//! Parsers for `<<set>>` / `<<declare>>` statements, the shared
//! parse-time expression validator, and text-interpolation segment splitting.

use std::sync::Arc;

use crate::compiler::ast::{Expr, Stmt, TextSegment};
use crate::error::{DialogueError, Result};

use super::command::split_first_word;

/// Parses an expression and wraps it in a shared pointer for the AST.
pub(super) fn parse_expr_arc(
    src: &str,
    context: &str,
    line: usize,
    file: &str,
) -> Result<Arc<Expr>> {
    crate::compiler::expr::parse_expr(src)
        .map_err(|_| DialogueError::Parse {
            file: file.to_owned(),
            line,
            message: format!("invalid expression in {context}: `{src}`"),
        })
        .map(Arc::new)
}

/// Splits `raw` text into [`TextSegment`]s, parsing every `{expr}` fragment.
///
/// Returns a `Parse` error if any fragment is syntactically invalid.
pub(super) fn parse_interpolated(
    raw: &str,
    context: &str,
    line: usize,
    file: &str,
) -> Result<Vec<TextSegment>> {
    let mut segments = Vec::new();
    let mut remaining = raw;

    while let Some(open) = remaining.find('{') {
        if open > 0 {
            segments.push(TextSegment::literal(&remaining[..open]));
        }
        let after = &remaining[open + 1..];
        let close = after.find('}').ok_or_else(|| DialogueError::Parse {
            file: file.to_owned(),
            line,
            message: format!("unclosed `{{` in {context}: `{raw}`"),
        })?;
        let src = &after[..close];
        let expr = parse_expr_arc(src, context, line, file)?;
        segments.push(TextSegment::Expr(expr));
        remaining = &after[close + 1..];
    }

    if !remaining.is_empty() {
        segments.push(TextSegment::literal(remaining));
    }

    Ok(segments)
}

pub(super) fn parse_set(inner: &str, line: usize, file: &str) -> Result<Stmt> {
    let rest = inner["set".len()..].trim();
    let (name, after) = split_first_word(rest);
    let rhs = after
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
    let expr = parse_expr_arc(&rhs, "<<set>>", line, file)?;
    Ok(Stmt::Set {
        name: name.to_owned(),
        expr,
    })
}

pub(super) fn parse_declare(inner: &str, line: usize, file: &str) -> Result<Stmt> {
    let rest = inner["declare".len()..].trim();
    let (name, after) = split_first_word(rest);
    let default_src = after
        .strip_prefix('=')
        .map(str::trim)
        .ok_or_else(|| DialogueError::Parse {
            file: file.to_owned(),
            line,
            message: format!("expected `= expr` after variable in `<<declare>>`, got `{after}`"),
        })?
        .to_owned();
    let expr = parse_expr_arc(&default_src, "<<declare>>", line, file)?;
    Ok(Stmt::Declare {
        name: name.to_owned(),
        expr,
        default_src,
    })
}
