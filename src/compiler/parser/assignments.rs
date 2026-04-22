//! Parsers for `<<set>>` / `<<declare>>` statements and the shared
//! parse-time expression validator they delegate to.

use crate::compiler::ast::Stmt;
use crate::error::{DialogueError, Result};

use super::command::split_first_word;

/// Validates an expression source at parse time, returning a clear error on the
/// correct line rather than a cryptic runtime failure later.
pub(super) fn validate_expr(src: &str, context: &str, line: usize, file: &str) -> Result<()> {
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
