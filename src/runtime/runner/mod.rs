//! [`Runner`] — the public entry point for executing a compiled [`Program`].

pub(super) mod evaluation;
pub(super) mod execute;
pub(super) mod node_body;

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, RwLock};

use crate::compiler::Program;
use crate::compiler::ast::Stmt;
use crate::error::{DialogueError, Result};
use crate::library::FunctionLibrary;
use crate::runtime::event::DialogueEvent;
use crate::runtime::provider::{LineProvider, PassthroughProvider};
use crate::saliency::{FirstAvailable, SaliencyStrategy};
use crate::value::{Value, VariableStorage};

/// Execution state of the runner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum State {
    Idle,
    Running,
    AwaitingOption,
    Done,
}

/// A frame on the call stack.
#[derive(Debug, Clone)]
pub(super) struct Frame {
    pub(super) node: String,
    pub(super) stmts: VecDeque<Stmt>,
}

impl Frame {
    pub(super) fn new(node: String, body: Vec<Stmt>) -> Self {
        Self {
            node,
            stmts: VecDeque::from(body),
        }
    }
}

/// Option bodies held during `AwaitingOption` state.
type OptionBodies = Vec<Vec<Stmt>>;

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
    pub(super) visits: Arc<RwLock<HashMap<String, usize>>>,
    pub(super) once_seen: HashSet<String>,
    pub(super) saliency: Box<dyn SaliencyStrategy>,
    pub(super) provider: Box<dyn LineProvider>,
}

impl<S: VariableStorage> Runner<S> {
    /// Creates a new runner for the given program and variable storage.
    ///
    /// # Panics
    /// Panics only if the internal `RwLock` is poisoned, which cannot happen in normal use.
    #[must_use]
    pub fn new(program: Program, storage: S) -> Self {
        let visits: Arc<RwLock<HashMap<String, usize>>> = Arc::default();
        let mut library = FunctionLibrary::new();

        let v1 = Arc::clone(&visits);
        library.register("visited", move |args| {
            let title = match args.as_slice() {
                [Value::Text(t)] => t.clone(),
                _ => {
                    return Err(DialogueError::Function {
                        name: "visited".into(),
                        message: "expected one string argument".into(),
                    });
                }
            };
            Ok(Value::Bool(
                *v1.read().unwrap().get(&title).unwrap_or(&0) > 0,
            ))
        });
        let v2 = Arc::clone(&visits);
        library.register("visited_count", move |args| {
            let title = match args.as_slice() {
                [Value::Text(t)] => t.clone(),
                _ => {
                    return Err(DialogueError::Function {
                        name: "visited_count".into(),
                        message: "expected one string argument".into(),
                    });
                }
            };
            let count = *v2.read().unwrap().get(&title).unwrap_or(&0);
            #[allow(clippy::cast_precision_loss)]
            Ok(Value::Number(count as f64))
        });

        Self {
            program,
            storage,
            state: State::Idle,
            stack: Vec::new(),
            pending: VecDeque::new(),
            option_bodies: Vec::new(),
            library,
            visits,
            once_seen: HashSet::new(),
            saliency: Box::new(FirstAvailable),
            provider: Box::new(PassthroughProvider),
        }
    }

    /// Starts execution at the given node.
    ///
    /// # Errors
    /// Returns [`DialogueError::UnknownNode`] if the title does not exist in the program.
    ///
    /// # Panics
    /// Panics only if the internal `RwLock` is poisoned, which cannot happen in normal use.
    pub fn start(&mut self, node: &str) -> Result<()> {
        if !self.program.node_exists(node) {
            return Err(DialogueError::UnknownNode(node.to_owned()));
        }
        let body = self.pick_node_body(node)?;
        self.stack.clear();
        self.stack.push(Frame::new(node.to_owned(), body));
        self.state = State::Running;
        *self
            .visits
            .write()
            .unwrap()
            .entry(node.to_owned())
            .or_insert(0) += 1;
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
            State::AwaitingOption => Err(DialogueError::Runtime(
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
    /// Returns [`DialogueError::Runtime`] if called when not awaiting an option or index is out of
    /// range.
    pub fn select_option(&mut self, index: usize) -> Result<()> {
        if self.state != State::AwaitingOption {
            return Err(DialogueError::Runtime(
                "select_option() called when not awaiting an option".into(),
            ));
        }
        let body = self.option_bodies.get(index).cloned().ok_or_else(|| {
            DialogueError::Runtime(format!(
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
    pub(super) fn push_inline_frame(&mut self, body: Vec<Stmt>) {
        if body.is_empty() {
            return;
        }
        let title = self
            .stack
            .last()
            .map(|f| f.node.clone())
            .unwrap_or_default();
        self.stack.push(Frame::new(title, body));
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

    // ── save / load ───────────────────────────────────────────────────────────

    /// Captures the current session state into a serialisable `RunnerSnapshot`.
    ///
    /// The snapshot records the active node title, visit counts, and the set of
    /// exhausted `<<once>>` blocks.  Variable storage is **not** included; serialise
    /// it via [`Runner::storage`] alongside the snapshot.
    ///
    /// Restoring with [`Runner::restore`] will restart execution from the beginning
    /// of the snapshotted node.
    ///
    /// Only available with the `serde` feature.
    ///
    /// # Panics
    ///
    /// Panics if the internal visits lock is poisoned (only possible if a previous
    /// thread panicked while holding it, which is not expected in normal use).
    #[cfg(feature = "serde")]
    #[must_use]
    pub fn snapshot(&self) -> crate::runtime::RunnerSnapshot {
        crate::runtime::RunnerSnapshot {
            current_node: self.stack.last().map(|f| f.node.clone()),
            visits: self.visits.read().unwrap().clone(),
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
    ///
    /// # Panics
    ///
    /// Panics if the internal visits lock is poisoned (not expected in normal use).
    ///
    /// Only available with the `serde` feature.
    #[cfg(feature = "serde")]
    pub fn restore(&mut self, snapshot: crate::runtime::RunnerSnapshot) -> Result<()> {
        *self.visits.write().unwrap() = snapshot.visits;
        self.once_seen = snapshot.once_seen;
        self.stack.clear();
        self.pending.clear();
        self.option_bodies.clear();
        self.state = State::Idle;
        if let Some(node) = snapshot.current_node {
            self.start(&node)?;
        }
        Ok(())
    }
}
