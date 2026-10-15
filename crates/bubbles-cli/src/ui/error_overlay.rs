//! Error-overlay popup drawn on top of the rest of the UI.

use ratatui::Frame;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

use super::popup::centered;
use crate::diagnostic::Diagnostic;

/// Width of the popup relative to the terminal, in percent.
const WIDTH_PERCENT: u16 = 70;
/// Height of the popup relative to the terminal, in percent.
const HEIGHT_PERCENT: u16 = 40;

/// Draws `overlay` centered on top of `frame`.
pub fn render(overlay: &Diagnostic, frame: &mut Frame<'_>) {
    let area = centered(frame.area(), WIDTH_PERCENT, HEIGHT_PERCENT);
    frame.render_widget(Clear, area); // clear background
    frame.render_widget(paragraph(overlay), area);
}

fn paragraph(overlay: &Diagnostic) -> Paragraph<'_> {
    let mut lines: Vec<Line<'_>> = Vec::new();

    if let Some(location) = overlay.location_string() {
        lines.push(Line::from(Span::styled(
            format!("at {location}"),
            Style::default().add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
    }
    for chunk in overlay.message.split('\n') {
        lines.push(Line::from(Span::raw(chunk.to_owned())));
    }
    if let Some(excerpt) = overlay.excerpt.as_deref() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("| {excerpt}"),
            Style::default().add_modifier(Modifier::DIM),
        )));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Ctrl+R: reload   x: dismiss   ?: keys   q/Esc: quit",
        Style::default().add_modifier(Modifier::DIM),
    )));

    let title = format!(" {} ", overlay.title);
    Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .title_style(Style::default().add_modifier(Modifier::BOLD)),
        )
        .wrap(Wrap { trim: false })
}
