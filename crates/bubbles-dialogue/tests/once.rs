//! Integration tests for <<once>> blocks.

mod common;

use bubbles::{HashMapStorage, Runner, compile};

use common::{drain, line_texts};

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
    let events1 = drain(&mut runner);
    assert_eq!(line_texts(&events1), ["First-time line.", "Done."]);

    runner.start("Start").unwrap();
    let events2 = drain(&mut runner);
    // once block should not run the second time
    assert_eq!(line_texts(&events2), ["Done."]);
}

#[test]
fn once_if_skips_body_when_condition_false() {
    let src = "\
title: Start
---
<<once if false>>
    Secret.
<<endonce>>
After.
===
";
    let prog = compile(src).unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.start("Start").unwrap();
    let events = drain(&mut runner);
    assert_eq!(line_texts(&events), ["After."]);
}

#[test]
fn once_in_detoured_node_persists_across_calls() {
    // Detour twice into the same node that contains a <<once>> block. The
    // block should fire exactly once across the two entries (it is tracked
    // by stable block id, which must remain unique after the Arc<[Stmt]>
    // refactor).
    let src = "\
title: Start
---
<<detour Greet>>
<<detour Greet>>
===
title: Greet
---
<<once>>
    First greeting.
<<else>>
    Welcome back.
<<endonce>>
===
";
    let prog = compile(src).unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.start("Start").unwrap();
    let events = drain(&mut runner);
    assert_eq!(line_texts(&events), ["First greeting.", "Welcome back."]);
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
    let ev1 = drain(&mut runner);
    assert_eq!(line_texts(&ev1), ["Intro."]);

    runner.start("Start").unwrap();
    let ev2 = drain(&mut runner);
    assert_eq!(line_texts(&ev2), ["Reminder."]);
}
