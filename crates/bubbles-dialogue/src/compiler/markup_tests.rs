use super::*;

fn open(name: &str) -> TextToken<'_> {
    TextToken::MarkupOpen {
        name,
        properties: vec![],
    }
}

fn open_props<'a>(name: &'a str, props: Vec<(&'a str, &'a str)>) -> TextToken<'a> {
    TextToken::MarkupOpen {
        name,
        properties: props,
    }
}

fn close(name: &str) -> TextToken<'_> {
    TextToken::MarkupClose { name }
}

fn self_close(name: &str) -> TextToken<'_> {
    TextToken::MarkupSelfClose {
        name,
        properties: vec![],
    }
}

fn self_close_props<'a>(name: &'a str, props: Vec<(&'a str, &'a str)>) -> TextToken<'a> {
    TextToken::MarkupSelfClose {
        name,
        properties: props,
    }
}

fn lit(s: &str) -> TextToken<'_> {
    TextToken::Literal(s)
}

fn expr(s: &str) -> TextToken<'_> {
    TextToken::Expr(s)
}

// ── basic passthrough ────────────────────────────────────────────────────

#[test]
fn empty_string_returns_empty() {
    assert_eq!(scan_text_segments("").unwrap(), vec![]);
}

#[test]
fn plain_literal_passthrough() {
    assert_eq!(
        scan_text_segments("Hello world").unwrap(),
        vec![lit("Hello world")]
    );
}

// ── open / close tags ────────────────────────────────────────────────────

#[test]
fn open_close_around_literal() {
    assert_eq!(
        scan_text_segments("[wave]hello[/wave]").unwrap(),
        vec![open("wave"), lit("hello"), close("wave")]
    );
}

#[test]
fn open_close_with_surrounding_text() {
    assert_eq!(
        scan_text_segments("[b]Hello[/b] world").unwrap(),
        vec![open("b"), lit("Hello"), close("b"), lit(" world")]
    );
}

// ── self-closing ─────────────────────────────────────────────────────────

#[test]
fn self_closing_no_text() {
    assert_eq!(
        scan_text_segments("[pause /]").unwrap(),
        vec![self_close("pause")]
    );
}

#[test]
fn self_closing_between_text() {
    assert_eq!(
        scan_text_segments("Wait[pause /]here").unwrap(),
        vec![lit("Wait"), self_close("pause"), lit("here")]
    );
}

// ── properties ───────────────────────────────────────────────────────────

#[test]
fn open_with_property() {
    assert_eq!(
        scan_text_segments("[color value=red]text[/color]").unwrap(),
        vec![
            open_props("color", vec![("value", "red")]),
            lit("text"),
            close("color"),
        ]
    );
}

#[test]
fn self_close_with_property() {
    assert_eq!(
        scan_text_segments("[sfx name=boom /]").unwrap(),
        vec![self_close_props("sfx", vec![("name", "boom")])]
    );
}

#[test]
fn open_with_multiple_properties() {
    assert_eq!(
        scan_text_segments("[tag a=1 b=2]x[/tag]").unwrap(),
        vec![
            open_props("tag", vec![("a", "1"), ("b", "2")]),
            lit("x"),
            close("tag"),
        ]
    );
}

// ── combined with {expr} ─────────────────────────────────────────────────

#[test]
fn markup_wrapping_expr() {
    assert_eq!(
        scan_text_segments("[b]{$name}[/b]").unwrap(),
        vec![open("b"), expr("$name"), close("b")]
    );
}

#[test]
fn expr_between_literals_no_markup() {
    assert_eq!(
        scan_text_segments("Hello {$name}!").unwrap(),
        vec![lit("Hello "), expr("$name"), lit("!")]
    );
}

#[test]
fn markup_and_expr_interleaved() {
    assert_eq!(
        scan_text_segments("[b]{$x}[/b] and {$y}").unwrap(),
        vec![open("b"), expr("$x"), close("b"), lit(" and "), expr("$y")]
    );
}

// ── nested tags ──────────────────────────────────────────────────────────

#[test]
fn nested_tags() {
    assert_eq!(
        scan_text_segments("[b][i]text[/i][/b]").unwrap(),
        vec![open("b"), open("i"), lit("text"), close("i"), close("b")]
    );
}

// ── non-markup brackets treated as literals ──────────────────────────────

#[test]
fn brackets_with_spaces_are_literal() {
    assert_eq!(
        scan_text_segments("[has spaces]").unwrap(),
        vec![lit("[has spaces]")]
    );
}

#[test]
fn brackets_with_non_identifier_content_are_literal() {
    // Starts with a digit – not a valid identifier
    assert_eq!(scan_text_segments("[123]").unwrap(), vec![lit("[123]")]);
}

#[test]
fn non_markup_brackets_inline_with_text() {
    assert_eq!(
        scan_text_segments("text [not markup] more").unwrap(),
        vec![lit("text [not markup] more")]
    );
}

#[test]
fn empty_brackets_are_literal() {
    assert_eq!(scan_text_segments("[]").unwrap(), vec![lit("[]")]);
}

#[test]
fn close_tag_without_name_is_literal() {
    assert_eq!(scan_text_segments("[/]").unwrap(), vec![lit("[/]")]);
}

// ── mixed: markup and non-markup brackets ────────────────────────────────

#[test]
fn markup_and_non_markup_in_same_string() {
    assert_eq!(
        scan_text_segments("[wave]Hi[/wave] [not a tag]").unwrap(),
        vec![open("wave"), lit("Hi"), close("wave"), lit(" [not a tag]")]
    );
}

// ── identifier edge cases ────────────────────────────────────────────────

#[test]
fn identifier_with_hyphens_and_underscores() {
    assert_eq!(
        scan_text_segments("[my-tag_1]x[/my-tag_1]").unwrap(),
        vec![open("my-tag_1"), lit("x"), close("my-tag_1")]
    );
}

// ── error cases ──────────────────────────────────────────────────────────

#[test]
fn unclosed_bracket_is_error() {
    assert_eq!(
        scan_text_segments("[wave"),
        Err(MarkupScanError::UnclosedBracket(0))
    );
}

#[test]
fn unclosed_bracket_mid_string_is_error() {
    // `[` is at byte offset 5
    assert_eq!(
        scan_text_segments("hello[wave"),
        Err(MarkupScanError::UnclosedBracket(5))
    );
}

#[test]
fn unclosed_brace_is_error() {
    assert_eq!(
        scan_text_segments("{$x"),
        Err(MarkupScanError::UnclosedBrace(0))
    );
}

#[test]
fn unclosed_brace_mid_string_is_error() {
    assert_eq!(
        scan_text_segments("hello {$x"),
        Err(MarkupScanError::UnclosedBrace(6))
    );
}
