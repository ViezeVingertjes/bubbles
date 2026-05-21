use super::{Runner, State};
use crate::error::Result;

impl<S: crate::value::VariableStorage> Runner<S> {
    /// Captures the current session state into a [`RunnerSnapshot`](crate::RunnerSnapshot).
    ///
    /// The snapshot records the active node title, visit counts, and the set of
    /// exhausted `<<once>>` blocks. Variable storage is **not** included; persist
    /// it via [`Runner::storage`] alongside the snapshot when saving.
    ///
    /// Restoring with [`Runner::restore`] will restart execution from the beginning
    /// of the snapshotted node.
    ///
    /// Enable the `serde` feature on `bubbles-dialogue` if you need `Serialize` /
    /// `Deserialize` on [`RunnerSnapshot`](crate::RunnerSnapshot).
    #[must_use]
    pub fn snapshot(&self) -> crate::runtime::RunnerSnapshot {
        crate::runtime::RunnerSnapshot {
            current_node: self.stack.last().map(|f| f.node.as_ref().to_owned()),
            visits: self.visits.clone(),
            once_seen: self.once_seen.clone(),
        }
    }

    /// Applies a previously captured `RunnerSnapshot`, restoring visit counts
    /// and the `<<once>>` exhaustion set, then re-enters the snapshotted node
    /// from its beginning.
    ///
    /// # Errors
    ///
    /// Returns [`crate::DialogueError::UnknownNode`] if the snapshotted node no longer
    /// exists in the program (e.g. after a script update).
    pub fn restore(&mut self, snapshot: crate::runtime::RunnerSnapshot) -> Result<()> {
        self.visits = snapshot.visits;
        self.once_seen = snapshot.once_seen;
        self.stack.clear();
        self.clear_event_queues();
        self.state = State::Idle;
        if let Some(node) = snapshot.current_node {
            self.start(&node)?;
        }
        Ok(())
    }
}
