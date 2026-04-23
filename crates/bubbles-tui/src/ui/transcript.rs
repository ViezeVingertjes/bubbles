//! Transcript pane: the scrollable running log of session events.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::AppState;
use crate::display::FocusPanel;
use crate::transcript::TranscriptEntry;

/// Draws the transcript pane into `area`.
pub fn render(state: &AppState, frame: &mut Frame<'_>, area: Rect) {
    let entries = state.transcript();
    let scroll = state.transcript_scroll();

    let visible_rows = usize::from(area.height.saturating_sub(2)); // minus borders
    let total = entries.len();
    let end = total.saturating_sub(scroll);
    let start = end.saturating_sub(visible_rows);

    let lines: Vec<Line<'_>> = if entries.is_empty() {
        let hint = if state.is_done() {
            "(empty)"
        } else {
            "Press Enter to begin."
        };
        vec![Line::from(Span::styled(
            hint,
            Style::default().add_modifier(Modifier::DIM),
        ))]
    } else {
        entries[start..end].iter().map(format_entry).collect()
    };

    let title = title(state, total, scroll);
    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(title))
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

fn title(state: &AppState, total: usize, scroll: usize) -> String {
    let focus_marker = if matches!(state.focus(), FocusPanel::Transcript) {
        "\u{2022} "
    } else {
        ""
    };
    if scroll == 0 {
        format!(" {focus_marker}transcript ({total}) ")
    } else {
        format!(" {focus_marker}transcript ({total}, \u{2191}{scroll}) ")
    }
}

fn format_entry(entry: &TranscriptEntry) -> Line<'_> {
    match entry {
        TranscriptEntry::NodeStarted(name) => Line::from(Span::styled(
            format!("[\u{2192} {name}]"),
            Style::default().add_modifier(Modifier::DIM),
        )),
        TranscriptEntry::NodeComplete(name) => Line::from(Span::styled(
            format!("[\u{2190} {name}]"),
            Style::default().add_modifier(Modifier::DIM),
        )),
        TranscriptEntry::Line { speaker, text } => {
            let mut spans = Vec::with_capacity(2);
            if let Some(spk) = speaker.as_deref() {
                spans.push(Span::styled(
                    format!("{spk}: "),
                    Style::default().add_modifier(Modifier::BOLD),
                ));
            }
            spans.push(Span::raw(text.clone()));
            Line::from(spans)
        }
        TranscriptEntry::Command { name, args, tags } => {
            let joined_args = args.join(" ");
            let mut text = format!("\u{2699} {name} {joined_args}");
            if !tags.is_empty() {
                text.push_str(" #");
                text.push_str(&tags.join(" #"));
            }
            Line::from(Span::styled(
                text,
                Style::default().add_modifier(Modifier::DIM),
            ))
        }
        TranscriptEntry::OptionChosen { text, index } => Line::from(Span::styled(
            format!("\u{2192} chose [{index}] {text}"),
            Style::default().add_modifier(Modifier::ITALIC),
        )),
    }
}
