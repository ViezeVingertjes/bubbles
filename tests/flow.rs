//! Integration tests for runner flow: node start, completion, and dialogue end.

use bubbles::{DialogueEvent, HashMapStorage, Runner, Value, VariableStorage, compile};

fn drain(runner: &mut Runner<HashMapStorage>) -> Vec<DialogueEvent> {
    let mut events = Vec::new();
    loop {
        match runner.next_event().unwrap() {
            Some(ev) => events.push(ev),
            None => break,
        }
    }
    events
}

#[test]
fn empty_node_end_to_end() {
    let prog = compile(
        "title: Start\n\
         ---\n\
         ===\n",
    )
    .unwrap();

    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.start("Start").unwrap();
    let events = drain(&mut runner);

    assert_eq!(
        events,
        vec![
            DialogueEvent::NodeStarted("Start".into()),
            DialogueEvent::NodeComplete("Start".into()),
            DialogueEvent::DialogueComplete,
        ]
    );
}

#[test]
fn start_unknown_node_errors() {
    let prog = compile("title: Real\n---\n===\n").unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    assert!(runner.start("Fake").is_err());
}

#[test]
fn next_event_before_start_returns_none() {
    let prog = compile("title: A\n---\n===\n").unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    // No start() called; runner is Idle → should return Ok(None)
    assert_eq!(runner.next_event().unwrap(), None);
}

#[test]
fn select_option_when_not_awaiting_errors() {
    let prog = compile("title: A\n---\n===\n").unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.start("A").unwrap();
    assert!(runner.select_option(0).is_err());
}

#[test]
fn select_option_out_of_range_errors() {
    let src = "\
title: A
---
-> Yes
-> No
===
";
    let prog = compile(src).unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.start("A").unwrap();
    while let Some(ev) = runner.next_event().unwrap() {
        if let DialogueEvent::Options(_) = ev {
            break;
        }
    }
    assert!(runner.select_option(99).is_err());
}

#[test]
fn multiple_jumps_chain() {
    let src = "\
title: A
---
<<jump B>>
===
title: B
---
<<jump C>>
===
title: C
---
End.
===
";
    let prog = compile(src).unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.start("A").unwrap();
    let events = drain(&mut runner);
    let line_text: Vec<_> = events
        .iter()
        .filter_map(|e| {
            if let DialogueEvent::Line { text, .. } = e {
                Some(text.as_str())
            } else {
                None
            }
        })
        .collect();
    assert_eq!(line_text, ["End."]);
}

#[test]
fn storage_getter_works() {
    use bubbles::{Value, VariableStorage};
    let mut storage = HashMapStorage::new();
    storage.set("$x", Value::Number(42.0));
    let prog = compile("title: A\n---\n===\n").unwrap();
    let runner = Runner::new(prog, storage);
    assert_eq!(runner.storage().get("$x"), Some(Value::Number(42.0)));
}

#[test]
fn division_by_zero_returns_error() {
    let prog = compile("title: A\n---\n<<set $x = 10 / 0>>\n===\n").unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.start("A").unwrap();
    let err = loop {
        match runner.next_event() {
            Err(e) => break e.to_string(),
            Ok(None) => panic!("dialogue ended without error"),
            Ok(Some(_)) => {}
        }
    };
    assert!(err.contains("division by zero"), "got: {err}");
}

#[test]
fn undefined_variable_returns_error() {
    let prog = compile("title: A\n---\n{$notset}\n===\n").unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.start("A").unwrap();
    let err = loop {
        match runner.next_event() {
            Err(e) => break e.to_string(),
            Ok(None) => panic!("dialogue ended without error"),
            Ok(Some(_)) => {}
        }
    };
    assert!(err.contains("$notset"), "got: {err}");
}
