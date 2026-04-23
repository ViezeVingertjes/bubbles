//! Shared test harness for integration tests.
//!
//! Each `tests/*.rs` file is compiled as its own binary and sees the *entire*
//! `common` module, but only uses a subset. The `dead_code` allow is therefore
//! intentional: unused helpers here are NOT dead code across the full suite.

#![allow(dead_code)]

use bubbles::{DialogueEvent, HashMapStorage, Runner, compile};

mod assertions;

/// Compiles `source` and drives the runner from `start_node` to completion,
/// collecting all events. Panics on any compile or runtime error.
pub fn play(source: &str, start_node: &str) -> Vec<DialogueEvent> {
    let prog = compile(source).expect("compile failed");
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.start(start_node).expect("start failed");
    drain(&mut runner)
}

/// Loads a fixture file from `tests/fixtures/<name>.bub` and runs it.
pub fn play_fixture(name: &str, start_node: &str) -> Vec<DialogueEvent> {
    let path = format!("tests/fixtures/{name}.bub");
    let source =
        std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("fixture not found: {path}"));
    play(&source, start_node)
}

/// Drains every remaining event from `runner`, panicking on any runtime error.
///
/// This is the shared counterpart to the ad-hoc `drain(runner)` helpers that
/// appear in many integration tests; new tests should prefer this one.
pub fn drain(runner: &mut Runner<HashMapStorage>) -> Vec<DialogueEvent> {
    let mut events = Vec::new();
    while let Some(ev) = runner
        .next_event()
        .unwrap_or_else(|e| panic!("runtime error: {e}"))
    {
        events.push(ev);
    }
    events
}

/// Extracts the text of every [`DialogueEvent::Line`] in order.
///
/// Returns borrowed slices into `events`; most test comparisons work with
/// `&str` literals, making this more ergonomic than an owned-`String` variant.
pub fn line_texts(events: &[DialogueEvent]) -> Vec<&str> {
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
