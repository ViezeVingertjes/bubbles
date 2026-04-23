//! Tests for [`RunnerBuilder`] — the ergonomic one-shot runner construction API.

mod common;

use bubbles::saliency::BestLeastRecentlyViewed;
use bubbles::{HashMapStorage, RunnerBuilder, Value, VariableStorage, compile};

use common::line_texts;

// ── basic construction ────────────────────────────────────────────────────────

#[test]
fn builder_default_behaves_like_runner_new() {
    let src = "title: A\n---\nHello.\n===\n";
    let prog = compile(src).unwrap();
    let mut runner = RunnerBuilder::new(prog, HashMapStorage::new()).build();
    runner.start("A").unwrap();
    let events = common::drain(&mut runner);
    assert_eq!(line_texts(&events), ["Hello."]);
}

// ── custom saliency ───────────────────────────────────────────────────────────

#[test]
fn builder_with_saliency_takes_effect() {
    // BestLeastRecentlyViewed rotates through line group variants across calls.
    let src = "\
title: Bark
---
=> Alpha.
=> Beta.
=> Gamma.
===
";
    let prog = compile(src).unwrap();
    let mut runner = RunnerBuilder::new(prog, HashMapStorage::new())
        .with_saliency(BestLeastRecentlyViewed::default())
        .build();
    runner.start("Bark").unwrap();
    let first_events = common::drain(&mut runner);
    let first = line_texts(&first_events);

    // Restart to get the second variant (BLRV rotates).
    runner.start("Bark").unwrap();
    let second_events = common::drain(&mut runner);
    let second = line_texts(&second_events);

    // With BLRV the two runs should pick different lines.
    assert_ne!(
        first, second,
        "BLRV saliency should rotate variants; got same line twice: {first:?}"
    );
}

// ── host function registration via closure ────────────────────────────────────

#[test]
fn builder_with_function_registers_host_fn() {
    let src = "title: A\n---\n<<set $result = double(21)>>\n{$result}\n===\n";
    let prog = compile(src).unwrap();
    let mut runner = RunnerBuilder::new(prog, HashMapStorage::new())
        .with_function("double", |args| {
            let n = args
                .into_iter()
                .next()
                .and_then(|v| {
                    if let bubbles::Value::Number(n) = v {
                        Some(n)
                    } else {
                        None
                    }
                })
                .unwrap_or(0.0);
            Ok(bubbles::Value::Number(n * 2.0))
        })
        .build();
    runner.start("A").unwrap();
    let events = common::drain(&mut runner);
    assert_eq!(line_texts(&events), ["42"]);
}

// ── initial variable storage ──────────────────────────────────────────────────

#[test]
fn builder_with_storage_carries_pre_set_variables() {
    let src = "title: A\n---\n{$name}\n===\n";
    let prog = compile(src).unwrap();
    let mut storage = HashMapStorage::new();
    storage.set("$name", Value::Text("Alice".into()));

    let mut runner = RunnerBuilder::new(prog, storage).build();
    runner.start("A").unwrap();
    let events = common::drain(&mut runner);
    assert_eq!(line_texts(&events), ["Alice"]);
}
