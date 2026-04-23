# Snippets

Small scripts for answering "can Bubbles do X?" Each one is 30-50 lines, runs independently through the TUI, and is built around a pirate scenario that puts the feature in context.

## Running a snippet

```sh
cargo run -p bubbles-tui -- examples/snippets/<name>.bub
```

Press `r` to reload after editing, `b` to step back through the session history.

## The snippets

| File | Scenario | Features |
|---|---|---|
| [`options.bub`](#options) | Insult sword-fight duel | Options, `<<if>>` / `<<elseif>>` / `<<else>>`, guarded options, `<<jump>>` |
| [`variables.bub`](#variables) | Grog shop with rising prices | `<<declare>>`, `<<set>>`, arithmetic, `{$interpolation}` |
| [`commands.bub`](#commands) | Treasure chest discovery | `<<command>>` dispatch, line tags (`#dramatic`, `#sfx`) |
| [`once.bub`](#once) | Barnacle Pete's kraken tale | `<<once>>` / `<<else>>` / `<<endonce>>` |
| [`saliency.bub`](#saliency) | Dockside chatter + time-of-day storyteller | Line groups (BLRV), node groups (`when:` conditions) |

---

## options

```sh
cargo run -p bubbles-tui -- examples/snippets/options.bub
```

A duelist challenges you. Whether you can deliver the devastating insult depends on `$sword_skill`. Change its `<<declare>>` value and reload to unlock the guarded option.

Shows options with `<<if>>` guards, an `<<elseif>>` chain for skill tiers, and `<<jump>>` sending each branch to a separate outcome node.

See [Options](../language/options.md) and [Conditionals](../language/conditionals.md).

---

## variables

```sh
cargo run -p bubbles-tui -- examples/snippets/variables.bub
```

Griselda's Grog Shack charges more for each mug you buy. The price and your doubloon count update in the dialogue text as they change.

Shows `<<declare>>` for typed variables, `<<set>>` with arithmetic, and `{$variable}` interpolation. The `<<if>>` guard on the buy option locks it when you run out of funds.

See [Variables](../language/variables.md), [Expressions](../language/expressions.md), and [Interpolation](../language/interpolation.md).

---

## commands

```sh
cargo run -p bubbles-tui -- examples/snippets/commands.bub
```

Prying open a cursed treasure chest fires `<<play_fanfare>>`, `<<spawn_particles>>`, and `<<apply_curse>>`. Key lines carry tags like `#dramatic` and `#sfx`.

In a real game, your event loop matches on `DialogueEvent::Command { name, args, .. }` and dispatches to audio/animation/inventory. The TUI shows commands in the transcript pane.

Line tags travel with `DialogueEvent::Line { tags, .. }`. Use them for voiceover cues, subtitle styles, camera hints, or any other per-line metadata your engine needs.

See [Commands](../language/commands.md) and [Tags and Metadata](../language/tags.md).

---

## once

```sh
cargo run -p bubbles-tui -- examples/snippets/once.bub
```

Old Barnacle Pete has a legendary kraken story. The first visit plays the full account. Every later visit plays the short acknowledgement. No flag variable needed.

Shows `<<once>>` / `<<else>>` / `<<endonce>>` with multiple independent once sequences in the same script. Use `b` to step back and watch the `<<else>>` branches fire.

See [Once Blocks](../language/once.md).

---

## saliency

```sh
cargo run -p bubbles-tui -- examples/snippets/saliency.bub
```

The dockside has two layers of variety:

**Line groups** (`=>`): five ambient worker barks cycle with `BestLeastRecentlyViewed`. Each visit picks the one seen least recently, so the scene never repeats immediately.

**Node groups**: the `Storyteller` NPC has four nodes with the same title. Bubbles picks the first whose `when:` condition is true, falling back to the final node with no condition. Change the `<<declare $time_of_day = "evening">>` value and reload (`r`) to hear the other variants.

See [Line Groups](../language/line-groups.md) and [Node Groups and Saliency](../language/node-groups.md).

---

> **Next:** [API Reference](../api-reference.md)
