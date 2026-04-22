//! Free helper functions used by the parser — string manipulation utilities.

use crate::compiler::ast::Stmt;
use crate::error::{DialogueError, Result};

pub(super) fn leading_spaces(s: &str) -> usize {
    s.len() - s.trim_start().len()
}

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

/// Validates an expression source at parse time, returning a clear error on the
/// correct line rather than a cryptic runtime failure later.
fn validate_expr(src: &str, context: &str, line: usize, file: &str) -> Result<()> {
    crate::compiler::expr::parse_expr(src).map_err(|_| DialogueError::Parse {
        file: file.to_owned(),
        line,
        message: format!("invalid expression in {context}: `{src}`"),
    })?;
    Ok(())
}

pub(super) fn parse_set(inner: &str, line: usize, file: &str) -> Result<Stmt> {
    let rest = inner["set".len()..].trim();
    let (name, after) = split_first_word(rest);
    let expr_src = after
        .strip_prefix('=')
        .or_else(|| after.strip_prefix("to "))
        .map(str::trim)
        .ok_or_else(|| DialogueError::Parse {
            file: file.to_owned(),
            line,
            message: format!(
                "expected `= expr` or `to expr` after variable in `<<set>>`, got `{after}`"
            ),
        })?
        .to_owned();
    validate_expr(&expr_src, "<<set>>", line, file)?;
    Ok(Stmt::Set {
        name: name.to_owned(),
        expr_src,
    })
}

pub(super) fn parse_declare(inner: &str, line: usize, file: &str) -> Result<Stmt> {
    let rest = inner["declare".len()..].trim();
    let (name, after) = split_first_word(rest);
    let expr_src = after
        .strip_prefix('=')
        .map(str::trim)
        .ok_or_else(|| DialogueError::Parse {
            file: file.to_owned(),
            line,
            message: format!("expected `= expr` after variable in `<<declare>>`, got `{after}`"),
        })?
        .to_owned();
    validate_expr(&expr_src, "<<declare>>", line, file)?;
    Ok(Stmt::Declare {
        name: name.to_owned(),
        expr_src,
    })
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

pub(super) fn parse_line_stmt(t: &str) -> Stmt {
    let (speaker, rest) = split_speaker(t);
    let (text, tags) = split_trailing_tags(&rest);
    Stmt::Line {
        speaker,
        text,
        tags,
    }
}
