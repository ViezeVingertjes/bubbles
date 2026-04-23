//! Integration tests for runner flow: node start, completion, and dialogue end.

mod common;

use bubbles::{DialogueError, DialogueEvent, HashMapStorage, Runner, compile};

use common::{drain, line_texts};

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
    assert_eq!(line_texts(&events), ["End."]);
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

#[test]
fn next_event_while_awaiting_option_errors() {
    let prog = compile("title: A\n---\n-> Only\n===\n").unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.start("A").unwrap();
    assert!(matches!(
        runner.next_event().unwrap(),
        Some(DialogueEvent::NodeStarted(_))
    ));
    assert!(matches!(
        runner.next_event().unwrap(),
        Some(DialogueEvent::Options(_))
    ));
    let err = runner.next_event().unwrap_err().to_string();
    assert!(
        err.contains("select_option") || err.contains("AwaitingOption"),
        "got: {err}"
    );
}

#[test]
fn runtime_jump_to_unknown_node_errors_even_without_validate() {
    let prog = compile("title: A\n---\n<<jump Nope>>\n===\n").unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.start("A").unwrap();
    runner.next_event().unwrap(); // NodeStarted
    let err = runner.next_event().unwrap_err().to_string();
    assert!(
        err.contains("Nope") || err.contains("unknown"),
        "got: {err}"
    );
}

#[test]
fn visited_builtin_wrong_arity_errors() {
    let prog = compile("title: A\n---\n<<set $x = visited()>>\n===\n").unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.start("A").unwrap();
    runner.next_event().unwrap();
    let err = runner.next_event().unwrap_err().to_string();
    assert!(err.contains("visited"), "got: {err}");
}

#[test]
fn visited_count_builtin_wrong_arity_errors() {
    let prog = compile("title: A\n---\n<<set $x = visited_count(1, 2)>>\n===\n").unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.start("A").unwrap();
    runner.next_event().unwrap();
    let err = runner.next_event().unwrap_err().to_string();
    assert!(err.contains("visited_count"), "got: {err}");
}

#[test]
fn return_from_entry_node_ends_dialogue() {
    let prog = compile("title: A\n---\n<<return>>\n===\n").unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.start("A").unwrap();
    let mut saw_complete = false;
    while let Some(ev) = runner.next_event().unwrap() {
        if matches!(ev, DialogueEvent::DialogueComplete) {
            saw_complete = true;
        }
    }
    assert!(saw_complete);
}

#[test]
fn stop_ends_dialogue_mid_node() {
    // `<<stop>>` terminates the dialogue immediately, skipping the rest of
    // the node body and any calling frames. No `NodeComplete` is emitted -
    // the stack is cleared and a single `DialogueComplete` closes things out.
    let prog = compile(
        "title: A\n\
         ---\n\
         first\n\
         <<stop>>\n\
         never shown\n\
         ===\n",
    )
    .unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.start("A").unwrap();
    let events = drain(&mut runner);

    assert!(matches!(events[0], DialogueEvent::NodeStarted(_)));
    assert!(matches!(
        &events[1],
        DialogueEvent::Line { text, .. } if text == "first"
    ));
    assert_eq!(events[2], DialogueEvent::DialogueComplete);
    assert_eq!(events.len(), 3, "unexpected events: {events:?}");
    assert_eq!(line_texts(&events), ["first"]);
}

#[test]
fn stop_terminates_across_detour() {
    // `<<stop>>` must unwind the full call stack: a detour into a node that
    // calls `<<stop>>` should end the entire dialogue, not just return from
    // the detour.
    let src = "\
title: Start
---
before
<<detour Sub>>
after
===
title: Sub
---
in sub
<<stop>>
===
";
    let events = common::play(src, "Start");
    assert_eq!(line_texts(&events), ["before", "in sub"]);
    assert_eq!(events.last(), Some(&DialogueEvent::DialogueComplete));
}

#[test]
fn detour_reenters_same_body_twice_without_mutation() {
    // Two detours to the same node must replay the same lines both times.
    // If bodies were mutated in-place through a shared `Arc`, the second
    // detour would observe missing statements.
    let src = "\
title: Start
---
<<detour Sub>>
<<detour Sub>>
Done.
===
title: Sub
---
alpha
beta
===
";
    let prog = compile(src).unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.start("Start").unwrap();
    let events = drain(&mut runner);
    assert_eq!(
        line_texts(&events),
        ["alpha", "beta", "alpha", "beta", "Done."]
    );
}

// ── structured error variants ─────────────────────────────────────────────────

#[test]
fn next_event_while_awaiting_option_returns_protocol_violation() {
    let prog = compile("title: A\n---\n-> Only\n===\n").unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.start("A").unwrap();
    runner.next_event().unwrap(); // NodeStarted
    runner.next_event().unwrap(); // Options
    let err = runner.next_event().unwrap_err();
    assert!(
        matches!(err, DialogueError::ProtocolViolation(_)),
        "expected ProtocolViolation, got: {err:?}"
    );
}

#[test]
fn select_option_when_not_awaiting_returns_protocol_violation() {
    let prog = compile("title: A\n---\n===\n").unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.start("A").unwrap();
    let err = runner.select_option(0).unwrap_err();
    assert!(
        matches!(err, DialogueError::ProtocolViolation(_)),
        "expected ProtocolViolation, got: {err:?}"
    );
}

#[test]
fn type_error_from_expression_returns_type_mismatch() {
    // Adding a number to a string is a type mismatch.
    let prog = compile("title: A\n---\n<<set $x = 1 + \"oops\">>\n===\n").unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.start("A").unwrap();
    runner.next_event().unwrap(); // NodeStarted
    let err = runner.next_event().unwrap_err();
    assert!(
        matches!(err, DialogueError::TypeMismatch { .. }),
        "expected TypeMismatch, got: {err:?}"
    );
}
