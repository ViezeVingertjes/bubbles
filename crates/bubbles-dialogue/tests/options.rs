//! Integration tests for shortcut options.

use bubbles::{DialogueError, DialogueEvent, HashMapStorage, Runner, RunnerPhase, compile};

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
            if let DialogueEvent::Line { text, .. } = e {
                Some(text.as_str())
            } else {
                None
            }
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
            if let DialogueEvent::Line { text, .. } = e {
                Some(text.as_str())
            } else {
                None
            }
        })
        .collect();
    assert_eq!(lines, ["Question?", "You run."]);
}

#[test]
fn option_with_no_body_just_selectable() {
    let src = "\
title: Start
---
What do you say?
-> Hello
-> Goodbye
===
";
    let prog = compile(src).unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.start("Start").unwrap();
    let mut got_options = false;
    while let Some(ev) = runner.next_event().unwrap() {
        if let DialogueEvent::Options(opts) = ev {
            got_options = true;
            assert_eq!(opts.len(), 2);
            assert_eq!(opts[0].text, "Hello");
            assert_eq!(opts[1].text, "Goodbye");
            runner.select_option(0).unwrap();
        }
    }
    assert!(got_options);
}

#[test]
fn option_multiple_trailing_tags_preserve_order() {
    let src = "\
title: Start
---
-> First #a #b #c
-> Second
===
";
    let prog = compile(src).unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.start("Start").unwrap();
    while let Some(ev) = runner.next_event().unwrap() {
        if let DialogueEvent::Options(opts) = ev {
            assert_eq!(opts[0].tags, vec!["a", "b", "c"]);
            assert!(opts[1].tags.is_empty());
            break;
        }
    }
}

#[test]
fn option_metadata_exposed() {
    let src = "\
title: Start
---
-> Buy #expensive
-> Sell
===
";
    let prog = compile(src).unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.start("Start").unwrap();
    while let Some(ev) = runner.next_event().unwrap() {
        if let DialogueEvent::Options(opts) = ev {
            assert!(opts[0].tags.contains(&"expensive".to_string()));
            runner.select_option(1).unwrap();
            break;
        }
    }
}

#[test]
fn select_unavailable_option_returns_protocol_violation() {
    let src = "\
title: Start
---
Pick?
-> Locked <<if false>>
    no
-> Go
    ok
===
";
    let prog = compile(src).unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.start("Start").unwrap();
    while let Some(ev) = runner.next_event().unwrap() {
        if let DialogueEvent::Options(opts) = ev {
            assert!(!opts[0].available);
            assert!(opts[1].available);
            let err = runner.select_option(0).unwrap_err();
            assert!(matches!(err, DialogueError::ProtocolViolation(_)));
            assert_eq!(runner.phase(), RunnerPhase::AwaitingOption);
            runner.select_option(1).unwrap();
            break;
        }
    }
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

#[test]
fn once_option_becomes_unavailable_after_selection() {
    let src = "\
title: Start
---
Pick?
-> once Ask about the map
    Asked.
-> Leave
    Left.
===
";
    let prog = compile(src).unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());

    runner.start("Start").unwrap();
    while let Some(ev) = runner.next_event().unwrap() {
        if let DialogueEvent::Options(opts) = ev {
            assert!(opts[0].available);
            runner.select_option(0).unwrap();
            break;
        }
    }

    runner.start("Start").unwrap();
    while let Some(ev) = runner.next_event().unwrap() {
        if let DialogueEvent::Options(opts) = ev {
            assert!(!opts[0].available, "once option should be exhausted");
            assert!(opts[1].available);
            let err = runner.select_option(0).unwrap_err();
            assert!(matches!(err, DialogueError::ProtocolViolation(_)));
            runner.select_option(1).unwrap();
            break;
        }
    }
}
