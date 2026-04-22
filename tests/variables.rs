//! Integration tests for <<set>>, <<declare>>, and variable storage.

mod common;

use bubbles::{DialogueEvent, HashMapStorage, Runner, Value, VariableStorage, compile};

#[test]
fn set_stores_value_in_storage() {
    let src = "title: Start\n---\n<<set $gold = 10>>\nDone.\n===\n";
    let prog = compile(src).unwrap();
    let storage = HashMapStorage::new();
    let mut runner = Runner::new(prog, storage);
    runner.start("Start").unwrap();

    // consume events until complete
    loop {
        match runner.next_event().unwrap() {
            Some(DialogueEvent::DialogueComplete) | None => break,
            _ => {}
        }
    }
    assert_eq!(runner.storage().get("$gold"), Some(Value::Number(10.0)));
}

#[test]
fn set_evaluated_expression() {
    let src = "title: Start\n---\n<<set $x = 3 * 4>>\n===\n";
    let prog = compile(src).unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.start("Start").unwrap();
    while let Some(ev) = runner.next_event().unwrap() {
        if ev == DialogueEvent::DialogueComplete {
            break;
        }
    }
    assert_eq!(runner.storage().get("$x"), Some(Value::Number(12.0)));
}

#[test]
fn set_boolean_from_comparison() {
    let src = "title: Start\n---\n<<set $gold = 10>>\n<<set $rich = $gold > 5>>\n===\n";
    let prog = compile(src).unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.start("Start").unwrap();
    while let Some(ev) = runner.next_event().unwrap() {
        if ev == DialogueEvent::DialogueComplete {
            break;
        }
    }
    assert_eq!(runner.storage().get("$rich"), Some(Value::Bool(true)));
}

#[test]
fn declare_initialises_once() {
    let src = "title: Start\n---\n<<declare $x = 5>>\n<<declare $x = 99>>\n===\n";
    let prog = compile(src).unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.start("Start").unwrap();
    while let Some(ev) = runner.next_event().unwrap() {
        if ev == DialogueEvent::DialogueComplete {
            break;
        }
    }
    // second declare is a no-op since $x was already set
    assert_eq!(runner.storage().get("$x"), Some(Value::Number(5.0)));
}
