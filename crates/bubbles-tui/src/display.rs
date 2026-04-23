//! View-model types consumed by both the app and the renderer.

use bubbles::DialogueOption;

/// A line of dialogue ready to be drawn on screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayedLine {
    /// Optional speaker prefix (the `Speaker:` part of a line).
    pub speaker: Option<String>,
    /// Fully interpolated line text (all `{expr}` fragments already resolved).
    pub text: String,
}

/// An option ready to be drawn on screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayedOption {
    /// Fully interpolated option text.
    pub text: String,
    /// Whether the option's guard currently passes; false options are
    /// displayed but not selectable.
    pub available: bool,
}

impl From<DialogueOption> for DisplayedOption {
    fn from(opt: DialogueOption) -> Self {
        Self {
            text: opt.text,
            available: opt.available,
        }
    }
}

/// Which pane currently owns the keyboard focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusPanel {
    /// The dialogue / options pane.
    Dialogue,
    /// The transcript pane.
    Transcript,
}

/// Direction for the option-focus / transcript-scroll cursor shifts.
#[derive(Debug, Clone, Copy)]
pub enum FocusShift {
    Next,
    Prev,
}
