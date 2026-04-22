//! Thin wrappers around expression evaluation, text interpolation, and
//! command-argument parsing, bridging the runner's state to the stateless
//! [`crate::runtime::eval`] and [`crate::runtime::interpolate`] modules.

use crate::error::Result;
use crate::runtime::eval::eval;
use crate::runtime::interpolate::interpolate;
use crate::value::{Value, VariableStorage};

use super::Runner;

impl<S: VariableStorage> Runner<S> {
    /// Evaluates an expression source string against the current variable storage.
    pub(super) fn eval_expr_src(&self, src: &str) -> Result<Value> {
        let expr = crate::compiler::expr::parse_expr(src)?;
        eval(&expr, &self.storage, &|name, args| {
            self.library.call(name, args)
        })
    }

    /// Runs inline `{expr}` substitution on `text` using current storage.
    pub(super) fn interpolate_text(&self, text: &str) -> Result<String> {
        interpolate(text, &self.storage, &|name, args| {
            self.library.call(name, args)
        })
    }

    /// Splits `args_src` into whitespace-separated tokens after interpolation.
    pub(super) fn parse_command_args(&self, args_src: &str) -> Result<Vec<String>> {
        if args_src.trim().is_empty() {
            return Ok(Vec::new());
        }
        let interpolated = self.interpolate_text(args_src)?;
        Ok(interpolated.split_whitespace().map(str::to_owned).collect())
    }
}
