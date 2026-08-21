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

        // Snapshot candidate bodies (O(1) `Arc` bumps) and availability so the
        // `&self.program` borrow is released before `self.saliency` is borrowed
        // mutably.
        //
        // Candidate IDs must be unique within the group.  Nodes in a group
        // share the same title, so we append the index to make IDs stable
        // across calls (`BestLeastRecentlyViewed` uses them as history keys).
        let bodies: Vec<StmtList> = group.iter().map(|n| StmtList::clone(&n.body)).collect();
        let candidate_ids: Vec<String> = (0..group.len()).map(|i| format!("{title}#{i}")).collect();
        let candidates: Vec<Candidate<'_>> = group
            .iter()
            .zip(&candidate_ids)
            .map(|(n, id)| Candidate {
                id,
                available: n
                    .when
                    .as_ref()
                    .is_none_or(|e| self.eval_expr(e.as_ref()).is_ok_and(|v| v.is_truthy())),
            })
            .collect();

        let idx = self.saliency.select(&candidates).ok_or_else(|| {
            DialogueError::ProtocolViolation(format!(
                "node group '{title}' has no available candidate"
            ))
        })?;
        let len = bodies.len();
        bodies.into_iter().nth(idx).ok_or_else(|| {
            DialogueError::ProtocolViolation(format!(
                "saliency strategy returned index {idx} but node group '{title}' \
                 only has {len} candidate(s)"
            ))
        })
    }
}
