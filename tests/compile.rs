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
