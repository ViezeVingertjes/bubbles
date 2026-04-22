//! [`DialogueEvent`] and [`DialogueOption`] — the output types of the runner.

/// An option presented to the player.
#[derive(Debug, Clone, PartialEq)]
pub struct DialogueOption {
    /// Display text of the option.
    pub text: String,
    /// Whether this option is currently available (guards that evaluate to false make it unavailable).
    pub available: bool,
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
