//! Integration tests for the compilation pipeline.

use bubbles::{compile, compile_many, validate};

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
fn validate_rejects_unknown_jump_target() {
    let prog = compile("title: A\n---\n<<jump Missing>>\n===\n").unwrap();
    assert!(validate(&prog).is_err());
}

#[test]
fn validate_accepts_valid_jump_target() {
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
