//! Integration tests for {expr} inline substitution.

mod common;

use bubbles::{HashMapProvider, HashMapStorage, RunnerBuilder, Value, VariableStorage, compile};

use common::line_texts as lines_from;

#[test]
fn variable_and_expression_substituted() {
    let events = common::play_fixture("interpolation", "Start");
    let lines = lines_from(&events);
    assert_eq!(
        lines,
        ["Hello Bob, you have 42 coins.", "The answer is 42.",]
    );
}

#[test]
fn no_braces_unchanged() {
    let src = "title: Start\n---\nHello world.\n===\n";
    let events = common::play(src, "Start");
    let lines = lines_from(&events);
    assert_eq!(lines, ["Hello world."]);
}

// ── plural() ──────────────────────────────────────────────────────────────────

#[test]
fn plural_singular_in_line() {
    let src = "\
title: Start
---
<<declare $n = 1>>
You have {$n} {plural($n, \"apple\", \"apples\")}.
===
";
    let events = common::play(src, "Start");
    assert_eq!(lines_from(&events), ["You have 1 apple."]);
}

#[test]
fn plural_plural_in_line() {
    let src = "\
title: Start
---
<<declare $n = 3>>
You have {$n} {plural($n, \"coin\", \"coins\")}.
===
";
    let events = common::play(src, "Start");
    assert_eq!(lines_from(&events), ["You have 3 coins."]);
}

#[test]
fn plural_zero_uses_plural_form() {
    let src = "\
title: Start
---
<<declare $n = 0>>
{plural($n, \"item\", \"items\")} remaining.
===
";
    let events = common::play(src, "Start");
    assert_eq!(lines_from(&events), ["items remaining."]);
}

// ── select() / gendered grammar ───────────────────────────────────────────────

#[test]
fn select_gendered_pronoun() {
    let src = "\
title: Start
---
<<declare $gender = \"f\">>
{select($gender, \"m:He|f:She|other:They\")} went to the store.
===
";
    let events = common::play(src, "Start");
    assert_eq!(lines_from(&events), ["She went to the store."]);
}

#[test]
fn select_uses_other_fallback() {
    let src = "\
title: Start
---
<<declare $gender = \"nb\">>
{select($gender, \"m:He|f:She|other:They\")} arrived.
===
";
    let events = common::play(src, "Start");
    assert_eq!(lines_from(&events), ["They arrived."]);
}

#[test]
fn select_combined_with_plural() {
    let src = "\
title: Start
---
<<declare $gender = \"m\">>
<<declare $n = 1>>
{select($gender, \"m:He|f:She|other:They\")} found {$n} {plural($n, \"coin\", \"coins\")}.
===
";
    let events = common::play(src, "Start");
    assert_eq!(lines_from(&events), ["He found 1 coin."]);
}

// ── eval_template: provider returns a template containing {expr} ──────────────
//
// This exercises the BraceSegment::Expr branch in evaluation.rs::eval_template.

#[test]
fn provider_template_with_expr_is_evaluated() {
    // Source text is plain; the provider overrides it with a template that
    // itself contains an expression placeholder.
    let src = "title: A\n---\nHello. #line:greeting\n===\n";
    let prog = compile(src).unwrap();

    let mut provider = HashMapProvider::new();
    // The translated template uses the variable $name.
    provider.insert("greeting", "Hola, {$name}!");

    let mut storage = HashMapStorage::new();
    storage.set("$name", Value::Text("Alice".into()));

    let mut runner = RunnerBuilder::new(prog, storage)
        .with_provider(provider)
        .build();
    runner.start("A").unwrap();
    let events = common::drain(&mut runner);
    assert_eq!(lines_from(&events), ["Hola, Alice!"]);
}

#[test]
fn provider_template_with_arithmetic_expr_is_evaluated() {
    // Provider template with a non-trivial expression: {$hp + 10}.
    let src = "title: A\n---\nYou have HP. #line:hp_line\n===\n";
    let prog = compile(src).unwrap();

    let mut provider = HashMapProvider::new();
    provider.insert("hp_line", "HP remaining: {$hp + 10}");

    let mut storage = HashMapStorage::new();
    storage.set("$hp", Value::Number(90.0));

    let mut runner = RunnerBuilder::new(prog, storage)
        .with_provider(provider)
        .build();
    runner.start("A").unwrap();
    let events = common::drain(&mut runner);
    assert_eq!(lines_from(&events), ["HP remaining: 100"]);
}
