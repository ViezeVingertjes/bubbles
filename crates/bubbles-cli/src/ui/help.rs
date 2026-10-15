//! Key-list popup drawn on top of the rest of the UI.
//!
//! The footer has room for six hints and the player binds twice that, so `?`
//! puts the whole table on screen.

use ratatui::Frame;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

use super::popup::centered;
use crate::help::KEYS;

/// Width of the popup relative to the terminal, in percent.
const WIDTH_PERCENT: u16 = 70;
/// Height of the popup relative to the terminal, in percent.
const HEIGHT_PERCENT: u16 = 80;
/// Column the descriptions start in, so the two columns line up.
const KEY_WIDTH: usize = 19;

/// Draws the key list centered on top of `frame`.
pub fn render(frame: &mut Frame<'_>) {
    let area = centered(frame.area(), WIDTH_PERCENT, HEIGHT_PERCENT);
    frame.render_widget(Clear, area);
    frame.render_widget(paragraph(), area);
}

fn paragraph() -> Paragraph<'static> {
    let lines: Vec<Line<'static>> = KEYS
        .iter()
        .map(|(keys, what)| {
            Line::from(vec![
                Span::styled(
                    format!("{keys:<KEY_WIDTH$}"),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw((*what).to_owned()),
            ])
        })
        .collect();

    Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Keys ")
            .title_style(Style::default().add_modifier(Modifier::BOLD)),
    )
}
