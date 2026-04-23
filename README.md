# bubbles

**A minimal, engine-agnostic dialogue runtime for Rust games.**

[![Crates.io](https://img.shields.io/crates/v/bubbles-dialogue.svg)](https://crates.io/crates/bubbles-dialogue)
[![docs.rs](https://docs.rs/bubbles-dialogue/badge.svg)](https://docs.rs/bubbles-dialogue)
[![Guide](https://img.shields.io/badge/guide-github%20pages-informational)](https://viezevingertjes.github.io/bubbles/)
[![CI](https://github.com/ViezeVingertjes/bubbles/actions/workflows/ci.yml/badge.svg)](https://github.com/ViezeVingertjes/bubbles/actions)
[![MSRV: 1.95](https://img.shields.io/badge/rustc-1.95%2B-orange.svg)](https://blog.rust-lang.org/2025/05/15/Rust-1.95.0/)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)

Write branching `.bub` scripts, compile them once at startup, then drive the
dialogue from any game loop with a simple pull-based event API.  Designed to
integrate cleanly into Bevy, Godot, or any custom Rust engine - zero async
primitives, zero allocations in the hot path beyond the events themselves.

> **Docs:** the full guide lives at **<https://viezevingertjes.github.io/bubbles/>** - a friendly
> walk through the `.bub` language and integration, plus the hosted rustdoc.
> The rest of this README is the quick-reference overview.

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
| Built-in functions | `visited`, `visited_count`, `random`, `random_range`, `dice`, `round`, `floor`, `ceil`, `min`, `max`, `abs`, `clamp`, `string`, `int`, `plural`, `select` |
| Custom functions | Host-registered closures callable inside any expression |
| Once blocks | `<<once>>` / `<<once if cond>>` / `<<endonce>>` with optional `<<else>>` |
| Detour / return | `<<detour Node>>` / `<<return>>` subroutine stack |
| Early termination | `<<stop>>` ends the dialogue from anywhere in the call stack |
| Smart variables | `<<declare $x = expr>>` initialises once, stored in `VariableStorage` |
| Line groups | `=>` alternatives selected by the active `SaliencyStrategy` |
| Node groups | Multiple nodes sharing a title with `when:` header conditions |
| Saliency strategies | `FirstAvailable`, `RandomAvailable`, `BestLeastRecentlyViewed`, or custom |
| Multi-file compile | `compile_many(&[(name, source)])` with duplicate-title error |
| Reference validation | Catches typo'd jump/detour targets at compile time |
| Program introspection | `node_titles()`, `node_tags()`, `node_exists()`, `variable_declarations()` |
| Localisation seam | `LineProvider` returns a **template** for `#line:id`-tagged lines; `{expr}` in the translated string is evaluated after lookup (translate-then-format) |
| Stable line ids | `#line:<id>` also sets `line_id` on `DialogueEvent::Line` / `DialogueOption` (for VO, analytics); see [`line_id_from_tags`](https://docs.rs/bubbles-dialogue/latest/bubbles/fn.line_id_from_tags.html) |
| Save / load | `RunnerSnapshot` (serde feature) captures visits, once-seen, active node |

---

## Quick start

Add to `Cargo.toml`:

```toml
[dependencies]
bubbles-dialogue = "0.5"
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
cargo run -p bubbles-dialogue --example tavern --all-features
```

Or drop into the writer's terminal UI - play a script, watch the
transcript, hit `b` to rewind, `r` to reload:

```bash
cargo run -p bubbles-tui -- path/to/script.bub Start
```

See the [TUI Runner guide](https://viezevingertjes.github.io/bubbles/examples/tui-runner.html) for the full keymap and architecture overview.

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
<<stop>>                       # end the whole dialogue immediately

<<once>>
    Fires the very first time only.
<<else>>
    Fires every subsequent time.
<<endonce>>

=> Line variant A.             # line group - one chosen by saliency strategy
=> Line variant B.
=> Line variant C.

<<my_command arg1 arg2>>       # host command - surfaced as DialogueEvent::Command
===
```

---

## Extension points

| Trait / Type | Purpose |
|---|---|
| `VariableStorage` | Pluggable variable store (default: `HashMapStorage`). Override `get_ref` to return a borrowed `Cow<Value>` and skip allocating on every `$var` read. |
| `SaliencyStrategy` | Line / node group selection policy |
| `LineProvider` | Localisation lookup for `#line:id`-tagged lines |
| `line_id_from_tags` | Same id rule as events: parse `#line:` from a tag slice (e.g. custom UI) |
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
// The value is a *template* - {expr} placeholders are evaluated
// against the current variable storage AFTER the lookup.
provider.insert("greeting_id", "Hallo {$name}!");
runner.set_provider(provider);
```

When a line carries `#line:greeting_id`, the provider is consulted
**first** (translate-then-format ordering).  The returned string may
contain `{expr}` placeholders just like the original source - they are
evaluated after translation, so translators can reorder or reshape
interpolations for their language.  If the provider returns `None`, the
source text is used unchanged.

### Pluralisation and gendered grammar

Two built-in functions handle the most common i18n patterns directly
inside `{expr}` substitutions:

```
# English source
You found {$n} {plural($n, "gem", "gems")}.

# Translated template stored in the provider (e.g. Spanish)
Encontraste {$n} {plural($n, "gema", "gemas")}.
```

```
# Gendered pronouns via select()
{select($gender, "m:He|f:She|other:They")} arrived at the tavern.
```

`plural(count, singular, plural)` - returns the singular form when
`|count| == 1`, the plural form otherwise.

`select(key, "k1:text1|k2:text2|other:fallback")` - returns the text
for the matching key, or the `other` fallback when the key is not in the
mapping.  The first colon on each entry is the separator, so values may
themselves contain colons.

### Host functions

```rust
runner.library_mut().register("double", |args| {
    if let Some(bubbles::Value::Number(n)) = args.first() {
        Ok(bubbles::Value::Number(n * 2.0))
    } else {
        Err(bubbles::DialogueError::Function {
            name: "double".into(),
            message: "expected one number argument".into(),
        })
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

## Prior art

Bubbles sits in the same space as [Yarn Spinner](https://yarnspinner.dev) and
[Ink](https://github.com/inkle/ink): a structured dialogue language with branching,
variables, and conditions. The differences are in scope and integration model:

- **Yarn Spinner** compiles to a binary format and ships its own Unity/Godot runtime.
  Bubbles compiles to a Rust struct and runs wherever your Rust binary runs.
- **Ink** has a richer expression language and a hosted editor (Inky).
  Bubbles is narrower: no JSON story format, no divert tunnels - just a clean
  pull-based API for a game loop.
- **Twee / Twine** target HTML/browser story formats. Bubbles targets native Rust.

If you want a battle-tested, Unity-first dialogue system, Yarn Spinner is the right
choice. If you want something that drops cleanly into a Bevy, Godot-rust, or Macroquad
project with no runtime dependency, that is what Bubbles is for.

---

## Save / load

Requires the `serde` feature (or `full`):

```toml
bubbles-dialogue = { version = "0.5", features = ["serde"] }
```

Serialise **both** the snapshot and variable storage - neither is complete
without the other:

```rust
// Save.
let snap_json = serde_json::to_string(&runner.snapshot())?;
let vars_json = serde_json::to_string(runner.storage())?;

// Restore in a new session.
let snap: bubbles::RunnerSnapshot = serde_json::from_str(&snap_json)?;
let vars: bubbles::HashMapStorage  = serde_json::from_str(&vars_json)?;
let mut runner = bubbles::Runner::new(program, vars);
runner.restore(snap)?;
```

`RunnerSnapshot` preserves visit counts and exhausted `<<once>>` blocks.
The runner is left in the `Idle` state after `restore`; call `start` again
only if you want to replay from the beginning of the saved node.

---

## License

Licensed under either of [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) at your option.
