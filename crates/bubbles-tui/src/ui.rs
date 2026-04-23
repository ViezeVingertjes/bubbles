//! Rendering: pure functions from [`AppState`] to a ratatui frame.
//!
//! Every test uses `ratatui::backend::TestBackend`, so no function in this
//! module touches the real terminal.

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};

use crate::app::{AppState, DisplayedLine, DisplayedOption};

/// Marker drawn in front of the focused option.
const FOCUS_MARKER: &str = "> ";
/// Marker drawn in front of unfocused options (same width as the focus marker
/// so text aligns).
const UNFOCUSED_MARKER: &str = "  ";
/// Marker drawn next to unavailable (guard-locked) options.
const LOCKED_MARKER: &str = " \u{2717}";

/// Draws the current [`AppState`] into `frame`.
///
/// The layout is a stacked dialogue pane + options pane + one-line footer.
/// Later milestones add side panes by extending the top-level layout.
pub fn render(state: &AppState, frame: &mut Frame<'_>) {
    let show_options = !state.options().is_empty();
    let constraints: &[Constraint] = if show_options {
        &[
            Constraint::Min(3),
            Constraint::Length(options_height(state)),
            Constraint::Length(1),
        ]
    } else {
        &[Constraint::Min(1), Constraint::Length(1)]
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(frame.area());

    frame.render_widget(dialogue_paragraph(state), chunks[0]);
    if show_options {
        frame.render_widget(options_list(state), chunks[1]);
        frame.render_widget(footer(state), chunks[2]);
    } else {
        frame.render_widget(footer(state), chunks[1]);
    }
}

fn options_height(state: &AppState) -> u16 {
    let rows = u16::try_from(state.options().len()).unwrap_or(u16::MAX);
    rows.saturating_add(2) // +2 for the block borders
}

fn dialogue_paragraph(state: &AppState) -> Paragraph<'_> {
    let lines: Vec<Line<'_>> = match state.current_line() {
        Some(line) => line_to_spans(line),
        None if state.is_done() => vec![Line::from(Span::raw("[end of dialogue]"))],
        None if !state.options().is_empty() => {
            vec![Line::from(Span::styled(
                "Choose an option below.",
                Style::default().add_modifier(Modifier::DIM),
            ))]
        }
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

fn options_list(state: &AppState) -> List<'_> {
    let focused = state.focused_option();
    let items: Vec<ListItem<'_>> = state
        .options()
        .iter()
        .enumerate()
        .map(|(i, opt)| option_item(i, opt, focused == Some(i)))
        .collect();

    List::new(items).block(Block::default().borders(Borders::ALL).title(" options "))
}

fn option_item(index: usize, opt: &DisplayedOption, focused: bool) -> ListItem<'_> {
    let marker = if focused {
        FOCUS_MARKER
    } else {
        UNFOCUSED_MARKER
    };
    let mut spans = vec![
        Span::raw(marker),
        Span::raw(format!("{}. ", index + 1)),
        Span::raw(opt.text.clone()),
    ];
    if !opt.available {
        spans.push(Span::styled(
            LOCKED_MARKER,
            Style::default().add_modifier(Modifier::DIM),
        ));
    }

    let style = if focused {
        Style::default().add_modifier(Modifier::BOLD)
    } else if opt.available {
        Style::default()
    } else {
        Style::default().add_modifier(Modifier::DIM)
    };
    ListItem::new(Line::from(spans)).style(style)
}

fn footer(state: &AppState) -> Paragraph<'_> {
    let text = if state.is_done() {
        "  q/Esc: quit"
    } else if state.options().is_empty() {
        "  Enter: advance    q/Esc: quit"
    } else {
        "  \u{2191}/\u{2193}: focus    Enter: choose    1-9: pick    q/Esc: quit"
    };
    Paragraph::new(Line::from(Span::styled(
        text,
        Style::default().add_modifier(Modifier::DIM),
    )))
}
