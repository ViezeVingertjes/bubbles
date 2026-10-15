# Using the CLI

`bubbles-cli` installs a single binary called `bubbles` with two verbs. `check` compiles your scripts and tells you what is wrong with them. `play` walks through one in a terminal. Both run the same `Runner` and the same compiler a real integration would, so neither can tell you something your game would disagree with.

```sh
cargo install bubbles-cli
```

The crate is not on crates.io yet. Until the next release publishes it, install from git instead — `cargo install --git https://github.com/ViezeVingertjes/bubbles bubbles-cli`, with the same flags wherever this page says `cargo install bubbles-cli`.

Working inside this repository, swap `bubbles` for `cargo run -p bubbles-cli --` in every command below.

## `bubbles check`

```sh
bubbles check dialogue/tavern.bub
```

It compiles the files, validates every `<<jump>>` and `<<detour>>` target, prints one diagnostic per line, and exits non-zero if anything failed. That is the whole command. It needs no configuration, no project file, and no Rust in the project it is checking.

```text
dialogue/tavern.bub:42: Parse error: in <<if>> ``: unexpected end of expression
```

A clean run prints a summary instead:

```text
ok: 3 files, 17 nodes
```

**Every file in one invocation is compiled together**, as a single programme. Node titles merge across files and cross-file jumps resolve, exactly as they do at runtime. This matters: checking files one at a time would report every cross-file jump as a broken reference.

```sh
# right - one programme, cross-file jumps resolve
bubbles check dialogue/*.bub

# wrong - each file checked alone, cross-file jumps look broken
for f in dialogue/*.bub; do bubbles check "$f"; done
```

### In CI

`--quiet` drops the success summary and leaves only diagnostics, which is what you want when a green step should be silent. Diagnostics go to stdout, so they pipe.

```yaml
- run: cargo install bubbles-cli --no-default-features
- run: bubbles check --quiet dialogue/*.bub
```

`--no-default-features` turns off the terminal UI and leaves a checker that adds nothing to the library's own three direct dependencies — `indexmap`, `logos`, `thiserror`. `check` behaves identically either way — it never reaches the parts of the runtime that the feature turns on.

## `bubbles play`

```sh
bubbles play examples/harbour/harbour.bub examples/harbour/services.bub
```

You arrive at Barnacle Bay and meet Stumpy McGee, the cantankerous harbormaster. He wants a permit fee. Try each option, run out of gold, find the map seller around the corner.

```sh
# single file
bubbles play my-scene.bub

# multiple files compiled together (cross-file jumps and detours work)
bubbles play main.bub services.bub characters.bub

# explicit start node (defaults to "Start" if omitted)
bubbles play my-scene.bub TownSquare
```

A bare path list with no verb does the same thing: `bubbles my-scene.bub` is `bubbles play my-scene.bub`.

### What you're looking at

**Transcript** (the main panel) logs every event the runner emits: node markers (`[→ Name]` / `[← Name]`), speaker lines, commands (`⚙ play_sfx clink`), and the option you chose each time. Scroll it to see the full history. The title bar shows the total event count and how far you've scrolled back.

**Options** appears at the bottom whenever a choice is active. `Tab` swaps focus between the options list and the transcript, so you can scroll back through history while a choice is waiting.

**Error overlay** pops up on any parse or runtime error, with the file, line number, and a snippet of the offending source. Press `x` to dismiss.

### Keys

Press `?` for the full list at any time. The footer shows the ones that apply to what is on screen.

| Key | What it does |
|---|---|
| `Enter` / `Space` | Advance; confirm the focused option |
| `↑` / `k`, `↓` / `j` | Navigate options or scroll the transcript |
| `1` ... `9` | Pick an option directly by number |
| `Tab` | Swap focus between the options list and transcript |
| `b` / `Backspace` | Step back one event |
| `r` | Rerun: restart from the start node, keeping variables and `<<once>>` history |
| `Ctrl+R` / `F5` | Reload: re-read the files from disk. **Discards every variable.** |
| `x` | Dismiss the error overlay |
| `?` | Show or hide the key list |
| `q` / `Esc` | Quit (closes the key list first) |

Every letter is case-insensitive. Reload is the one action that throws your runtime state away with no confirmation and no undo, so it is deliberately not on a letter at all.

### Rerun vs reload

**`r` (rerun)** restarts from the beginning of the dialogue but keeps the current variable values and `<<once>>` seen history. Use it to reach lines that only appear on a second visit — the `<<else>>` branch of a `<<once>>` block, a node that only triggers after buying something, a counter that changes behaviour on the fifth loop.

**`Ctrl+R` (reload)** picks up every change you've made to the `.bub` files on disk. Variables reset, `<<once>>` history resets, everything starts fresh.

The workflow: write a scene, `Ctrl+R` to pick up the edit, play through it, `r` to get the second-visit state, `Ctrl+R` again after the next edit.

## How the player is built

The crate is split so the entire UI can be exercised without a real terminal, which is also why the event API gets a thorough workout here:

- `AppState` owns the compiled program and exposes read-only accessors (`current_line`, `options`, `transcript`, `error_overlay`, etc.).
- `Intent` captures every user-visible command (`Advance`, `FocusNext`, `SelectOption`, `Reload`, `Rerun`, `StepBack`, etc.) decoupled from key codes.
- `render(&AppState, frame)` draws the state with ratatui. Tests call it with `TestBackend` and assert on buffer contents.
- `terminal` is the only module that touches raw mode / stdin / stdout, translating crossterm key events into `Intent`s for the loop.

Step-back works by replaying intent history: on `StepBack`, the session is re-created from the stored source and the log is replayed minus its last entry. No snapshots needed — just a deterministic log.

## Using it while reading the guide

Each language chapter has a **Try it** block with the snippet command to run. Keep the player handy, run the snippet, poke at the script, then come back. The guide covers the mechanics; the player shows you exactly what lands in your event loop.

---

> **Next:** [What We're Building](../tutorial/overview.md)
