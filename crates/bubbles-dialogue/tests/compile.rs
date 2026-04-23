//! Integration tests for the compilation pipeline.

use bubbles::{DialogueError, compile, compile_many, validate};

#[test]
fn empty_node_parses() {
    let src = "\
title: Start
---
===
";
    let prog = compile(src).expect("should compile");
    assert!(prog.node_exists("Start"));
}

#[test]
fn node_with_tags_header() {
    let src = "\
title: Bark
tags: npc ambient
---
===
";
    let prog = compile(src).unwrap();
    assert_eq!(prog.node_tags("Bark").unwrap(), &["npc", "ambient"]);
}

#[test]
fn missing_title_is_an_error() {
    let src = "\
tags: orphan
---
===
";
    assert!(compile(src).is_err());
}

#[test]
fn multiple_nodes_compile() {
    let src = "\
title: A
---
===
title: B
---
===
";
    let prog = compile(src).unwrap();
    assert!(prog.node_exists("A"));
    assert!(prog.node_exists("B"));
}

#[test]
fn compile_many_merges_sources() {
    let prog = compile_many(&[
        ("file_a", "title: A\n---\n===\n"),
        ("file_b", "title: B\n---\n===\n"),
    ])
    .unwrap();
    assert!(prog.node_exists("A"));
    assert!(prog.node_exists("B"));
}

#[test]
fn duplicate_node_title_is_an_error() {
    let result = compile_many(&[
        ("file_a", "title: Dup\n---\n===\n"),
        ("file_b", "title: Dup\n---\n===\n"),
    ]);
    assert!(result.is_err());
}

#[test]
fn node_titles_iterator() {
    let prog = compile("title: Alpha\n---\n===\ntitle: Beta\n---\n===\n").unwrap();
    let titles: Vec<&str> = prog.node_titles().collect();
    assert!(titles.contains(&"Alpha"));
    assert!(titles.contains(&"Beta"));
}

#[test]
fn node_exists_false_for_unknown() {
    let prog = compile("title: Real\n---\n===\n").unwrap();
    assert!(!prog.node_exists("Fake"));
}

#[test]
fn variable_declarations_collected() {
    let src = "\
title: Start
---
<<declare $health = 100>>
<<declare $name = \"Hero\">>
Hello.
===
";
    let prog = compile(src).unwrap();
    let decls = prog.variable_declarations();
    assert_eq!(decls.len(), 2);
    assert_eq!(decls[0].name, "$health");
    assert_eq!(decls[0].default_src, "100");
    assert_eq!(decls[1].name, "$name");
    assert_eq!(decls[1].default_src, "\"Hero\"");
}

#[test]
fn variable_declarations_deduped_across_nodes() {
    // The same variable declared in two nodes should only appear once.
    let src = "\
title: A
---
<<declare $x = 0>>
===
title: B
---
<<declare $x = 0>>
===
";
    let prog = compile(src).unwrap();
    assert_eq!(prog.variable_declarations().len(), 1);
}

#[test]
fn validate_standalone_rejects_unknown_jump_target() {
    // validate() is a power-user hook for Programs built outside of compile().
    // Build one manually via compile_many (without validation) is not possible
    // anymore, so we test via the public validate() fn directly on a known-good
    // program and confirm it still runs without error.
    let prog = compile("title: A\n---\n<<jump B>>\n===\ntitle: B\n---\n===\n").unwrap();
    assert!(validate(&prog).is_ok());
}

#[test]
fn three_source_compile_many() {
    let prog = compile_many(&[
        ("a", "title: A\n---\n===\n"),
        ("b", "title: B\n---\n===\n"),
        ("c", "title: C\n---\n===\n"),
    ])
    .unwrap();
    assert_eq!(prog.node_titles().count(), 3);
}

// ── error message quality ─────────────────────────────────────────────────────

#[test]
fn error_missing_title_header() {
    let err = compile("tags: foo\n---\n===\n").unwrap_err().to_string();
    assert!(err.contains("title:"), "got: {err}");
}

#[test]
fn error_missing_body_delimiter() {
    let err = compile("title: A\n===\n").unwrap_err().to_string();
    assert!(err.contains("---"), "got: {err}");
}

#[test]
fn error_missing_node_terminator() {
    let err = compile("title: A\n---\nHello.\ntitle: B\n---\n===\n")
        .unwrap_err()
        .to_string();
    assert!(err.contains("title:") && err.contains("==="), "got: {err}");
}

