# Introduction

Welcome to **Bubbles** - a small, friendly dialogue runtime for Rust games.

You write branching dialogue in `.bub` scripts. Bubbles compiles them once at startup and hands you a simple loop: ask for the next event, show it on screen, select an option, repeat. That is the whole API.

```rust,ignore
while let Some(event) = runner.next_event()? {
    // draw, wait, choose, continue
}
```

No async. No global state. No engine lock-in. Bubbles runs wherever Rust runs - Bevy, Godot, Macroquad, a custom engine, the web via WebAssembly, even a terminal. **Unity and other native hosts** can use the **`bubbles-ffi`** C library (JSON events, P/Invoke); see [Unity and native hosts (C ABI)](./integration/unity-and-native.md).

## What Bubbles gives you

- A tiny text format for nodes, lines, options, and branching
- Typed variables (`Number`, `Text`, `Bool`) with a real expression language
- Jumps, detours, conditionals, `<<once>>` blocks, interpolation, host commands
- Inline markup (`[b]text[/b]`, `[wave]text[/wave]`, `[pause /]`) stripped to byte-precise spans your renderer consumes
- Line groups and node groups for variety (no more hearing the same bark twice)
- A pluggable localisation seam, custom functions, and custom saliency strategies
- Save/load via serde snapshots
- An allocation-conscious runtime with zero async primitives

## Who Bubbles is for

You want dialogue in a Rust game. You do not want a 30 MB runtime, a scripting VM, or a DSL that fights with your borrow checker. You want something you can read the source of in an afternoon and drop into a release build without a second thought.

If that sounds like you, you are in the right place.

## A taste of `.bub`

```text
title: Tavern
---
Barkeep: Evening, stranger.
-> A mug of ale <<if $gold >= 5>>
    <<set $gold = $gold - 5>>
    Barkeep: Here you are.
-> Ask about rumours
    <<jump Rumours>>
-> Nothing, just passing through.
    Barkeep: Safe travels, then.
===
```

That is a complete, working dialogue. A speaker line, three options (one guarded by a condition), a variable assignment, and a jump. We will build this exact scene step by step in [The Harbour](./examples/harbour.md).

## Jump right in

Want to skip the setup and play first?

```sh
cargo run -p bubbles-tui -- examples/harbour/harbour.bub examples/harbour/services.bub
```

That's the harbour example: a playable pirate dockside scene that covers most of the language in one go. Come back here when you're done.

## How to read this guide

The chapters are meant to be read in order, but each one stands on its own:

- **[Getting Started](./getting-started/first-dialogue.md)** - go from zero to a running dialogue in ten lines of Rust.
- **[Tutorial](./tutorial/overview.md)** - build a real NPC scene from scratch, one feature at a time.
- **[The .bub Language](./language/nodes-and-lines.md)** - every piece of the script format, one concept per page.
- **[Integrating with Your Engine](./integration/runner.md)** - wiring Bubbles into rendering, input, and save systems, including [Unity and other native hosts via the C ABI](./integration/unity-and-native.md).
- **[Advanced](./advanced/save-load.md)** - snapshots, multi-file projects, WebAssembly.
- **[Examples](./examples/tui-runner.md)** - annotated walkthroughs of the demos shipped with the crate.
- **[API Reference](./api-reference.md)** - the full rustdoc, generated fresh for every release.

Ready? Let's write a dialogue.

---

> **Next:** [Your First Dialogue](./getting-started/first-dialogue.md)
