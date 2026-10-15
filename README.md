# bubbles

**An embeddable, deterministic dialogue runtime for Rust. Scripted conversation as a state machine you drive, not a narrative authoring suite.**

[![Crates.io](https://img.shields.io/crates/v/bubbles-dialogue.svg)](https://crates.io/crates/bubbles-dialogue)
[![docs.rs](https://docs.rs/bubbles-dialogue/badge.svg)](https://docs.rs/bubbles-dialogue)
[![Guide](https://img.shields.io/badge/guide-github%20pages-informational)](https://viezevingertjes.github.io/bubbles/)
[![CI](https://github.com/ViezeVingertjes/bubbles/actions/workflows/ci.yml/badge.svg)](https://github.com/ViezeVingertjes/bubbles/actions)
[![MSRV: 1.94](https://img.shields.io/badge/rustc-1.94%2B-orange.svg)](https://blog.rust-lang.org/2026/03/27/Rust-1.94.0/)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)

Bubbles is a Rust library you call, not a framework that calls you. A `.bub` script compiles to a plain Rust struct, and the runtime gives you one method: ask for the next event. There is no async, no global state, and nothing to load at runtime, so it drops into a loop you already own - a Bevy system, a Godot-rust node, a Macroquad or Fyrox frame, a plain `while let` in a console application, or a browser animation frame under WebAssembly.

If you are building education software, a training simulation, an onboarding flow, a kiosk, or a museum exhibit, that shape is probably why you are here. You need scripted conversation that behaves the same way every time, and you are not going to install a game engine to get it.

It works well for games too, and for the same reason: if your engine needs replay, lockstep, or server-authoritative dialogue, a runtime with no hidden clock and no ambient randomness is much easier to trust.

---

## Why you would choose this

**It embeds without inverting your control flow.** The runtime is pull-based. You call `next_event()` when your application is ready for the next thing to happen, and nothing happens in between. There are no `async` functions in the library, no background tasks, no callbacks you must register, and no global or thread-local state, so a `Runner` is just a value you own and you can hold as many as you like. Bubbles never asks for your main loop.

**It is deterministic by construction.** Nothing in the runtime reads the clock, the environment, or a global RNG on its own initiative. The only nondeterminism is what you opt into, and you can take all of it back:

- `rand` is a feature. Build with `default-features = false` and the RNG is not compiled in at all.
- With it on, `RunnerBuilder::with_seed(u64)` makes randomness reproducible. `random`, `random_range`, `dice`, and `RandomAvailable` all draw from one generator the runner owns, the stream position travels in the snapshot, and a strategy you write yourself is handed that generator rather than reaching for a global. The three built-ins are also ordinary entries in a function registry, so you can replace them outright if you want the rolls to come from somewhere else.
- Line-group and node-group selection goes through an injected `SaliencyStrategy`. The default, `FirstAvailable`, is deterministic; `RandomAvailable` is the opt-in random one.

Given the same script, the same starting variables, the same seed, and the same option choices, you get the same sequence of events. Saves are canonical as well: `RunnerSnapshot` and `HashMapStorage` serialise to identical bytes for identical state, so you can hash, diff, or golden-test them.

**It is small and auditable.** Three required dependencies: `indexmap`, `logos`, `thiserror`. The library denies `unsafe_code`; the one crate that opts out is the optional `bubbles-ffi` C wrapper, which cannot be written in safe Rust. No bytecode, no binary format, no asset pipeline. `compile()` returns a struct, so there is nothing to version, load, or ship alongside your binary. CI keeps the library building for `wasm32-unknown-unknown` (with `default-features = false`, see below) and enforces a 300-non-blank-line cap per source file, which is mostly a way of keeping the code readable by someone who did not write it.

**Where the edges are.** `snapshot()` / `restore()` records the active node, visit counts, and which `<<once>>` blocks have fired, which is enough for save-and-resume at node granularity. It deliberately does not serialise the in-progress statement list, so `restore()` re-enters the snapshotted node from its beginning, and neither the `<<detour>>` return stack nor saliency history carries across. It is a bookmark, not a time-travel debugger - though the random stream does survive it, so rolls continue rather than restart. See [Save and Load](https://viezevingertjes.github.io/bubbles/advanced/save-load.html) for what round-trips and what does not.

---

## Quick start

```toml
[dependencies]
bubbles-dialogue = "2.0.0"
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

That loop is the whole integration surface. Replace the `println!`s with your renderer and `select_option(0)` with your input handling, and you are done.

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

## Prior art

[Yarn Spinner](https://yarnspinner.dev) and [Ink](https://github.com/inkle/ink) are the established tools in this space, and they are good at a job Bubbles is not trying to do.

They sell **narrative authoring suites to writers**, and the tooling is the product: editor extensions, graph views, live preview, engine plugins for Unity and Unreal, established localisation workflows, years of shipped titles, and communities large enough that your question has probably already been answered. Both are more mature than Bubbles and both have far richer writer-facing features. If a writer who does not use Rust needs to author and preview dialogue themselves, pick one of those; that is genuinely the better choice.

Bubbles is a **runtime for the program that embeds it**. The audience is the engineer wiring dialogue into an application, and the design follows from that: no engine assumption, no authoring tool to install, no bytecode or binary format in the middle, a pull-based API that does not want your main loop, and determinism you can turn all the way on. The two markets that care most about those properties are non-game interactive software (education, training, simulation, onboarding, kiosks, exhibits) and deterministic game systems such as replay, lockstep, and server-authoritative dialogue.

Put shortly: they optimise for the person writing the dialogue, we optimise for the program running it.

---

## Reference case: SiliSim

[SiliSim](https://silisim.com/) is a browser-based logic-gate course and silicon simulator. It is **our own project, not third-party adoption**. We describe it here because it is the clearest illustration of the problem shape Bubbles is built for, and because it is why the library looks the way it does.

SiliSim is not a game. It is a course: a lesson introduces a concept, the learner wires up a circuit, and the lesson reacts to what they built. That needs speaker lines, hints, and branching tutorial copy conditioned on simulation state, which is a dialogue problem, but it does not need a game engine, and it has to run in a browser tab.

That ruled out the alternatives on integration grounds rather than feature grounds:

- **It is already a WASM binary with its own event loop.** The circuit builder owns the frame loop. A dialogue runtime that wanted to own it, or that assumed an async runtime underneath, would have been the wrong shape. Pull-based `next_event()` calls drop into the existing loop directly.
- **There is no engine and no asset pipeline to hang things off.** Because `compile()` returns a plain struct, lesson scripts are just data the page already has: no bytecode to load, no binary format to serve.
- **Lessons must behave identically for every learner.** A hint that fires for one student and not another is a bug report nobody can reproduce. Determinism is a correctness requirement here, not a nice-to-have.

Someone else's event loop, no engine, reproducible behaviour: the same three constraints describe most of the training and simulation software we have looked at.

---

## Language reference

Bubbles is a small language. The table below is a reference for what the script format supports, not an argument for choosing the library; the reasons to choose it are above.

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
| Seeded replay | `RunnerBuilder::with_seed` - same program, same inputs, same seed, same event stream, including `dice()` and `RandomAvailable` |
| Host commands | `<<play_sound bell>>` surfaces as `DialogueEvent::Command` |
| Built-in functions | `visited`, `visited_count`, `random`, `random_range`, `dice`, `round`, `floor`, `ceil`, `min`, `max`, `abs`, `clamp`, `string`, `int`, `plural`, `select` |
| Custom functions | Register closures callable from any expression; registering a built-in's name replaces it |
| Localisation | `LineProvider` for `#line:id` lookup; translated templates can contain `{expr}` and `[markup]` |
| Multi-file | `compile_many(&[(name, src)])` with duplicate-title detection and cross-file jump validation |
| Bookmarks / save | `Runner::snapshot` / `Runner::restore` with `RunnerSnapshot` (always available); enable `serde` to persist snapshot and `HashMapStorage` to disk. Saves serialise to canonical bytes, so they can be hashed, diffed, and golden-tested |
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

Two things worth knowing about `rand`:

- In the default build, `random()`, `random_range()`, `dice()`, and `RandomAvailable` draw from the runner's own generator, seeded from the operating system unless you call `RunnerBuilder::with_seed`. Seed it and the session replays identically; the seed also covers the saliency strategy, which replacing the functions would not.
- **For `wasm32-unknown-unknown` you need `default-features = false`.** The default build pulls in `getrandom`, which does not support that target unless you opt into a JavaScript backend. This is why the CI WebAssembly job builds the no-default-features configurations, and it means the browser-friendly build and the deterministic build are the same build.

```toml
# Browser / deterministic configuration.
bubbles-dialogue = { version = "2.0.0", default-features = false, features = ["serde"] }
```

---

## Learn more

The full guide - language reference, integration walkthrough, localisation, save/load, WebAssembly, and annotated examples - lives at **<https://viezevingertjes.github.io/bubbles/>**. Start with [Embedding and Determinism](https://viezevingertjes.github.io/bubbles/why.html) if you are deciding whether this library fits.

Release history, including the breaking changes in 1.0.0 and 2.0.0, is in [CHANGELOG.md](CHANGELOG.md).

Rust hosts use the library directly, whether that is Bevy, Godot-rust, Macroquad, Fyrox, or a binary with a loop of its own. For hosts that are not written in Rust, [`bubbles-ffi`](crates/bubbles-ffi/) is a maintained C ABI wrapper that surfaces the same events as JSON.

---

## Contributing

Contributions of every size are welcome, including documentation and typo fixes.

```bash
git clone https://github.com/ViezeVingertjes/bubbles
cd bubbles
cargo test --all-features
```

Good places to start:

- Issues labelled [`good first issue`](https://github.com/ViezeVingertjes/bubbles/labels/good%20first%20issue), which are scoped to an evening and name the file to edit
- A confusing passage in the [guide](https://viezevingertjes.github.io/bubbles/). The source is in `book/src/`, and a wording fix needs no tests
- A bug you hit. A minimal `.bub` reproduction is the most useful report there is

[CONTRIBUTING.md](CONTRIBUTING.md) covers setup, the quality gates, and how much
process applies to which kind of change (a typo fix needs very little). Commits
need a [DCO](DCO) sign-off, which `git commit -s` adds.

Changes land on `main` through pull requests, with CI and MSRV checks required.
Draft PRs are welcome if you want feedback before a change is finished.

---

## License

Licensed under either of [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) at your option.
