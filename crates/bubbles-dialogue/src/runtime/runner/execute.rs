//! `step` and `execute_stmt` - the core statement dispatch loop.

use std::sync::Arc;

use crate::compiler::ast::{Expr, IfBranch, Stmt, StmtList, TextSegment};
use crate::error::Result;
use crate::runtime::event::{
    DialogueEvent, DialogueOption, line_mode_from_tags, option_group_from_tags,
};
use crate::saliency::Candidate;
use crate::value::VariableStorage;

use super::{Runner, RunnerPhase};

impl<S: VariableStorage> Runner<S> {
    pub(super) fn step(&mut self) -> Result<Option<DialogueEvent>> {
        // Clone the StmtList (Arc bump, O(1)) to obtain independent ownership of the
        // body slice. This releases the mutable borrow of self.stack before we call
        // execute_stmt, which needs &mut self.  The individual Stmt is then borrowed
        // from our locally-owned Arc rather than from the stack frame, so the borrow
        // checker is satisfied without cloning the Stmt itself.
        let (body, ip) = loop {
            let Some(frame) = self.stack.last_mut() else {
                self.state = RunnerPhase::Done;
                return Ok(Some(DialogueEvent::DialogueComplete));
            };
            if frame.ip < frame.body.len() {
                let body = frame.body.clone(); // O(1) Arc reference-count bump
                let ip = frame.ip;
                frame.ip += 1;
                break (body, ip);
            }
            let finished_node = frame.node.as_ref().to_owned();
            self.stack.pop();
            if self.stack.is_empty() {
                self.state = RunnerPhase::Done;
                self.pending.push_back(DialogueEvent::DialogueComplete);
                return Ok(Some(DialogueEvent::NodeComplete(finished_node)));
            }
        };

        self.execute_stmt(&body[ip])
    }

    pub(super) fn execute_stmt(&mut self, stmt: &Stmt) -> Result<Option<DialogueEvent>> {
        match stmt {
            Stmt::Line {
                speaker,
                text,
                tags,
            } => self.exec_line(speaker.clone(), text, tags.clone()),
            Stmt::Set { name, expr } => {
                let value = self.eval_expr(expr.as_ref())?;
                self.storage.set(name, value);
                Ok(None)
            }
            Stmt::Declare { name, expr, .. } => {
                if self.storage.get(name).is_none() {
                    let value = self.eval_expr(expr.as_ref())?;
                    self.storage.set(name, value);
                }
                Ok(None)
            }
            Stmt::LineGroup(variants) => self.exec_line_group(variants),
            Stmt::Options(items) => self.exec_options(items),
            Stmt::If {
                branches,
                else_body,
            } => self.exec_if(branches, else_body),
            Stmt::Once {
                block_id,
                cond,
                body,
                else_body,
            } => self.exec_once(block_id, cond.as_ref(), body, else_body),
            Stmt::Jump(target) => self.exec_jump(target),
            Stmt::Detour(target) => self.exec_detour(target),
            Stmt::Return => {
                self.stack.pop();
                Ok(None)
            }
            Stmt::Stop => {
                self.stack.clear();
                self.pending.clear();
                self.option_bodies.clear();
                self.state = RunnerPhase::Done;
                Ok(Some(DialogueEvent::DialogueComplete))
            }
            Stmt::Command { name, args, tags } => {
                let args = self.eval_segments_as_args(args)?;
                Ok(Some(DialogueEvent::Command {
                    name: name.clone(),
                    args,
                    tags: tags.clone(),
                }))
            }
        }
    }

    fn exec_line(
        &self,
        speaker: Option<String>,
        text: &[TextSegment],
        tags: Vec<String>,
    ) -> Result<Option<DialogueEvent>> {
        let (text, spans, line_id) = self.eval_line_text(text, &tags)?;
        let line_mode = line_mode_from_tags(&tags);
        Ok(Some(DialogueEvent::Line {
            speaker,
            text,
            line_id,
            tags,
            line_mode,
            spans,
        }))
    }

