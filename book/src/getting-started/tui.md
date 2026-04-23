# Using the TUI

Before diving into the language guide, set up a playground. Trust me, it's worth it.

The `bubbles-tui` crate is a terminal dialogue player built for iterating on `.bub` scripts. Load any file, play through it, edit it, and reload it in place. It runs the same `Runner` your game would use, so what you see is exactly what a real integration would receive.

Open a second terminal alongside this guide. You'll want them side by side.

## Run the harbour

The harbour is the main example: a two-file pirate dockside scene that covers most of the language in one playable go. Start there:

```sh
cargo run -p bubbles-tui -- examples/harbour/harbour.bub examples/harbour/services.bub
```

You arrive at Barnacle Bay and meet Stumpy McGee, the cantankerous harbormaster. He wants a permit fee. Try each option, run out of gold, find the map seller around the corner. Come back to this guide when you're done.

## What you're looking at

**Transcript** (the main panel) logs every event the runner emits: node markers (`[→ Name]` / `[← Name]`), speaker lines, commands (`⚙ play_sfx clink`), and the option you chose each time. Scroll it to see the full history. The title bar shows the total event count and how far you've scrolled back.

**Options** appears at the bottom whenever a choice is active. `Tab` swaps focus between the options list and the transcript, so you can scroll back through history while a choice is waiting.

**Error overlay** pops up on any parse or runtime error. It shows the file, line number, and a snippet of the offending source. Press `x` to dismiss, or `r` to reload and try again.

## Keys

| Key | What it does |
|---|---|
| `Enter` / `Space` | Advance; confirm the focused option |
| `↑` / `k`, `↓` / `j` | Navigate options or scroll the transcript |
| `1` ... `9` | Pick an option directly by number |
| `Tab` | Swap focus between the options list and transcript |
| `b` / `Backspace` | Step back one event |
| `r` | Reload: re-read files from disk, reset everything |
| `R` (Shift+r) | Rerun: restart from the beginning, keeping variables and `<<once>>` history |
| `q` / `Esc` | Quit |

### `r` vs `R`

These two are the most important keys to know:

**`r` (reload)** picks up every change you've made to the `.bub` files on disk. Use it after editing a script. Variables reset, `<<once>>` history resets, everything starts fresh.

**`R` (rerun)** restarts from the beginning of the dialogue but keeps the current variable values and `<<once>>` seen history. Use it to reach lines that only appear on a second visit - the `<<else>>` branch of a `<<once>>` block, a node that only triggers after buying something, a counter that changes behaviour on the fifth loop.

The workflow: write a scene, press `r` to reload, play through it, press `R` to get the second-visit state, press `r` to pick up new edits.

## Run your own scripts

```sh
# single file
cargo run -p bubbles-tui -- my-scene.bub

# multiple files compiled together (cross-file jumps and detours work)
cargo run -p bubbles-tui -- main.bub services.bub characters.bub

# explicit start node (defaults to "Start" if omitted)
cargo run -p bubbles-tui -- my-scene.bub TownSquare
```

## Using it while reading the guide

Each language chapter has a **Try it** block with the snippet command to run. Keep the TUI handy, run the snippet, poke at the script, then come back. The guide covers the mechanics; the TUI shows you exactly what lands in your event loop.

---

> **Next:** [What We're Building](../tutorial/overview.md)
