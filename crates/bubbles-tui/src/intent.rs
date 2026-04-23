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
    /// Advance to the next line, option set, command, or node boundary.
    Advance,
    /// Quit the application (the binary exits; no-op in tests).
    Quit,
}
