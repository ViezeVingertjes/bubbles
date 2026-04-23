//! Parsers for `<<set>>` / `<<declare>>` statements, the shared
//! parse-time expression validator, and text-interpolation segment splitting.

use std::sync::Arc;

use crate::compiler::ast::{Expr, Stmt, TextSegment};
use crate::compiler::interpolation::{BraceSegment, scan_brace_segments};
use crate::error::{DialogueError, Result};

use super::command::split_first_word;

/// Parses an expression and wraps it in a shared pointer for the AST.
///
/// Parse failures surface with the enclosing statement's file/line so error
/// messages point at the real `.bub` location rather than the `<expr>`
/// placeholder.  The `context` hint (e.g. `"<<set>>"`, `"<<if>>"`) is
/// prefixed onto the message so the reader knows which clause failed.
pub(super) fn parse_expr_arc(
    src: &str,
    context: &str,
    line: usize,
    file: &str,
) -> Result<Arc<Expr>> {
    crate::compiler::expr::parse_expr_at(src, file, line)
        .map_err(|e| match e {
            DialogueError::Parse {
                file: f,
                line: l,
                message,
            } => DialogueError::Parse {
                file: f,
                line: l,
                message: format!("in {context} `{src}`: {message}"),
            },
            other => other,
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
    let brace_segments = scan_brace_segments(raw).map_err(|_| DialogueError::Parse {
        file: file.to_owned(),
        line,
        message: format!("unclosed `{{` in {context}: `{raw}`"),
    })?;

    let mut segments = Vec::with_capacity(brace_segments.len());
    for seg in brace_segments {
        match seg {
            BraceSegment::Literal(s) => segments.push(TextSegment::literal(s)),
            BraceSegment::Expr(src) => {
                segments.push(TextSegment::Expr(parse_expr_arc(src, context, line, file)?));
            }
        }
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
