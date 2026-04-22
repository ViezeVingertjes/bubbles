# bubbles

**A minimal, engine-agnostic dialogue runtime for Rust games.**

[![Crates.io](https://img.shields.io/crates/v/bubbles-dialogue.svg)](https://crates.io/crates/bubbles-dialogue)
[![docs.rs](https://docs.rs/bubbles-dialogue/badge.svg)](https://docs.rs/bubbles-dialogue)
[![CI](https://github.com/ViezeVingertjes/bubbles/actions/workflows/ci.yml/badge.svg)](https://github.com/ViezeVingertjes/bubbles/actions)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)

Crates.io and docs.rs badges above fill in after the first release on crates.io.

Write branching `.bub` scripts, compile them once at startup, then drive the
dialogue from any game loop with a simple pull-based event API.  Designed to
integrate cleanly into Bevy, Godot, or any custom Rust engine — zero async
primitives, zero allocations in the hot path beyond the events themselves.

**Requirements:** Rust **1.95** or later (see `rust-version` in `Cargo.toml`).

---

## Features

| Feature | Description |
|---|---|
| Nodes | `title:` / `---` / `===` structure; optional `tags:` metadata |
| Lines | Plain text, `Speaker: attribution`, trailing `#hashtag` metadata |
| Shortcut options | `->` with optional `<<if cond>>` guard and indented body |
| Jumps | `<<jump NodeName>>` |
| Conditionals | `<<if>>` / `<<elseif>>` / `<<else>>` / `<<endif>>` |
| Typed variables | `Number`, `Text`, `Bool`; `<<set $x = expr>>` |
| Expressions | Arithmetic, comparison, boolean, unary, parens, proper precedence |
| Inline substitution | `{expr}` anywhere in line / option / command text |
| Host commands | `<<verb arg1 arg2>>` surfaced as `DialogueEvent::Command` |
| Built-in functions | `visited`, `visited_count`, `random`, `random_range`, `dice`, `round`, `floor`, `ceil`, `min`, `max`, `abs`, `clamp`, `string`, `int` |
| Custom functions | Host-registered closures callable inside any expression |
| Once blocks | `<<once>>` / `<<once if cond>>` / `<<endonce>>` with optional `<<else>>` |
| Detour / return | `<<detour Node>>` / `<<return>>` subroutine stack |
| Smart variables | `<<declare $x = expr>>` initialises once, stored in `VariableStorage` |
| Line groups | `=>` alternatives selected by the active `SaliencyStrategy` |
| Node groups | Multiple nodes sharing a title with `when:` header conditions |
| Saliency strategies | `FirstAvailable`, `RandomAvailable`, `BestLeastRecentlyViewed`, or custom |
| Multi-file compile | `compile_many(&[(name, source)])` with duplicate-title error |
| Reference validation | Catches typo'd jump/detour targets at compile time |
| Program introspection | `node_titles()`, `node_tags()`, `node_exists()`, `variable_declarations()` |
| Localisation seam | `LineProvider` trait consulted for `#line:id`-tagged lines |
| Save / load | `RunnerSnapshot` (serde feature) captures visits, once-seen, active node |

---

## Quick start

Add to `Cargo.toml`:

```toml
[dependencies]
bubbles-dialogue = "0.1"
```

```rust
use bubbles::{compile, DialogueEvent, HashMapStorage, Runner};

fn main() -> Result<(), bubbles::DialogueError> {
    let program = compile(r#"
        title: Start
        ---
        Alice: Welcome.
        -> Ready
            Alice: Here we go.
        -> Not yet
            Alice: Take your time.
        ===
    "#)?;

    let mut runner = Runner::new(program, HashMapStorage::new());
    runner.start("Start")?;

    while let Some(event) = runner.next_event()? {
        match event {
            DialogueEvent::Line { speaker, text, .. } => {
                println!("{}: {text}", speaker.as_deref().unwrap_or("*"));
            }
            DialogueEvent::Options(opts) => {
                for (i, o) in opts.iter().enumerate() {
                    println!("  {i}) {}", o.text);
                }
                runner.select_option(0)?;
            }
            _ => {}
        }
    }
    Ok(())
}
```

Run the included examples:

```bash
cargo run --example cli_runner --all-features
cargo run --example tavern --all-features
```

---

## Script syntax

```
title: NodeName
tags: optional space-separated-tags
when: <condition expression>   # node-group header
---
Speaker: A plain line.
A narrator line.               # no speaker
The count is {$count}.         # inline expression substitution

<<set $x = $x + 1>>           # typed variable assignment
<<declare $hp = 100>>          # initialise-once (skips if already set)

<<if $x > 10>>
    A conditional line.
<<elseif $x == 5>>
    Another branch.
<<else>>
    The fallback.
<<endif>>

-> Option text
    Lines under the option.
-> Only when rich <<if $gold >= 100>>
    You splurge.

<<jump OtherNode>>             # jump (clears call stack)
<<detour SubScene>>            # call-stack push
<<return>>                     # pop back to caller

<<once>>
    Fires the very first time only.
<<else>>
    Fires every subsequent time.
<<endonce>>

=> Line variant A.             # line group — one chosen by saliency strategy
=> Line variant B.
=> Line variant C.

<<my_command arg1 arg2>>       # host command — surfaced as DialogueEvent::Command
===
```

---

## Extension points

| Trait / Type | Purpose |
|---|---|
| `VariableStorage` | Pluggable variable store (default: `HashMapStorage`) |
| `SaliencyStrategy` | Line / node group selection policy |
| `LineProvider` | Localisation lookup for `#line:id`-tagged lines |
| `FunctionLibrary` | Register host functions callable inside expressions |

### Custom saliency

```rust
use bubbles::saliency::{BestLeastRecentlyViewed, RandomAvailable};

runner.set_saliency(BestLeastRecentlyViewed::new()); // maximum variation
runner.set_saliency(RandomAvailable);                 // uniform random
```

### Localisation

```rust
use bubbles::HashMapProvider;

let mut provider = HashMapProvider::new();
provider.insert("greeting_id", "Hallo!");
runner.set_provider(provider);
```

A line tagged `#line:greeting_id` will be substituted with the provider's
translation before the event is emitted.

### Host functions

```rust
runner.library_mut().register("double", |args| {
    if let Some(bubbles::Value::Number(n)) = args.first() {
        Ok(bubbles::Value::Number(n * 2.0))
    } else {
        Err(bubbles::DialogueError::Runtime("expected number".into()))
    }
});
```

---

## Feature flags

| Flag | Default | Description |
|---|---|---|
| `rand` | **on** | Enables `random()`, `random_range()`, `dice()` builtins and `RandomAvailable` saliency |
| `serde` | off | Derives `Serialize` / `Deserialize` on `Value`, `HashMapStorage`, and `RunnerSnapshot` |
| `full` | off | Shorthand for `rand` and `serde` together (`features = ["full"]`) |

---

## Save / load

```rust
// Capture session state mid-dialogue (requires serde feature).
let snap = runner.snapshot();
let json = serde_json::to_string(&snap)?;

// … restore later in a new session …
let snap: bubbles::RunnerSnapshot = serde_json::from_str(&json)?;
runner.restore(snap)?;
```

`RunnerSnapshot` preserves visit counts and exhausted `<<once>>` blocks.
Variable storage is serialised separately via `runner.storage()`.

---

## License

Licensed under either of [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) at your option.
