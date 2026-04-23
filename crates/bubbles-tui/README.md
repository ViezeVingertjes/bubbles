# bubbles-tui

Writer-focused terminal UI for the [`bubbles-dialogue`](../bubbles-dialogue)
runtime.  Play any `.bub` script in your terminal, inspect the full event
transcript, and step back through the session while iterating on content.

## Run it

Run the harbour showcase (two files compiled together):

```sh
cargo run -p bubbles-tui -- examples/harbour/harbour.bub examples/harbour/services.bub Harbour
```

Or run any single `.bub` file:

```sh
cargo run -p bubbles-tui -- path/to/script.bub
```

An optional trailing argument picks the start node (defaults to `Start`):

```sh
cargo run -p bubbles-tui -- path/to/script.bub MyNode
```

Multiple files are compiled into one programme, so cross-file jumps and detours work across all of them:

```sh
cargo run -p bubbles-tui -- scene.bub characters.bub shared/barks.bub
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

See the [TUI Runner guide](https://viezevingertjes.github.io/bubbles/examples/tui-runner.html)
for the full architecture write-up, and the [Bubbles guide](https://viezevingertjes.github.io/bubbles/)
for the underlying language and runtime.
