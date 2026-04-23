//! Rendering: pure functions from [`AppState`] to a ratatui frame.
//!
//! Every test uses `ratatui::backend::TestBackend`, so no function in this
//! module touches the real terminal.

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::{AppState, DisplayedLine};

/// Draws the current [`AppState`] into `frame`.
///
/// The layout is a single dialogue pane stacked above a one-line footer
/// with keybinding hints.  Later milestones add side panes in the same
/// top-level `Layout`.
pub fn render(state: &AppState, frame: &mut Frame<'_>) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(frame.area());

    frame.render_widget(dialogue_paragraph(state), chunks[0]);
    frame.render_widget(footer(state), chunks[1]);
}

fn dialogue_paragraph(state: &AppState) -> Paragraph<'_> {
    let lines: Vec<Line<'_>> = match state.current_line() {
        Some(line) => line_to_spans(line),
        None if state.is_done() => vec![Line::from(Span::raw("[end of dialogue]"))],
        None => vec![Line::from(Span::styled(
            "Press Enter to begin.",
            Style::default().add_modifier(Modifier::DIM),
        ))],
    };

    Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" dialogue "))
        .wrap(Wrap { trim: false })
}

fn line_to_spans(line: &DisplayedLine) -> Vec<Line<'_>> {
    let mut spans = Vec::with_capacity(2);
    if let Some(speaker) = line.speaker.as_deref() {
        spans.push(Span::styled(
            format!("{speaker}: "),
            Style::default().add_modifier(Modifier::BOLD),
        ));
    }
    spans.push(Span::raw(line.text.as_str()));
    vec![Line::from(spans)]
}

fn footer(state: &AppState) -> Paragraph<'_> {
    let text = if state.is_done() {
        "  q/Esc: quit"
    } else {
        "  Enter: advance    q/Esc: quit"
    };
    Paragraph::new(Line::from(Span::styled(
        text,
        Style::default().add_modifier(Modifier::DIM),
    )))
}
