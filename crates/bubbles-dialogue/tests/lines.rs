//! Integration tests for line events.

mod common;

use bubbles::DialogueEvent;

#[test]
fn plain_lines_emitted_in_order() {
    let events = common::play_fixture("lines", "Start");
    assert_eq!(
        events,
        vec![
            DialogueEvent::NodeStarted("Start".into()),
            DialogueEvent::Line {
                speaker: None,
                text: "Hello there.".into(),
                line_id: None,
                tags: vec![],
                spans: vec![],
            },
            DialogueEvent::Line {
                speaker: Some("Alice".into()),
                text: "Hi, how are you?".into(),
                line_id: None,
                tags: vec![],
                spans: vec![],
            },
            DialogueEvent::NodeComplete("Start".into()),
            DialogueEvent::DialogueComplete,
        ]
    );
}
