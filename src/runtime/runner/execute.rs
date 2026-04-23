//! `step` and `execute_stmt` — the core statement dispatch loop.

use std::sync::Arc;

use crate::compiler::ast::{Expr, IfBranch, Stmt, TextSegment};
use crate::error::{DialogueError, Result};
use crate::runtime::event::{DialogueEvent, DialogueOption, line_id_from_tags};
use crate::saliency::Candidate;
use crate::value::VariableStorage;

use super::{Frame, Runner, State};

impl<S: VariableStorage> Runner<S> {
    pub(super) fn step(&mut self) -> Result<Option<DialogueEvent>> {
        let stmt = loop {
            let Some(frame) = self.stack.last_mut() else {
                self.state = State::Done;
                return Ok(Some(DialogueEvent::DialogueComplete));
            };
            if let Some(s) = frame.stmts.pop_front() {
                break s;
            }
            let finished_node = frame.node.clone();
            self.stack.pop();
            if self.stack.is_empty() {
                self.state = State::Done;
                self.pending.push_back(DialogueEvent::DialogueComplete);
                return Ok(Some(DialogueEvent::NodeComplete(finished_node)));
            }
        };

        self.execute_stmt(stmt)
    }

    pub(super) fn execute_stmt(&mut self, stmt: Stmt) -> Result<Option<DialogueEvent>> {
        match stmt {
            Stmt::Line {
                speaker,
                text,
                tags,
            } => self.exec_line(speaker, &text, tags),
            Stmt::Set { name, expr } => {
                let value = self.eval_expr(expr.as_ref())?;
                self.storage.set(&name, value);
                Ok(None)
            }
            Stmt::Declare { name, expr, .. } => {
                if self.storage.get(&name).is_none() {
                    let value = self.eval_expr(expr.as_ref())?;
                    self.storage.set(&name, value);
                }
                Ok(None)
            }
            Stmt::LineGroup(variants) => self.exec_line_group(&variants),
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
            Stmt::Command { name, args, tags } => {
                let args = self.eval_segments_as_args(&args)?;
                Ok(Some(DialogueEvent::Command { name, args, tags }))
            }
        }
    }

    fn exec_line(
        &self,
        speaker: Option<String>,
        text: &[TextSegment],
        tags: Vec<String>,
    ) -> Result<Option<DialogueEvent>> {
        let text = self.eval_line_text(text, &tags)?;
        let line_id = line_id_from_tags(&tags);
        Ok(Some(DialogueEvent::Line {
            speaker,
            text,
            line_id,
            tags,
        }))
    }

    fn exec_line_group(
        &mut self,
        variants: &[crate::compiler::ast::LineVariant],
    ) -> Result<Option<DialogueEvent>> {
        // Collect availability into a local Vec so `self.saliency` can be mutably
        // borrowed during `select` without conflicting with the prior immutable reads.
        let availability: Vec<bool> = variants
            .iter()
            .map(|v| {
                v.cond
                    .as_ref()
                    .is_none_or(|e| self.eval_expr(e.as_ref()).is_ok_and(|val| val.is_truthy()))
                    && !(v.once && self.once_seen.contains(&v.id))
            })
            .collect();

        let candidate_ids: Vec<&str> = variants.iter().map(|v| v.id.as_str()).collect();
        let candidates: Vec<Candidate<'_>> = candidate_ids
            .iter()
            .zip(availability.iter())
            .map(|(&id, &available)| Candidate { id, available })
            .collect();

        if let Some(idx) = self.saliency.select(&candidates) {
            let chosen = &variants[idx];
            if chosen.once {
                self.once_seen.insert(chosen.id.clone());
            }
            let text = self.eval_line_text(&chosen.text, &chosen.tags)?;
            let line_id = line_id_from_tags(&chosen.tags);
            return Ok(Some(DialogueEvent::Line {
                speaker: chosen.speaker.clone(),
                text,
                line_id,
                tags: chosen.tags.clone(),
            }));
        }
        Ok(None)
    }

    fn exec_options(
        &mut self,
        items: Vec<crate::compiler::ast::OptionItem>,
    ) -> Result<Option<DialogueEvent>> {
        let mut options = Vec::with_capacity(items.len());
        let mut bodies = Vec::with_capacity(items.len());
        for item in items {
            let available = match &item.cond {
                Some(e) => self.eval_expr(e.as_ref())?.is_truthy(),
                None => true,
            };
            let text = self.eval_line_text(&item.text, &item.tags)?;
            let line_id = line_id_from_tags(&item.tags);
            options.push(DialogueOption {
                text,
                available,
                line_id,
                tags: item.tags.clone(),
            });
            bodies.push(item.body);
        }
        self.option_bodies = bodies;
        self.state = State::AwaitingOption;
        Ok(Some(DialogueEvent::Options(options)))
    }

    fn exec_if(
        &mut self,
        branches: Vec<IfBranch>,
        else_body: Vec<Stmt>,
    ) -> Result<Option<DialogueEvent>> {
        let mut chosen = None;
        for b in branches {
            if self.eval_expr(b.cond.as_ref())?.is_truthy() {
                chosen = Some(b.body);
                break;
            }
        }
        self.push_inline_frame(chosen.unwrap_or(else_body));
        Ok(None)
    }

    fn exec_once(
        &mut self,
        block_id: String,
        cond: Option<&Arc<Expr>>,
        body: Vec<Stmt>,
        else_body: Vec<Stmt>,
    ) -> Result<Option<DialogueEvent>> {
        let cond_ok = match cond {
            None => true,
            Some(e) => self.eval_expr(e.as_ref())?.is_truthy(),
        };
        let first_time = !self.once_seen.contains(&block_id);
        let run_body = cond_ok && first_time;
        if run_body {
            self.once_seen.insert(block_id);
        }
        self.push_inline_frame(if run_body { body } else { else_body });
        Ok(None)
    }

    fn exec_jump(&mut self, target: String) -> Result<Option<DialogueEvent>> {
        if !self.program.node_exists(&target) {
            return Err(DialogueError::UnknownNode(target));
        }
        let body = self.pick_node_body(&target)?;
        self.stack.clear();
        self.stack.push(Frame::new(target.clone(), body));
        *self.visits.entry(target.clone()).or_insert(0) += 1;
        self.pending.push_front(DialogueEvent::NodeStarted(target));
        Ok(None)
    }

    fn exec_detour(&mut self, target: String) -> Result<Option<DialogueEvent>> {
        if !self.program.node_exists(&target) {
            return Err(DialogueError::UnknownNode(target));
        }
        let body = self.pick_node_body(&target)?;
        *self.visits.entry(target.clone()).or_insert(0) += 1;
        self.stack.push(Frame::new(target.clone(), body));
        self.pending.push_front(DialogueEvent::NodeStarted(target));
        Ok(None)
    }
}
