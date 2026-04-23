# TUI Runner

The [`bubbles-tui`](https://github.com/ViezeVingertjes/bubbles/tree/main/crates/bubbles-tui) crate is a writer-focused terminal UI for iterating on `.bub` scripts. It drives the same `Runner` you would in a game and shows exactly what a real integration would see: node markers, speaker lines, option prompts, command emissions, and runtime errors.

## Quick start

Run the harbour showcase, a two-file pirate scene covering most language features:

```sh
cargo run -p bubbles-tui -- examples/harbour/harbour.bub examples/harbour/services.bub
```

Or jump straight to any of the focused snippets:

```sh
cargo run -p bubbles-tui -- examples/snippets/variables.bub
cargo run -p bubbles-tui -- examples/snippets/once.bub
cargo run -p bubbles-tui -- examples/snippets/saliency.bub
```

Then read [The Harbour](./harbour.md) for a feature-by-feature walkthrough, or [Snippets](./snippets.md) for a complete index.

## Running your own scripts

```sh
# single file
cargo run -p bubbles-tui -- path/to/script.bub

# multiple files compiled together (cross-file jumps and detours work)
cargo run -p bubbles-tui -- main.bub services.bub shared/barks.bub

# explicit start node (defaults to "Start" if omitted)
cargo run -p bubbles-tui -- path/to/script.bub MyStartNode
```

## Keybindings

| Key                | Action                                                |
| ------------------ | ----------------------------------------------------- |
| `Enter` / `Space`  | Advance the dialogue; commit the focused option       |
| `↑` / `k`          | Focus the previous option / scroll the transcript up  |
| `↓` / `j`          | Focus the next option / scroll the transcript down    |
| `1` … `9`          | Pick the option at that 1-based index                 |
| `Tab`              | Swap focus between the options list and the transcript |
| `PageUp` / `PageDown` | Scroll the transcript regardless of focused pane   |
| `b` / `Backspace`  | Step back one visible event                           |
| `r`                | Reload: re-read files from disk and reset all state   |
| `R` (Shift+r)      | Rerun: run again from the start, keeping variables and `<<once>>` history |
| `x`                | Dismiss the active error overlay                      |
| `q` / `Esc`        | Quit                                                  |

The difference between `r` and `R` matters when your script uses variables
or `<<once>>` blocks. Use `r` after editing a `.bub` file to pick up your
changes. Use `R` to reach lines that only appear on a second visit (e.g. an
`<<once>>..<<else>>` branch or a condition that checks a counter).

## Panels

**Transcript.** A running log of every event the runner has emitted:
node boundaries (`[→ Name]` / `[← Name]`), lines, commands (`⚙ name args
#tags`), and the options you picked (`→ chose [index] text`). It scrolls
to keep the latest entry visible. The title shows the total entry count
and the current scroll offset.

**Options.** Appears below the transcript when a choice is active.
Arrow keys or `1`-`9` pick an option; `Tab` switches focus back to the
transcript for scrolling.

**Error overlay.** Parse errors and runtime errors surface as a modal
popup with the file, line, message, and a short excerpt from the
offending source line. Press `x` to dismiss without reloading, or `r`
to re-read and recompile from disk.

## How it is architected

The crate is split so the entire UI can be exercised without a real
terminal:

- `AppState` owns the compiled program and exposes read-only accessors
  (`current_line`, `options`, `transcript`, `error_overlay`, …).
- `Intent` captures every user-visible command (`Advance`, `FocusNext`,
  `SelectOption`, `Reload`, `Rerun`, `StepBack`, …) decoupled from key codes.
- `render(&AppState, frame)` draws the state with ratatui. Tests call it
  with `TestBackend` and assert on the buffer contents.
- `terminal` is the only module that touches raw mode / stdin / stdout,
  translating `crossterm` key events into `Intent`s for the loop.

Step-back uses the recorded `Intent` history: on `StepBack` the session is
re-created from the stored source and the log is replayed minus its last
entry. That keeps the implementation snapshot-free, deterministic, and
compatible with any dialogue the runner can run.

---

> **Next:** [The Harbour](./harbour.md)
