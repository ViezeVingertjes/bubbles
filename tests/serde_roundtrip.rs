//! Snapshot / restore tests gated on the `serde` feature.

#[cfg(feature = "serde")]
mod serde_tests {
    use bubbles::{
        DialogueEvent, HashMapStorage, Runner, RunnerSnapshot, Value, VariableStorage, compile,
    };

    // ── Value round-trips ─────────────────────────────────────────────────────

    #[test]
    fn value_number_round_trips() {
        let v = Value::Number(3.14);
        let json = serde_json::to_string(&v).unwrap();
        assert_eq!(serde_json::from_str::<Value>(&json).unwrap(), v);
    }

    #[test]
    fn value_text_round_trips() {
        let v = Value::Text("hello".into());
        let json = serde_json::to_string(&v).unwrap();
        assert_eq!(serde_json::from_str::<Value>(&json).unwrap(), v);
    }

    #[test]
    fn value_bool_round_trips() {
        let v = Value::Bool(true);
        let json = serde_json::to_string(&v).unwrap();
        assert_eq!(serde_json::from_str::<Value>(&json).unwrap(), v);
    }

    // ── HashMapStorage round-trip ─────────────────────────────────────────────

    #[test]
    fn hashmap_storage_round_trips() {
        let mut s = HashMapStorage::new();
        s.set("$gold", Value::Number(42.0));
        s.set("$name", Value::Text("Bob".into()));
        s.set("$alive", Value::Bool(true));

        let json = serde_json::to_string(&s).unwrap();
        let s2: HashMapStorage = serde_json::from_str(&json).unwrap();

        assert_eq!(s2.get("$gold"), Some(Value::Number(42.0)));
        assert_eq!(s2.get("$name"), Some(Value::Text("Bob".into())));
        assert_eq!(s2.get("$alive"), Some(Value::Bool(true)));
    }

    // ── RunnerSnapshot round-trip ─────────────────────────────────────────────

    const SCRIPT: &str = "\
title: Intro
---
Hello, traveller.
===
title: Once
---
<<once>>
    A secret passage opens.
<<endonce>>
Goodbye.
===
";

    /// `snapshot()` captures the active node.
    #[test]
    fn snapshot_captures_current_node() {
        let prog = compile(SCRIPT).unwrap();
        let mut runner = Runner::new(prog, HashMapStorage::new());
        runner.start("Intro").unwrap();
        let _ = runner.next_event(); // NodeStarted

        let snap = runner.snapshot();
        assert_eq!(snap.current_node.as_deref(), Some("Intro"));
    }

    /// Snapshot serialises and deserialises without loss.
    #[test]
    fn snapshot_round_trips_json() {
        let prog = compile(SCRIPT).unwrap();
        let mut runner = Runner::new(prog, HashMapStorage::new());
        runner.start("Intro").unwrap();

        let snap = runner.snapshot();
        let json = serde_json::to_string(&snap).unwrap();
        let snap2: RunnerSnapshot = serde_json::from_str(&json).unwrap();

        assert_eq!(snap.current_node, snap2.current_node);
        assert_eq!(snap.visits, snap2.visits);
        assert_eq!(snap.once_seen, snap2.once_seen);
    }

    /// After a detour the snapshot records the innermost (detoured) node.
    #[test]
    fn snapshot_records_visit_counts() {
        let prog = compile(SCRIPT).unwrap();
        let mut runner = Runner::new(prog, HashMapStorage::new());
        runner.start("Intro").unwrap();
        // Drain all events to completion.
        while runner.next_event().unwrap().is_some() {}

        let snap = runner.snapshot();
        // Intro was visited once.
        assert_eq!(snap.visits.get("Intro").copied().unwrap_or(0), 1);
    }

    /// `restore()` preserves once_seen so blocks don't re-fire after load.
    #[test]
    fn restore_preserves_once_seen() {
        let prog = compile(SCRIPT).unwrap();
        let mut runner = Runner::new(prog.clone(), HashMapStorage::new());
        runner.start("Once").unwrap();

        // Drain until we have seen the "secret" line but before the node ends,
        // then snapshot with the node still active on the stack.
        let mut saw_secret = false;
        let mut snap_json = String::new();
        loop {
            let ev = runner.next_event().unwrap();
            match ev {
                Some(DialogueEvent::Line { ref text, .. }) if text.contains("secret") => {
                    saw_secret = true;
                    // Snapshot right after the secret line, node still running.
                    let snap = runner.snapshot();
                    snap_json = serde_json::to_string(&snap).unwrap();
                }
                Some(DialogueEvent::DialogueComplete) | None => break,
                _ => {}
            }
        }
        assert!(saw_secret, "first run should show the secret passage");

        // Restore the mid-dialogue snapshot into a fresh runner.
        let mut runner2 = Runner::new(prog, HashMapStorage::new());
        let snap2: RunnerSnapshot = serde_json::from_str(&snap_json).unwrap();
        runner2.restore(snap2).unwrap();

        // The once block was in `once_seen` at snapshot time, so it must NOT re-fire.
        let mut texts2: Vec<String> = Vec::new();
        while let Some(ev) = runner2.next_event().unwrap() {
            if let DialogueEvent::Line { text, .. } = ev {
                texts2.push(text);
            }
        }
        assert!(
            !texts2.iter().any(|t| t.contains("secret")),
            "restored runner must not show the secret passage again"
        );
        assert!(
            texts2.iter().any(|t| t.contains("Goodbye")),
            "restored runner should still emit Goodbye: {texts2:?}"
        );
    }

    /// `restore()` preserves visit counts so `visited()` still returns true.
    #[test]
    fn restore_preserves_visit_counts() {
        let src = "title: A\n---\nHello.\n===\n";
        let prog = compile(src).unwrap();
        let mut runner = Runner::new(prog.clone(), HashMapStorage::new());
        runner.start("A").unwrap();
        while runner.next_event().unwrap().is_some() {}

        let snap = runner.snapshot();
        let json = serde_json::to_string(&snap).unwrap();

        let mut runner2 = Runner::new(prog, HashMapStorage::new());
        let snap2: RunnerSnapshot = serde_json::from_str(&json).unwrap();
        runner2.restore(snap2).unwrap();
        while runner2.next_event().unwrap().is_some() {}

        // After restoring, the runner knows A was visited at least once before.
        let snap3 = runner2.snapshot();
        assert!(snap3.visits.get("A").copied().unwrap_or(0) >= 1);
    }
}
