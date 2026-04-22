//! Integration tests for the compilation pipeline.

use bubbles::{compile, compile_many};

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
