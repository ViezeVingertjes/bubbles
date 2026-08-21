//! Thin wrappers around expression evaluation and pre-parsed text segment
//! rendering, bridging the runner's state to the stateless
//! [`crate::runtime::eval`] module.

use crate::compiler::ast::{Expr, TextSegment};
use crate::compiler::expr::parse_expr_at;
use crate::compiler::markup::{TextToken, owned_properties, scan_text_segments};
use crate::error::{DialogueError, Result};
use crate::runtime::eval::eval;
use crate::runtime::event::{MarkupSpan, line_id_from_tags};
use crate::value::{Value, VariableStorage};

/// Stack entry used while resolving open markup tags: (name, properties, `start_byte`).
type OpenTag = (String, Vec<(String, String)>, usize);

use super::Runner;

/// Appends the display form of `value` to `out` without an intermediate `String`.
fn push_value(out: &mut String, value: &Value) {
    use std::fmt::Write as _;
    write!(out, "{value}").expect("writing to a String cannot fail");
}

fn close_markup(
    open_stack: &mut Vec<OpenTag>,
    spans: &mut Vec<MarkupSpan>,
    name: &str,
    end: usize,
) {
    if let Some(pos) = open_stack.iter().rposition(|(n, _, _)| n == name) {
        let (open_name, properties, start) = open_stack.remove(pos);
        spans.push(MarkupSpan {
            name: open_name,
            start,
            length: end - start,
            properties,
        });
    }
}

fn self_closing_markup(
    spans: &mut Vec<MarkupSpan>,
    name: String,
    properties: Vec<(String, String)>,
    start: usize,
) {
    spans.push(MarkupSpan {
        name,
        start,
        length: 0,
        properties,
    });
}

fn finish_open_markup(open_stack: Vec<OpenTag>, spans: &mut Vec<MarkupSpan>) {
    for (name, properties, start) in open_stack {
        spans.push(MarkupSpan {
            name,
            start,
            length: 0,
            properties,
        });
    }
}

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

    /// Renders pre-parsed text segments into a final `(text, spans)` pair.
    ///
    /// Literal segments are appended verbatim; `Expr` segments are evaluated
    /// and stringified. Markup open/close/self-close segments are stripped from
    /// the text and recorded as [`MarkupSpan`]s with byte offsets into the
    /// returned string.
    pub(super) fn eval_segments(
        &self,
        segments: &[TextSegment],
    ) -> Result<(String, Vec<MarkupSpan>)> {
        let mut out = String::new();
        let mut spans: Vec<MarkupSpan> = Vec::new();
        let mut open_stack: Vec<OpenTag> = Vec::new();

        for seg in segments {
            match seg {
                TextSegment::Literal(s) => out.push_str(s),
                TextSegment::Expr(e) => push_value(&mut out, &self.eval_expr(e.as_ref())?),
                TextSegment::MarkupOpen { name, properties } => {
                    open_stack.push((name.clone(), properties.clone(), out.len()));
                }
                TextSegment::MarkupClose { name } => {
                    close_markup(&mut open_stack, &mut spans, name, out.len());
                }
                TextSegment::MarkupSelfClose { name, properties } => {
                    self_closing_markup(&mut spans, name.clone(), properties.clone(), out.len());
                }
            }
        }

        finish_open_markup(open_stack, &mut spans);

        Ok((out, spans))
    }

    /// Renders pre-parsed segments then splits the result on whitespace.
    ///
    /// Markup spans are discarded; command argument strings do not carry markup.
    /// Returns an empty `Vec` when all segments are empty or whitespace-only.
    pub(super) fn eval_segments_as_args(&self, segments: &[TextSegment]) -> Result<Vec<String>> {
        let (text, _spans) = self.eval_segments(segments)?;
        Ok(text.split_whitespace().map(str::to_owned).collect())
    }

    /// Parses a translated `template` string for `{expr}` placeholders and
    /// `[markup]` tags at runtime, evaluating each expression against current
    /// storage and recording markup spans.
    ///
    /// Used after a [`crate::runtime::provider::LineProvider`] returns a
    /// translated string, enabling translate-then-format ordering. Markup in
    /// translated strings is processed with the same scanner as source markup;
    /// mismatched or unclosed tags are handled leniently (silently dropped).
    pub(super) fn eval_template(&self, template: &str) -> Result<(String, Vec<MarkupSpan>)> {
        let tokens = scan_text_segments(template).map_err(|e| DialogueError::Parse {
            file: "<translation>".into(),
            line: 0,
            message: e.describe("translated template", template),
        })?;

        let mut out = String::with_capacity(template.len());
        let mut spans: Vec<MarkupSpan> = Vec::new();
        let mut open_stack: Vec<OpenTag> = Vec::new();

        for tok in tokens {
            match tok {
                TextToken::Literal(s) => out.push_str(s),
                TextToken::Expr(src) => {
                    let expr = parse_expr_at(src, "<translation>", 0)?;
                    push_value(&mut out, &self.eval_expr(&expr)?);
                }
                TextToken::MarkupOpen { name, properties } => {
                    open_stack.push((name.to_owned(), owned_properties(&properties), out.len()));
                }
                TextToken::MarkupClose { name } => {
                    close_markup(&mut open_stack, &mut spans, name, out.len());
                }
                TextToken::MarkupSelfClose { name, properties } => self_closing_markup(
                    &mut spans,
                    name.to_owned(),
                    owned_properties(&properties),
                    out.len(),
                ),
            }
        }

        finish_open_markup(open_stack, &mut spans);

        Ok((out, spans))
    }

    /// Resolves the final `(text, spans, line_id)` for a line: looks up the
    /// provider first so that translators receive raw templates they can still
    /// use `{expr}` and `[markup]` in, then falls back to evaluating the
    /// compile-time-parsed segments. The line id extracted from `tags` is
    /// returned so callers do not have to scan the tags a second time.
    pub(super) fn eval_line_text(
        &self,
        segments: &[TextSegment],
        tags: &[String],
    ) -> Result<(String, Vec<MarkupSpan>, Option<String>)> {
        let line_id = line_id_from_tags(tags);
        let (text, spans) = line_id
            .as_deref()
            .and_then(|id| self.provider.get(id))
            .map_or_else(
                || self.eval_segments(segments),
                |template| self.eval_template(&template),
            )?;
        Ok((text, spans, line_id))
    }
}