    fn exec_line_group(
        &mut self,
        variants: &[crate::compiler::ast::LineVariant],
    ) -> Result<Option<DialogueEvent>> {
        // Candidates borrow `variants` (not `self`), so `self.saliency` can be
        // mutably borrowed during `select` once this Vec is built.
        let candidates: Vec<Candidate<'_>> =
            variants
                .iter()
                .map(|v| Candidate {
                    id: v.id.as_str(),
                    available: v.cond.as_ref().is_none_or(|e| {
                        self.eval_expr(e.as_ref()).is_ok_and(|val| val.is_truthy())
                    }) && !(v.once && self.once_seen.contains(&v.id)),
                })
                .collect();

        let Some(idx) = self.saliency.select(&candidates) else {
            return Ok(None);
        };
        let chosen = &variants[idx];
        if chosen.once {
            self.once_seen.insert(chosen.id.clone());
        }
        self.exec_line(chosen.speaker.clone(), &chosen.text, chosen.tags.clone())
    }

    fn exec_options(
        &mut self,
        items: &[crate::compiler::ast::OptionItem],
    ) -> Result<Option<DialogueEvent>> {
        let mut options = Vec::with_capacity(items.len());
        let mut bodies = Vec::with_capacity(items.len());
        for item in items {
            let guard_available = match &item.cond {
                Some(e) => self.eval_expr(e.as_ref())?.is_truthy(),
                None => true,
            };
            let once_available = !(item.once && self.once_seen.contains(&item.id));
            let available = guard_available && once_available;
            let (text, spans, line_id) = self.eval_line_text(&item.text, &item.tags)?;
            let group = option_group_from_tags(&item.tags);
            options.push(DialogueOption {
                text,
                available,
                line_id,
                tags: item.tags.clone(),
                group,
                spans,
            });
            let once_id = item.once.then(|| item.id.clone());
            bodies.push((available, once_id, item.body.clone())); // Arc<[Stmt]> - O(1)
        }
        self.option_bodies = bodies;
        self.state = RunnerPhase::AwaitingOption;
        Ok(Some(DialogueEvent::Options(options)))
    }

    fn exec_if(
        &mut self,
        branches: &[IfBranch],
        else_body: &StmtList,
    ) -> Result<Option<DialogueEvent>> {
        let mut chosen: Option<StmtList> = None;
        for b in branches {
            if self.eval_expr(b.cond.as_ref())?.is_truthy() {
                chosen = Some(b.body.clone()); // Arc<[Stmt]> - O(1) reference-count bump
                break;
            }
        }
        self.push_inline_frame(chosen.unwrap_or_else(|| else_body.clone()));
        Ok(None)
    }

    fn exec_once(
        &mut self,
        block_id: &str,
        cond: Option<&Arc<Expr>>,
        body: &StmtList,
        else_body: &StmtList,
    ) -> Result<Option<DialogueEvent>> {
        let cond_ok = match cond {
            None => true,
            Some(e) => self.eval_expr(e.as_ref())?.is_truthy(),
        };
        let first_time = !self.once_seen.contains(block_id);
        let run_body = cond_ok && first_time;
        if run_body {
            self.once_seen.insert(block_id.to_owned());
        }
        // Arc<[Stmt]> clones are O(1) reference-count bumps
        self.push_inline_frame(if run_body {
            body.clone()
        } else {
            else_body.clone()
        });
        Ok(None)
    }

    fn exec_jump(&mut self, target: &str) -> Result<Option<DialogueEvent>> {
        self.enter_node(target, true)
    }

    fn exec_detour(&mut self, target: &str) -> Result<Option<DialogueEvent>> {
        self.enter_node(target, false)
    }

    /// Common plumbing for `<<jump>>` / `<<detour>>`: resolve the node body,
    /// bump the visit counter, push a frame, and enqueue `NodeStarted`.
    ///
    /// When `replace_stack` is `true` the entire stack is cleared first
    /// (`<<jump>>` semantics); otherwise the new frame is pushed on top
    /// (`<<detour>>` semantics).
    fn enter_node(&mut self, target: &str, replace_stack: bool) -> Result<Option<DialogueEvent>> {
        let body = self.pick_node_body(target)?;
        if replace_stack {
            self.stack.clear();
        }
        self.record_visit(target);
        self.push_node_frame(target, body);
        self.pending
            .push_front(DialogueEvent::NodeStarted(target.to_owned()));
        Ok(None)
    }
}