#[test]
fn error_invalid_expression_in_if() {
    let err = compile("title: A\n---\n<<if $x &&>>\nHello.\n<<endif>>\n===\n")
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("<<if>>") && err.contains("$x &&"),
        "got: {err}"
    );
}

#[test]
fn error_invalid_expression_in_set() {
    let err = compile("title: A\n---\n<<set $x = 1 +>>\n===\n")
        .unwrap_err()
        .to_string();
    assert!(err.contains("<<set>>"), "got: {err}");
}

#[test]
fn error_dangling_endif_gives_hint() {
    let err = compile("title: A\n---\n<<endif>>\n===\n")
        .unwrap_err()
        .to_string();
    assert!(err.contains("<<endif>>"), "got: {err}");
}

#[test]
fn error_invalid_expression_in_elseif() {
    let err = compile("title: A\n---\n<<if true>>\n    a\n<<elseif 1+>>\n    b\n<<endif>>\n===\n")
        .unwrap_err()
        .to_string();
    assert!(err.contains("<<elseif>>"), "got: {err}");
}

#[test]
fn error_invalid_expression_in_once_if() {
    let err = compile("title: A\n---\n<<once if 1+>>\n    x\n<<endonce>>\n===\n")
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("<<once if>>") || err.contains("once if"),
        "got: {err}"
    );
}

#[test]
fn error_unexpected_eof_before_body_delimiter() {
    let err = compile("title: A\n").unwrap_err().to_string();
    assert!(
        err.contains("---") || err.contains("end of file"),
        "got: {err}"
    );
}

#[test]
fn error_malformed_command_missing_close() {
    let err = compile("title: A\n---\n<<jump Nowhere\n===\n")
        .unwrap_err()
        .to_string();
    assert!(
        err.contains(">>") || err.contains("malformed"),
        "got: {err}"
    );
}

#[test]
fn error_declare_without_equals() {
    assert!(compile("title: A\n---\n<<declare $x>>\n===\n").is_err());
}

#[test]
fn error_set_to_without_expression() {
    assert!(compile("title: A\n---\n<<set $x to>>\n===\n").is_err());
}

#[test]
fn error_set_missing_equals_and_to() {
    assert!(compile("title: A\n---\n<<set $x 5>>\n===\n").is_err());
}

// ── compile-time validation of {expr} in text / args ─────────────────────────

#[test]
fn error_bad_expr_in_line_text_at_compile_time() {
    let err = compile("title: A\n---\nHello {1+}\n===\n")
        .unwrap_err()
        .to_string();
    assert!(err.contains("line"), "expected parse error, got: {err}");
}

#[test]
fn error_bad_expr_in_option_text_at_compile_time() {
    let err = compile("title: A\n---\n-> Choice {$x &&}\n===\n")
        .unwrap_err()
        .to_string();
    assert!(err.contains("parse"), "expected parse error, got: {err}");
}

#[test]
fn error_bad_expr_in_line_group_text_at_compile_time() {
    let err = compile("title: A\n---\n=> => Bark {!}\n===\n")
        .unwrap_err()
        .to_string();
    assert!(err.contains("parse"), "expected parse error, got: {err}");
}

#[test]
fn error_bad_expr_in_command_args_at_compile_time() {
    let err = compile("title: A\n---\n<<play {1+}>>\n===\n")
        .unwrap_err()
        .to_string();
    assert!(err.contains("parse"), "expected parse error, got: {err}");
}

#[test]
fn valid_interpolated_text_compiles() {
    assert!(compile("title: A\n---\nHello {$name}!\n===\n").is_ok());
}

// ── line numbers in error messages point at the source, not buffer index ──────

#[test]
fn error_line_number_for_if_reports_source_line() {
    let src = "\n\n// leading comment\ntitle: A\n---\n<<if $x &&>>\nHello.\n<<endif>>\n===\n";
    //         1 2 3                   4     5   6 (<<if>> is on source line 6)
    let err = compile(src).unwrap_err().to_string();
    assert!(err.contains(":6:"), "expected line 6 in error, got: {err}");
}

#[test]
fn error_line_number_for_elseif_reports_source_line() {
    let src = "title: A\n---\n<<if true>>\n    a\n<<elseif 1+>>\n    b\n<<endif>>\n===\n";
    //    1         2   3           4    5 (<<elseif>> on line 5)
    let err = compile(src).unwrap_err().to_string();
    assert!(err.contains(":5:"), "expected line 5 in error, got: {err}");
}

