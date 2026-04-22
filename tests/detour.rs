//! Integration tests for <<detour>> and <<return>>.

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
fn detour_executes_sub_then_returns() {
    let src = "\
title: Start
---
Before detour.
<<detour Sub>>
After detour.
===
title: Sub
---
Inside sub.
===
";
    let events = common::play(src, "Start");
    assert_eq!(
        line_texts(&events),
        ["Before detour.", "Inside sub.", "After detour."]
    );
}

#[test]
fn return_exits_node_early() {
    let src = "\
title: Start
---
Before detour.
<<detour Sub>>
After sub.
===
title: Sub
---
Sub line.
<<return>>
Should not appear.
===
";
    let events = common::play(src, "Start");
    assert_eq!(
        line_texts(&events),
        ["Before detour.", "Sub line.", "After sub."]
    );
}
