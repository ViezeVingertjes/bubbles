//! Thin wrappers around expression evaluation and pre-parsed text segment
//! rendering, bridging the runner's state to the stateless
//! [`crate::runtime::eval`] module.

use crate::compiler::ast::{Expr, TextSegment};
use crate::error::Result;
use crate::runtime::eval::eval;
use crate::value::{Value, VariableStorage};

use super::Runner;

impl<S: VariableStorage> Runner<S> {
    /// Evaluates a compile-time-parsed expression against current storage and the
    /// function library.
    pub(super) fn eval_expr(&self, expr: &Expr) -> Result<Value> {
        eval(expr, &self.storage, &|name, args| {
            self.library.call(name, args)
        })
    }

    /// Renders pre-parsed text segments into a final string.
    ///
    /// Literal segments are appended verbatim; `Expr` segments are evaluated
    /// against current storage and converted to their string representation.
    pub(super) fn eval_segments(&self, segments: &[TextSegment]) -> Result<String> {
        let mut out = String::new();
        for seg in segments {
            match seg {
                TextSegment::Literal(s) => out.push_str(s),
                TextSegment::Expr(e) => out.push_str(&self.eval_expr(e.as_ref())?.to_string()),
            }
        }
        Ok(out)
    }

    /// Renders pre-parsed segments then splits the result on whitespace.
    ///
    /// Returns an empty `Vec` when all segments are empty or whitespace-only.
    pub(super) fn eval_segments_as_args(&self, segments: &[TextSegment]) -> Result<Vec<String>> {
        let text = self.eval_segments(segments)?;
        if text.trim().is_empty() {
            return Ok(Vec::new());
        }
        Ok(text.split_whitespace().map(str::to_owned).collect())
    }
}
