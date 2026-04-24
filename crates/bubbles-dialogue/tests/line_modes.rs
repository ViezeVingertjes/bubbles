//! Line mode metadata on line events.

use bubbles::{DialogueEvent, HashMapStorage, LineMode, Runner, compile};

#[test]
fn line_without_mode_tag_is_normal() {
    let src = "title: A\n---\nPlain.\n===\n";
    let prog = compile(src).unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.start("A").unwrap();
    let _ = runner.next_event(); // NodeStarted
    let DialogueEvent::Line { line_mode, .. } = runner.next_event().unwrap().unwrap() else {
        panic!("expected Line");
    };
    assert_eq!(line_mode, LineMode::Normal);
}

#[test]
fn narration_tag_sets_line_mode() {
    let src = "title: A\n---\nWide shot. #narration\n===\n";
    let prog = compile(src).unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.start("A").unwrap();
    let _ = runner.next_event();
    let DialogueEvent::Line {
        line_mode, tags, ..
    } = runner.next_event().unwrap().unwrap()
    else {
        panic!("expected Line");
    };
    assert_eq!(line_mode, LineMode::Narration);
    assert!(tags.contains(&"narration".into()));
}

#[test]
fn debug_tag_sets_line_mode_and_wins_over_narration() {
    let src = "title: A\n---\nQA note #narration #debug\n===\n";
    let prog = compile(src).unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.start("A").unwrap();
    let _ = runner.next_event();
    let DialogueEvent::Line { line_mode, .. } = runner.next_event().unwrap().unwrap() else {
        panic!("expected Line");
    };
    assert_eq!(line_mode, LineMode::Debug);
}
