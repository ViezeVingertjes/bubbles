//! AST types: node/statement types and expression tree — data only, no logic.

use std::sync::Arc;

use indexmap::IndexMap;

/// A shared, read-only slice of [`Stmt`]s used for every body throughout the
/// AST (node bodies, `if` branches, `once` bodies, option bodies, line groups).
///
/// Stored behind an [`Arc`] so the runner can push them onto the call stack
/// without cloning the underlying statement list — frame pushes become a
/// reference-count bump regardless of body size.
pub type StmtList = Arc<[Stmt]>;

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
    /// The statements making up the node body.
    pub body: StmtList,
}

/// One branch of an `<<if>>` chain: condition AST + body statements.
#[derive(Debug, Clone)]
pub struct IfBranch {
    /// Parsed condition (same as would be produced from the source string at compile time).
    pub cond: Arc<Expr>,
    /// Statements when this branch is taken.
    pub body: StmtList,
}

/// A statement in a node body.
#[derive(Debug, Clone)]
pub enum Stmt {
    /// A line of dialogue.
    Line {
        /// Optional speaker prefix (`Alice:`).
        speaker: Option<String>,
        /// Pre-parsed text segments (literals and `{expr}` fragments).
        text: Vec<TextSegment>,
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
        else_body: StmtList,
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
        /// Pre-parsed argument segments (literals and `{expr}` fragments).
        args: Vec<TextSegment>,
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
        body: StmtList,
        /// Body that runs after the first time.
        else_body: StmtList,
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
    /// Pre-parsed display text (literals and `{expr}` fragments).
    pub text: Vec<TextSegment>,
    /// Optional guard (`-> text <<if cond>>`); `None` = always available if not `once` exhausted.
    pub cond: Option<Arc<Expr>>,
    /// Whether this option is a once-option.
    pub once: bool,
    /// Trailing tags.
    pub tags: Vec<String>,
    /// Indented body statements executed after selection.
    pub body: StmtList,
}

/// A line variant inside a `=>` line-group.
#[derive(Debug, Clone)]
pub struct LineVariant {
    /// Stable id.
    pub id: String,
    /// Optional speaker.
    pub speaker: Option<String>,
    /// Pre-parsed text segments (literals and `{expr}` fragments).
    pub text: Vec<TextSegment>,
    /// Optional guard; `None` = always considered with saliency.
    pub cond: Option<Arc<Expr>>,
    /// Whether this variant is a once-variant.
    pub once: bool,
    /// Trailing tags.
    pub tags: Vec<String>,
}

// ── interpolated text ─────────────────────────────────────────────────────────

/// One segment of text that may contain `{expr}` fragments.
///
/// Line text, option text, line-variant text, and command argument strings are
/// all stored as `Vec<TextSegment>` so that `{expr}` fragments are parsed once
/// at compile time and evaluated cheaply at runtime.
#[derive(Debug, Clone)]
pub enum TextSegment {
    /// A literal string with no interpolation.
    Literal(String),
    /// An `{expr}` fragment whose source has already been parsed.
    Expr(Arc<Expr>),
}

impl TextSegment {
    /// Convenience: construct a literal segment from any `Into<String>`.
    pub fn literal(s: impl Into<String>) -> Self {
        Self::Literal(s.into())
    }
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
