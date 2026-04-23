# TUI Runner

The [`bubbles-tui`](https://github.com/ViezeVingertjes/bubbles/tree/main/crates/bubbles-tui) crate is a writer-focused terminal UI for iterating on `.bub` scripts. It drives the same `Runner` you'd use in a real game and shows exactly what a working integration sees: node markers, lines, options, commands, and runtime errors.

For the full usage guide (panels, keys, `r` vs `R`, running your own files), see [Using the TUI](../getting-started/tui.md) in Getting Started.

## Quick start

Run the harbour showcase:

```sh
cargo run -p bubbles-tui -- examples/harbour/harbour.bub examples/harbour/services.bub
```

Or jump to a focused snippet:

```sh
cargo run -p bubbles-tui -- examples/snippets/variables.bub
cargo run -p bubbles-tui -- examples/snippets/once.bub
cargo run -p bubbles-tui -- examples/snippets/saliency.bub
```

See [The Harbour](./harbour.md) for a feature walkthrough, or [Snippets](./snippets.md) for a full recipe index.

## How it's built

The crate is split so the entire UI can be exercised without a real terminal:

- `AppState` owns the compiled program and exposes read-only accessors (`current_line`, `options`, `transcript`, `error_overlay`, etc.).
- `Intent` captures every user-visible command (`Advance`, `FocusNext`, `SelectOption`, `Reload`, `Rerun`, `StepBack`, etc.) decoupled from key codes.
- `render(&AppState, frame)` draws the state with ratatui. Tests call it with `TestBackend` and assert on buffer contents.
- `terminal` is the only module that touches raw mode / stdin / stdout, translating crossterm key events into `Intent`s for the loop.

Step-back works by replaying intent history: on `StepBack`, the session is re-created from the stored source and the log is replayed minus its last entry. No snapshots needed - just a deterministic log.

---

> **Next:** [The Harbour](./harbour.md)
