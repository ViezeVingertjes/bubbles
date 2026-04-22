//! Integration tests for node groups (same title, multiple when: variants).

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
fn node_group_selects_first_available_when() {
    let src = "\
title: Bark
when: false
---
Should not see this.
===
title: Bark
when: true
---
Selected bark.
===
";
    let events = common::play(src, "Bark");
    assert_eq!(line_texts(&events), ["Selected bark."]);
}
