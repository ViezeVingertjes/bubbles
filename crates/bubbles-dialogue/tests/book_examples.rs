//! Integration tests for dialogue snippets from the mdBook getting-started chapters.

mod common;

use bubbles::{DialogueEvent, HashMapStorage, Runner, compile};
use common::drain;

const FIRST_DIALOGUE: &str = r"
title: Start
---
Narrator: Hi. I'm the narrator.
===
";

const EVENT_LOOP_ADVENTURE: &str = r"
title: Adventure
---
Narrator: Where do you go?
-> The forest.
    Narrator: You hear wolves.
-> The mountain.
    Narrator: The view is worth it.
===
";

#[test]
fn getting_started_first_dialogue_compiles_and_runs() {
    let prog = compile(FIRST_DIALOGUE).unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.start("Start").unwrap();
    let events = drain(&mut runner);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, DialogueEvent::Line { text, .. } if text.contains("narrator")))
    );
}

#[test]
fn getting_started_event_loop_compiles() {
    let prog = compile(EVENT_LOOP_ADVENTURE).unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.start("Adventure").unwrap();
    let mut saw_options = false;
    while let Some(ev) = runner.next_event().unwrap() {
        if let DialogueEvent::Options(opts) = ev {
            saw_options = true;
            let idx = opts
                .iter()
                .position(|o| o.available)
                .expect("available option");
            runner.select_option(idx).unwrap();
        }
    }
    assert!(saw_options);
}
