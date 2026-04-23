//! Rendering: pure functions from [`AppState`] to a ratatui frame.
//!
//! Every test uses `ratatui::backend::TestBackend`, so nothing under this
//! module touches the real terminal.

mod dialogue;
mod footer;
mod transcript;

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};

use crate::app::AppState;

/// Draws the current [`AppState`] into `frame`.
///
/// The layout is a horizontal split:
///
/// ```text
/// ┌ dialogue ──────────┐┌ transcript ────────┐
/// │ …                  ││ …                  │
/// └────────────────────┘└────────────────────┘
///   footer keybinds
/// ```
pub fn render(state: &AppState, frame: &mut Frame<'_>) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(frame.area());

    let panes = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(outer[0]);

    dialogue::render(state, frame, panes[0]);
    transcript::render(state, frame, panes[1]);
    frame.render_widget(footer::footer(state), outer[1]);
}
