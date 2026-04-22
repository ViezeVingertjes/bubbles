//! Internal helper methods on [`Runner`] — node body selection, expression
//! evaluation, text interpolation, and visit-count queries.

use crate::compiler::ast::Stmt;
use crate::error::{DialogueError, Result};
use crate::runtime::eval::eval;
use crate::runtime::interpolate::interpolate;
use crate::saliency::Candidate;
use crate::value::{Value, VariableStorage};

use super::Runner;

impl<S: VariableStorage> Runner<S> {
    /// Picks the body of the node (or node-group variant) with the given title.
    ///
    /// When the group has a single node with no `when:` clause the body is returned
    /// directly.  For node groups each variant's `when:` condition is evaluated and
    /// the active [`SaliencyStrategy`] selects among the eligible candidates.
    ///
    /// [`SaliencyStrategy`]: crate::saliency::SaliencyStrategy
    pub(super) fn pick_node_body(&mut self, title: &str) -> Result<Vec<Stmt>> {
        let Some(group) = self.program.node_group(title) else {
            return Err(DialogueError::UnknownNode(title.to_owned()));
        };

        let has_when = group.iter().any(|n| n.when_src.is_some());
        if !has_when {
            return Ok(group[0].body.clone());
        }

        // Collect candidate metadata into owned data so we can borrow `self.saliency`
        // mutably afterwards without a conflict with the `&self.program` borrow.
        let candidate_info: Vec<(String, bool, Vec<Stmt>)> = group
            .iter()
            .map(|n| {
                let available = n.when_src.as_deref().is_none_or(|src| {
                    self.eval_expr_src(src)
                        .map(|v| v.is_truthy())
                        .unwrap_or(false)
                });
                (n.title.clone(), available, n.body.clone())
            })
            .collect();

        // Build candidate IDs that are unique within the group.
        // Nodes share the same title, so we append the index to make IDs stable
        // across calls (required for BestLeastRecentlyViewed to track history).
        let candidate_ids: Vec<String> = candidate_info
            .iter()
            .enumerate()
            .map(|(i, (t, _, _))| format!("{t}#{i}"))
            .collect();

        // Candidates now reference local data, not `self.program`.
        let candidates: Vec<Candidate<'_>> = candidate_ids
            .iter()
            .zip(candidate_info.iter())
            .map(|(id, (_, available, _))| Candidate {
                id: id.as_str(),
                available: *available,
            })
            .collect();

        let idx = self.saliency.select(&candidates).ok_or_else(|| {
            DialogueError::Runtime(format!("node group '{title}' has no available candidate"))
        })?;
        Ok(candidate_info.into_iter().nth(idx).unwrap().2)
    }

    /// Evaluates an expression source string against the current variable storage.
    pub(super) fn eval_expr_src(&self, src: &str) -> Result<Value> {
        let expr = crate::compiler::expr::parse_expr(src)?;
        eval(&expr, &self.storage, &|name, args| {
            self.library.call(name, args)
        })
    }

    /// Runs inline `{expr}` substitution on `text` using current storage.
    pub(super) fn interpolate_text(&self, text: &str) -> Result<String> {
        interpolate(text, &self.storage, &|name, args| {
            self.library.call(name, args)
        })
    }

    /// Splits `args_src` into whitespace-separated tokens after interpolation.
    pub(super) fn parse_command_args(&self, args_src: &str) -> Result<Vec<String>> {
        if args_src.trim().is_empty() {
            return Ok(Vec::new());
        }
        let interpolated = self.interpolate_text(args_src)?;
        Ok(interpolated.split_whitespace().map(str::to_owned).collect())
    }
}
