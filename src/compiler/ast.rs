//! AST types: node/statement types and expression tree — data only, no logic.

use std::sync::Arc;

use indexmap::IndexMap;

/// A complete parsed node from a `.bub` script.
#[derive(Debug, Clone)]
pub struct Node {
    /// The node title (must be unique within the program, or share a `when:` clause for groups).
    pub title: String,
    /// Tags declared in the `tags:` header.
    pub tags: Vec<String>,
    /// All other header key-value pairs, preserved verbatim (minus `title` / `tags` / `when`).
    pub headers: IndexMap<String, String>,
    /// Optional `when:` condition for node-group selection (parsed at compile time).
    pub when: Option<Arc<Expr>>,
    /// The statements making up the node body (shared [`Arc`] so pickers can clone cheaply).
    pub body: Arc<Vec<Stmt>>,
}

/// One branch of an `<<if>>` chain: condition AST + body statements.
#[derive(Debug, Clone)]
pub struct IfBranch {
    /// Parsed condition (same as would be produced from the source string at compile time).
    pub cond: Arc<Expr>,
    /// Statements when this branch is taken.
    pub body: Vec<Stmt>,
}

/// A statement in a node body.
#[derive(Debug, Clone)]
pub enum Stmt {
    /// A line of dialogue.
    Line {
        /// Optional speaker prefix (`Alice:`).
        speaker: Option<String>,
        /// Raw text, may contain `{expr}` fragments.
        text: String,
        /// Trailing `#tag` metadata.
        tags: Vec<String>,
    },
    /// A `<<set $var = expr>>` assignment.
    Set {
        /// Variable name including the `$` sigil.
        name: String,
        /// Parsed right-hand expression (compile-time).
        expr: Arc<Expr>,
    },
    /// A conditional block.
    If {
        /// `if` / `elseif` branches in order.
        branches: Vec<IfBranch>,
    /// `else` body.
    else_body: Vec<Self>,
    },
    /// A `<<jump NodeTitle>>` statement.
    Jump(String),
    /// A `<<detour NodeTitle>>` statement.
    Detour(String),
    /// A `<<return>>` statement.
    Return,
    /// A generic host command `<<name args…>>`.
    Command {
        /// Command name.
        name: String,
        /// Raw argument string (may contain `{expr}` fragments).
        args_src: String,
        /// Trailing `#tag` metadata.
        tags: Vec<String>,
    },
    /// A `<<once>>` … `<<endonce>>` block.
    Once {
        /// Stable block id (line number–based), used to track seen state.
        block_id: String,
        /// Optional condition for `<<once if …>>` (parsed at compile time).
        cond: Option<Arc<Expr>>,
        /// Body that runs the first time.
        body: Vec<Self>,
        /// Body that runs after the first time.
        else_body: Vec<Self>,
    },
    /// A `<<declare $var = expr>>` smart-variable declaration.
    Declare {
        /// Variable name.
        name: String,
        /// Parsed default expression.
        expr: Arc<Expr>,
        /// Expression source as written (for [`crate::VariableDecl`] / tooling).
        default_src: String,
    },
    /// A shortcut-option block.
    Options(Vec<OptionItem>),
    /// A line-group block (alternatives selected by saliency).
    LineGroup(Vec<LineVariant>),
}

/// A single shortcut option.
#[derive(Debug, Clone)]
pub struct OptionItem {
    /// Stable id for once/saliency tracking.
    pub id: String,
    /// Display text, may contain `{expr}`.
    pub text: String,
    /// Optional guard (`-> text <<if cond>>`); `None` = always available if not `once` exhausted.
    pub cond: Option<Arc<Expr>>,
    /// Whether this option is a once-option.
    pub once: bool,
    /// Trailing tags.
    pub tags: Vec<String>,
    /// Indented body statements executed after selection.
    pub body: Vec<Stmt>,
}

/// A line variant inside a `=>` line-group.
#[derive(Debug, Clone)]
pub struct LineVariant {
    /// Stable id.
    pub id: String,
    /// Optional speaker.
    pub speaker: Option<String>,
    /// Text.
    pub text: String,
    /// Optional guard; `None` = always considered with saliency.
    pub cond: Option<Arc<Expr>>,
    /// Whether this variant is a once-variant.
    pub once: bool,
    /// Trailing tags.
    pub tags: Vec<String>,
}

// ── expression AST ────────────────────────────────────────────────────────────

/// A node in the expression AST.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Numeric literal.
    Number(f64),
    /// String literal.
    Text(String),
    /// Boolean literal.
    Bool(bool),
    /// Variable read, e.g. `$gold`.
    Var(String),
    /// Function call, e.g. `random(1, 6)`.
    Call {
        /// Function name.
        name: String,
        /// Argument expressions.
        args: Vec<Self>,
    },
    /// Unary operator.
    Unary {
        /// Operator.
        op: UnOp,
        /// Operand.
        expr: Box<Self>,
    },
    /// Binary operator.
    Binary {
        /// Left operand.
        left: Box<Self>,
        /// Operator.
        op: BinOp,
        /// Right operand.
        right: Box<Self>,
    },
}

/// Binary operator kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    /// `+`
    Add,
    /// `-`
    Sub,
    /// `*`
    Mul,
    /// `/`
    Div,
    /// `%`
    Rem,
    /// `==`
    Eq,
    /// `!=`
    Neq,
    /// `<`
    Lt,
    /// `<=`
    Lte,
    /// `>`
    Gt,
    /// `>=`
    Gte,
    /// `&&`
    And,
    /// `||`
    Or,
}

/// Unary operator kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    /// Arithmetic negation `-`.
    Neg,
    /// Logical negation `!`.
    Not,
}
