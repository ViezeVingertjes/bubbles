//! A tavern conversation demonstrating a broad set of bubbles features:
//!
//! - Multiple nodes and `<<jump>>`
//! - `<<set>>` / `<<declare>>` typed variables
//! - `<<if>>` / `<<elseif>>` / `<<else>>` conditionals
//! - Shortcut options with `<<if>>` guards
//! - Inline `{expr}` substitution
//! - Host commands via `DialogueEvent::Command`
//! - `<<once>>` / `<<endonce>>` blocks
//! - `<<detour>>` / `<<return>>`
//! - Line groups (`=>`) with `BestLeastRecentlyViewed` saliency
//! - Program introspection (`node_titles`, `variable_declarations`)
//! - Save / load via `RunnerSnapshot` (requires `serde` feature)

use bubbles::saliency::BestLeastRecentlyViewed;
use bubbles::{DialogueEvent, HashMapStorage, Runner, Value, VariableStorage, compile_many};

// Each source file is compiled together — demonstrating multi-file support.
const TAVERN: &str = r#"
title: Tavern
tags: scene indoor
---
<<declare $gold = 50>>
<<declare $visited_barkeep = false>>

Barkeep: Evening, stranger.

<<if $visited_barkeep>>
    Barkeep: Back again so soon?
<<else>>
    Barkeep: First time here, is it?
    <<set $visited_barkeep = true>>
<<endif>>

=> Barkeep: The fire crackles nearby.
=> Barkeep: A minstrel plucks softly in the corner.
=> Barkeep: The smell of roasting meat fills the air.

Barkeep: What'll it be?

-> A mug of ale <<if $gold >= 5>>
    <<detour PourAle>>
-> Ask about rumours
    <<jump Rumours>>
-> Nothing, just passing through.
    Barkeep: Safe travels, then.
    <<jump End>>
===
"#;

const SERVICES: &str = r#"
title: PourAle
---
<<pour_ale>>
<<set $gold = $gold - 5>>
Barkeep: Here you are. You have {$gold} gold left.
<<return>>
===

title: Rumours
---
<<once>>
    Barkeep: Word has it there's treasure north of the Ashen Pass.
    Barkeep: Goblin activity is up though — watch yourself.
<<else>>
    Barkeep: Nothing new to report since last we spoke.
<<endonce>>
Barkeep: Anything else?
-> Head back
    <<jump Tavern>>
===

title: End
---
===
"#;

fn main() {
    // ── compile ──────────────────────────────────────────────────────────────
    let prog = compile_many(&[("tavern.bub", TAVERN), ("services.bub", SERVICES)])
        .expect("compile failed");

    // ── introspect before running ─────────────────────────────────────────────
    println!("Nodes in program:");
    for title in prog.node_titles() {
        let tags = prog.node_tags(title).unwrap_or_default();
        if tags.is_empty() {
            println!("  {title}");
        } else {
            println!("  {title}  [{}]", tags.join(", "));
        }
    }
    println!("\nDeclared variables:");
    for decl in prog.variable_declarations() {
        println!("  {} = {}", decl.name, decl.default_src);
    }

    // ── build runner with BLRV so barks cycle ────────────────────────────────
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.set_saliency(BestLeastRecentlyViewed::new());

    // Simulate two visits to demonstrate once-blocks and BLRV variation.
    let all_choices: [&[usize]; 2] = [
        &[0, 1, 0], // Visit 1: order ale → ask about rumours → head back → leave
        &[2],       // Visit 2: leave immediately
    ];

    for (visit_idx, choices) in all_choices.iter().enumerate() {
        println!("\n{}", "─".repeat(48));
        println!("  Visit {}", visit_idx + 1);
        println!("{}\n", "─".repeat(48));

        runner.start("Tavern").expect("start failed");
        let mut choice_iter = choices.iter();

        loop {
            match runner.next_event().expect("runtime error") {
                Some(DialogueEvent::NodeStarted(name)) => {
                    println!("[→ {name}]");
                }
                Some(DialogueEvent::Line {
                    speaker,
                    text,
                    tags,
                    ..
                }) => {
                    let spk = speaker.as_deref().unwrap_or("*");
                    if tags.is_empty() {
                        println!("{spk}: {text}");
                    } else {
                        println!("{spk}: {text}  #{}", tags.join(" #"));
                    }
                }
                Some(DialogueEvent::Options(opts)) => {
                    println!("\nChoices:");
                    for (i, opt) in opts.iter().enumerate() {
                        let note = if opt.available {
                            ""
                        } else {
                            "  ⚠ unavailable"
                        };
                        println!("  {i}. {}{note}", opt.text);
                    }
                    let idx = *choice_iter.next().unwrap_or(&0);
                    println!("» {idx}\n");
                    runner.select_option(idx).expect("select failed");
                }
                Some(DialogueEvent::Command { name, args, .. }) => {
                    // In a real game, dispatch to your audio/animation system here.
                    println!("[⚙ command: {name}({args})]", args = args.join(", "));
                }
                Some(DialogueEvent::NodeComplete(name)) => {
                    println!("[← {name}]");
                }
                Some(DialogueEvent::DialogueComplete) | None => {
                    println!("\n[dialogue complete]\n");
                    break;
                }
                Some(_) => {}
            }
        }

        if let Some(Value::Number(g)) = runner.storage().get("$gold") {
            println!("Gold: {g}");
        }
    }

    // ── save / load snapshot ─────────────────────────────────────────────────
    #[cfg(feature = "serde")]
    {
        runner.start("Tavern").expect("start failed");
        let _ = runner.next_event(); // NodeStarted

        let snap = runner.snapshot();
        let json = serde_json::to_string_pretty(&snap).expect("serialise failed");
        println!("\n[snapshot]\n{json}");

        // Restore demonstrates that once_seen and visits survive serialisation.
        runner
            .restore(serde_json::from_str(&json).unwrap())
            .unwrap();
        // Drain the restored dialogue, selecting the first option whenever prompted.
        loop {
            match runner.next_event().unwrap() {
                Some(DialogueEvent::Options(_)) => {
                    runner.select_option(0).unwrap();
                }
                Some(DialogueEvent::DialogueComplete) | None => break,
                _ => {}
            }
        }
        println!("[restored and drained successfully]");
    }

    println!("\nDone.");
}
