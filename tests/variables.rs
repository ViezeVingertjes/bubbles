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
fn set_using_to_keyword() {
    let prog = compile("title: A\n---\n<<set $x to 2 + 3>>\n{$x}\n===\n").unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.start("A").unwrap();
    let mut saw = false;
    while let Some(ev) = runner.next_event().unwrap() {
        if let DialogueEvent::Line { text, .. } = ev {
            if text == "5" {
                saw = true;
            }
        }
    }
    assert!(saw);
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

#[test]
fn declare_read_in_line_interpolation() {
    let src = "\
title: Start
---
<<declare $hp = 100>>
You have {$hp} health.
===
";
    let prog = compile(src).unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.start("Start").unwrap();
    let mut line_text = String::new();
    while let Some(ev) = runner.next_event().unwrap() {
        if let DialogueEvent::Line { text, .. } = ev {
            line_text = text;
        }
    }
    assert_eq!(line_text, "You have 100 health.");
}

#[test]
fn set_string_variable() {
    let src = "title: S\n---\n<<set $name = \"Alice\">>\n===\n";
    let prog = compile(src).unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.start("S").unwrap();
    while runner.next_event().unwrap().is_some() {}
    assert_eq!(
        runner.storage().get("$name"),
        Some(Value::Text("Alice".into()))
    );
}

#[test]
fn set_overrides_existing_value() {
    let src = "\
title: S
---
<<set $x = 1>>
<<set $x = 2>>
===
";
    let prog = compile(src).unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.start("S").unwrap();
    while runner.next_event().unwrap().is_some() {}
    assert_eq!(runner.storage().get("$x"), Some(Value::Number(2.0)));
}

#[test]
fn storage_mut_allows_external_set() {
    let src = "title: S\n---\n{$x}\n===\n";
    let prog = compile(src).unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.storage_mut().set("$x", Value::Number(77.0));
    runner.start("S").unwrap();
    let mut texts = Vec::new();
    while let Some(ev) = runner.next_event().unwrap() {
        if let DialogueEvent::Line { text, .. } = ev {
            texts.push(text);
        }
    }
    assert_eq!(texts, ["77"]);
}
