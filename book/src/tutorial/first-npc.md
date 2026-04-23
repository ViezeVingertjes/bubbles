# Your First NPC

Let's get Mira talking. Paste this into `mira.bub`:

```text
title: Start
---
Mira: Hello, traveller. Potions? I've got potions.
Mira: Best prices in the harbour district. Probably.
-> How much for a health potion?
    Mira: Five gold. Non-negotiable.
-> Just looking.
    Mira: You've been just looking for ten minutes.
===
```

Run it:

```sh
cargo run -p bubbles-tui -- mira.bub
```

Pick an option. Notice what happens: both branches lead to a line from Mira, then the dialogue ends. Not much of a sales pitch yet, but it works.

## What's in here

**The node.** Everything between `title: Start`, the `---` separator, and the closing `===` is one node. Nodes are the chunks your game jumps between. This one is called `Start`, which is the default entry point.

**Speaker lines.** `Mira: ...` is a speaker line. The name before the colon becomes the `speaker` field in `DialogueEvent::Line`. Lines with no name are narrator lines - `speaker` arrives as `None`.

**Options.** Lines starting with `->` are choices. When the runner hits a set of them, it emits `DialogueEvent::Options(...)` and waits. Your game shows the choices, the player picks one, you call `runner.select_option(i)`, and execution continues into that option's indented body.

```rust,ignore
DialogueEvent::Options(opts) => {
    // opts[0].text = "How much for a health potion?"
    // opts[1].text = "Just looking."
    runner.select_option(player_choice)?;
}
```

**Fall-through.** After both option bodies finish, there's nothing else in the node - so the dialogue ends. That's fine for now.

## Try it

Change Mira's greeting to something grumpier. Press `r` in the TUI to reload. The change shows up immediately.

Add a third option. Reload again. The new choice appears in the options list.

Try putting some lines *after* the options block (outside the indented bodies, back at zero indent). Those lines run after whichever option the player picks.

```text
===          <- add something before this
===
```

Something like:

```text
Mira: Anyway. Come back if you change your mind.
===
```

That line runs no matter which option was chosen. Handy for "and then we get back to business" beats.

## Where this is going

The dialogue works, but Mira doesn't care whether you can actually pay. Let's add gold.

---

> **Next:** [Add State](./add-state.md)
