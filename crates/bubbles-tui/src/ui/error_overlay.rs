//! Error-overlay popup drawn on top of the rest of the UI.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

use crate::overlay::ErrorOverlay;

/// Width of the popup relative to the terminal, in percent.
const WIDTH_PERCENT: u16 = 70;
/// Height of the popup relative to the terminal, in percent.
const HEIGHT_PERCENT: u16 = 40;

/// Draws `overlay` centered on top of `frame`.
pub fn render(overlay: &ErrorOverlay, frame: &mut Frame<'_>) {
    let area = centered(frame.area(), WIDTH_PERCENT, HEIGHT_PERCENT);
    frame.render_widget(Clear, area); // clear background
    frame.render_widget(paragraph(overlay), area);
}

fn paragraph(overlay: &ErrorOverlay) -> Paragraph<'_> {
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
        "r: reload   x: dismiss   q/Esc: quit",
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

/// Returns a rectangle with `width_pct`% × `height_pct`% of `area`, centered.
const fn centered(area: Rect, width_pct: u16, height_pct: u16) -> Rect {
    let width = area.width.saturating_mul(width_pct) / 100;
    let height = area.height.saturating_mul(height_pct) / 100;
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    Rect {
        x,
        y,
        width,
        height,
    }
}
