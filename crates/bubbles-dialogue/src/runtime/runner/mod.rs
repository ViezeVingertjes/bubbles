//! [`Runner`] - the public entry point for executing a compiled [`Program`].

pub(super) mod evaluation;
pub(super) mod execute;
pub(super) mod node_body;

use std::borrow::Cow;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

use crate::compiler::Program;
use crate::compiler::ast::StmtList;
use crate::error::{DialogueError, Result};
use crate::library::FunctionLibrary;
use crate::runtime::event::DialogueEvent;
use crate::runtime::provider::{LineProvider, PassthroughProvider};
use crate::saliency::{FirstAvailable, SaliencyStrategy};
use crate::value::{Value, VariableStorage};

/// Where the [`Runner`] is in its `start` / `next_event` / [`Runner::select_option`] protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunnerPhase {
    /// No dialogue running; call [`Runner::start`].
    Idle,
    /// Advancing lines and statements; call [`Runner::next_event`].
    Running,
    /// The last event was [`DialogueEvent::Options`]; call [`Runner::select_option`] before
    /// [`Runner::next_event`].
    AwaitingOption,
    /// The current node finished; the stream is finished until the next [`Runner::start`].
    Done,
}

/// Execution state of the runner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum State {
    Idle,
    Running,
    AwaitingOption,
    Done,
}

/// A frame on the call stack.
///
/// Frames reference a shared [`StmtList`] and advance a program counter.
/// No statements are cloned when a frame is pushed - only the `Arc` is
/// bumped - so control-flow constructs (`<<if>>`, `<<once>>`, options,
/// detours, jumps) are O(1) regardless of body size.
#[derive(Debug, Clone)]
pub(super) struct Frame {
    pub(super) node: Arc<str>,
    pub(super) body: StmtList,
    pub(super) ip: usize,
}

impl Frame {
    pub(super) const fn new(node: Arc<str>, body: StmtList) -> Self {
        Self { node, body, ip: 0 }
    }
}

/// Option bodies held during `AwaitingOption` state.
type OptionBodies = Vec<StmtList>;

/// Drives execution of a compiled [`Program`], yielding [`DialogueEvent`]s one at a time.
///
/// # Pull model
/// The host calls [`Runner::next_event`] in a loop until it returns `Ok(None)` (dialogue
/// ended) or until it receives a [`DialogueEvent::Options`], at which point it must call
/// [`Runner::select_option`] before continuing.
pub struct Runner<S: VariableStorage> {
    pub(super) program: Program,
    pub(super) storage: S,
    pub(super) state: State,
    pub(super) stack: Vec<Frame>,
    pub(super) pending: VecDeque<DialogueEvent>,
    pub(super) option_bodies: OptionBodies,
    pub(super) library: FunctionLibrary,
    pub(super) visits: HashMap<String, u32>,
    pub(super) once_seen: HashSet<String>,
    pub(super) saliency: Box<dyn SaliencyStrategy>,
    pub(super) provider: Box<dyn LineProvider>,
}

impl<S: VariableStorage> Runner<S> {
    fn clear_event_queues(&mut self) {
        self.pending.clear();
        self.option_bodies.clear();
    }

    /// Returns read-only access to the compiled program (node titles, declarations, etc.).
    #[must_use]
    pub const fn program(&self) -> &Program {
        &self.program
    }

    /// Returns the runner’s current phase for UI or protocol guards.
    #[must_use]
    pub const fn phase(&self) -> RunnerPhase {
        match self.state {
            State::Idle => RunnerPhase::Idle,
            State::Running => RunnerPhase::Running,
            State::AwaitingOption => RunnerPhase::AwaitingOption,
            State::Done => RunnerPhase::Done,
        }
    }

    /// Creates a new runner for the given program and variable storage.
    #[must_use]
    pub fn new(program: Program, storage: S) -> Self {
        Self {
            program,
            storage,
            state: State::Idle,
            stack: Vec::new(),
            pending: VecDeque::new(),
            option_bodies: Vec::new(),
            library: FunctionLibrary::new(),
            visits: HashMap::new(),
            once_seen: HashSet::new(),
            saliency: Box::new(FirstAvailable),
            provider: Box::new(PassthroughProvider),
        }
    }

    /// Starts execution at the given node.
    ///
    /// Clears any queued events and abandons an in-flight choice so a new conversation
    /// cannot inherit stale [`DialogueEvent::DialogueComplete`] or option state from a
    /// prior [`Runner::start`].
    ///
    /// # Errors
    /// Returns [`DialogueError::UnknownNode`] if the title does not exist in the program.
    pub fn start(&mut self, node: &str) -> Result<()> {
        if !self.program.node_exists(node) {
            return Err(DialogueError::UnknownNode(node.to_owned()));
        }
        let body = self.pick_node_body(node)?;
        self.clear_event_queues();
        let title: Arc<str> = Arc::from(node);
        self.stack.clear();
        self.stack.push(Frame::new(title, body));
        self.state = State::Running;
        *self.visits.entry(node.to_owned()).or_insert(0) += 1;
        self.pending
            .push_back(DialogueEvent::NodeStarted(node.to_owned()));
        Ok(())
    }

