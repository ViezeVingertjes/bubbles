//! End-to-end smoke tests for the compile → run pipeline when scripts use
//! expressions in every place the grammar allows them. These guard against
//! regressions when the AST stores parsed [`Expr`](bubbles::compiler::ast::Expr)
//! trees and node bodies are shared via `Arc` instead of re-parsing at runtime.
//!
//! If this file fails, compare event output against a known-good run — the
//! intent is *behavioural* parity, not a particular internal representation.

use bubbles::{DialogueEvent, HashMapStorage, Runner, Value, VariableStorage, compile};

/// Runs the runner to completion, auto-picking the first available option
/// whenever `Options` is encountered. Returns the runner (for `storage()`,
/// etc.) and all line texts in order.
fn run_until_done(mut r: Runner<HashMapStorage>) -> (Runner<HashMapStorage>, Vec<String>) {
    let mut out = Vec::new();
    loop {
        let ev = match r.next_event() {
            Ok(e) => e,
            Err(e) => panic!("{e}"),
        };
        let Some(ev) = ev else {
            return (r, out);
        };
        match ev {
            DialogueEvent::Line { text, .. } => out.push(text),
            DialogueEvent::Options(opts) => {
                let idx = opts
                    .iter()
                    .position(|o| o.available)
                    .expect("no available option");
                r.select_option(idx).expect("select_option");
            }
            _ => {}
        }
    }
}

/// Exercises: `<<declare>>`, `<<set>>`, `<<if>>` / `<<elseif>>` / `<<else>>`, `<<once if>>`,
/// node `when:` for a group, line-group `<<if>>` guards, and option `<<if>>` guards.
#[test]
fn full_expression_surface_emits_expected_lines() {
    let src = r"
title: A
when: 1 < 2
---
<<declare $t = 0>>
<<if $t < 1>>
    First branch.
<<elseif 2 * 2 == 4>>
    Second branch.
<<else>>
    Else branch.
<<endif>>
=> => Line one.
=> => Line two <<if true>>
-> Opt <<if 1+1==2>>
    <<set $t = 1>>
    After opt.
-> Skip <<if false>>
    hidden
<<once if 5 > 3>>
    Once line.
<<endonce>>
===
title: B
---
Other node.
===
";
    let prog = compile(src).unwrap();
    let mut r = Runner::new(prog, HashMapStorage::new());
    r.start("A").unwrap();
    let lines = run_until_done(r).1;

    // First if branch: $t is 0, so $t < 1 is true → "First branch." only from the if.
    assert!(
        lines.iter().any(|l| l.contains("First branch")),
        "expected first if branch, got {lines:?}"
    );
    assert!(
        !lines.iter().any(|l| l.contains("Second branch")),
        "did not expect elseif when first if matched: {lines:?}"
    );
    // Line group: first variant wins with default saliency (Line one).
    assert!(lines.iter().any(|l| l.contains("Line one")));
    // Option taken when guard is true; body runs.
    assert!(lines.iter().any(|l| l.contains("After opt")));
    // Once with condition.
    assert!(lines.iter().any(|l| l.contains("Once line.")));
    // $t was set in option body
    let prog2 = compile(src).unwrap();
    let mut r2 = Runner::new(prog2, HashMapStorage::new());
    r2.start("A").unwrap();
    let r2 = run_until_done(r2).0;
    assert_eq!(r2.storage().get("$t"), Some(Value::Number(1.0)));
}

#[test]
fn node_group_when_uses_parsed_expression() {
    let src = r"
title: G
when: 10 >= 5
---
Hello
===
title: G
when: 1 == 0
---
Wrong
===
";
    let prog = compile(src).unwrap();
    let mut r = Runner::new(prog, HashMapStorage::new());
    r.start("G").unwrap();
    let lines = run_until_done(r).1;
    assert_eq!(lines, vec!["Hello"]);
}
