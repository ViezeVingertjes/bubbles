//! Integration tests for <<jump>> and <<command>> execution.

mod common;

use bubbles::DialogueEvent;

fn line_texts(events: &[DialogueEvent]) -> Vec<&str> {
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
            matches!(
                e,
                DialogueEvent::NodeStarted(_) | DialogueEvent::NodeComplete(_)
            )
        })
        .collect();
    assert!(node_events.len() >= 2);
}

// ── commands ──────────────────────────────────────────────────────────────────

#[test]
fn command_brace_interpolation_in_arguments() {
    let src = "\
title: Start
---
<<ping {1 + 2}>>
===
";
    let events = common::play(src, "Start");
    let cmds: Vec<_> = events
        .iter()
        .filter_map(|e| {
            if let DialogueEvent::Command { name, args, .. } = e {
                Some((name.as_str(), args.clone()))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0].0, "ping");
    assert_eq!(cmds[0].1, vec!["3".to_owned()]);
}

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
            if let DialogueEvent::Command { name, args, .. } = e {
                Some((name.as_str(), args))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0].0, "shake");
    assert_eq!(cmds[0].1, &["camera", "5"]);
}

#[test]
fn jump_line_may_carry_trailing_tags_after_command() {
    use bubbles::compile;
    let prog = compile("title: A\n---\n<<jump B #cut>>\n===\ntitle: B\n---\nOK\n===\n").unwrap();
    assert!(prog.node_exists("B"));
}

#[test]
fn detour_to_unknown_node_errors_at_runtime() {
    use bubbles::{HashMapStorage, Runner, compile};
    let prog = compile("title: A\n---\n<<detour Missing>>\n===\n").unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.start("A").unwrap();
    runner.next_event().unwrap();
    assert!(runner.next_event().is_err());
}

#[test]
fn command_no_args() {
    let src = "title: Start\n---\n<<fade_out>>\n===\n";
    let events = common::play(src, "Start");
    let cmds: Vec<_> = events
        .iter()
        .filter_map(|e| {
            if let DialogueEvent::Command { name, args, .. } = e {
                Some((name.as_str(), args.as_slice()))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(cmds, [("fade_out", [].as_slice())]);
}
