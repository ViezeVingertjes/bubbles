//! Integration tests for if/elseif/else/endif conditional execution.

mod common;

use bubbles::DialogueEvent;

#[test]
fn if_true_branch_taken() {
    let events = common::play_fixture("conditionals", "Start");
    let lines: Vec<_> = events
        .iter()
        .filter_map(|e| {
            if let DialogueEvent::Line { text, .. } = e {
                Some(text.as_str())
            } else {
                None
            }
        })
        .collect();
    assert_eq!(lines, ["Rich path."]);
}

#[test]
fn if_false_takes_else() {
    let src = "\
title: Start
---
<<set $gold = 2>>
<<if $gold > 5>>
    Rich path.
<<else>>
    Poor path.
<<endif>>
===
";
    let events = common::play(src, "Start");
    let lines: Vec<_> = events
        .iter()
        .filter_map(|e| {
            if let DialogueEvent::Line { text, .. } = e {
                Some(text.as_str())
            } else {
                None
            }
        })
        .collect();
    assert_eq!(lines, ["Poor path."]);
}

#[test]
fn if_else_body_reused_across_repeated_detours() {
    // Detour twice into a node whose body contains an <<if>> whose else
    // branch fires on both entries. The else body is shared via Arc<[Stmt]>
    // and must not be consumed by the first execution.
    let src = "\
title: Start
---
<<detour Sub>>
<<detour Sub>>
===
title: Sub
---
<<if false>>
    skipped
<<else>>
    fallback
<<endif>>
===
";
    let events = common::play(src, "Start");
    let lines: Vec<_> = events
        .iter()
        .filter_map(|e| {
            if let DialogueEvent::Line { text, .. } = e {
                Some(text.as_str())
            } else {
                None
            }
        })
        .collect();
    assert_eq!(lines, ["fallback", "fallback"]);
}

#[test]
fn elseif_chain() {
    let src = "\
title: Start
---
<<set $x = 5>>
<<if $x > 10>>
    High.
<<elseif $x > 3>>
    Mid.
<<else>>
    Low.
<<endif>>
===
";
    let events = common::play(src, "Start");
    let lines: Vec<_> = events
        .iter()
        .filter_map(|e| {
            if let DialogueEvent::Line { text, .. } = e {
                Some(text.as_str())
            } else {
                None
            }
        })
        .collect();
    assert_eq!(lines, ["Mid."]);
}
