//! One-line footer with the keybind hints that apply to the current state.
//!
//! Six hints is all that reliably fits, so the footer carries the verbs you
//! reach for mid-scene and ends with `?` for the rest.  The `Ctrl+R` hint
//! here spells out that it is destructive: it is the only key that throws
//! variables away, and it does so without a confirmation step.

use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::app::AppState;
use crate::display::FocusPanel;

/// The reload hint, worded the same way everywhere it is shown.
const RELOAD: &str = "Ctrl+R: reload (wipes variables)";

/// Returns the footer paragraph for `state`.
pub fn footer(state: &AppState) -> Paragraph<'_> {
    let text = hint_text(state);
    Paragraph::new(Line::from(Span::styled(
        text,
        Style::default().add_modifier(Modifier::DIM),
    )))
}

fn hint_text(state: &AppState) -> String {
    if state.help_visible() {
        return "  ?/q/Esc: close".to_owned();
    }
    let body = if state.error_overlay().is_some() {
        format!("r: rerun    {RELOAD}    x: dismiss")
    } else if state.is_done() {
        format!("b: back    r: rerun    {RELOAD}    Tab: scroll transcript")
    } else {
        match state.focus() {
            FocusPanel::Options if state.options().is_empty() => {
                format!("Enter: advance    b: back    r: rerun    {RELOAD}")
            }
            FocusPanel::Options => {
                format!(
                    "\u{2191}/\u{2193}: option    Enter: choose    1-9: pick    b: back    {RELOAD}"
                )
            }
            FocusPanel::Transcript => {
                format!(
                    "\u{2191}/\u{2193}: scroll    b: back    r: rerun    {RELOAD}    Tab: options"
                )
            }
        }
    };
    format!("  {body}    ?: keys    q/Esc: quit")
}
