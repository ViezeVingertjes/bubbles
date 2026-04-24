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
