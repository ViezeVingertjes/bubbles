//! Error overlay: a view-model for compile- and runtime-error popups.

use bubbles::DialogueError;

/// A file + line location associated with an error, when the underlying
/// [`DialogueError`] variant carries one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorLocation {
    /// Source file name as reported by the compiler.
    pub file: String,
    /// 1-based line number.
    pub line: usize,
}

/// View-model for the error overlay popup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorOverlay {
    /// Short category string drawn in the overlay title
    /// (e.g. `"Parse error"`, `"Runtime error"`).
    pub title: String,
    /// Human-readable error message.
    pub message: String,
    /// Optional source location (`file:line`).
    pub location: Option<ErrorLocation>,
    /// Optional excerpt of the offending source line, for parse errors.
    pub excerpt: Option<String>,
}

impl ErrorOverlay {
    /// Builds an overlay from a [`DialogueError`].  When the error is a
    /// [`DialogueError::Parse`] and `source` is provided, the offending line
    /// is attached as an excerpt.
    #[must_use]
    pub fn from_error(error: &DialogueError, source: Option<&str>) -> Self {
        match error {
            DialogueError::Parse {
                file,
                line,
                message,
            } => Self {
                title: "Parse error".to_owned(),
                message: message.clone(),
                location: Some(ErrorLocation {
                    file: file.clone(),
                    line: *line,
                }),
                excerpt: source.and_then(|s| excerpt_at(s, *line)),
            },
            DialogueError::UnknownNode(name) => Self {
                title: "Unknown node".to_owned(),
                message: format!("unknown node '{name}'"),
                location: None,
                excerpt: None,
            },
            DialogueError::DuplicateNode(name) => Self {
                title: "Duplicate node".to_owned(),
                message: format!("duplicate node title '{name}'"),
                location: None,
                excerpt: None,
            },
            DialogueError::Validation(msg) => Self {
                title: "Validation error".to_owned(),
                message: msg.clone(),
                location: None,
                excerpt: None,
            },
            DialogueError::UndefinedVariable(name) => Self {
                title: "Undefined variable".to_owned(),
                message: format!("undefined variable '{name}'"),
                location: None,
                excerpt: None,
            },
            DialogueError::Function { name, message } => Self {
                title: "Function error".to_owned(),
                message: format!("function '{name}': {message}"),
                location: None,
                excerpt: None,
            },
            DialogueError::ProtocolViolation(msg) => Self {
                title: "Protocol violation".to_owned(),
                message: msg.clone(),
                location: None,
                excerpt: None,
            },
            DialogueError::TypeMismatch {
                expected,
                got,
                context,
            } => Self {
                title: "Type mismatch".to_owned(),
                message: format!("in {context}: expected {expected}, got {got}"),
                location: None,
                excerpt: None,
            },
            // Forward-compatible fallback for future variants (#[non_exhaustive]).
            _ => Self {
                title: "Error".to_owned(),
                message: error.to_string(),
                location: None,
                excerpt: None,
            },
        }
    }

    /// `"<file>:<line>"` when a location is present.
    #[must_use]
    pub fn location_string(&self) -> Option<String> {
        self.location
            .as_ref()
            .map(|loc| format!("{}:{}", loc.file, loc.line))
    }
}

/// Returns the 1-based line `line` from `source`, trimmed of trailing
/// whitespace.  Returns `None` when the line is missing or empty.
fn excerpt_at(source: &str, line: usize) -> Option<String> {
    let idx = line.checked_sub(1)?;
    let snippet = source.lines().nth(idx)?.trim_end();
    if snippet.is_empty() {
        None
    } else {
        Some(snippet.to_owned())
    }
}
