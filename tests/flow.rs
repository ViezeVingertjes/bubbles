//! Integration tests for runner flow: node start, completion, and dialogue end.

use bubbles::{DialogueEvent, HashMapStorage, Runner, compile};

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
