//! Runner session hygiene: `start` clears stale queues, and read-only introspection.

mod common;

use bubbles::{DialogueError, DialogueEvent, HashMapStorage, Runner, RunnerPhase, compile};

#[test]
fn start_clears_queued_dialogue_complete() {
    let prog = compile(
        "title: A\n---\nHi\n===\n\
         title: B\n---\nBye\n===\n",
    )
    .unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.start("A").unwrap();
    loop {
        match runner.next_event().unwrap() {
            Some(DialogueEvent::NodeComplete(_)) => break,
            Some(_) => {}
            None => panic!("expected NodeComplete before stream end"),
        }
    }
    assert_eq!(runner.phase(), RunnerPhase::Done);
    runner.start("B").unwrap();
    assert_eq!(runner.phase(), RunnerPhase::Running);
    match runner.next_event().unwrap() {
        Some(DialogueEvent::NodeStarted(n)) => assert_eq!(n, "B"),
        other => panic!("expected NodeStarted(B), got {other:?}"),
    }
}

#[test]
fn start_after_options_without_select_abandons_choice() {
    let prog = compile(
        "title: A\n---\nQ?\n-> X\n  old\n===\n\
         title: B\n---\nFresh\n===\n",
    )
    .unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.start("A").unwrap();
    loop {
        match runner.next_event().unwrap() {
            Some(DialogueEvent::Options(_)) => break,
            Some(_) => {}
            None => panic!("expected Options"),
        }
    }
    assert_eq!(runner.phase(), RunnerPhase::AwaitingOption);
    runner.start("B").unwrap();
    assert_eq!(runner.phase(), RunnerPhase::Running);
    assert!(matches!(
        runner.select_option(0),
        Err(DialogueError::ProtocolViolation(_))
    ));
    match runner.next_event().unwrap() {
        Some(DialogueEvent::NodeStarted(n)) => assert_eq!(n, "B"),
        other => panic!("expected NodeStarted(B), got {other:?}"),
    }
    match runner.next_event().unwrap() {
        Some(DialogueEvent::Line { text, .. }) => assert_eq!(text, "Fresh"),
        other => panic!("expected Line Fresh, got {other:?}"),
    }
}

#[test]
fn program_accessor_matches_compiled_program() {
    let prog = compile("title: Only\n---\n===\n").unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    assert!(runner.program().node_exists("Only"));
    assert!(!runner.program().node_exists("Missing"));
    runner.start("Only").unwrap();
    assert!(runner.program().node_exists("Only"));
}

#[test]
fn phase_tracks_idle_running_awaiting_done() {
    let prog = compile("title: S\n---\nPick?\n-> A\n  done\n===\n").unwrap();
    let mut r = Runner::new(prog, HashMapStorage::new());
    assert_eq!(r.phase(), RunnerPhase::Idle);
    r.start("S").unwrap();
    assert_eq!(r.phase(), RunnerPhase::Running);
    loop {
        match r.next_event().unwrap() {
            Some(DialogueEvent::Options(_)) => {
                assert_eq!(r.phase(), RunnerPhase::AwaitingOption);
                break;
            }
            Some(_) => {}
            None => panic!("expected Options"),
        }
    }
    r.select_option(0).unwrap();
    assert_eq!(r.phase(), RunnerPhase::Running);
    while r.next_event().unwrap().is_some() {}
    assert_eq!(r.phase(), RunnerPhase::Done);
    assert_eq!(r.next_event().unwrap(), None);
}
