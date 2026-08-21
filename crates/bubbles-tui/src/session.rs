//! Thin wrapper around [`bubbles::Runner`] that exposes exactly the slice
//! of runtime behaviour the TUI needs.
//!
//! Keeping the `Runner` behind `Session` lets the rest of the crate stay
//! ignorant of pull-event details and makes it obvious where every
//! `next_event` / `select_option` call happens.

use bubbles::{DialogueError, DialogueEvent, HashMapStorage, Runner, RunnerPhase, compile_many};

use crate::source_set::SourceSet;

/// Wraps a compiled program and its runner.
pub struct Session {
    runner: Runner<HashMapStorage>,
}

impl Session {
    /// Compiles all files in `sources` together via `compile_many` and primes
    /// a runner on `start_node`.
    ///
    /// Typo'd `<<jump>>`/`<<detour>>` targets that cross file boundaries are
    /// caught here as [`DialogueError::Validation`] errors.
    pub fn from_source_set(sources: &SourceSet, start_node: &str) -> Result<Self, DialogueError> {
        let slices = sources.as_named_slices();
        let program = compile_many(&slices)?;
        let mut runner = Runner::new(program, HashMapStorage::new());
        runner.start(start_node)?;
        Ok(Self { runner })
    }

    /// Returns `true` once the underlying dialogue has fully completed.
    pub fn is_done(&self) -> bool {
        self.runner.phase() == RunnerPhase::Done
    }

    /// Pulls the next event from the runner; yields `None` once the dialogue
    /// is finished.
    pub fn next_event(&mut self) -> Result<Option<DialogueEvent>, DialogueError> {
        self.runner.next_event()
    }

    /// Runs again from `start_node` without clearing variables or once-seen
    /// state.  The runner's internal storage and saliency history are
    /// preserved; only the execution stack is reset.
    ///
    /// # Errors
    /// Returns [`DialogueError::UnknownNode`] if `start_node` does not exist.
    pub fn rerun(&mut self, start_node: &str) -> Result<(), DialogueError> {
        self.runner.start(start_node)
    }

    /// Commits an option choice to the runner.
    ///
    /// # Errors
    /// Forwards any runtime error produced by [`Runner::select_option`]
    /// (e.g. out-of-range index).
    pub fn select_option(&mut self, index: usize) -> Result<(), DialogueError> {
        self.runner.select_option(index)
    }
}
