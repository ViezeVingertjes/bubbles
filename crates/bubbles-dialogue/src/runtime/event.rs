//! [`DialogueEvent`], [`DialogueOption`], and [`MarkupSpan`] - the output types of the runner.

/// How a [`DialogueEvent::Line`] should be treated for display or logging.
///
/// The runner sets this from trailing `#tag` metadata on the source line:
/// `#debug` yields [`LineMode::Debug`], `#narration` yields [`LineMode::Narration`].
/// If both are present, [`LineMode::Debug`] wins. All other lines use [`LineMode::Normal`].
///
/// Tags that only exist to set the mode remain in [`DialogueEvent::Line::tags`]; your
/// host can ignore them once you branch on [`LineMode`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LineMode {
    /// Ordinary character or narrator line.
    #[default]
    Normal,
    /// System or omniscient narration (subtitle style, VO bus, etc.).
    Narration,
    /// Developer or QA line you may want to hide in release builds.
    Debug,
}

/// Derives [`LineMode`] from trailing `#tag` strings (without the `#` prefix).
#[must_use]
pub fn line_mode_from_tags(tags: &[String]) -> LineMode {
    if tags.iter().any(|t| t == "debug") {
        LineMode::Debug
    } else if tags.iter().any(|t| t == "narration") {
        LineMode::Narration
    } else {
        LineMode::Normal
    }
}

/// Returns the group from a `#group:<name>` tag in `tags`, if any (first match wins).
///
/// Used for UI constraints (radio-button semantics, mutually exclusive option sets).
/// The group tag itself remains in [`DialogueOption::tags`]; your UI can use both
/// the `group` field for constraint logic and `tags` for styling/metadata.
#[must_use]
pub fn option_group_from_tags(tags: &[String]) -> Option<String> {
    tags.iter()
        .find_map(|t| t.strip_prefix("group:"))
        .map(ToOwned::to_owned)
        .filter(|s| !s.is_empty())
}

/// A resolved inline markup span: a named annotation over a byte range in the
/// stripped display text.
///
/// Spans are produced at runtime, after expression substitution, so [`start`]
/// and [`length`] are byte offsets into the final [`DialogueEvent::Line::text`]
/// / [`DialogueOption::text`] string.
///
/// The runtime assigns no meaning to span names or properties. Your game
/// decides what `[wave]`, `[color value=red]`, or `[pause /]` means and how
/// to render it.
///
/// [`start`]: MarkupSpan::start
/// [`length`]: MarkupSpan::length
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkupSpan {
    /// The markup tag name, e.g. `wave` for `[wave]text[/wave]`.
    pub name: String,
    /// Byte offset of the first character of the spanned text in the display string.
    pub start: usize,
    /// Byte length of the spanned text. Zero for self-closing tags.
    pub length: usize,
    /// Zero or more `(key, value)` pairs from the tag, e.g. `[("value", "red")]`
    /// for `[color value=red]`.
    pub properties: Vec<(String, String)>,
}

/// Returns the id from a `#line:<id>` tag in `tags`, if any (first match wins).
///
/// This matches the id passed to [`crate::LineProvider`]. Use it to key voice-over or analytics
/// without re-parsing [`DialogueEvent::Line::tags`] or [`DialogueOption::tags`].
#[must_use]
pub fn line_id_from_tags(tags: &[String]) -> Option<String> {
    tags.iter()
        .find_map(|t| t.strip_prefix("line:"))
        .map(str::to_owned)
        .filter(|s| !s.is_empty())
}

/// An option presented to the player.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DialogueOption {
    /// Display text of the option (markup tags stripped, expressions evaluated).
    pub text: String,
    /// Whether this option is currently available (guards that evaluate to false make it unavailable).
    pub available: bool,
    /// If the option text was tagged with `#line:<id>`, the stable id (no `line:` prefix).
    pub line_id: Option<String>,
    /// Trailing `#tag` metadata.
    pub tags: Vec<String>,
    /// If the option was tagged with `#group:<name>`, the group name for UI constraints (radio buttons, etc.).
    pub group: Option<String>,
    /// Inline markup spans over [`text`](DialogueOption::text), in source order.
    /// Empty when the option text contains no markup tags.
    pub spans: Vec<MarkupSpan>,
}

/// Events emitted by [`crate::Runner`] one at a time via [`crate::Runner::next_event`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum DialogueEvent {
    /// A node has started executing.
    NodeStarted(String),
    /// A line of dialogue ready to display.
    Line {
        /// Optional speaker name.
        speaker: Option<String>,
        /// Display text with all `{expr}` fragments evaluated and markup tags stripped.
        text: String,
        /// If the line was tagged with `#line:<id>`, the stable id (no `line:` prefix).
        line_id: Option<String>,
        /// Trailing `#tag` metadata.
        tags: Vec<String>,
        /// Hint for filtering or routing (from `#narration` / `#debug` when present).
        line_mode: LineMode,
        /// Inline markup spans over [`text`](DialogueEvent::Line::text), in source order.
        /// Empty when the line contains no markup tags.
        spans: Vec<MarkupSpan>,
    },
    /// A set of options for the player to choose from.
    Options(Vec<DialogueOption>),
    /// A host command to execute.
    Command {
        /// Command name.
        name: String,
        /// Arguments with `{expr}` substituted.
        args: Vec<String>,
        /// Trailing tags.
        tags: Vec<String>,
    },
    /// The current node has finished.
    NodeComplete(String),
    /// All dialogue has finished.
    DialogueComplete,
}

#[cfg(test)]
#[path = "event_tests.rs"]
mod tests;
