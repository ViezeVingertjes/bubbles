//! [`Runner`] — the public entry point for executing a compiled [`Program`].

use crate::compiler::ast::Stmt;
use crate::compiler::Program;
use crate::error::{DialogueError, Result};
use crate::runtime::event::DialogueEvent;
use crate::value::VariableStorage;

/// Execution state of the runner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Idle,
    Running,
    AwaitingOption,
    Done,
}

/// Drives execution of a compiled [`Program`], yielding [`DialogueEvent`]s one at a time.
///
/// # Pull model
/// The host calls [`Runner::next_event`] in a loop until it returns `Ok(None)` (dialogue
/// ended) or until it receives a [`DialogueEvent::Options`], at which point it must call
/// [`Runner::select_option`] before continuing.
pub struct Runner<S: VariableStorage> {
    program: Program,
    storage: S,
    state: State,
    /// Current node title being executed.
    current_node: Option<String>,
    /// Cursor into the current node's body.
    cursor: usize,
    /// Pending events queued up ready to be returned by `next_event`.
    pending: std::collections::VecDeque<DialogueEvent>,
}

impl<S: VariableStorage> Runner<S> {
    /// Creates a new runner for the given program and variable storage.
    #[must_use]
    pub fn new(program: Program, storage: S) -> Self {
        Self {
            program,
            storage,
            state: State::Idle,
            current_node: None,
            cursor: 0,
            pending: std::collections::VecDeque::new(),
        }
    }

    /// Starts execution at the given node.
    ///
    /// # Errors
    /// Returns [`DialogueError::UnknownNode`] if the title does not exist in the program.
    pub fn start(&mut self, node: &str) -> Result<()> {
        if !self.program.node_exists(node) {
            return Err(DialogueError::UnknownNode(node.to_owned()));
        }
        self.current_node = Some(node.to_owned());
        self.cursor = 0;
        self.state = State::Running;
        self.pending.push_back(DialogueEvent::NodeStarted(node.to_owned()));
        Ok(())
    }

    /// Returns the next event, or `Ok(None)` when dialogue is finished.
    ///
    /// # Errors
    /// Returns a [`DialogueError`] on runtime failures.
    pub fn next_event(&mut self) -> Result<Option<DialogueEvent>> {
        // drain any pre-queued events first
        if let Some(ev) = self.pending.pop_front() {
            return Ok(Some(ev));
        }

        match self.state {
            State::Idle | State::Done => Ok(None),
            State::AwaitingOption => Err(DialogueError::Runtime(
                "call select_option() before next_event()".into(),
            )),
            State::Running => self.step(),
        }
    }

    /// Selects an option by index after receiving [`DialogueEvent::Options`].
    ///
    /// # Errors
    /// Returns [`DialogueError::Runtime`] if called when no options are pending or if
    /// the index is out of range.
    pub fn select_option(&mut self, _index: usize) -> Result<()> {
        if self.state != State::AwaitingOption {
            return Err(DialogueError::Runtime(
                "select_option() called when not awaiting an option".into(),
            ));
        }
        // Will be expanded once options are implemented
        self.state = State::Running;
        Ok(())
    }

    /// Returns a shared reference to the variable storage.
    #[must_use]
    pub fn storage(&self) -> &S {
        &self.storage
    }

    /// Returns a mutable reference to the variable storage.
    pub fn storage_mut(&mut self) -> &mut S {
        &mut self.storage
    }

    // ── execution engine (grows with each todo step) ──────────────────────────

    fn execute_stmt(&mut self, stmt: Stmt) -> Result<Option<DialogueEvent>> {
        match stmt {
            Stmt::Line { speaker, text, tags } => Ok(Some(DialogueEvent::Line { speaker, text, tags })),
            _ => Err(DialogueError::Runtime("unimplemented statement type".into())),
        }
    }

    fn step(&mut self) -> Result<Option<DialogueEvent>> {
        let node_title = match &self.current_node {
            Some(t) => t.clone(),
            None => {
                self.state = State::Done;
                return Ok(Some(DialogueEvent::DialogueComplete));
            }
        };

        let body_len = self
            .program
            .node_group(&node_title)
            .and_then(|g| g.first())
            .map(|n| n.body.len())
            .unwrap_or(0);

        if self.cursor >= body_len {
            // node finished
            self.state = State::Done;
            self.pending.push_back(DialogueEvent::DialogueComplete);
            return Ok(Some(DialogueEvent::NodeComplete(node_title)));
        }

        // Clone what we need before borrowing mutably.
        let stmt = self
            .program
            .node_group(&node_title)
            .and_then(|g| g.first())
            .map(|n| n.body[self.cursor].clone())
            .ok_or_else(|| DialogueError::Runtime("cursor out of range".into()))?;
        self.cursor += 1;

        self.execute_stmt(stmt)
    }
}
