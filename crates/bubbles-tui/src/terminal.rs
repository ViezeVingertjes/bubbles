//! Terminal lifecycle: the only place allowed to touch raw mode, the
//! alternate screen, stdin/stdout.
//!
//! Everything else in the crate renders into a ratatui `Frame` (real or
//! `TestBackend`) and translates `crossterm` key events into
//! [`crate::Intent`]s.

use std::io::{self, Stdout, Write as _};
use std::time::Duration;

use crossterm::ExecutableCommand as _;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use crate::Intent;

/// Polling interval for keyboard input.  Short enough to stay responsive,
/// long enough to keep the CPU idle in the steady state.
const POLL_INTERVAL: Duration = Duration::from_millis(100);

/// A configured terminal ready for `Terminal::draw` calls.
pub type Tui = Terminal<CrosstermBackend<Stdout>>;

/// Puts the terminal into raw mode + alternate screen and returns a ratatui
/// terminal handle.  Pair with [`restore`] in a `Drop`-safe way.
///
/// # Errors
/// Forwards any `crossterm` or `ratatui` initialisation failure.
pub fn init() -> io::Result<Tui> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    stdout.flush()?;
    Terminal::new(CrosstermBackend::new(stdout))
}

/// Restores the terminal to its original state.  Called after the event
/// loop exits (normally or via panic hook).
///
/// # Errors
/// Forwards any `crossterm` failure while leaving the alt screen or
/// disabling raw mode.
pub fn restore() -> io::Result<()> {
    let mut stdout = io::stdout();
    stdout.execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

/// Polls `crossterm` for up to [`POLL_INTERVAL`] and maps any key press
/// into an [`Intent`].
///
/// Returns `Ok(None)` when the poll timed out with no actionable input so
/// the event loop can do periodic work (e.g. future hot-reload checks).
///
/// # Errors
/// Any `crossterm` read error is forwarded to the caller.
pub fn next_intent() -> io::Result<Option<Intent>> {
    if !event::poll(POLL_INTERVAL)? {
        return Ok(None);
    }
    match event::read()? {
        Event::Key(key) if key.kind == KeyEventKind::Press => Ok(key_to_intent(key)),
        _ => Ok(None),
    }
}

const fn key_to_intent(key: KeyEvent) -> Option<Intent> {
    match key.code {
        KeyCode::Enter | KeyCode::Char(' ') => Some(Intent::Advance),
        KeyCode::Esc | KeyCode::Char('q' | 'Q') => Some(Intent::Quit),
        KeyCode::Tab => Some(Intent::ToggleFocus),
        KeyCode::Down | KeyCode::Char('j' | 'J') => Some(Intent::FocusNext),
        KeyCode::Up | KeyCode::Char('k' | 'K') => Some(Intent::FocusPrev),
        KeyCode::PageUp => Some(Intent::ScrollUp),
        KeyCode::PageDown => Some(Intent::ScrollDown),
        KeyCode::Char(c @ '1'..='9') => Some(Intent::SelectOption((c as usize) - ('1' as usize))),
        _ => None,
    }
}
