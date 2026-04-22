//! `<<…>>` command-syntax helpers: extraction, first-word splitting, and
//! trailing-tag peeling specific to command lines.

use crate::error::{DialogueError, Result};

use super::text::split_trailing_tags;

/// Splits `<<cmd>> #tag1 #tag2` into `("<<cmd>>", vec!["tag1","tag2"])`.
pub(super) fn extract_cmd_line_tags(t: &str) -> (&str, Vec<String>) {
    if let Some(close) = t.find(">>") {
        let cmd_part = &t[..close + 2];
        let after = t[close + 2..].trim();
        if after.is_empty() {
            return (cmd_part, Vec::new());
        }
        let padded = format!(" {after}");
        let (_, tags) = split_trailing_tags(&padded);
        (cmd_part, tags)
    } else {
        (t, Vec::new())
    }
}

/// Extracts the inner text of `<<…>>`.
pub(super) fn extract_cmd<'a>(t: &'a str, line: usize, file: &str) -> Result<&'a str> {
    let close = t.find(">>").ok_or_else(|| DialogueError::Parse {
        file: file.to_owned(),
        line,
        message: format!("malformed command (missing `>>`): `{t}`"),
    })?;
    let with_close = &t[..close + 2];
    let inner = with_close
        .strip_prefix("<<")
        .and_then(|s| s.strip_suffix(">>"))
        .ok_or_else(|| DialogueError::Parse {
            file: file.to_owned(),
            line,
            message: format!("malformed command `{t}`"),
        })?;
    Ok(inner.trim())
}

pub(super) fn first_word(s: &str) -> &str {
    s.split_whitespace().next().unwrap_or("")
}

pub(super) fn split_first_word(s: &str) -> (&str, &str) {
    s.find(|c: char| c.is_ascii_whitespace())
        .map_or((s, ""), |i| (&s[..i], s[i..].trim_start()))
}
