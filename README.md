# bubbles

**Branching dialogue for Rust games. Write scripts, compile once, drive from any game loop.**

[![Crates.io](https://img.shields.io/crates/v/bubbles-dialogue.svg)](https://crates.io/crates/bubbles-dialogue)
[![docs.rs](https://docs.rs/bubbles-dialogue/badge.svg)](https://docs.rs/bubbles-dialogue)
[![Guide](https://img.shields.io/badge/guide-github%20pages-informational)](https://viezevingertjes.github.io/bubbles/)
[![CI](https://github.com/ViezeVingertjes/bubbles/actions/workflows/ci.yml/badge.svg)](https://github.com/ViezeVingertjes/bubbles/actions)
[![MSRV: 1.94](https://img.shields.io/badge/rustc-1.94%2B-orange.svg)](https://blog.rust-lang.org/2026/03/27/Rust-1.94.0/)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)

No async. No global state. No engine lock-in. Pull events when your game is ready, pick an option, continue. That's the whole API.

Works with Bevy, Godot-rust, Macroquad, or any custom engine. Unity and other native hosts can use [`bubbles-ffi`](crates/bubbles-ffi/) - a C ABI wrapper with JSON events for P/Invoke.

---

## What a script looks like

```text
title: Start
tags: scene docks outdoor
---
<<declare $gold = 25>>

=> Dockworker: Oi, watch yer step!
=> Dockworker: These crates won't unload themselves, ye know.
=> Dockworker: Smells like low tide and regret out here.

Stumpy: Name's McGee. Stumpy McGee, Harbormaster.
Stumpy: Ye can't sail from Barnacle Bay without a travel permit.

<<jump Options>>
===
```

The harbour demo continues in `examples/harbour/` with an `Options` node (guarded choices, `<<detour MapSeller>>`, `<<jump Depart>>`). Run it with:

```bash
cargo run -p bubbles-tui -- examples/harbour/harbour.bub examples/harbour/services.bub
```

---

## Quick start

```toml
[dependencies]
bubbles-dialogue = "1.0.0"
```

On crates.io the package is **`bubbles-dialogue`** (the name **`bubbles`** was already taken). The Rust library is still **`bubbles`**, so you write `use bubbles::{…}` in code.

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

---

## Features

| Feature | Description |
|---|---|
| Lines and options | Plain text, `Speaker:` attribution, `->` branches with optional `<<if>>` guards |
| Variables | `Number`, `Text`, `Bool`; `<<declare>>`, `<<set>>`, full expression language |
| Inline interpolation | `{$var}`, `{$gold / 2}`, `{plural($n, "gem", "gems")}` - evaluated before the event arrives |
| Inline markup | `[b]text[/b]`, `[color value=red]...[/color]`, `[pause /]` - stripped from text, returned as byte-precise `MarkupSpan`s |
| Jumps and detours | `<<jump Node>>` replaces the call stack; `<<detour Node>>` / `<<return>>` push/pop it |
| Conditionals | `<<if>>` / `<<elseif>>` / `<<else>>` / `<<endif>>` |
| Once blocks | `<<once>>` content that fires exactly once, with optional `<<else>>` for every subsequent visit |
| Line groups | `=>` alternatives chosen by the active saliency strategy - no repeated barks |
| Node groups | Multiple nodes sharing a title with `when:` conditions - great for time-of-day, relationship state |
| Saliency | `FirstAvailable`, `RandomAvailable`, `BestLeastRecentlyViewed`, or your own |
| Host commands | `<<play_sound bell>>` surfaces as `DialogueEvent::Command` |
| Built-in functions | `visited`, `visited_count`, `random`, `random_range`, `dice`, `round`, `floor`, `ceil`, `min`, `max`, `abs`, `clamp`, `string`, `int`, `plural`, `select` |
| Custom functions | Register closures callable from any expression |
| Localisation | `LineProvider` for `#line:id` lookup; translated templates can contain `{expr}` and `[markup]` |
| Multi-file | `compile_many(&[(name, src)])` with duplicate-title detection and cross-file jump validation |
| Bookmarks / save | `Runner::snapshot` / `Runner::restore` with `RunnerSnapshot` (always available); enable `serde` to persist snapshot and `HashMapStorage` to disk |
| Line modes | `#narration` and `#debug` on lines set `DialogueEvent::Line::line_mode` for filtering or routing |
| Variable inspection | `Runner::all_variables`, `Runner::variable`, `Runner::variable_ref`; `VariableStorage::all_variables` (override on custom stores) |
| Option groups | `#group:<name>` on options for UI constraints (radio buttons, mutually exclusive choices) |

---

## Feature flags

| Flag | Default | |
|---|---|---|
| `rand` | on | `random()`, `random_range()`, `dice()`, `RandomAvailable` |
| `serde` | off | `Serialize` / `Deserialize` on `Value`, `HashMapStorage`, `RunnerSnapshot` |
| `full` | off | Both of the above |

---

## Prior art

Bubbles sits in the same space as [Yarn Spinner](https://yarnspinner.dev) and [Ink](https://github.com/inkle/ink). The difference is integration model: Bubbles compiles to a plain Rust struct and runs wherever your binary runs - no binary format, no external runtime, no Unity package. If you want something that drops into a Bevy, Godot-rust, or Macroquad project in an afternoon, that's what this is for.

---

## Learn more

The full guide - language reference, integration walkthrough, localisation, save/load, WebAssembly, and annotated examples - lives at **<https://viezevingertjes.github.io/bubbles/>**.

---

## Contributing

All changes to `main` go through a **pull request** (no direct pushes). These status checks must pass before merge:

| Check | Source |
|-------|--------|
| `ci` | GitHub Actions (format, clippy, tests, docs, …) |
| `msrv` | GitHub Actions (MSRV compile) |

The branch must be **up to date** with `main` before merge. Conversation threads on the PR must be resolved. Force-push and branch deletion on `main` are blocked.

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and code quality gates.

---

## License

Licensed under either of [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) at your option.
