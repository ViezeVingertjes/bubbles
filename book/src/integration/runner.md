# The Runner Lifecycle

The [`Runner`](https://docs.rs/bubbles-dialogue/latest/bubbles/struct.Runner.html) is where compiled dialogue meets your game. Create one, start it on a node, pump events until it's done. That's the whole shape.

## Create

```rust,ignore
use bubbles::{compile, HashMapStorage, Runner};

let program = compile(source)?;
let mut runner = Runner::new(program, HashMapStorage::new());
```

Two inputs:

- A compiled [`Program`](https://docs.rs/bubbles-dialogue/latest/bubbles/struct.Program.html). Build this once at load time; it's cheap to clone if you need per-runner copies.
- A [`VariableStorage`](./storage.md). `HashMapStorage::new()` is the batteries-included option. Your own implementation works too.

> **Tip:** Keep one `Program` per scripting asset, and spin up a fresh `Runner` whenever you start a conversation. Runners are cheap; programs are the expensive thing you compiled once.

## Configure (optional)

Before starting, you can swap defaults:

```rust,ignore
use bubbles::saliency::BestLeastRecentlyViewed;
use bubbles::HashMapProvider;

runner.set_saliency(BestLeastRecentlyViewed::new());   // variant picking
runner.set_provider(HashMapProvider::new());           // localisation
runner.library_mut().register("faction", |args| { /* … */ Ok(todo!()) });
```

These all return the runner (or `&mut` to it) so you can chain or call them one after the other.

## Start

```rust,ignore
runner.start("Intro")?;
```

[`start`](https://docs.rs/bubbles-dialogue/latest/bubbles/struct.Runner.html#method.start) validates the node exists and primes the runner. Calling it a second time resets execution - handy for "replay this scene" and for restoring from a snapshot.

An error at this stage is almost always a typo in the node name. Bubbles tells you which one.

## Pump events

```rust,ignore
while let Some(event) = runner.next_event()? {
    match event {
        DialogueEvent::Line { .. } => { /* render */ }
        DialogueEvent::Options(opts) => {
            runner.select_option(choose(opts))?;
        }
        DialogueEvent::Command { .. } => { /* dispatch */ }
        _ => {}
    }
}
```

`next_event` returns `None` when dialogue completes. Until then, it either:

- Returns the next event, or
- Returns an error (runtime type mismatch, bad option index, etc).

If you hit an `Options` event and call `next_event` again without a `select_option`, you get back `DialogueError::Runtime("call select_option() before next_event()")`. Bubbles refuses to guess.

## Completion

You'll see `DialogueEvent::DialogueComplete` on the last meaningful step, and then `None` on the next call. Both are fine to treat as "we're done" - pick whichever fits your loop.

```rust,ignore
match runner.next_event()? {
    Some(DialogueEvent::DialogueComplete) | None => break,
    Some(other) => handle(other),
}
```

## Inspecting state mid-run

Mid-run, you can read (and write) state directly:

```rust,ignore
let storage = runner.storage();          // &S
let storage_mut = runner.storage_mut();  // &mut S
```

Same for the function library:

```rust,ignore
runner.library_mut().register("reroll", |_| Ok(/* … */));
```

This is useful for late-binding custom functions (e.g. when a menu unlocks new capabilities) or for sneaking a variable in from outside:

```rust,ignore
use bubbles::{Value, VariableStorage};

runner.storage_mut().set("$player_name", Value::Text(player.name.clone()));
```

## Threading

`Runner` is `Send` when its storage is - that's true for `HashMapStorage`. Run dialogue on any thread you like; just don't share a single runner across threads without synchronisation. The pull-based API is designed to slot into whatever update scheme your engine uses (single thread, job system, task pool).

---

> **Next:** [Handling Events](./events.md)
