//! Integration tests for {expr} inline substitution.

mod common;

use bubbles::DialogueEvent;

fn lines_from(events: &[DialogueEvent]) -> Vec<&str> {
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
fn variable_and_expression_substituted() {
    let events = common::play_fixture("interpolation", "Start");
    let lines = lines_from(&events);
    assert_eq!(
        lines,
        ["Hello Bob, you have 42 coins.", "The answer is 42.",]
    );
}

#[test]
fn no_braces_unchanged() {
    let src = "title: Start\n---\nHello world.\n===\n";
    let events = common::play(src, "Start");
    let lines = lines_from(&events);
    assert_eq!(lines, ["Hello world."]);
}

// ── plural() ──────────────────────────────────────────────────────────────────

#[test]
fn plural_singular_in_line() {
    let src = "\
title: Start
---
<<declare $n = 1>>
You have {$n} {plural($n, \"apple\", \"apples\")}.
===
";
    let events = common::play(src, "Start");
    assert_eq!(lines_from(&events), ["You have 1 apple."]);
}

#[test]
fn plural_plural_in_line() {
    let src = "\
title: Start
---
<<declare $n = 3>>
You have {$n} {plural($n, \"coin\", \"coins\")}.
===
";
    let events = common::play(src, "Start");
    assert_eq!(lines_from(&events), ["You have 3 coins."]);
}

#[test]
fn plural_zero_uses_plural_form() {
    let src = "\
title: Start
---
<<declare $n = 0>>
{plural($n, \"item\", \"items\")} remaining.
===
";
    let events = common::play(src, "Start");
    assert_eq!(lines_from(&events), ["items remaining."]);
}
