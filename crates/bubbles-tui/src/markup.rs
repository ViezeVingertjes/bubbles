//! Converts [`MarkupSpan`] metadata into styled ratatui [`Line`]s.
//!
//! Supported tag names (case-sensitive):
//!
//! | Tag                       | Effect            |
//! |---------------------------|-------------------|
//! | `[b]` / `[bold]`          | bold              |
//! | `[i]` / `[italic]` / `[em]` | italic          |
//! | `[u]` / `[underline]`     | underlined        |
//! | `[dim]`                   | dim               |
//! | `[color value=NAME]`      | foreground colour |
//! | anything else             | ignored           |
//!
//! Spans may overlap (e.g. `[b][i]text[/i][/b]`). Styles are merged at each
//! character position so both modifiers apply.

use bubbles::MarkupSpan;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

/// Turns `text` + `spans` into a ratatui [`Line`] with inline styling.
///
/// When `spans` is empty, the function returns immediately with a single
/// unstyled span, avoiding any allocation beyond the string itself.
pub fn markup_line(text: &str, spans: &[MarkupSpan]) -> Line<'static> {
    if spans.is_empty() || text.is_empty() {
        return Line::from(text.to_owned());
    }

    // One style accumulator per byte. Overlapping spans are merged.
    let len = text.len();
    let mut styles = vec![Style::default(); len];

    for span in spans {
        let end = (span.start + span.length).min(len);
        if span.start >= end {
            continue;
        }
        let delta = style_for_span(&span.name, &span.properties);
        for s in &mut styles[span.start..end] {
            *s = merge(*s, delta);
        }
    }

    // Compress adjacent bytes with identical styles into one Span each.
    let mut ratatui_spans: Vec<Span<'static>> = Vec::new();
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let mut i = 0;
    while i < chars.len() {
        let (byte_start, _) = chars[i];
        let style = styles[byte_start];
        let mut j = i + 1;
        while j < chars.len() {
            if styles[chars[j].0] != style {
                break;
            }
            j += 1;
        }
        let byte_end = if j < chars.len() { chars[j].0 } else { len };
        ratatui_spans.push(Span::styled(text[byte_start..byte_end].to_owned(), style));
        i = j;
    }

    Line::from(ratatui_spans)
}

/// Returns the [`Style`] delta for a given tag name and properties.
///
/// Unknown tags return [`Style::default()`] (no-op).
fn style_for_span(name: &str, properties: &[(String, String)]) -> Style {
    match name {
        "b" | "bold" => Style::default().add_modifier(Modifier::BOLD),
        "i" | "italic" | "em" => Style::default().add_modifier(Modifier::ITALIC),
        "u" | "underline" => Style::default().add_modifier(Modifier::UNDERLINED),
        "dim" => Style::default().add_modifier(Modifier::DIM),
        "color" => {
            let val = properties
                .iter()
                .find(|(k, _)| k == "value")
                .map_or("", |(_, v)| v.as_str());
            Style::default().fg(parse_color(val))
        }
        _ => Style::default(),
    }
}

/// Merges `delta` into `base`: OR the modifier bits and override fg/bg when set.
fn merge(base: Style, delta: Style) -> Style {
    let mut out = base;
    out.add_modifier |= delta.add_modifier;
    if delta.fg.is_some() {
        out.fg = delta.fg;
    }
    if delta.bg.is_some() {
        out.bg = delta.bg;
    }
    out
}

/// Maps a colour name string to a ratatui [`Color`].
///
/// Supports the 16 ANSI names; anything unrecognised falls back to
/// [`Color::Reset`] (terminal default).
fn parse_color(s: &str) -> Color {
    match s {
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "white" => Color::White,
        "gray" | "grey" | "darkgray" | "darkgrey" => Color::DarkGray,
        "lightred" => Color::LightRed,
        "lightgreen" => Color::LightGreen,
        "lightyellow" => Color::LightYellow,
        "lightblue" => Color::LightBlue,
        "lightmagenta" => Color::LightMagenta,
        "lightcyan" => Color::LightCyan,
        _ => Color::Reset,
    }
}
