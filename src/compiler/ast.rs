//! AST node types produced by the parser — data only, no logic.

use indexmap::IndexMap;

/// A complete parsed node from a `.bub` script.
#[derive(Debug, Clone)]
pub struct Node {
    /// The node title (must be unique within the program, or share a `when:` clause for groups).
    pub title: String,
    /// Tags declared in the `tags:` header.
    pub tags: Vec<String>,
    /// All other header key-value pairs, preserved verbatim.
    pub headers: IndexMap<String, String>,
    /// Optional `when:` condition source for node-group selection.
    pub when_src: Option<String>,
    /// The statements making up the node body.
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
        /// Expression source string.
        expr_src: String,
    },
    /// A conditional block.
    If {
        /// Ordered list of `(condition_src, body)` branches.
        branches: Vec<(String, Vec<Stmt>)>,
        /// Optional else body.
        else_body: Vec<Stmt>,
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
        /// Optional condition source for `<<once if …>>`.
        cond_src: Option<String>,
        /// Body that runs the first time.
        body: Vec<Stmt>,
        /// Body that runs after the first time.
        else_body: Vec<Stmt>,
    },
    /// A `<<declare $var = expr>>` smart-variable declaration.
    Declare {
        /// Variable name.
        name: String,
        /// Computed expression source.
        expr_src: String,
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
    /// Optional condition source (for `-> text <<if …>>`).
    pub cond_src: Option<String>,
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
    /// Optional condition source.
    pub cond_src: Option<String>,
    /// Whether this variant is a once-variant.
    pub once: bool,
    /// Trailing tags.
    pub tags: Vec<String>,
}
