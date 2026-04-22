//! [`Runner`] — the public entry point for executing a compiled [`Program`].

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, RwLock};

use crate::compiler::ast::Stmt;
use crate::compiler::expr::parse_expr;
use crate::compiler::Program;
use crate::error::{DialogueError, Result};
use crate::library::FunctionLibrary;
use crate::runtime::eval::eval;
use crate::runtime::event::DialogueEvent;
use crate::runtime::interpolate::interpolate;
use crate::saliency::{Candidate, FirstAvailable, SaliencyStrategy};
use crate::value::{Value, VariableStorage};

/// Execution state of the runner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Idle,
    Running,
    AwaitingOption,
    Done,
}

/// A frame on the call stack.
#[derive(Debug, Clone)]
struct Frame {
    /// Node title.
    node: String,
    /// Pending statements yet to be executed in this frame.
    stmts: VecDeque<Stmt>,
}

impl Frame {
    fn new(node: String, body: Vec<Stmt>) -> Self {
        Self { node, stmts: VecDeque::from(body) }
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
    program: Program,
    storage: S,
    state: State,
    /// The call stack. The top frame is the current one.
    stack: Vec<Frame>,
    /// Pending events ready to be returned.
    pending: VecDeque<DialogueEvent>,
    /// Bodies for each option while awaiting selection.
    option_bodies: OptionBodies,
    /// Function registry; defaults to built-ins.
    library: FunctionLibrary,
    /// Visit counts keyed by node title (shared with visit-tracking closures).
    visits: Arc<RwLock<HashMap<String, usize>>>,
    /// Seen `<<once>>` block IDs.
    once_seen: HashSet<String>,
    /// Strategy used for line-group and node-group selection.
    saliency: Box<dyn SaliencyStrategy>,
}

impl<S: VariableStorage> Runner<S> {
    /// Creates a new runner for the given program and variable storage.
    #[must_use]
    pub fn new(program: Program, storage: S) -> Self {
        let visits: Arc<RwLock<HashMap<String, usize>>> = Arc::default();
        let mut library = FunctionLibrary::new();

        // Register visit-tracking functions; they close over `visits`.
        let v1 = Arc::clone(&visits);
        library.register("visited", move |args| {
            let title = match args.as_slice() {
                [Value::Text(t)] => t.clone(),
                _ => return Err(DialogueError::Function {
                    name: "visited".into(),
                    message: "expected one string argument".into(),
                }),
            };
            Ok(Value::Bool(*v1.read().unwrap().get(&title).unwrap_or(&0) > 0))
        });
        let v2 = Arc::clone(&visits);
        library.register("visited_count", move |args| {
            let title = match args.as_slice() {
                [Value::Text(t)] => t.clone(),
                _ => return Err(DialogueError::Function {
                    name: "visited_count".into(),
                    message: "expected one string argument".into(),
                }),
            };
            let count = *v2.read().unwrap().get(&title).unwrap_or(&0);
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
        let body = self.node_body(node)?;
        self.stack.clear();
        self.stack.push(Frame::new(node.to_owned(), body));
        self.state = State::Running;
        *self.visits.write().unwrap().entry(node.to_owned()).or_insert(0) += 1;
        self.pending.push_back(DialogueEvent::NodeStarted(node.to_owned()));
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
            State::Running => {
                loop {
                    if let Some(ev) = self.pending.pop_front() {
                        return Ok(Some(ev));
                    }
                    if self.state != State::Running {
                        return Ok(None);
                    }
                    match self.step()? {
                        Some(ev) => return Ok(Some(ev)),
                        None => {}
                    }
                }
            }
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
            let title = self.stack.last().map(|f| f.node.clone()).unwrap_or_default();
            self.stack.push(Frame::new(title, body));
        }
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

    /// Returns a mutable reference to the function library (for registering host functions).
    pub fn library_mut(&mut self) -> &mut FunctionLibrary {
        &mut self.library
    }

    /// Replaces the saliency strategy used for line and node group selection.
    pub fn set_saliency(&mut self, strategy: impl SaliencyStrategy) {
        self.saliency = Box::new(strategy);
    }

    // ── internals ─────────────────────────────────────────────────────────────

    fn node_body(&self, title: &str) -> Result<Vec<Stmt>> {
        self.program
            .node_group(title)
            .and_then(|g| g.first())
            .map(|n| n.body.clone())
            .ok_or_else(|| DialogueError::UnknownNode(title.to_owned()))
    }

    fn eval_expr_src(&self, src: &str) -> Result<Value> {
        let expr = parse_expr(src)?;
        eval(&expr, &self.storage, &|name, args| self.library.call(name, args))
    }

    fn interpolate_text(&self, text: &str) -> Result<String> {
        interpolate(text, &self.storage, &|name, args| self.library.call(name, args))
    }

    fn parse_command_args(&self, args_src: &str) -> Result<Vec<String>> {
        if args_src.trim().is_empty() {
            return Ok(Vec::new());
        }
        let interpolated = self.interpolate_text(args_src)?;
        Ok(interpolated
            .split_whitespace()
            .map(str::to_owned)
            .collect())
    }

    fn step(&mut self) -> Result<Option<DialogueEvent>> {
        // pop current frame's next statement
        let stmt = loop {
            let frame = match self.stack.last_mut() {
                Some(f) => f,
                None => {
                    self.state = State::Done;
                    return Ok(Some(DialogueEvent::DialogueComplete));
                }
            };
            if let Some(s) = frame.stmts.pop_front() {
                break s;
            }
            // frame exhausted — pop it
            let finished_node = frame.node.clone();
            self.stack.pop();
            if self.stack.is_empty() {
                // top-level node complete
                self.state = State::Done;
                self.pending.push_back(DialogueEvent::DialogueComplete);
                return Ok(Some(DialogueEvent::NodeComplete(finished_node)));
            }
        };

        self.execute_stmt(stmt)
    }

    fn execute_stmt(&mut self, stmt: Stmt) -> Result<Option<DialogueEvent>> {
        match stmt {
            Stmt::Line { speaker, text, tags } => {
                let text = self.interpolate_text(&text)?;
                Ok(Some(DialogueEvent::Line { speaker, text, tags }))
            }
            Stmt::Set { name, expr_src } => {
                let value = self.eval_expr_src(&expr_src)?;
                self.storage.set(&name, value);
                Ok(None)
            }
            Stmt::Declare { name, expr_src } => {
                // Only initialise if not already present (respects saved-game state).
                if self.storage.get(&name).is_none() {
                    let value = self.eval_expr_src(&expr_src)?;
                    self.storage.set(&name, value);
                }
                Ok(None)
            }
            Stmt::LineGroup(variants) => {
                let candidates: Vec<Candidate<'_>> = variants
                    .iter()
                    .map(|v| {
                        let available = match &v.cond_src {
                            Some(src) => self.eval_expr_src(src).map(|v| v.is_truthy()).unwrap_or(false),
                            None => true,
                        } && !(v.once && self.once_seen.contains(&v.id));
                        Candidate { id: &v.id, available }
                    })
                    .collect();

                if let Some(idx) = self.saliency.select(&candidates) {
                    let chosen = &variants[idx];
                    if chosen.once {
                        self.once_seen.insert(chosen.id.clone());
                    }
                    let text = self.interpolate_text(&chosen.text)?;
                    let line = DialogueEvent::Line {
                        speaker: chosen.speaker.clone(),
                        text,
                        tags: chosen.tags.clone(),
                    };
                    return Ok(Some(line));
                }
                Ok(None)
            }
            Stmt::Options(items) => {
                let mut options = Vec::with_capacity(items.len());
                let mut bodies = Vec::with_capacity(items.len());
                for item in items {
                    let available = match &item.cond_src {
                        Some(src) => self.eval_expr_src(src)?.is_truthy(),
                        None => true,
                    };
                    let text = self.interpolate_text(&item.text)?;
                    options.push(crate::runtime::event::DialogueOption {
                        text,
                        available,
                        tags: item.tags.clone(),
                    });
                    bodies.push(item.body);
                }
                self.option_bodies = bodies;
                self.state = State::AwaitingOption;
                Ok(Some(DialogueEvent::Options(options)))
            }
            Stmt::If { branches, else_body } => {
                let mut chosen = None;
                for (cond_src, body) in branches {
                    let v = self.eval_expr_src(&cond_src)?;
                    if v.is_truthy() {
                        chosen = Some(body);
                        break;
                    }
                }
                let body = chosen.unwrap_or(else_body);
                if !body.is_empty() {
                    let title = self
                        .stack
                        .last()
                        .map(|f| f.node.clone())
                        .unwrap_or_default();
                    self.stack.push(Frame::new(title, body));
                }
                Ok(None)
            }
            Stmt::Once { block_id, cond_src, body, else_body } => {
                // Check optional condition
                let cond_ok = match cond_src {
                    Some(src) => self.eval_expr_src(&src)?.is_truthy(),
                    None => true,
                };
                let first_time = !self.once_seen.contains(&block_id);
                let run_body = cond_ok && first_time;
                if run_body {
                    self.once_seen.insert(block_id);
                }
                let chosen = if run_body { body } else { else_body };
                if !chosen.is_empty() {
                    let title = self.stack.last().map(|f| f.node.clone()).unwrap_or_default();
                    self.stack.push(Frame::new(title, chosen));
                }
                Ok(None)
            }
            Stmt::Jump(target) => {
                if !self.program.node_exists(&target) {
                    return Err(DialogueError::UnknownNode(target));
                }
                let body = self.node_body(&target)?;
                self.stack.clear();
                self.stack.push(Frame::new(target.clone(), body));
                *self.visits.write().unwrap().entry(target.clone()).or_insert(0) += 1;
                self.pending.push_front(DialogueEvent::NodeStarted(target));
                Ok(None)
            }
            Stmt::Detour(target) => {
                if !self.program.node_exists(&target) {
                    return Err(DialogueError::UnknownNode(target));
                }
                let body = self.node_body(&target)?;
                *self.visits.write().unwrap().entry(target.clone()).or_insert(0) += 1;
                self.stack.push(Frame::new(target.clone(), body));
                self.pending.push_front(DialogueEvent::NodeStarted(target));
                Ok(None)
            }
            Stmt::Return => {
                self.stack.pop();
                Ok(None)
            }
            Stmt::Command { name, args_src, tags } => {
                let args = self.parse_command_args(&args_src)?;
                Ok(Some(DialogueEvent::Command { name, args, tags }))
            }
            _ => Err(DialogueError::Runtime("unimplemented statement type".into())),
        }
    }
}
