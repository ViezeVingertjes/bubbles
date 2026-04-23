//! One-line footer with the keybind hints that apply to the current state.

use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::app::AppState;
use crate::display::FocusPanel;

/// Returns the footer paragraph for `state`.
pub fn footer(state: &AppState) -> Paragraph<'_> {
    let text = hint_text(state);
    Paragraph::new(Line::from(Span::styled(
        text,
        Style::default().add_modifier(Modifier::DIM),
    )))
}

fn hint_text(state: &AppState) -> &'static str {
    if state.error_overlay().is_some() {
        return "  r: reload    x: dismiss    q/Esc: quit";
    }
    if state.is_done() {
        return "  q/Esc: quit    Tab: focus transcript";
    }
    match state.focus() {
        FocusPanel::Dialogue if state.options().is_empty() => {
            "  Enter: advance    Tab: focus transcript    q/Esc: quit"
        }
        FocusPanel::Dialogue => {
            "  \u{2191}/\u{2193}: focus    Enter: choose    1-9: pick    Tab: transcript    q/Esc: quit"
        }
        FocusPanel::Transcript => "  \u{2191}/\u{2193}: scroll    Tab: dialogue    q/Esc: quit",
    }
}
