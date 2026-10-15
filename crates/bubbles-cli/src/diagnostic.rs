//! [`Diagnostic`] - the one place a [`DialogueError`] is turned into words.
//!
//! Both front ends consume it, so they can never drift apart: `bubbles check`
//! prints a diagnostic as a single line, and the player draws the same one in
//! its error overlay.

use bubbles::DialogueError;

/// A file + line location associated with an error, when the underlying
/// [`DialogueError`] variant carries one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticLocation {
    /// Source file name as reported by the compiler.
    pub file: String,
    /// 1-based line number.
    pub line: usize,
}

/// A compile- or runtime-error rendered for a human.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// Short category string (e.g. `"Parse error"`, `"Runtime error"`), drawn
    /// in the overlay title and printed before the message by `check`.
    pub title: String,
    /// Human-readable error message.
    pub message: String,
    /// Optional source location (`file:line`).
    pub location: Option<DiagnosticLocation>,
    /// Optional excerpt of the offending source line, for parse errors.
    pub excerpt: Option<String>,
}

impl Diagnostic {
    /// Builds a diagnostic from a [`DialogueError`].  When the error is a
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
                location: Some(DiagnosticLocation {
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

    /// Diagnostic with `title` and the error's own `Display` text, no location.
    fn plain(title: &str, error: &DialogueError) -> Self {
        Self::with_message(title, error.to_string())
    }

    /// Diagnostic with `title` and `message`, no location or excerpt.
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

    /// The diagnostic as exactly one line of text, for `bubbles check`.
    ///
    /// `"<file>:<line>: <title>: <message>"`, or `"<title>: <message>"` when
    /// the error carries no location.  Embedded newlines are folded to spaces
    /// so a diagnostic always occupies a single line of output.
    #[must_use]
    pub fn one_line(&self) -> String {
        let message = self.message.split('\n').collect::<Vec<_>>().join(" ");
        self.location_string().map_or_else(
            || format!("{}: {message}", self.title),
            |at| format!("{at}: {}: {message}", self.title),
        )
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