    /// Returns the next event, or `Ok(None)` when dialogue is finished.
    ///
    /// # Errors
    /// Returns a [`DialogueError`] on runtime failures.
    pub fn next_event(&mut self) -> Result<Option<DialogueEvent>> {
        if let Some(ev) = self.pending.pop_front() {
            return Ok(Some(ev));
        }
        match self.state {
            State::Idle | State::Done => Ok(None),
            State::AwaitingOption => Err(DialogueError::ProtocolViolation(
                "call select_option() before next_event()".into(),
            )),
            State::Running => loop {
                if let Some(ev) = self.pending.pop_front() {
                    return Ok(Some(ev));
                }
                if self.state != State::Running {
                    return Ok(None);
                }
                if let Some(ev) = self.step()? {
                    return Ok(Some(ev));
                }
            },
        }
    }

    /// Selects an option by index after receiving [`DialogueEvent::Options`].
    ///
    /// # Errors
    ///
    /// Returns [`DialogueError::ProtocolViolation`] if called when not awaiting an option
    /// selection, or if `index` is out of range.
    pub fn select_option(&mut self, index: usize) -> Result<()> {
        if self.state != State::AwaitingOption {
            return Err(DialogueError::ProtocolViolation(
                "select_option() called when not awaiting an option".into(),
            ));
        }
        let body = self.option_bodies.get(index).cloned().ok_or_else(|| {
            DialogueError::ProtocolViolation(format!(
                "option index {index} out of range ({})",
                self.option_bodies.len()
            ))
        })?;
        self.option_bodies.clear();
        self.state = State::Running;
        self.push_inline_frame(body);
        Ok(())
    }

    /// Pushes `body` as a new frame on top of the stack, inheriting the
    /// current frame's node title. No-op when `body` is empty.
    ///
    /// Used by `<<if>>`, `<<once>>`, and option-body execution.
    pub(super) fn push_inline_frame(&mut self, body: StmtList) {
        if body.is_empty() {
            return;
        }
        let title = self
            .stack
            .last()
            .map_or_else(|| Arc::from(""), |f| Arc::clone(&f.node));
        self.stack.push(Frame::new(title, body));
    }

    /// Pushes a frame whose node title matches the supplied owned string.
    ///
    /// Used by `<<jump>>` / `<<detour>>` / `start()` - sites that already
    /// know the target node title and want to install it as the top-of-stack
    /// frame in one call.
    pub(super) fn push_node_frame(&mut self, node: &str, body: StmtList) {
        self.stack.push(Frame::new(Arc::from(node), body));
    }

    /// Returns a shared reference to the variable storage.
    #[must_use]
    pub const fn storage(&self) -> &S {
        &self.storage
    }

    /// Returns a mutable reference to the variable storage.
    pub const fn storage_mut(&mut self) -> &mut S {
        &mut self.storage
    }

    /// Returns all variables known to storage (see [`VariableStorage::all_variables`]).
    ///
    /// For [`HashMapStorage`](crate::HashMapStorage) this is every key/value pair.
    /// Custom stores return whatever their [`VariableStorage::all_variables`]
    /// implementation provides (often empty unless overridden).
    #[must_use]
    pub fn all_variables(&self) -> Vec<(String, Value)> {
        self.storage.all_variables()
    }

    /// Returns a clone of the value for `name`, or `None` if unset.
    #[must_use]
    pub fn variable(&self, name: &str) -> Option<Value> {
        self.storage.get(name)
    }

    /// Borrows the value for `name` when the storage can lend it without cloning.
    #[must_use]
    pub fn variable_ref(&self, name: &str) -> Option<Cow<'_, Value>> {
        self.storage.get_ref(name)
    }

    /// Returns a mutable reference to the function library (for registering host functions).
    pub const fn library_mut(&mut self) -> &mut FunctionLibrary {
        &mut self.library
    }

    /// Replaces the saliency strategy used for line and node group selection.
    pub fn set_saliency(&mut self, strategy: impl SaliencyStrategy) {
        self.saliency = Box::new(strategy);
    }

    /// Sets the line provider used for localisation lookup.
    pub fn set_provider(&mut self, provider: impl LineProvider) {
        self.provider = Box::new(provider);
    }

    /// Replaces the saliency strategy from an already-boxed value.
    ///
    /// Used by [`crate::runtime::RunnerBuilder`] to avoid double-boxing.
    pub(crate) fn set_saliency_box(&mut self, strategy: Box<dyn SaliencyStrategy>) {
        self.saliency = strategy;
    }

    /// Sets the line provider from an already-boxed value.
    ///
    /// Used by [`crate::runtime::RunnerBuilder`] to avoid double-boxing.
    pub(crate) fn set_provider_box(&mut self, provider: Box<dyn LineProvider>) {
        self.provider = provider;
    }

    // ── save / load ───────────────────────────────────────────────────────────

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
    /// Returns [`DialogueError::UnknownNode`] if the snapshotted node no longer
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
