//! Snapshot and restore without the `serde` feature.

use std::collections::{HashMap, HashSet};

use bubbles::{DialogueEvent, HashMapStorage, Runner, RunnerSnapshot, compile};

#[test]
fn snapshot_restore_round_trip_without_serde() {
    let src = "\
title: A
---
Line one.
===
title: B
---
Line two.
===
";
    let prog = compile(src).unwrap();
    let mut runner = Runner::new(prog.clone(), HashMapStorage::new());
    runner.start("A").unwrap();
    assert!(matches!(
        runner.next_event().unwrap(),
        Some(DialogueEvent::NodeStarted(_))
    ));
    assert!(matches!(
        runner.next_event().unwrap(),
        Some(DialogueEvent::Line { .. })
    ));

    let snap = runner.snapshot();
    assert_eq!(
        snap,
        RunnerSnapshot {
            current_node: Some("A".into()),
            visits: HashMap::from([("A".into(), 1)]),
            once_seen: HashSet::new(),
        }
    );

    let mut runner2 = Runner::new(prog, HashMapStorage::new());
    runner2.restore(snap).unwrap();
    assert!(matches!(
        runner2.next_event().unwrap(),
        Some(DialogueEvent::NodeStarted(_))
    ));
    let DialogueEvent::Line { text, .. } = runner2.next_event().unwrap().unwrap() else {
        panic!("expected Line");
    };
    assert_eq!(text, "Line one.");
}

#[test]
fn restore_unknown_node_errors() {
    let prog = compile("title: Only\n---\nHi.\n===\n").unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    let bad = RunnerSnapshot {
        current_node: Some("Missing".into()),
        visits: HashMap::new(),
        once_seen: HashSet::new(),
    };
    assert!(runner.restore(bad).is_err());
}
