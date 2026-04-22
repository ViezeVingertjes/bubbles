//! Integration tests for => line groups and saliency selection.

mod common;

use bubbles::DialogueEvent;

fn line_texts(events: &[DialogueEvent]) -> Vec<&str> {
    events
        .iter()
        .filter_map(|e| {
            if let DialogueEvent::Line { text, .. } = e {
                Some(text.as_str())
            } else {
                None
            }
        })
        .collect()
}

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
