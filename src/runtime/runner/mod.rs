//! [`Runner`] — the public entry point for executing a compiled [`Program`].

pub(super) mod execute;

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, RwLock};

use crate::compiler::Program;
use crate::compiler::ast::Stmt;
use crate::error::{DialogueError, Result};
use crate::library::FunctionLibrary;
use crate::runtime::eval::eval;
use crate::runtime::event::DialogueEvent;
use crate::runtime::interpolate::interpolate;
use crate::runtime::provider::{LineProvider, PassthroughProvider};
use crate::saliency::{Candidate, FirstAvailable, SaliencyStrategy};
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
        if !body.is_empty() {
            let title = self
                .stack
                .last()
                .map(|f| f.node.clone())
                .unwrap_or_default();
            self.stack.push(Frame::new(title, body));
        }
        Ok(())
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

    // ── internals ─────────────────────────────────────────────────────────────

    pub(super) fn node_body(&self, title: &str) -> Result<Vec<Stmt>> {
        self.program
            .node_group(title)
            .and_then(|g| g.first())
            .map(|n| n.body.clone())
            .ok_or_else(|| DialogueError::UnknownNode(title.to_owned()))
    }

    pub(super) fn pick_node_body(&self, title: &str) -> Result<Vec<Stmt>> {
        let group = self
            .program
            .node_group(title)
            .ok_or_else(|| DialogueError::UnknownNode(title.to_owned()))?;

        let has_when = group.iter().any(|n| n.when_src.is_some());
        if !has_when {
            return Ok(group[0].body.clone());
        }

        let candidates: Vec<Candidate<'_>> = group
            .iter()
            .map(|n| {
                let available = n.when_src.as_deref().is_none_or(|src| {
                    self.eval_expr_src(src)
                        .map(|v| v.is_truthy())
                        .unwrap_or(false)
                });
                Candidate {
                    id: &n.title,
                    available,
                }
            })
            .collect();

        let idx = self.saliency.select(&candidates).ok_or_else(|| {
            DialogueError::Runtime(format!("node group '{title}' has no available candidate"))
        })?;
        Ok(group[idx].body.clone())
    }

    pub(super) fn eval_expr_src(&self, src: &str) -> Result<Value> {
        let expr = crate::compiler::expr::parse_expr(src)?;
        eval(&expr, &self.storage, &|name, args| {
            self.library.call(name, args)
        })
    }

    pub(super) fn interpolate_text(&self, text: &str) -> Result<String> {
        interpolate(text, &self.storage, &|name, args| {
            self.library.call(name, args)
        })
    }

    pub(super) fn parse_command_args(&self, args_src: &str) -> Result<Vec<String>> {
        if args_src.trim().is_empty() {
            return Ok(Vec::new());
        }
        let interpolated = self.interpolate_text(args_src)?;
        Ok(interpolated.split_whitespace().map(str::to_owned).collect())
    }
}
