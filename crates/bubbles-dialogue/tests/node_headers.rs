//! Integration tests for node headers the compiler does not interpret.
//!
//! Only `title:`, `tags:` and `when:` mean anything. Every other header is
//! read and discarded, which lets a script carry authoring metadata the
//! runtime never sees. Nothing else in the suite compiles a node with such a
//! header, so these tests are what holds that promise: a change that dropped
//! the extras a step earlier in the parser, or that added an "unknown header"
//! diagnostic, would turn every script carrying metadata into a compile error
//! with the rest of the suite still green.

mod common;

use bubbles::compile;
use common::{line_texts, play};

#[test]
fn unknown_node_headers_are_accepted_and_ignored() {
    let src = "\
title: Start
author: Kompel Media
speaker_default: Narrator
note: a value: containing a colon
---
Hi.
===
";
    let prog = compile(src).expect("unknown headers should compile");
    assert!(prog.node_exists("Start"));
    // The body must be untouched: a header the parser failed to consume would
    // surface here as an extra line, or swallow the real one.
    assert_eq!(line_texts(&play(src, "Start")), ["Hi."]);
}

#[test]
fn unknown_headers_do_not_displace_the_reserved_ones() {
    // Unknown headers before, between and after `title:`, `tags:` and `when:`
    // must not stop the reserved ones from being read.
    let src = "\
author: Kompel Media
title: Bark
licence: MIT
tags: npc ambient
colour: red
when: false
---
Gated out.
===
title: Bark
---
Fallback.
===
";
    let prog = compile(src).unwrap();
    // `node_tags` reports the first group member, which is the one carrying
    // the `tags:` header above.
    assert_eq!(prog.node_tags("Bark").unwrap(), &["npc", "ambient"]);
    // Its `when: false` is still live, so the fallback member is selected.
    assert_eq!(line_texts(&play(src, "Bark")), ["Fallback."]);
}

#[test]
fn reserved_header_names_are_matched_exactly() {
    // A near miss is an unknown header, not a header the parser guesses at.
    let prog = compile("title: A\ntag: npc\n---\n===\n").unwrap();
    assert_eq!(prog.node_tags("A").unwrap(), [] as [String; 0]);

    let err = compile("titles: A\n---\n===\n").unwrap_err().to_string();
    assert!(err.contains("title:"), "got: {err}");
}
