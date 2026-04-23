//! Thin wrappers around expression evaluation and pre-parsed text segment
//! rendering, bridging the runner's state to the stateless
//! [`crate::runtime::eval`] module.

use crate::compiler::ast::{Expr, TextSegment};
use crate::compiler::expr::parse_expr_at;
use crate::error::{DialogueError, Result};
use crate::runtime::eval::eval;
use crate::value::{Value, VariableStorage};

use super::Runner;

impl<S: VariableStorage> Runner<S> {
    /// Evaluates a compile-time-parsed expression against current storage and the
    /// function library.
    pub(super) fn eval_expr(&self, expr: &Expr) -> Result<Value> {
        eval(expr, &self.storage, &|name, args| {
            self.call_function(name, args)
        })
    }

    /// Dispatches a function call, short-circuiting the built-in `visited` and
    /// `visited_count` lookups against the runner-local visit table before
    /// delegating to the [`crate::library::FunctionLibrary`].
    ///
    /// Keeping these two builtins out of the [`FunctionLibrary`] means the
    /// visits map is not shared across threads, so we can store it as a plain
    /// `HashMap` instead of `Arc<Mutex<_>>`.
    fn call_function(&self, name: &str, args: Vec<Value>) -> Result<Value> {
        match (name, args.as_slice()) {
            ("visited", [Value::Text(title)]) => Ok(Value::Bool(
                self.visits.get(title).copied().unwrap_or(0) > 0,
            )),
            ("visited", _) => Err(DialogueError::Function {
                name: "visited".into(),
                message: "expected one string argument".into(),
            }),
            ("visited_count", [Value::Text(title)]) => Ok(Value::Number(f64::from(
                self.visits.get(title).copied().unwrap_or(0),
            ))),
            ("visited_count", _) => Err(DialogueError::Function {
                name: "visited_count".into(),
                message: "expected one string argument".into(),
            }),
            _ => self.library.call(name, args),
        }
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

    /// Parses `template` for `{expr}` placeholders at runtime and evaluates
    /// each one against the current storage and function library.
    ///
    /// Used after a [`crate::runtime::provider::LineProvider`] returns a
    /// translated string that may still contain `{expr}` syntax, enabling
    /// translate-then-format ordering.
    pub(super) fn eval_template(&self, template: &str) -> Result<String> {
        let mut out = String::with_capacity(template.len());
        let mut remaining = template;
        while let Some(open) = remaining.find('{') {
            out.push_str(&remaining[..open]);
            let after = &remaining[open + 1..];
            let close = after.find('}').ok_or_else(|| DialogueError::Parse {
                file: "<translation>".into(),
                line: 0,
                message: format!("unclosed `{{` in translated template: `{template}`"),
            })?;
            let expr_src = &after[..close];
            let expr = parse_expr_at(expr_src, "<translation>", 0)?;
            let value = self.eval_expr(&expr)?;
            out.push_str(&value.to_string());
            remaining = &after[close + 1..];
        }
        out.push_str(remaining);
        Ok(out)
    }

    /// Resolves the final text for a line: looks up the provider first so that
    /// translators receive raw templates they can still use `{expr}` in, then
    /// falls back to evaluating the compile-time-parsed segments.
    pub(super) fn eval_line_text(
        &self,
        segments: &[TextSegment],
        tags: &[String],
    ) -> Result<String> {
        let line_id = crate::runtime::event::line_id_from_tags(tags);
        line_id
            .as_deref()
            .and_then(|id| self.provider.get(id))
            .map_or_else(
                || self.eval_segments(segments),
                |template| self.eval_template(&template),
            )
    }
}
