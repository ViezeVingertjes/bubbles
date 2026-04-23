# TUI Runner

The [`bubbles-tui`](https://github.com/ViezeVingertjes/bubbles/tree/main/crates/bubbles-tui) crate is a writer-focused terminal UI for iterating on `.bub` scripts. It drives the same `Runner` you would in a game and shows exactly what a real integration would see: node markers, speaker lines, option prompts, command emissions, and runtime errors.

```sh
cargo run -p bubbles-tui -- path/to/script.bub Start
```

## Keybindings

| Key                | Action                                                |
| ------------------ | ----------------------------------------------------- |
| `Enter` / `Space`  | Advance the dialogue; commit the focused option       |
| `↑` / `k`          | Focus the previous option / scroll the transcript up  |
| `↓` / `j`          | Focus the next option / scroll the transcript down    |
| `1` … `9`          | Pick the option at that 1-based index                 |
| `Tab`              | Swap focus between the dialogue and transcript panes  |
| `PageUp` / `PageDown` | Scroll the transcript regardless of focused pane   |
| `b` / `Backspace`  | Step back one visible event                           |
| `r`                | Reload the script from disk                           |
| `x`                | Dismiss the active error overlay                      |
| `q` / `Esc`        | Quit                                                  |

## Panels

**Dialogue pane.** The active line (speaker + text) plus the currently
available options, with focus and guard markers. When the script ends, the
pane shows `[end of dialogue]`.

**Transcript pane.** A running log of every event the runner has emitted —
node boundaries (`[→ Name]` / `[← Name]`), lines, commands (`⚙ name args
#tags`), and the options you picked (`→ chose [index] text`). The title
shows the total entry count and the current scroll offset.

**Error overlay.** Parse errors and runtime errors surface as a modal
popup with the file, line, message, and a short excerpt from the
offending source line. Press `x` to dismiss without reloading, or `r`
to recompile from the stored source.

## How it is architected

The crate is split so the entire UI can be exercised without a real
terminal:

- `AppState` owns the compiled program and exposes read-only accessors
  (`current_line`, `options`, `transcript`, `error_overlay`, …).
- `Intent` captures every user-visible command (`Advance`, `FocusNext`,
  `SelectOption`, `Reload`, `StepBack`, …) decoupled from key codes.
- `render(&AppState, frame)` draws the state with ratatui. Tests call it
  with `TestBackend` and assert on the buffer contents.
- `terminal` is the only module that touches raw mode / stdin / stdout,
  translating `crossterm` key events into `Intent`s for the loop.

Step-back uses the recorded `Intent` history: on `StepBack` the session is
re-created from the stored source and the log is replayed minus its last
entry. That keeps the implementation snapshot-free, deterministic, and
compatible with any dialogue the runner can run.

---

> **Next:** [The Tavern](./tavern.md)
