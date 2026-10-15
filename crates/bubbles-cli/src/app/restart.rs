//! The three ways a session starts over: reload from disk, rerun in place,
//! and step back by replaying the input log.
//!
//! Kept beside `mod.rs` to stay within the project's per-file line limit, and
//! because the distinction between them is the thing a reader most often
//! comes here to check.

use super::AppState;
use crate::history::HistoryStep;
use crate::session::Session;
use crate::transcript::Transcript;

impl AppState {
    /// `true` when there is a previous visible step to rewind to.
    #[must_use]
    pub const fn can_step_back(&self) -> bool {
        self.history.len() > 1
    }

    /// Rewinds one visible step by replaying the log minus its last entry.
    pub(super) fn step_back(&mut self) {
        if !self.can_step_back() {
            return;
        }
        let mut history = std::mem::take(&mut self.history);
        history.pop();
        self.reset_runtime_state();
        self.recording = false;
        for step in &history {
            match *step {
                HistoryStep::Advance => self.guarded(Self::advance),
                HistoryStep::SelectOption(i) => self.guarded(|s| s.commit_option(i)),
            }
        }
        self.recording = true;
        self.history = history;
    }

    /// Re-reads the stored sources and throws every scrap of runtime state
    /// away: variables, once-seen flags, transcript, and replay history.
    pub(super) fn reload(&mut self) {
        self.reset_runtime_state();
        self.history.clear();
    }

    /// Restarts at the start node while the runner keeps its storage and
    /// saliency history, so second-visit lines become reachable.
    pub(super) fn rerun(&mut self) {
        let result = self.session.as_mut().map(|s| s.rerun(&self.start_node));

        match result {
            Some(Ok(())) => self.error_overlay = None,
            Some(Err(err)) => self.fail(&err),
            // No active session (compile error on load) - full reload.
            None => self.reset_runtime_state(),
        }
        self.transcript = Transcript::new();
        self.clear_prompt();
        self.history.clear();
    }

    /// Rebuilds the session from the stored sources, discarding runtime state.
    pub(super) fn reset_runtime_state(&mut self) {
        match Session::from_source_set(&self.sources, &self.start_node) {
            Ok(session) => {
                self.session = Some(session);
                self.error_overlay = None;
            }
            Err(err) => self.fail(&err),
        }
        self.transcript = Transcript::new();
        self.clear_prompt();
    }
}
