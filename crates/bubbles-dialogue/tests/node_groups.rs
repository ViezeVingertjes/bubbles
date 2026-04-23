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

#[test]
fn node_group_all_when_false_errors_on_start() {
    use bubbles::{HashMapStorage, Runner, compile};
    let src = "\
title: Empty
when: false
---
A
===
title: Empty
when: false
---
B
===
";
    let prog = compile(src).unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    let err = runner.start("Empty").unwrap_err().to_string();
    assert!(
        err.contains("no available candidate") || err.contains("Empty"),
        "got {err}"
    );
}
