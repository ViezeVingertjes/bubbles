//! Integration tests for <<once>> blocks.

use bubbles::{DialogueEvent, HashMapStorage, Runner, compile};

fn line_texts(events: &[DialogueEvent]) -> Vec<String> {
    events
        .iter()
        .filter_map(|e| {
            if let DialogueEvent::Line { text, .. } = e {
                Some(text.clone())
            } else {
                None
            }
        })
        .collect()
}

fn play_to_end(runner: &mut Runner<HashMapStorage>) -> Vec<DialogueEvent> {
    let mut events = Vec::new();
    loop {
        match runner.next_event().unwrap() {
            Some(ev) => {
                let done = ev == DialogueEvent::DialogueComplete;
                events.push(ev);
                if done {
                    break;
                }
            }
            None => break,
        }
    }
    events
}

#[test]
fn once_runs_first_time_only() {
    let src = "\
title: Start
---
<<once>>
    First-time line.
<<endonce>>
Done.
===
";
    let prog = compile(src).unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());

    runner.start("Start").unwrap();
    let events1 = play_to_end(&mut runner);
    assert_eq!(line_texts(&events1), ["First-time line.", "Done."]);

    runner.start("Start").unwrap();
    let events2 = play_to_end(&mut runner);
    // once block should not run the second time
    assert_eq!(line_texts(&events2), ["Done."]);
}

#[test]
fn once_else_runs_after_first() {
    let src = "\
title: Start
---
<<once>>
    Intro.
<<else>>
    Reminder.
<<endonce>>
===
";
    let prog = compile(src).unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());

    runner.start("Start").unwrap();
    let ev1 = play_to_end(&mut runner);
    assert_eq!(line_texts(&ev1), ["Intro."]);

    runner.start("Start").unwrap();
    let ev2 = play_to_end(&mut runner);
    assert_eq!(line_texts(&ev2), ["Reminder."]);
}
