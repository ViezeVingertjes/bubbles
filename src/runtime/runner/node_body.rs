//! Node-body selection, including node-group resolution via the active
//! [`SaliencyStrategy`].
//!
//! [`SaliencyStrategy`]: crate::saliency::SaliencyStrategy

use crate::compiler::ast::Stmt;
use crate::error::{DialogueError, Result};
use crate::saliency::Candidate;
use crate::value::VariableStorage;

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

        let has_when = group.iter().any(|n| n.when.is_some());
        if !has_when {
            return Ok(group[0].body.as_ref().clone());
        }

        // Collect candidate metadata into owned data so we can borrow `self.saliency`
        // mutably afterwards without a conflict with the `&self.program` borrow.
        let candidate_info: Vec<(String, bool, Vec<Stmt>)> = group
            .iter()
            .map(|n| {
                let available = n
                    .when
                    .as_ref()
                    .is_none_or(|e| self.eval_expr(e.as_ref()).is_ok_and(|v| v.is_truthy()));
                (n.title.clone(), available, n.body.as_ref().clone())
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
        Ok(candidate_info
            .into_iter()
            .nth(idx)
            .expect("saliency returned an in-bounds index")
            .2)
    }
}
