//! Dialogue + options pane rendering.

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};

use crate::app::{AppState, DisplayedLine, DisplayedOption, FocusPanel};

/// Marker drawn in front of the focused option.
const FOCUS_MARKER: &str = "> ";
/// Marker drawn in front of unfocused options (same width as the focus marker
/// so text aligns).
const UNFOCUSED_MARKER: &str = "  ";
/// Marker drawn next to unavailable (guard-locked) options.
const LOCKED_MARKER: &str = " \u{2717}";

/// Renders the dialogue pane (and options, when active) into `area`.
pub fn render(state: &AppState, frame: &mut Frame<'_>, area: Rect) {
    if state.options().is_empty() {
        frame.render_widget(dialogue_paragraph(state), area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),
            Constraint::Length(options_height(state)),
        ])
        .split(area);
    frame.render_widget(dialogue_paragraph(state), chunks[0]);
    frame.render_widget(options_list(state), chunks[1]);
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
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title_with_focus(" dialogue ", state)),
        )
        .wrap(Wrap { trim: false })
}

fn title_with_focus(text: &'static str, state: &AppState) -> &'static str {
    if matches!(state.focus(), FocusPanel::Dialogue) {
        // Subtle focus cue: prefix the title with a bullet. We keep it as a
        // 'static str so widgets stay `'static`-friendly.
        match text {
            " dialogue " => "• dialogue ",
            other => other,
        }
    } else {
        text
    }
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
