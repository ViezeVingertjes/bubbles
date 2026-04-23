//! Options list rendering.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem};

use crate::app::AppState;
use crate::display::DisplayedOption;

/// Marker drawn in front of the focused option.
const FOCUS_MARKER: &str = "> ";
/// Marker drawn in front of unfocused options (same width as the focus marker
/// so text aligns).
const UNFOCUSED_MARKER: &str = "  ";
/// Marker drawn next to unavailable (guard-locked) options.
const LOCKED_MARKER: &str = " \u{2717}";

/// Height of the options list widget (rows + 2 border lines).
pub fn options_height(state: &AppState) -> u16 {
    let rows = u16::try_from(state.options().len()).unwrap_or(u16::MAX);
    rows.saturating_add(2)
}

/// Renders the options list into `area`.
pub fn render_options(state: &AppState, frame: &mut Frame<'_>, area: Rect) {
    frame.render_widget(options_list(state), area);
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
