//! User-facing commands that drive [`crate::AppState`].
//!
//! `Intent` is deliberately decoupled from the terminal: tests construct
//! intents directly, while the binary translates `crossterm` key events into
//! them.  Adding a new verb is just adding a variant and handling it in
//! [`crate::AppState::apply`].

/// A high-level command applied to the app state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Intent {
    /// Advance to the next line.  When an option set is currently showing,
    /// this commits the focused option instead.
    Advance,
    /// Quit the application (the binary exits; no-op in tests).
    Quit,
    /// Move option focus down, wrapping to the top at the end.
    FocusNext,
    /// Move option focus up, wrapping to the bottom at the start.
    FocusPrev,
    /// Commit option `index` directly.  No-op when no options are showing or
    /// the index is out of range / unavailable.
    SelectOption(usize),
}
