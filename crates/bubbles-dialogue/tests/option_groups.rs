//! Option groups for UI constraints (radio buttons, mutually exclusive sets).

use bubbles::{DialogueEvent, HashMapStorage, Runner, compile};

#[test]
fn option_group_from_tag_extraction() {
    let src = "\
title: Start
---
Choose a faction:
-> Join Reds #group:faction
-> Join Blues #group:faction
-> Stay neutral #group:meta
===
";
    let prog = compile(src).unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.start("Start").unwrap();
    let _ = runner.next_event(); // NodeStarted
    let _ = runner.next_event(); // Line
    let DialogueEvent::Options(opts) = runner.next_event().unwrap().unwrap() else {
        panic!("expected Options");
    };
    assert_eq!(opts.len(), 3);
    assert_eq!(opts[0].group, Some("faction".into()));
    assert_eq!(opts[1].group, Some("faction".into()));
    assert_eq!(opts[2].group, Some("meta".into()));
}

#[test]
fn option_without_group_has_none() {
    let src = "\
title: Start
---
-> Yes
-> No
===
";
    let prog = compile(src).unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.start("Start").unwrap();
    let _ = runner.next_event();
    let DialogueEvent::Options(opts) = runner.next_event().unwrap().unwrap() else {
        panic!("expected Options");
    };
    assert!(opts[0].group.is_none());
    assert!(opts[1].group.is_none());
}

#[test]
fn group_remains_in_tags() {
    let src = "title: S\n---\n-> Option #group:test #style\n===\n";
    let prog = compile(src).unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.start("S").unwrap();
    let _ = runner.next_event();
    let DialogueEvent::Options(opts) = runner.next_event().unwrap().unwrap() else {
        panic!("expected Options");
    };
    assert_eq!(opts[0].group, Some("test".into()));
    assert!(opts[0].tags.contains(&"group:test".into()));
    assert!(opts[0].tags.contains(&"style".into()));
}
