//! Shared test harness for integration tests.

use bubbles::{DialogueEvent, HashMapStorage, Runner, compile};

mod assertions;

/// Compiles `source` and drives the runner from `start_node` to completion,
/// collecting all events. Panics on any compile or runtime error.
pub fn play(source: &str, start_node: &str) -> Vec<DialogueEvent> {
    let prog = compile(source).expect("compile failed");
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.start(start_node).expect("start failed");
    collect(&mut runner)
}

/// Like [`play`] but allows customising the runner before starting (e.g. registering
/// functions, setting variables, installing saliency strategies).
pub fn play_with<F>(source: &str, start_node: &str, setup: F) -> Vec<DialogueEvent>
where
    F: FnOnce(&mut Runner<HashMapStorage>),
{
    let prog = compile(source).expect("compile failed");
    let mut runner = Runner::new(prog, HashMapStorage::new());
    setup(&mut runner);
    runner.start(start_node).expect("start failed");
    collect(&mut runner)
}

/// Loads a fixture file from `tests/fixtures/<name>.bub` and runs it.
pub fn play_fixture(name: &str, start_node: &str) -> Vec<DialogueEvent> {
    let path = format!("tests/fixtures/{name}.bub");
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("fixture not found: {path}"));
    play(&source, start_node)
}

fn collect(runner: &mut Runner<HashMapStorage>) -> Vec<DialogueEvent> {
    let mut events = Vec::new();
    loop {
        match runner.next_event().unwrap_or_else(|e| panic!("runtime error: {e}")) {
            Some(ev) => events.push(ev),
            None => break,
        }
    }
    events
}
