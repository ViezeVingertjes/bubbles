//! Line-level text helpers: indentation, speaker prefixes, trailing tags,
//! and plain-line statement construction.

use crate::compiler::ast::Stmt;
use crate::error::Result;

use super::assignments::parse_interpolated;

pub(super) fn leading_spaces(s: &str) -> usize {
    s.len() - s.trim_start().len()
}

/// Extracts trailing ` #tag` tokens from the end of a string.
pub(super) fn split_trailing_tags(s: &str) -> (String, Vec<String>) {
    let mut text = s.trim_end().to_owned();
    let mut tags = Vec::new();
    loop {
        let trimmed = text.trim_end();
        if let Some(hash_pos) = trimmed.rfind(" #") {
            let tag_candidate = &trimmed[hash_pos + 2..];
            if !tag_candidate.is_empty() && !tag_candidate.contains(' ') {
                tags.push(tag_candidate.to_owned());
                text = trimmed[..hash_pos].trim_end().to_owned();
                continue;
            }
        }
        break;
    }
    tags.reverse();
    (text, tags)
}

/// Splits a line into `(speaker, text)` if it looks like `Name: text`.
pub(super) fn split_speaker(s: &str) -> (Option<String>, String) {
    if let Some(colon) = s.find(':') {
        let candidate = &s[..colon];
        if !candidate.contains(' ') && !candidate.is_empty() && colon + 1 < s.len() {
            let text = s[colon + 1..].trim().to_owned();
            return (Some(candidate.trim().to_owned()), text);
        }
    }
    (None, s.to_owned())
}

pub(super) fn parse_line_stmt(t: &str, lineno: usize, file: &str) -> Result<Stmt> {
    let (speaker, rest) = split_speaker(t);
    let (raw, tags) = split_trailing_tags(&rest);
    let text = parse_interpolated(&raw, "line text", lineno, file)?;
    Ok(Stmt::Line {
        speaker,
        text,
        tags,
    })
}
