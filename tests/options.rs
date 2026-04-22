//! Integration tests for shortcut options.

mod common;

use bubbles::{DialogueEvent, HashMapStorage, Runner, compile};

fn play_select(source: &str, node: &str, choice: usize) -> Vec<DialogueEvent> {
    let prog = compile(source).unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.start(node).unwrap();
    let mut events = Vec::new();
    loop {
        match runner.next_event().unwrap() {
            Some(DialogueEvent::Options(_)) => {
                events.push(DialogueEvent::Options(vec![])); // record marker
                runner.select_option(choice).unwrap();
            }
            Some(ev) => events.push(ev),
            None => break,
        }
    }
    events
}

#[test]
fn two_options_first_selected() {
    let events = play_select(
        "title: Start\n---\nQuestion?\n-> Fight\n    You fight.\n-> Run\n    You run.\n===\n",
        "Start",
        0,
    );
    let lines: Vec<_> = events
        .iter()
        .filter_map(|e| {
            if let DialogueEvent::Line { text, .. } = e { Some(text.as_str()) } else { None }
        })
        .collect();
    assert_eq!(lines, ["Question?", "You fight."]);
}

#[test]
fn two_options_second_selected() {
    let events = play_select(
        "title: Start\n---\nQuestion?\n-> Fight\n    You fight.\n-> Run\n    You run.\n===\n",
        "Start",
        1,
    );
    let lines: Vec<_> = events
        .iter()
        .filter_map(|e| {
            if let DialogueEvent::Line { text, .. } = e { Some(text.as_str()) } else { None }
        })
        .collect();
    assert_eq!(lines, ["Question?", "You run."]);
}

#[test]
fn option_guard_marks_unavailable() {
    let src = "\
title: Start
---
<<set $gold = 3>>
-> Buy sword <<if $gold >= 10>>
    Bought!
-> Leave
    Left.
===
";
    let prog = compile(src).unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.start("Start").unwrap();
    while let Some(ev) = runner.next_event().unwrap() {
        if let DialogueEvent::Options(opts) = ev {
            assert!(!opts[0].available, "buy sword should be unavailable");
            assert!(opts[1].available, "leave should be available");
            runner.select_option(1).unwrap();
            break;
        }
    }
}
