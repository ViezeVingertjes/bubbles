# Introduction

Welcome to **Bubbles** - an embeddable, deterministic dialogue runtime for Rust. Scripted conversation as a state machine you drive, not a narrative authoring suite.

You write branching dialogue in `.bub` scripts. Bubbles compiles them to a plain Rust struct and hands you one loop: ask for the next event, show it, select an option, repeat.

```rust,ignore
while let Some(event) = runner.next_event()? {
    // draw, wait, choose, continue
}
```

That loop is the entire integration surface. Nothing happens between calls to `next_event()`, there is no async anywhere in the library, and there is no global state, so Bubbles fits inside a loop you already own rather than asking for one of its own. There is no bytecode and no binary format either: `compile()` returns a struct, so there is nothing to load at runtime.

So Bubbles runs wherever Rust runs: a Bevy system, a Godot-rust node, a Macroquad or Fyrox frame, a custom engine, a plain binary, a terminal, or a browser tab via WebAssembly.

## Who this is for

**If you build non-game interactive software** (education, training, simulation, onboarding, kiosks, exhibits), you have probably found that the established dialogue tools assume a game engine, an authoring suite, and an asset pipeline you have no use for. Bubbles assumes none of them. It is a library you call. It runs wherever Rust runs, including a browser tab via WebAssembly, and it behaves identically every run, which matters a great deal when "this hint appeared for one learner and not another" is a bug you have to reproduce.

**If you build deterministic game systems** (replay, lockstep netcode, server-authoritative dialogue), the same properties are what you were looking for. A runtime with no hidden clock and no ambient randomness is one you can put inside a simulation you need to reproduce exactly.

If you want a tool that lets a non-programmer write and preview dialogue with a graph editor and live reload, [Yarn Spinner](https://yarnspinner.dev) or [Ink](https://github.com/inkle/ink) will serve you better, and the [Embedding and Determinism](./why.md) chapter is honest about why.

## What Bubbles gives you

- A pull-based event API with no async, no callbacks, and no global state
- A tiny text format for nodes, lines, options, and branching
- Typed variables (`Number`, `Text`, `Bool`) with a real expression language
- Jumps, detours, conditionals, `<<once>>` blocks, interpolation, host commands
- Inline markup (`[b]text[/b]`, `[wave]text[/wave]`, `[pause /]`) stripped to byte-precise spans your renderer consumes
- Line groups and node groups for variety, chosen by a saliency strategy you can replace
- A pluggable localisation seam and custom functions that can override the built-ins
- Node-granularity save/load snapshots, with serde support for persistence
- Three required dependencies, `unsafe_code` denied, and a WebAssembly build checked in CI

Read [Embedding and Determinism](./why.md) next if you are still deciding whether this library fits your project: it covers both guarantees in detail, and the places where they stop.

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

That is a complete, working dialogue. A speaker line, three options (one guarded by a condition), a variable assignment, and a jump. The harbour walkthrough uses the same building blocks - speaker lines, guarded options, and jumps - to build a full dockside scene from scratch in [The Harbour](./examples/harbour.md).

## Jump right in

Want to skip the setup and play first?

```sh
cargo run -p bubbles-cli -- play examples/harbour/harbour.bub examples/harbour/services.bub
```

That's the harbour example: a playable pirate dockside scene that covers most of the language in one go. Come back here when you're done.

## How to read this guide

The chapters are meant to be read in order, but each one stands on its own:

- **[Embedding and Determinism](./why.md)** - the two guarantees this library is built around, and their limits.
- **[Getting Started](./getting-started/first-dialogue.md)** - go from zero to a running dialogue in ten lines of Rust.
- **[Tutorial](./tutorial/overview.md)** - build a real NPC scene from scratch, one feature at a time.
- **[The .bub Language](./language/nodes-and-lines.md)** - every piece of the script format, one concept per page.
- **[Integrating with Your Engine](./integration/runner.md)** - wiring Bubbles into rendering, input, and save systems, including [native hosts via the C ABI](./integration/native-hosts.md).
- **[Advanced](./advanced/save-load.md)** - snapshots, multi-file projects, WebAssembly.
- **[Examples](./examples/harbour.md)** - annotated walkthroughs of the demos shipped with the crate.
- **[API Reference](./api-reference.md)** - the full rustdoc, generated fresh for every release.

Ready? Let's write a dialogue.

---

> **Next:** [Embedding and Determinism](./why.md)
