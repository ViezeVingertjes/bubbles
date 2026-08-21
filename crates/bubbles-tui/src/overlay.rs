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
            DialogueError::UnknownNode(_) => Self::plain("Unknown node", error),
            DialogueError::DuplicateNode(_) => Self::plain("Duplicate node", error),
            DialogueError::Validation(msg) => Self::with_message("Validation error", msg.clone()),
            DialogueError::UndefinedVariable(name) => Self::with_message(
                "Undefined variable",
                format!(
                    "undefined variable '{name}' - add `<<declare {name} = ...>>` or seed it in storage before playback"
                ),
            ),
            DialogueError::Function { .. } => Self::plain("Function error", error),
            DialogueError::ProtocolViolation(msg) => {
                Self::with_message("Protocol violation", msg.clone())
            }
            // Worded for writers: point at the expression rather than the operator.
            DialogueError::TypeMismatch {
                expected,
                got,
                context,
            } => Self::with_message(
                "Type mismatch",
                format!("in expression {context}: expected {expected}, got {got}"),
            ),
            // Forward-compatible fallback for future variants (#[non_exhaustive]).
            _ => Self::plain("Error", error),
        }
    }

    /// Overlay with `title` and the error's own `Display` text, no location.
    fn plain(title: &str, error: &DialogueError) -> Self {
        Self::with_message(title, error.to_string())
    }

    /// Overlay with `title` and `message`, no location or excerpt.
    fn with_message(title: &str, message: String) -> Self {
        Self {
            title: title.to_owned(),
            message,
            location: None,
            excerpt: None,
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
