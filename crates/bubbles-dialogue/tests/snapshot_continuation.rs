//! Property tests for snapshot/restore continuation, which is what replay and
//! save-state correctness rest on.
//!
//! The gap: `tests/snapshots.rs` checks that a snapshot round-trips to an equal
//! value and that the runner resumes in the right node. Nothing checked the
//! property a save file actually has to satisfy — that **the rest of the
//! dialogue plays out the same way** after a restore as it would have without
//! one. An equal snapshot with a divergent continuation would have passed the
//! old tests.
//!
//! `restore` resumes at the *start* of the recorded node, not at the exact
//! statement, so every snapshot here is taken at a node boundary. That is the
//! granularity the API offers and therefore the granularity the property is
//! stated at.
//!
//! Storage is captured alongside the snapshot because `RunnerSnapshot`
//! deliberately excludes variables; a restore given only the snapshot would
//! diverge for any script that reads a variable, which is a documented host
//! responsibility rather than a runtime defect.

use std::fmt::Write as _;

use bubbles::{
    DialogueEvent, HashMapStorage, Program, Runner, RunnerSnapshot, Value, VariableStorage, compile,
};
use proptest::prelude::*;

/// Branching script with visits, `<<once>>`, variables and options, but no
/// `visited_count` interpolation - see `visit_count_is_re_incremented_on_restore`
/// for why that case is called out separately.
const SCRIPT: &str = "\
title: Start
---
<<declare $gold = 12>>
<<declare $trust = 0>>
Innkeeper: Evening.
<<once>>
    Innkeeper: New face.
<<endonce>>
-> Take a room.
    <<set $gold = $gold - 5>>
    <<jump Room>>
-> Ask about the harbour.
    <<set $trust = $trust + 1>>
    <<jump Harbour>>
===
title: Room
---
Innkeeper: Up the stairs. {$gold} gold left.
<<if $gold > 5>>
    Innkeeper: And a meal with it.
<<endif>>
<<jump Harbour>>
===
title: Harbour
---
Clerk: The tide turns at dawn.
<<once>>
    Clerk: Take this chart.
<<endonce>>
-> Sail.
    <<set $trust = $trust + 1>>
    <<jump Leave>>
-> Wait.
    <<jump Leave>>
===
title: Leave
---
Narrator: Trust stands at {$trust}. Gold at {$gold}.
===
";

fn render(events: &[DialogueEvent]) -> String {
    let mut out = String::new();
    for event in events {
        match event {
            DialogueEvent::NodeStarted(n) => {
                let _ = writeln!(out, "NodeStarted({n})");
            }
            DialogueEvent::NodeComplete(n) => {
                let _ = writeln!(out, "NodeComplete({n})");
            }
            DialogueEvent::Line { speaker, text, .. } => {
                let _ = writeln!(out, "Line {speaker:?} {text:?}");
            }
            DialogueEvent::Options(options) => {
                out.push_str("Options(\n");
                for opt in options {
                    let _ = writeln!(out, "  {:?} available={}", opt.text, opt.available);
                }
                out.push_str(")\n");
            }
            DialogueEvent::Command { name, args, .. } => {
                let _ = writeln!(out, "Command {name:?} {args:?}");
            }
            DialogueEvent::DialogueComplete => out.push_str("DialogueComplete\n"),
            other => {
                let _ = writeln!(out, "{other:?}");
            }
        }
    }
    out
}

/// A saved game: the runtime snapshot plus the host-owned variable storage.
/// Neither half is a complete save on its own, which is what the guide says.
#[derive(Clone)]
struct Save {
    snapshot: RunnerSnapshot,
    storage: HashMapStorage,
}

/// Drives `runner` to completion, taking choices from `choices` starting at
/// `choice_index`, and calls `on_node_start` as each node begins so a caller can
/// capture a save at that boundary.
fn drive(
    runner: &mut Runner<HashMapStorage>,
    choices: &[usize],
    choice_index: &mut usize,
    mut on_node_start: impl FnMut(usize, &Runner<HashMapStorage>, usize),
) -> Vec<DialogueEvent> {
    let mut events = Vec::new();
    for _ in 0..500 {
        let Some(event) = runner.next_event().unwrap() else {
            break;
        };
        if matches!(event, DialogueEvent::NodeStarted(_)) {
            on_node_start(events.len(), runner, *choice_index);
        }
        let available: Vec<usize> = if let DialogueEvent::Options(ref options) = event {
            options
                .iter()
                .enumerate()
                .filter(|(_, o)| o.available)
                .map(|(i, _)| i)
                .collect()
        } else {
            Vec::new()
        };
        events.push(event);
        if !available.is_empty() {
            let pick = available[choices[*choice_index % choices.len()] % available.len()];
            *choice_index += 1;
            runner.select_option(pick).unwrap();
        }
    }
    events
}

fn fresh(program: &Program) -> Runner<HashMapStorage> {
    let mut runner = Runner::new(program.clone(), HashMapStorage::new());
    runner.start("Start").unwrap();
    runner
}

