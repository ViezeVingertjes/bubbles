# The Tavern

[`examples/tavern.rs`](https://github.com/ViezeVingertjes/bubbles/blob/main/examples/tavern.rs) is the "everything example." It exercises most of Bubbles in one short scene: multiple files, variables, conditionals, guarded options, line groups, `<<once>>`, `<<detour>>`, commands, introspection, and (with the `serde` feature) save/load.

Run it:

```sh
cargo run --example tavern --all-features
```

You'll see two simulated visits to the tavern, followed by a snapshot round-trip demonstrating save/load. Let's walk through how it's built.

## Two files, one program

```rust,ignore
let prog = compile_many(&[
    ("tavern.bub", TAVERN),
    ("services.bub", SERVICES),
]).expect("compile failed");
```

`TAVERN` holds the main scene (`Tavern`). `SERVICES` holds the reusable beats (`PourAle`, `Rumours`, `End`). [`compile_many`](../advanced/multi-file.md) welds them into a single `Program` that sees every node regardless of file.

## The main scene

```text
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
```

Everything from the language chapters, working together in one node:

- Node **tags** (`scene indoor`) exposed via `program.node_tags("Tavern")`.
- **Declare** for stateful variables so they persist across visits.
- **Conditional** that diverges on first vs return visits.
- **Line group** (`=>`) for ambient barks - with BLRV saliency, each visit picks a different one.
- **Guarded option** (`<<if $gold >= 5>>`) - locked when you can't afford ale.
- **`<<detour>>`** into `PourAle` and come back; **`<<jump>>`** for one-way transitions.

## Reusable beats

```text
title: PourAle
---
<<pour_ale>>
<<set $gold = $gold - 5>>
Barkeep: Here you are. You have {$gold} gold left.
<<return>>
===
```

`<<pour_ale>>` is a **command** - an event the Rust side can react to (play audio, animate, whatever). Then the script deducts gold, reports the new total via interpolation, and `<<return>>`s to whoever detoured in.

```text
title: Rumours
---
<<once>>
    Barkeep: Word has it there's treasure north of the Ashen Pass.
    Barkeep: Goblin activity is up though - watch yourself.
<<else>>
    Barkeep: Nothing new to report since last we spoke.
<<endonce>>
Barkeep: Anything else?
-> Head back
    <<jump Tavern>>
===
```

The rumour beat uses `<<once>>` so the first telling is juicy and subsequent ones acknowledge the repeat, without a flag variable in sight.

## Introspection before running

```rust,ignore
println!("Nodes in program:");
for title in prog.node_titles() {
    let tags = prog.node_tags(title).unwrap_or_default();
    println!("  {title}  [{}]", tags.join(", "));
}

println!("\nDeclared variables:");
for decl in prog.variable_declarations() {
    println!("  {} = {}", decl.name, decl.default_src);
}
```

Before the dialogue even starts, the example prints every node title (with tags) and every declared variable. Tools and editors can use the same APIs to generate scene lists, debug overlays, or save-migration helpers.

## Configuring the runner

```rust,ignore
let mut runner = Runner::new(prog, HashMapStorage::new());
runner.set_saliency(BestLeastRecentlyViewed::new());
```

One line swaps in BLRV. With three `=>` fire-crackle variants, the player hears a fresh one every visit until they've cycled through all three.

## Simulated visits

The example scripts two visits instead of asking for input:

```rust,ignore
let all_choices: [&[usize]; 2] = [
    &[0, 1, 0], // Visit 1: ale → rumours → head back → leave
    &[2],       // Visit 2: leave immediately
];
```

A normal integration would read input from the player. Here the sequence is hard-coded so the output is reproducible.

Inside the loop, the event handling is the same shape as [the TUI runner](./tui-runner.md): react to node boundaries, lines, options, and commands as they are pulled from the runner.

## Save and load

Wrapped in `#[cfg(feature = "serde")]` so it only fires when the feature is on:

```rust,ignore
runner.start("Tavern").expect("start failed");
let _ = runner.next_event(); // NodeStarted

let snap = runner.snapshot();
let json = serde_json::to_string_pretty(&snap).expect("serialise failed");
println!("\n[snapshot]\n{json}");

runner.restore(serde_json::from_str(&json).unwrap()).unwrap();
```

Snapshot → JSON → back into the runner → drain the restored dialogue. The once-seen set and visit counts survive the round-trip; if you ask for rumours again after restoring, the Barkeep (correctly) says "nothing new."

## What to learn from this example

1. **Split scripts by concern.** One file for the main scene, one for services. `compile_many` pulls them together.
2. **Use `<<declare>>` for stateful values.** It's the hero of save/load.
3. **Line groups + BLRV** is the shortest path to an NPC that feels alive.
4. **`<<detour>>` for reusable beats.** `PourAle` could be called from ten different scenes; it always returns cleanly.
5. **Introspection isn't just for tools.** Node tags let you pre-load music; variable declarations let you generate save migrations.
6. **Feature-gate your save/load.** `#[cfg(feature = "serde")]` keeps the code path honest for builds that don't need it.

## Your turn

The Tavern is under 200 lines of Rust plus two small `.bub` scripts. It covers 80% of what a real game needs from a dialogue system. Try:

- **Add a "Landlady" NPC** in a third file, with her own greetings using a node group.
- **Register a `has_item` custom function** so an option unlocks when the player holds a key.
- **Swap `compile_many` for a loop** that reads every `.bub` file in an `assets/` directory.

That's the on-ramp. Everything else is more nodes, more variables, more flair.

---

> **Next:** [API Reference](../api-reference.md)
