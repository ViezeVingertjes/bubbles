//! [`DialogueEvent`] and [`DialogueOption`] - the output types of the runner.

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
    /// Display text of the option.
    pub text: String,
    /// Whether this option is currently available (guards that evaluate to false make it unavailable).
    pub available: bool,
    /// If the option text was tagged with `#line:<id>`, the stable id (no `line:` prefix).
    pub line_id: Option<String>,
    /// Trailing `#tag` metadata.
    pub tags: Vec<String>,
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
        /// Text with all `{expr}` fragments already substituted.
        text: String,
        /// If the line was tagged with `#line:<id>`, the stable id (no `line:` prefix).
        line_id: Option<String>,
        /// Trailing `#tag` metadata.
        tags: Vec<String>,
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
