//! Integration tests for <<jump>> and <<command>> execution.

mod common;

use bubbles::DialogueEvent;

use common::line_texts;

// ── jumps ─────────────────────────────────────────────────────────────────────

#[test]
fn jump_transitions_to_node() {
    let events = common::play_fixture("jump", "Start");
    assert_eq!(line_texts(&events), ["Before jump.", "After jump."]);
}

#[test]
fn jump_emits_node_events() {
    let events = common::play_fixture("jump", "Start");
    let node_event_count = events
        .iter()
        .filter(|e| {
            matches!(
                e,
                DialogueEvent::NodeStarted(_) | DialogueEvent::NodeComplete(_)
            )
        })
        .count();
    assert!(node_event_count >= 2);
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
fn jump_with_trailing_tag_is_rejected_at_compile_time() {
    // The parser includes the trailing `#cut` in the jump target name,
    // producing an unknown-node reference that compile() catches during
    // validation.  This test documents that behaviour: authors must not
    // put tags on jump lines.
    use bubbles::{DialogueError, compile};
    let err = compile("title: A\n---\n<<jump B #cut>>\n===\ntitle: B\n---\nOK\n===\n").unwrap_err();
    assert!(
        matches!(err, DialogueError::Validation(_)),
        "expected Validation error, got: {err:?}"
    );
}

#[test]
fn detour_to_unknown_node_errors_at_compile_time() {
    use bubbles::{DialogueError, compile};
    let err = compile("title: A\n---\n<<detour Missing>>\n===\n").unwrap_err();
    assert!(
        matches!(err, DialogueError::Validation(_)),
        "expected Validation error, got: {err:?}"
    );
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
