//! Integration tests for saliency strategies: `FirstAvailable`,
//! `RandomAvailable`, and `BestLeastRecentlyViewed`, used with line groups
//! and node groups.

use bubbles::saliency::{BestLeastRecentlyViewed, FirstAvailable};
use bubbles::{DialogueEvent, HashMapStorage, Runner, compile};

fn line_texts(runner: &mut Runner<HashMapStorage>) -> Vec<String> {
    let mut out = Vec::new();
    while let Some(ev) = runner.next_event().unwrap() {
        if let DialogueEvent::Line { text, .. } = ev {
            out.push(text);
        }
    }
    out
}

// ── FirstAvailable (default) ──────────────────────────────────────────────────

#[test]
fn first_available_line_group_always_picks_first() {
    let src = "\
title: Bark
---
=> Line A.
=> Line B.
=> Line C.
===
";
    let prog = compile(src).unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.start("Bark").unwrap();
    assert_eq!(line_texts(&mut runner), ["Line A."]);
    runner.start("Bark").unwrap();
    assert_eq!(line_texts(&mut runner), ["Line A."]);
}

// ── BestLeastRecentlyViewed ────────────────────────────────────────────────────

#[test]
fn blrv_line_group_cycles_through_variants() {
    let src = "\
title: Bark
---
=> Line A.
=> Line B.
=> Line C.
===
";
    let prog = compile(src).unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.set_saliency(BestLeastRecentlyViewed::new());

    runner.start("Bark").unwrap();
    assert_eq!(line_texts(&mut runner), ["Line A."]);

    runner.start("Bark").unwrap();
    assert_eq!(line_texts(&mut runner), ["Line B."]);

    runner.start("Bark").unwrap();
    assert_eq!(line_texts(&mut runner), ["Line C."]);

    // Fourth visit wraps back to A (oldest).
    runner.start("Bark").unwrap();
    assert_eq!(line_texts(&mut runner), ["Line A."]);
}

#[test]
fn blrv_node_group_cycles() {
    let src = "\
title: Shop
when: true
---
Still stocked.
===
title: Shop
when: true
---
Almost empty.
===
";
    let prog = compile(src).unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.set_saliency(BestLeastRecentlyViewed::new());

    runner.start("Shop").unwrap();
    let t1 = line_texts(&mut runner);

    runner.start("Shop").unwrap();
    let t2 = line_texts(&mut runner);

    // The two visits should yield different lines.
    assert_ne!(t1, t2, "BLRV should alternate between node variants");
}

#[test]
fn blrv_skips_unavailable_guarded_variant() {
    let src = "\
title: Bark
---
=> Line A if false.  <<if false>>
=> Line B.
===
";
    let prog = compile(src).unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.set_saliency(BestLeastRecentlyViewed::new());

    runner.start("Bark").unwrap();
    // Line A is guarded false, so BLRV picks Line B even though A is "newer".
    assert_eq!(line_texts(&mut runner), ["Line B."]);
}

// ── custom strategy ────────────────────────────────────────────────────────────

#[test]
fn custom_strategy_always_picks_last() {
    use bubbles::saliency::{Candidate, SaliencyStrategy};

    struct AlwaysLast;
    impl SaliencyStrategy for AlwaysLast {
        fn select(&mut self, candidates: &[Candidate<'_>]) -> Option<usize> {
            candidates
                .iter()
                .enumerate()
                .rev()
                .find(|(_, c)| c.available)
                .map(|(i, _)| i)
        }
    }

    let src = "\
title: Lines
---
=> First.
=> Second.
=> Third.
===
";
    let prog = compile(src).unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.set_saliency(AlwaysLast);

    runner.start("Lines").unwrap();
    assert_eq!(line_texts(&mut runner), ["Third."]);
}

// ── line provider ─────────────────────────────────────────────────────────────

#[test]
fn hashmap_provider_translates_line_ids() {
    use bubbles::{HashMapProvider, PassthroughProvider};

    let src = "\
title: Intro
---
Greetings! #line:greeting_id
===
";
    let prog = compile(src).unwrap();

    // Default (passthrough) provider emits the original text.
    let mut runner = Runner::new(prog.clone(), HashMapStorage::new());
    runner.set_provider(PassthroughProvider);
    runner.start("Intro").unwrap();
    let mut default_text = String::new();
    while let Some(ev) = runner.next_event().unwrap() {
        if let DialogueEvent::Line { text, .. } = ev {
            default_text = text;
        }
    }
    assert_eq!(default_text, "Greetings!");

    // HashMapProvider returns the translated string.
    let mut provider = HashMapProvider::new();
    provider.insert("greeting_id", "Hallo!");
    let mut runner2 = Runner::new(prog, HashMapStorage::new());
    runner2.set_provider(provider);
    runner2.start("Intro").unwrap();
    let mut translated = String::new();
    while let Some(ev) = runner2.next_event().unwrap() {
        if let DialogueEvent::Line { text, .. } = ev {
            translated = text;
        }
    }
    assert_eq!(translated, "Hallo!");
}

// ── FirstAvailable public strategy type ──────────────────────────────────────

#[test]
fn first_available_strategy_set_explicitly() {
    let src = "\
title: T
---
=> Alpha.
=> Beta.
===
";
    let prog = compile(src).unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.set_saliency(FirstAvailable);

    runner.start("T").unwrap();
    assert_eq!(line_texts(&mut runner), ["Alpha."]);
}