proptest! {
    /// Restoring a save taken at a node boundary must replay the remainder of the
    /// dialogue exactly as the uninterrupted run produced it.
    #[test]
    fn restore_at_node_boundary_continues_identically(
        choices in proptest::collection::vec(0usize..4, 1..5),
    ) {
        let program = compile(SCRIPT).unwrap();

        // Baseline run, capturing a save at every node boundary.
        let mut saves: Vec<(usize, Save, usize)> = Vec::new();
        let mut runner = fresh(&program);
        let mut index = 0;
        let baseline = drive(&mut runner, &choices, &mut index, |at, r, choice_index| {
            saves.push((
                at,
                Save {
                    snapshot: r.snapshot(),
                    storage: r.storage().clone(),
                },
                choice_index,
            ));
        });

        prop_assert!(!saves.is_empty(), "script produced no node boundaries");

        for (at, save, choice_index) in saves {
            let mut restored = Runner::new(program.clone(), save.storage.clone());
            restored.restore(save.snapshot.clone()).unwrap();
            let mut idx = choice_index;
            let tail = drive(&mut restored, &choices, &mut idx, |_, _, _| {});

            prop_assert_eq!(
                render(&tail),
                render(&baseline[at..]),
                "continuation diverged after restoring at event {}", at
            );
        }
    }

    /// A save restored twice must produce the same continuation both times:
    /// restoring must not consume or mutate the state it was given.
    #[test]
    fn restore_is_repeatable(choices in proptest::collection::vec(0usize..4, 1..5)) {
        let program = compile(SCRIPT).unwrap();
        let mut runner = fresh(&program);
        let mut index = 0;
        let mut first_save: Option<Save> = None;
        let _ = drive(&mut runner, &choices, &mut index, |_, r, _| {
            if first_save.is_none() {
                first_save = Some(Save {
                    snapshot: r.snapshot(),
                    storage: r.storage().clone(),
                });
            }
        });
        let save = first_save.unwrap();

        let mut a = Runner::new(program.clone(), save.storage.clone());
        a.restore(save.snapshot.clone()).unwrap();
        let mut ia = 0;
        let first = drive(&mut a, &choices, &mut ia, |_, _, _| {});

        let mut b = Runner::new(program, save.storage.clone());
        b.restore(save.snapshot).unwrap();
        let mut ib = 0;
        let second = drive(&mut b, &choices, &mut ib, |_, _, _| {});

        prop_assert_eq!(render(&first), render(&second));
    }
}

/// Restoring re-runs the recorded node from its start, and the runner counts
/// that as another visit — so `visited_count` of the restored node reads one
/// higher than it did in the uninterrupted run.
///
/// This is a direct consequence of the documented "restart from the beginning of
/// this node" behaviour rather than a separate bug, but it means a replay is not
/// faithful for any script that branches on `visited_count` of the node it was
/// saved in. Pinned here so the behaviour is a recorded decision instead of a
/// surprise, and so a future change to it is deliberate.
#[test]
fn visit_count_is_re_incremented_on_restore() {
    let src = "\
title: Start
---
Clerk: Visit number {visited_count(\"Start\")}.
===
";
    let program = compile(src).unwrap();

    let mut runner = Runner::new(program.clone(), HashMapStorage::new());
    runner.start("Start").unwrap();
    let _ = runner.next_event().unwrap(); // NodeStarted
    let snapshot = runner.snapshot();
    let line = loop {
        match runner.next_event().unwrap() {
            Some(DialogueEvent::Line { text, .. }) => break text,
            Some(_) => {}
            None => panic!("no line"),
        }
    };
    assert_eq!(line, "Visit number 1.");

    let mut restored = Runner::new(program, HashMapStorage::new());
    restored.restore(snapshot).unwrap();
    let restored_line = loop {
        match restored.next_event().unwrap() {
            Some(DialogueEvent::Line { text, .. }) => break text,
            Some(_) => {}
            None => panic!("no line"),
        }
    };
    assert_eq!(
        restored_line, "Visit number 2.",
        "restore re-enters the node, so the visit is counted again"
    );
}

/// Storage is not part of the snapshot, so a restore that forgets to carry
/// variables across diverges. Documented host responsibility - pinned so the
/// division of labour is visible in the test suite rather than only in prose.
#[test]
fn restore_without_storage_diverges() {
    let program = compile(SCRIPT).unwrap();
    let mut runner = fresh(&program);
    let mut index = 0;
    let mut save: Option<Save> = None;
    let _ = drive(&mut runner, &[0], &mut index, |_, r, _| {
        if save.is_none() && r.snapshot().current_node.as_deref() == Some("Start") {
            let mut storage = r.storage().clone();
            storage.set("$gold", Value::Number(999.0));
            save = Some(Save {
                snapshot: r.snapshot(),
                storage,
            });
        }
    });
    let save = save.expect("no save captured");

    let mut with_vars = Runner::new(program.clone(), save.storage.clone());
    with_vars.restore(save.snapshot.clone()).unwrap();
    let mut i1 = 0;
    let with = drive(&mut with_vars, &[0], &mut i1, |_, _, _| {});

    let mut without_vars = Runner::new(program, HashMapStorage::new());
    without_vars.restore(save.snapshot).unwrap();
    let mut i2 = 0;
    let without = drive(&mut without_vars, &[0], &mut i2, |_, _, _| {});

    assert_ne!(
        render(&with),
        render(&without),
        "dropping storage should change the run; if this passes, the script stopped reading variables"
    );
}
