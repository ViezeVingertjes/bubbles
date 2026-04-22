//! Shared error and result types for the crate.

use thiserror::Error;

/// A byte-offset span in a source string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// Start offset (inclusive).
    pub start: usize,
    /// End offset (exclusive).
    pub end: usize,
}

/// Alias for `Result<T, DialogueError>`.
pub type Result<T> = core::result::Result<T, DialogueError>;

/// All errors that can be produced by compilation or runtime execution.
#[non_exhaustive]
#[derive(Debug, Error, Clone)]
pub enum DialogueError {
    /// A parse-time error, optionally localised to a source span.
    #[error("parse error at {file}:{line}: {message}")]
    Parse {
        /// Source file name or `"<source>"`.
        file: String,
        /// 1-based line number.
        line: usize,
        /// Human-readable description.
        message: String,
    },
    /// A reference to an unknown node.
    #[error("unknown node '{0}'")]
    UnknownNode(String),
    /// A duplicate node title was found across merged sources.
    #[error("duplicate node title '{0}'")]
    DuplicateNode(String),
    /// A validation failure detected after all sources are merged.
    #[error("validation error: {0}")]
    Validation(String),
    /// A runtime execution error.
    #[error("runtime error: {0}")]
    Runtime(String),
    /// A type mismatch in an expression.
    #[error("type error: {0}")]
    Type(String),
    /// An unknown variable was referenced.
    #[error("undefined variable '{0}'")]
    UndefinedVariable(String),
    /// A function call failed.
    #[error("function '{name}' error: {message}")]
    Function {
        /// Function name.
        name: String,
        /// Description.
        message: String,
    },
}
