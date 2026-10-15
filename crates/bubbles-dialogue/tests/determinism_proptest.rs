//! Property tests for the determinism guarantee: identical script plus identical
//! inputs must produce an identical event stream.
//!
//! The gap these close is that nothing previously asserted reproducibility at
//! all. The runtime keeps session state in `HashMap`/`HashSet`
//! (`Runner::visits`, `Runner::once_seen`, `BestLeastRecentlyViewed::last_seen`),
//! and `std`'s hasher is randomly seeded, so a future change that iterated any of
//! those while building an event would make the event stream vary between runs
//! with nothing to catch it. Every run below therefore compares **separately
//! constructed** runners rather than reusing one: `RandomState::new()` yields a
//! different hash order per instance, so two runners in one process already
//! disagree about iteration order. If ordering ever reaches an event, these fail.
//!
//! What is deliberately *not* asserted here: anything touching `random()`,
//! `random_range()`, `dice()`, or `RandomAvailable`. Those read a thread-local
//! OS-seeded RNG and take no seed, so they are not reproducible and no test can
//! pretend otherwise.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use bubbles::{
    BestLeastRecentlyViewed, DialogueEvent, HashMapStorage, Program, Runner, Value,
    VariableStorage, compile,
};
use proptest::prelude::*;

/// A script exercising the constructs whose ordering could plausibly vary:
/// visit counts, `<<once>>` ids, conditionals over variables, interpolation of
/// both text and numbers, and options.
const SCRIPT: &str = "\
title: Start
---
<<declare $gold = 10>>
<<declare $name = \"traveller\">>
<<declare $trust = 0>>
Innkeeper: Welcome, {$name}.
<<once>>
    Innkeeper: First time here, I see.
<<endonce>>
<<if $gold > 5>>
    Innkeeper: You can afford a room. {$gold} gold buys a week.
<<else>>
    Innkeeper: Times are hard, {$name}.
<<endif>>
-> Take a room.
    <<set $gold = $gold - 5>>
    <<set $trust = $trust + 1>>
    <<jump Room>>
-> Ask about the harbour.
    <<set $trust = $trust + 1>>
    <<jump Harbour>>
-> Say nothing.
    <<jump Leave>>