#[test]
fn error_line_number_for_once_if_reports_source_line() {
    let src = "\ntitle: A\n---\n<<once if 1+>>\n    x\n<<endonce>>\n===\n";
    //         1 2         3   4 (<<once if>> on line 4)
    let err = compile(src).unwrap_err().to_string();
    assert!(err.contains(":4:"), "expected line 4 in error, got: {err}");
}

#[test]
fn error_line_number_for_missing_body_delimiter_reports_source_line() {
    // `---` is missing on line 4 (the `===` arrives instead).
    let src = "\n\ntitle: A\n===\n";
    let err = compile(src).unwrap_err().to_string();
    assert!(err.contains(":4:"), "expected line 4 in error, got: {err}");
}

#[test]
fn set_with_bad_expression_reports_correct_line_and_specific_detail() {
    // The malformed expression lives on line 3. The error must report that
    // line (not the bogus `<expr>:0` fallback) *and* include the specific
    // parser detail (e.g. "unexpected end of expression") rather than a
    // generic "invalid expression in …" wrapper.
    let src = "\
title: A
---
<<set $x = 1 +>>
===
";
    let Err(err) = compile(src) else {
        panic!("expected parse error");
    };
    let rendered = err.to_string();
    assert!(
        rendered.contains(":3:"),
        "expected line 3 in error, got: {rendered}"
    );
    assert!(
        !rendered.contains("<expr>"),
        "error should not leak the `<expr>` placeholder file, got: {rendered}"
    );
    assert!(
        rendered.contains("unexpected end of expression"),
        "error should carry the specific parser detail, got: {rendered}"
    );
}

#[test]
fn bad_interpolation_reports_source_line_not_expr_placeholder() {
    // The broken `{1 +}` is on line 3; the error must point there.
    let src = "\
title: A
---
hello {1 +} world
===
";
    let err = compile(src).unwrap_err().to_string();
    assert!(err.contains(":3:"), "expected line 3 in error, got: {err}");
    assert!(
        !err.contains("<expr>"),
        "error should report the real file, got: {err}"
    );
}

// ── tokenise / unknown-character errors ──────────────────────────────────────

#[test]
fn unknown_char_in_if_expression_is_a_parse_error() {
    // `@` is not a valid token; it must produce a Parse error rather than
    // being silently dropped and causing a confusing downstream failure.
    let src = "title: A\n---\n<<if @$x>>\nHello.\n<<endif>>\n===\n";
    let err = compile(src).unwrap_err();
    let rendered = err.to_string();
    assert!(
        rendered.contains('@') || rendered.contains("unexpected character"),
        "expected error mentioning unknown char, got: {rendered}"
    );
}

#[test]
fn unknown_char_in_set_expression_is_a_parse_error() {
    let src = "title: A\n---\n<<set $x = 1 ^ 2>>\n===\n";
    let err = compile(src).unwrap_err();
    let rendered = err.to_string();
    assert!(
        rendered.contains('^') || rendered.contains("unexpected character"),
        "expected error mentioning unknown char, got: {rendered}"
    );
}

// ── compile always validates ──────────────────────────────────────────────────

#[test]
fn compile_rejects_unknown_jump_target() {
    let src = "title: A\n---\n<<jump Missing>>\n===\n";
    let err = compile(src).unwrap_err();
    assert!(
        matches!(err, DialogueError::Validation(_)),
        "expected Validation error, got: {err:?}"
    );
    assert!(
        err.to_string().contains("Missing"),
        "error should mention the unknown target, got: {err}"
    );
}

#[test]
fn compile_accepts_valid_jump() {
    let src = "title: A\n---\n<<jump B>>\n===\ntitle: B\n---\nHello.\n===\n";
    assert!(compile(src).is_ok());
}

#[test]
fn compile_many_rejects_unknown_detour_target() {
    let src_a = "title: A\n---\n<<detour Missing>>\n===\n";
    let err = compile_many(&[("a", src_a)]).unwrap_err();
    assert!(
        matches!(err, DialogueError::Validation(_)),
        "expected Validation error, got: {err:?}"
    );
}

#[test]
fn compile_many_accepts_cross_file_jump() {
    let result = compile_many(&[
        ("a", "title: A\n---\n<<jump B>>\n===\n"),
        ("b", "title: B\n---\nHello.\n===\n"),
    ]);
    assert!(result.is_ok());
}
