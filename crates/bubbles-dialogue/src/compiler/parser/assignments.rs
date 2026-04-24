//! Parsers for `<<set>>` / `<<declare>>` statements, the shared
//! parse-time expression validator, and text-interpolation segment splitting.

use std::sync::Arc;

use crate::compiler::ast::{Expr, Stmt, TextSegment};
use crate::compiler::markup::{MarkupScanError, TextToken, scan_text_segments};
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

/// Splits `raw` text into [`TextSegment`]s, parsing every `{expr}` fragment
/// and recording inline markup open/close/self-close tags.
///
/// Returns a `Parse` error if any fragment is syntactically invalid or an
/// unclosed `{` / `[` is found.
pub(super) fn parse_interpolated(
    raw: &str,
    context: &str,
    line: usize,
    file: &str,
) -> Result<Vec<TextSegment>> {
    let tokens = scan_text_segments(raw).map_err(|e| {
        let msg = match e {
            MarkupScanError::UnclosedBrace(_) => format!("unclosed `{{` in {context}: `{raw}`"),
            MarkupScanError::UnclosedBracket(_) => {
                format!("unclosed `[` in {context}: `{raw}`")
            }
        };
        DialogueError::Parse {
            file: file.to_owned(),
            line,
            message: msg,
        }
    })?;

    let mut segments = Vec::with_capacity(tokens.len());
    for tok in tokens {
        match tok {
            TextToken::Literal(s) => segments.push(TextSegment::literal(s)),
            TextToken::Expr(src) => {
                segments.push(TextSegment::Expr(parse_expr_arc(src, context, line, file)?));
            }
            TextToken::MarkupOpen { name, properties } => {
                segments.push(TextSegment::MarkupOpen {
                    name: name.to_owned(),
                    properties: properties
                        .into_iter()
                        .map(|(k, v)| (k.to_owned(), v.to_owned()))
                        .collect(),
                });
            }
            TextToken::MarkupClose { name } => {
                segments.push(TextSegment::MarkupClose {
                    name: name.to_owned(),
                });
            }
            TextToken::MarkupSelfClose { name, properties } => {
                segments.push(TextSegment::MarkupSelfClose {
                    name: name.to_owned(),
                    properties: properties
                        .into_iter()
                        .map(|(k, v)| (k.to_owned(), v.to_owned()))
                        .collect(),
                });
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