===
title: Room
---
Innkeeper: Up the stairs. You have {$gold} left.
<<if visited(\"Harbour\")>>
    Innkeeper: Mind what they told you at the harbour.
<<endif>>
<<jump Leave>>
===
title: Harbour
---
Clerk: The tide turns at dawn. Visit {visited_count(\"Harbour\")} so far.
<<once>>
    Clerk: Here, take this chart.
<<endonce>>
-> Back to the inn.
    <<jump Start>>
-> Leave the harbour.
    <<jump Leave>>
===
title: Leave
---
Narrator: You step outside. Trust stands at {$trust}.
===
";

/// Renders an event stream to a single canonical string.
///
/// Comparing rendered text rather than `Vec<DialogueEvent>` means a failure
/// prints a readable diff, and it is also the closest thing to the "byte-identical
/// stream" the guarantee is phrased in terms of.
fn render(events: &[DialogueEvent]) -> String {
    let mut out = String::new();
    for event in events {
        match event {
            DialogueEvent::NodeStarted(node) => {
                let _ = writeln!(out, "NodeStarted({node})");
            }
            DialogueEvent::NodeComplete(node) => {
                let _ = writeln!(out, "NodeComplete({node})");
            }
            DialogueEvent::Line {
                speaker,
                text,
                line_id,
                tags,
                line_mode,
                spans,
            } => {
                let _ = writeln!(
                    out,
                    "Line speaker={speaker:?} text={text:?} id={line_id:?} \
                     tags={tags:?} mode={line_mode:?} spans={spans:?}"
                );
            }
            DialogueEvent::Options(options) => {
                out.push_str("Options(\n");
                for opt in options {
                    let _ = writeln!(
                        out,
                        "  text={:?} available={} spans={:?}",
                        opt.text, opt.available, opt.spans
                    );
                }
                out.push_str(")\n");
            }
            DialogueEvent::Command { name, args, tags } => {
                let _ = writeln!(out, "Command name={name:?} args={args:?} tags={tags:?}");
            }
            DialogueEvent::DialogueComplete => out.push_str("DialogueComplete\n"),
            other => {
                let _ = writeln!(out, "{other:?}");
            }
        }
    }
    out
}

/// The outcome of a run: the events it produced, and the error that stopped it
/// if one did.
///
/// Errors are part of what must be reproducible. A host that replays a session
/// needs the *same* failure at the same point, so a run that ends in
/// `TypeMismatch` is as much a determinism case as one that completes — and
/// capturing it here is what lets the property tests feed in ill-typed inputs
/// instead of having to avoid them.
struct Run {
    events: Vec<DialogueEvent>,
    error: Option<String>,
}

/// Drives a fresh runner through `program`, taking option choices from `choices`
/// (cycled, so a short choice list still drives a long dialogue).
///
/// `vars` seeds storage before the run, standing in for host-supplied inputs.
fn run(
    program: &Program,
    start: &str,
    vars: &[(String, Value)],
    choices: &[usize],
    blrv: bool,
) -> Run {
    let mut storage = HashMapStorage::new();
    for (name, value) in vars {
        storage.set(name, value.clone());
    }
    let mut runner = if blrv {
        bubbles::RunnerBuilder::new(program.clone(), storage)
            .with_saliency(BestLeastRecentlyViewed::new())
            .build()
    } else {
        Runner::new(program.clone(), storage)
    };
    runner.start(start).unwrap();

    let mut events = Vec::new();
    let mut taken = 0;
    // A jump cycle (Start -> Harbour -> Start) can run forever if every choice
    // keeps returning, so the loop is bounded. The bound is a safety net, not a
    // tuning knob: both compared runs stop at the same place.
    for _ in 0..2_000 {
        let event = match runner.next_event() {
            Ok(Some(event)) => event,
            Ok(None) => break,
            Err(e) => {
                return Run {
                    events,
                    error: Some(e.to_string()),
                };
            }
        };
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
            let pick = available[choices[taken % choices.len()] % available.len()];
            taken += 1;
            runner.select_option(pick).unwrap();
        }
    }
    Run {
        events,
        error: None,
    }
}

/// Renders a whole run, error included, to one comparable string.
fn render_run(run: &Run) -> String {
    let mut out = render(&run.events);
    if let Some(error) = &run.error {
        let _ = writeln!(out, "Error({error})");
    }
    out
}

fn var_strategy() -> impl Strategy<Value = Vec<(String, Value)>> {
    let value = prop_oneof![
        (-1000.0f64..1000.0).prop_map(Value::Number),
        "[a-zA-Z ]{0,12}".prop_map(Value::Text),
        any::<bool>().prop_map(Value::Bool),
    ];
    proptest::collection::vec(
        (
            prop_oneof![
                Just("$gold".to_owned()),
                Just("$name".to_owned()),
                Just("$trust".to_owned()),
            ],
            value,
        ),
        0..4,
    )
}

proptest! {
    /// Two independently built runners, same script and same inputs, must emit
    /// the same stream. Each runner has its own hash seeds, so this fails the
    /// moment map iteration order influences an event.
    #[test]
    fn same_inputs_produce_same_event_stream(
        vars in var_strategy(),
        choices in proptest::collection::vec(0usize..8, 1..6),
    ) {
        let program = compile(SCRIPT).unwrap();
        let first = run(&program, "Start", &vars, &choices, false);
        let second = run(&program, "Start", &vars, &choices, false);
        prop_assert_eq!(render_run(&first), render_run(&second));
    }

    /// The same, with the stateful saliency strategy selected. `BestLeastRecentlyViewed`
    /// keeps a `HashMap` of its own, so it gets its own case.
    #[test]
    fn blrv_saliency_is_reproducible(
        vars in var_strategy(),
        choices in proptest::collection::vec(0usize..8, 1..6),
    ) {
        let program = compile(SCRIPT).unwrap();
        let first = run(&program, "Start", &vars, &choices, true);
        let second = run(&program, "Start", &vars, &choices, true);
        prop_assert_eq!(render_run(&first), render_run(&second));
    }

    /// Recompiling the same source must give a program that behaves identically.
    /// Compilation interns node titles and builds the node table, so this covers
    /// ordering introduced at compile time rather than at run time.
    #[test]
    fn recompiling_does_not_change_behaviour(
        vars in var_strategy(),
        choices in proptest::collection::vec(0usize..8, 1..6),
    ) {
        let first = run(&compile(SCRIPT).unwrap(), "Start", &vars, &choices, false);
        let second = run(&compile(SCRIPT).unwrap(), "Start", &vars, &choices, false);
        prop_assert_eq!(render_run(&first), render_run(&second));
    }

    /// Number formatting in interpolation must not depend on the platform's
    /// locale or `printf`. Rust's `f64` `Display` is shortest-round-trip and
    /// target-independent; this pins that the runtime does not reintroduce a
    /// locale-sensitive path, and that whole numbers render without a `.0`.
    #[test]
    fn number_interpolation_is_stable(n in -1.0e12f64..1.0e12) {
        let src = "title: N\n---\nvalue {$n}\n===\n";
        let program = compile(src).unwrap();
        let vars = vec![("$n".to_owned(), Value::Number(n))];
        let first = run(&program, "N", &vars, &[0], false);
        let second = run(&program, "N", &vars, &[0], false);
        prop_assert_eq!(render_run(&first), render_run(&second));

        let expected = if n.fract() == 0.0 && n.abs() < 1e15 {
            format!("{n:.0}")
        } else {
            format!("{n}")
        };
        let line = first.events.iter().find_map(|e| match e {
            DialogueEvent::Line { text, .. } => Some(text.clone()),
            _ => None,
        }).unwrap();
        prop_assert_eq!(line, format!("value {expected}"));
    }
}

/// Drives a fresh runner like [`run`] but returns the end-of-run session state
/// instead of the event stream.
fn run_to_snapshot(program: &Program, choices: &[usize]) -> bubbles::RunnerSnapshot {
    let mut storage = HashMapStorage::new();
    storage.set("$gold", Value::Number(10.0));
    let mut runner = Runner::new(program.clone(), storage);
    runner.start("Start").unwrap();
    let mut taken = 0;
    for _ in 0..2_000 {
        let Some(event) = runner.next_event().unwrap() else {
            break;
        };
        if let DialogueEvent::Options(options) = event {
            let available: Vec<usize> = options
                .iter()
                .enumerate()
                .filter(|(_, o)| o.available)
                .map(|(i, _)| i)
                .collect();
            if !available.is_empty() {
                let pick = available[choices[taken % choices.len()] % available.len()];
                taken += 1;
                runner.select_option(pick).unwrap();
            }
        }
    }
    runner.snapshot()
}

/// Visit counts and `<<once>>` ids are the session state held in hash containers.
/// Two identical runs must accumulate the same state by content, whatever order
/// the containers hold it in — compared through sorted views so the assertion
/// tests the content and not the ordering.
#[test]
fn session_state_content_is_order_independent() {
    let program = compile(SCRIPT).unwrap();
    let a = run_to_snapshot(&program, &[0, 1, 2]);
    let b = run_to_snapshot(&program, &[0, 1, 2]);

    assert_eq!(
        a.visits.iter().collect::<BTreeMap<_, _>>(),
        b.visits.iter().collect::<BTreeMap<_, _>>(),
        "visit counts differed between two identical runs"
    );
    assert_eq!(
        a.once_seen.iter().collect::<BTreeSet<_>>(),
        b.once_seen.iter().collect::<BTreeSet<_>>(),
        "once ids differed between two identical runs"
    );
    assert_eq!(a.current_node, b.current_node);
}
