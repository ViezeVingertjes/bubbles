//! Integration tests for trailing #tag metadata on lines and commands.

mod common;

use bubbles::{DialogueEvent, HashMapProvider, HashMapStorage, Runner, compile};

#[test]
fn line_tags_emitted() {
    let src = "title: Start\n---\nHello. #greeting #important\n===\n";
    let events = common::play(src, "Start");
    let tags = events.iter().find_map(|e| {
        if let DialogueEvent::Line { tags, .. } = e {
            Some(tags.clone())
        } else {
            None
        }
    });
    assert_eq!(
        tags.as_deref(),
        Some(["greeting".to_owned(), "important".to_owned()].as_slice())
    );
}

#[test]
fn line_prefix_tag_uses_provider_for_text() {
    let prog = compile("title: A\n---\nHi. #line:abc\n===\n").unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    let mut provider = HashMapProvider::new();
    provider.insert("abc", "Hola");
    runner.set_provider(provider);
    runner.start("A").unwrap();
    let mut line = None;
    while let Some(ev) = runner.next_event().unwrap() {
        if let DialogueEvent::Line { text, .. } = ev {
            line = Some(text);
            break;
        }
    }
    assert_eq!(line.as_deref(), Some("Hola"));
}

#[test]
fn line_prefix_not_in_provider_keeps_interpolated_source() {
    let prog = compile("title: A\n---\nHello. #line:missing\n===\n").unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.set_provider(HashMapProvider::new());
    runner.start("A").unwrap();
    let mut line = None;
    while let Some(ev) = runner.next_event().unwrap() {
        if let DialogueEvent::Line { text, .. } = ev {
            line = Some(text);
            break;
        }
    }
    assert_eq!(line.as_deref(), Some("Hello."));
}

#[test]
fn command_tags_emitted() {
    let src = "title: Start\n---\n<<shake camera>> #vfx\n===\n";
    let events = common::play(src, "Start");
    let cmd_tags = events.iter().find_map(|e| {
        if let DialogueEvent::Command { tags, .. } = e {
            Some(tags.clone())
        } else {
            None
        }
    });
    assert_eq!(cmd_tags.as_deref(), Some(["vfx".to_owned()].as_slice()));
}
