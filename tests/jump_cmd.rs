//! Integration tests for <<jump>> and <<command>> execution.

mod common;

use bubbles::DialogueEvent;

fn line_texts(events: &[DialogueEvent]) -> Vec<&str> {
    events
        .iter()
        .filter_map(|e| {
            if let DialogueEvent::Line { text, .. } = e { Some(text.as_str()) } else { None }
        })
        .collect()
}

// ── jumps ─────────────────────────────────────────────────────────────────────

#[test]
fn jump_transitions_to_node() {
    let events = common::play_fixture("jump", "Start");
    assert_eq!(line_texts(&events), ["Before jump.", "After jump."]);
}

#[test]
fn jump_emits_node_events() {
    let events = common::play_fixture("jump", "Start");
    let node_events: Vec<_> = events
        .iter()
        .filter(|e| {
            matches!(e, DialogueEvent::NodeStarted(_) | DialogueEvent::NodeComplete(_))
        })
        .collect();
    assert!(node_events.len() >= 2);
}

// ── commands ──────────────────────────────────────────────────────────────────

#[test]
fn command_emitted_with_args() {
    let src = "\
title: Start
---
<<shake camera 5>>
===
";
    let events = common::play(src, "Start");
    let cmds: Vec<_> = events
        .iter()
        .filter_map(|e| {
            if let DialogueEvent::Command { name, args, .. } = e { Some((name.as_str(), args)) } else { None }
        })
        .collect();
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0].0, "shake");
    assert_eq!(cmds[0].1, &["camera", "5"]);
}

#[test]
fn command_no_args() {
    let src = "title: Start\n---\n<<fade_out>>\n===\n";
    let events = common::play(src, "Start");
    let cmds: Vec<_> = events
        .iter()
        .filter_map(|e| {
            if let DialogueEvent::Command { name, args, .. } = e { Some((name.as_str(), args.as_slice())) } else { None }
        })
        .collect();
    assert_eq!(cmds, [("fade_out", [].as_slice())]);
}
