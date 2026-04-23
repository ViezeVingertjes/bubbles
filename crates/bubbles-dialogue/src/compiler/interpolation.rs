//! Shared `{expr}` brace-scanning used by both the compile-time interpolation
//! parser and the runtime translation-template evaluator.
//!
//! Both sites need to walk a raw string looking for `{…}` placeholders; this
//! module provides one implementation that they both call.

/// A segment produced by scanning `{expr}` interpolation syntax.
#[derive(Debug, PartialEq, Eq)]
pub enum BraceSegment<'a> {
    /// A literal run of text with no substitution.
    Literal(&'a str),
    /// The source text between `{` and `}`.
    Expr(&'a str),
}

/// Scans `text` for `{expr}` placeholder syntax, yielding segments in order.
///
/// Returns `Ok(Vec<BraceSegment>)` on success, or `Err(offset)` where
/// `offset` is the byte position of an unclosed `{` so the caller can build
/// an appropriate error message with its own file/line context.
///
/// # Errors
///
/// Returns the byte offset of the unclosed `{` when no matching `}` is found.
pub fn scan_brace_segments(text: &str) -> Result<Vec<BraceSegment<'_>>, usize> {
    let mut segments = Vec::new();
    let mut remaining = text;
    let mut consumed = 0;

    while let Some(open) = remaining.find('{') {
        if open > 0 {
            segments.push(BraceSegment::Literal(&remaining[..open]));
        }
        let after = &remaining[open + 1..];
        let close = after.find('}').ok_or(consumed + open)?;
        segments.push(BraceSegment::Expr(&after[..close]));
        consumed += open + 1 + close + 1;
        remaining = &after[close + 1..];
    }

    if !remaining.is_empty() {
        segments.push(BraceSegment::Literal(remaining));
    }

    Ok(segments)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_braces_returns_single_literal() {
        assert_eq!(
            scan_brace_segments("hello world").unwrap(),
            vec![BraceSegment::Literal("hello world")]
        );
    }

    #[test]
    fn empty_string_returns_empty_vec() {
        assert_eq!(scan_brace_segments("").unwrap(), vec![]);
    }

    #[test]
    fn single_expr_only() {
        assert_eq!(
            scan_brace_segments("{$x}").unwrap(),
            vec![BraceSegment::Expr("$x")]
        );
    }

    #[test]
    fn literal_then_expr_then_literal() {
        assert_eq!(
            scan_brace_segments("Hello {$name}!").unwrap(),
            vec![
                BraceSegment::Literal("Hello "),
                BraceSegment::Expr("$name"),
                BraceSegment::Literal("!"),
            ]
        );
    }

    #[test]
    fn multiple_exprs() {
        assert_eq!(
            scan_brace_segments("{$a} and {$b}").unwrap(),
            vec![
                BraceSegment::Expr("$a"),
                BraceSegment::Literal(" and "),
                BraceSegment::Expr("$b"),
            ]
        );
    }

    #[test]
    fn unclosed_brace_returns_err_with_offset() {
        // `{unclosed` — the `{` is at offset 7 ("hello: ")
        let result = scan_brace_segments("hello: {unclosed");
        assert_eq!(result, Err(7));
    }

    #[test]
    fn unclosed_brace_at_start_returns_offset_zero() {
        assert_eq!(scan_brace_segments("{oops"), Err(0));
    }
}
