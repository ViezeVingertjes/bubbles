//! Rendering: pure functions from [`AppState`] to a ratatui frame.
//!
//! Every test uses `ratatui::backend::TestBackend`, so nothing under this
//! module touches the real terminal.

mod dialogue;
mod error_overlay;
mod footer;
mod transcript;

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};

use crate::app::AppState;

/// Draws the current [`AppState`] into `frame`.
///
/// Layout:
///
/// ```text
/// ┌ transcript ─────────────────────────────────┐
/// │ [→ Node]                                     │
/// │ NPC: Hello!                                  │
/// └─────────────────────────────────────────────┘
/// ┌ options (only when active) ─────────────────┐
/// │ > 1. Reply A    2. Reply B                   │
/// └─────────────────────────────────────────────┘
///   footer keybinds (1 line)
/// ```
pub fn render(state: &AppState, frame: &mut Frame<'_>) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(frame.area());

    let content = outer[0];

    if state.options().is_empty() {
        transcript::render(state, frame, content);
    } else {
        let options_height = dialogue::options_height(state);
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(options_height)])
            .split(content);
        transcript::render(state, frame, chunks[0]);
        dialogue::render_options(state, frame, chunks[1]);
    }

    frame.render_widget(footer::footer(state), outer[1]);

    if let Some(overlay) = state.error_overlay() {
        error_overlay::render(overlay, frame);
    }
}
