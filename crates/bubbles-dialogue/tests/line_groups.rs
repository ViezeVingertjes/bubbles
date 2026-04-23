//! Integration tests for => line groups and saliency selection.

mod common;

use bubbles::DialogueEvent;

use common::line_texts;

#[test]
fn first_available_variant_selected() {
    let src = "\
title: Start
---
=> Variant A.
=> Variant B.
===
";
    // default saliency = FirstAvailable → picks variant A
    let events = common::play(src, "Start");
    assert_eq!(line_texts(&events), ["Variant A."]);
}

#[test]
fn guarded_variant_skips_to_available() {
    let src = "\
title: Start
---
<<set $done = true>>
=> Unavailable. <<if !$done>>
=> Available now.
===
";
    let events = common::play(src, "Start");
    assert_eq!(line_texts(&events), ["Available now."]);
}

#[test]
fn line_group_all_guards_false_falls_through_to_next_line() {
    let src = "\
title: Start
---
<<set $x = true>>
=> Skipped A. <<if !$x>>
=> Skipped B. <<if !$x>>
After group.
===
";
    let events = common::play(src, "Start");
    assert_eq!(line_texts(&events), ["After group."]);
}

#[test]
fn line_group_once_prefix_skips_after_first_pick() {
    use bubbles::{HashMapStorage, Runner, compile};
    let src = "\
title: Start
---
=> once Shown once.
=> Repeatable.
===
";
    let prog = compile(src).unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());

    runner.start("Start").unwrap();
    let mut first_lines: Vec<String> = Vec::new();
    while let Some(ev) = runner.next_event().unwrap() {
        if let DialogueEvent::Line { text, .. } = ev {
            first_lines.push(text);
        }
    }
    assert!(
        first_lines.iter().any(|t| t.contains("Shown once")),
        "first={first_lines:?}"
    );

    runner.start("Start").unwrap();
    let mut second_first = None;
    while let Some(ev) = runner.next_event().unwrap() {
        if let DialogueEvent::Line { text, .. } = ev {
            second_first = Some(text);
            break;
        }
    }
    assert_eq!(
        second_first.as_deref(),
        Some("Repeatable."),
        "once-variant should be skipped on revisit"
    );
}
