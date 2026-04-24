//! [`DialogueEvent`], [`DialogueOption`], and [`MarkupSpan`] - the output types of the runner.

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
mod tests {
    use super::line_id_from_tags;

    #[test]
    fn line_id_from_tags_first_line_prefix() {
        assert_eq!(
            line_id_from_tags(&["foo".into(), "line:abc".into(), "line:ignored".into()]),
            Some("abc".into())
        );
    }

    #[test]
    fn line_id_from_tags_none_without_prefix() {
        assert_eq!(line_id_from_tags(&["foo".into(), "bar".into()]), None);
    }

    #[test]
    fn line_id_from_tags_empty_after_prefix_is_none() {
        assert_eq!(line_id_from_tags(&["line:".into()]), None);
    }
}
