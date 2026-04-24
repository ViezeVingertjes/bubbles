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
#[path = "interpolation_tests.rs"]
mod tests;
