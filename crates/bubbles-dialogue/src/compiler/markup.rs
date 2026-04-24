//! Scanner for inline markup (`[name]…[/name]`, `[name /]`) combined with
//! `{expr}` interpolation. Both are parsed in one left-to-right pass.

/// A token produced by scanning text that may contain `{expr}` and `[markup]` syntax.
#[derive(Debug, PartialEq, Eq)]
pub enum TextToken<'a> {
    /// A literal run of text with no substitution or markup.
    Literal(&'a str),
    /// The source text between `{` and `}`.
    Expr(&'a str),
    /// An opening markup tag: `[name]` or `[name key=val …]`.
    MarkupOpen {
        /// Tag name, e.g. `wave` in `[wave]`.
        name: &'a str,
        /// Zero or more `key=value` pairs.
        properties: Vec<(&'a str, &'a str)>,
    },
    /// A closing markup tag: `[/name]`.
    MarkupClose {
        /// Tag name, e.g. `wave` in `[/wave]`.
        name: &'a str,
    },
    /// A self-closing markup tag: `[name /]` or `[name key=val … /]`.
    MarkupSelfClose {
        /// Tag name, e.g. `pause` in `[pause /]`.
        name: &'a str,
        /// Zero or more `key=value` pairs.
        properties: Vec<(&'a str, &'a str)>,
    },
}

/// Errors returned by [`scan_text_segments`].
#[derive(Debug, PartialEq, Eq)]
pub enum MarkupScanError {
    /// An unclosed `{` at the given byte offset.
    UnclosedBrace(usize),
    /// An unclosed `[` at the given byte offset.
    UnclosedBracket(usize),
}

/// Scans `text` for `{expr}` and `[markup]` syntax, yielding tokens in order.
///
/// **Markup rules:**
/// - `[identifier]` or `[identifier key=val …]` → [`TextToken::MarkupOpen`]
/// - `[/identifier]` → [`TextToken::MarkupClose`]
/// - `[identifier /]` or `[identifier key=val … /]` → [`TextToken::MarkupSelfClose`]
/// - `[…]` whose content does not match any of the above → emitted verbatim
///   as part of a [`TextToken::Literal`]
///
/// An unclosed `{` or `[` (no matching `}` / `]` before end of input) is
/// always an error regardless of the content inside.
///
/// # Errors
///
/// Returns [`MarkupScanError::UnclosedBrace`] or [`MarkupScanError::UnclosedBracket`]
/// with the byte offset of the unmatched delimiter.
pub fn scan_text_segments(text: &str) -> Result<Vec<TextToken<'_>>, MarkupScanError> {
    let mut tokens = Vec::new();
    let bytes = text.as_bytes();
    let mut i = 0usize;
    let mut lit_start = 0usize;

    macro_rules! flush_literal {
        () => {
            if lit_start < i {
                tokens.push(TextToken::Literal(&text[lit_start..i]));
            }
        };
    }

    while i < bytes.len() {
        match bytes[i] {
            b'{' => {
                let brace_start = i;
                let rest = &text[i + 1..];
                let close = rest
                    .find('}')
                    .ok_or(MarkupScanError::UnclosedBrace(brace_start))?;
                flush_literal!();
                tokens.push(TextToken::Expr(&rest[..close]));
                i = i + 1 + close + 1;
                lit_start = i;
            }
            b'[' => {
                let bracket_start = i;
                let rest = &text[i + 1..];
                let close_rel = rest
                    .find(']')
                    .ok_or(MarkupScanError::UnclosedBracket(bracket_start))?;
                let inner = &rest[..close_rel];
                if let Some(tok) = try_parse_markup(inner) {
                    flush_literal!();
                    tokens.push(tok);
                    i = i + 1 + close_rel + 1;
                    lit_start = i;
                } else {
                    // Not markup – include the `[` in the current literal run
                    // and let the scanner continue character-by-character.
                    i += 1;
                }
            }
            _ => {
                i += 1;
            }
        }
    }

    if lit_start < text.len() {
        tokens.push(TextToken::Literal(&text[lit_start..]));
    }

    Ok(tokens)
}

/// Returns `true` if `s` is a valid markup identifier (`[a-zA-Z_][a-zA-Z0-9_-]*`).
fn is_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    chars.next().is_some_and(|c| {
        (c.is_ascii_alphabetic() || c == '_')
            && chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    })
}

/// Parses zero or more `key=value` pairs separated by whitespace.
///
/// Returns `None` if any pair is malformed (missing `=` or non-identifier key).
fn parse_properties(s: &str) -> Option<Vec<(&str, &str)>> {
    if s.is_empty() {
        return Some(Vec::new());
    }
    let mut props = Vec::new();
    for part in s.split_whitespace() {
        let eq = part.find('=')?;
        let key = &part[..eq];
        let val = &part[eq + 1..];
        if !is_identifier(key) {
            return None;
        }
        props.push((key, val));
    }
    Some(props)
}

/// Attempts to parse the content between `[` and `]` as a markup token.
///
/// Returns `None` if the content does not match the markup grammar, in which
/// case the caller should treat the entire `[…]` as literal text.
fn try_parse_markup(inner: &str) -> Option<TextToken<'_>> {
    // Close tag: `/identifier`
    if let Some(name_part) = inner.strip_prefix('/') {
        let name = name_part.trim_start();
        if is_identifier(name) && name.len() == name_part.len() {
            return Some(TextToken::MarkupClose { name });
        }
        return None;
    }

    // Self-closing: content ends with ` /`
    let (content, self_close) = inner
        .strip_suffix(" /")
        .map_or((inner, false), |rest| (rest, true));

    // Split into name and optional property string on the first space
    let (name, props_src) = content
        .find(' ')
        .map_or((content, ""), |sp| (&content[..sp], &content[sp + 1..]));

    if !is_identifier(name) {
        return None;
    }

    let properties = parse_properties(props_src)?;

    if self_close {
        Some(TextToken::MarkupSelfClose { name, properties })
    } else {
        Some(TextToken::MarkupOpen { name, properties })
    }
}

#[cfg(test)]
#[path = "markup_tests.rs"]
mod tests;
