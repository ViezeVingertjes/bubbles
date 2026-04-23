//! Node-body selection, including node-group resolution via the active
//! [`SaliencyStrategy`].
//!
//! [`SaliencyStrategy`]: crate::saliency::SaliencyStrategy

use crate::compiler::ast::StmtList;
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
    /// Bodies are shared via [`StmtList`] (i.e. `Arc<[Stmt]>`), so the return
    /// value is a cheap reference-count bump rather than a deep copy.
    ///
    /// [`SaliencyStrategy`]: crate::saliency::SaliencyStrategy
    pub(super) fn pick_node_body(&mut self, title: &str) -> Result<StmtList> {
        let Some(group) = self.program.node_group(title) else {
            return Err(DialogueError::UnknownNode(title.to_owned()));
        };

        let has_when = group.iter().any(|n| n.when.is_some());
        if !has_when {
            return Ok(StmtList::clone(&group[0].body));
        }

        // Snapshot candidate metadata so the `&self.program` borrow can be
        // released before we mutably borrow `self.saliency`.  Each body is
        // cloned as an `Arc` so this loop is O(group_len) regardless of body
        // size.
        let candidate_info: Vec<(String, bool, StmtList)> = group
            .iter()
            .map(|n| {
                let available = n
                    .when
                    .as_ref()
                    .is_none_or(|e| self.eval_expr(e.as_ref()).is_ok_and(|v| v.is_truthy()));
                (n.title.clone(), available, StmtList::clone(&n.body))
            })
            .collect();

        // Candidate IDs must be unique within the group.  Nodes in a group
        // share the same title, so we append the index to make IDs stable
        // across calls (`BestLeastRecentlyViewed` uses them as history keys).
        let candidate_ids: Vec<String> = candidate_info
            .iter()
            .enumerate()
            .map(|(i, (t, _, _))| format!("{t}#{i}"))
            .collect();

        let candidates: Vec<Candidate<'_>> = candidate_ids
            .iter()
            .zip(candidate_info.iter())
            .map(|(id, (_, available, _))| Candidate {
                id: id.as_str(),
                available: *available,
            })
            .collect();

        let idx = self.saliency.select(&candidates).ok_or_else(|| {
            DialogueError::ProtocolViolation(format!(
                "node group '{title}' has no available candidate"
            ))
        })?;
        let len = candidate_info.len();
        candidate_info
            .into_iter()
            .nth(idx)
            .map(|(_, _, body)| body)
            .ok_or_else(|| {
                DialogueError::ProtocolViolation(format!(
                    "saliency strategy returned index {idx} but node group '{title}' \
                     only has {len} candidate(s)"
                ))
            })
    }
}
