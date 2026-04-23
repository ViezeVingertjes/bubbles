//! Shared error and result types for the crate.

use thiserror::Error;

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
    /// A general runtime execution error.
    ///
    /// # Deprecation
    ///
    /// Prefer [`DialogueError::ProtocolViolation`] for API-misuse errors and
    /// [`DialogueError::TypeMismatch`] for type errors.  This variant is kept
    /// for backwards compatibility and for host-function error messages
    /// (see [`DialogueError::Function`]).
    #[deprecated(
        since = "0.5.0",
        note = "use ProtocolViolation or TypeMismatch for structured errors; \
                use Function { name, message } for host-function errors"
    )]
    #[error("runtime error: {0}")]
    Runtime(String),
    /// The [`Runner`](crate::runtime::Runner) API was called in the wrong
    /// order (e.g. [`next_event`](crate::runtime::Runner::next_event) while
    /// awaiting an option, or
    /// [`select_option`](crate::runtime::Runner::select_option) when not
    /// awaiting).
    ///
    /// Embedders can match on this variant to detect integration bugs
    /// without parsing the error message string.
    #[error("protocol violation: {0}")]
    ProtocolViolation(String),
    /// A type mismatch in an expression (e.g. adding a number to a string).
    ///
    /// `expected` and `got` are human-readable descriptions of the involved
    /// types; `context` is the operation that triggered the mismatch.
    #[error("type mismatch in {context}: expected {expected}, got {got}")]
    TypeMismatch {
        /// The type the operation expected.
        expected: String,
        /// The type actually encountered.
        got: String,
        /// The operation that produced the mismatch (e.g. `"+"`).
        context: String,
    },
    /// A general type error message.
    ///
    /// # Deprecation
    ///
    /// Prefer [`DialogueError::TypeMismatch`], which carries structured
    /// `expected`, `got`, and `context` fields instead of an opaque string.
    #[deprecated(
        since = "0.5.0",
        note = "use TypeMismatch { expected, got, context } instead"
    )]
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
