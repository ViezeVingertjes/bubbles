//! Integration tests for trailing #tag metadata on lines and commands.

mod common;

use bubbles::DialogueEvent;

#[test]
fn line_tags_emitted() {
    let src = "title: Start\n---\nHello. #greeting #important\n===\n";
    let events = common::play(src, "Start");
    let tags = events.iter().find_map(|e| {
        if let DialogueEvent::Line { tags, .. } = e { Some(tags.clone()) } else { None }
    });
    assert_eq!(tags.as_deref(), Some(["greeting".to_owned(), "important".to_owned()].as_slice()));
}

#[test]
fn command_tags_emitted() {
    let src = "title: Start\n---\n<<shake camera>> #vfx\n===\n";
    let events = common::play(src, "Start");
    let cmd_tags = events.iter().find_map(|e| {
        if let DialogueEvent::Command { tags, .. } = e { Some(tags.clone()) } else { None }
    });
    assert_eq!(cmd_tags.as_deref(), Some(["vfx".to_owned()].as_slice()));
}
