# What We're Building

This tutorial builds a real game dialogue from scratch in four sessions. Each one adds a layer, explains why it works, and links to the full language reference when there's more to know.

By the end you'll have Mira, a potion seller who remembers your first visit, locks her prices when you're broke, plays ambient sounds via commands, and never repeats the same harbour noise twice.

Here's the complete `mira.bub` you'll end up with:

```text
title: Start
---
<<declare $gold = 8>>

=> Seagull: Squawk!
=> A cart rumbles past with barrels of fish.
=> The salt air stings your eyes.

<<once>>
    Mira: Oh! A new customer. Fresh stock just arrived, as luck would have it.
<<else>>
    Mira: Back again? The price hasn't changed.
<<endonce>>

Mira: Five gold for a health potion. Best deal in the harbour district.
-> Buy a health potion (5 gold) <<if $gold >= 5>>
    <<set $gold = $gold - 5>>
    <<play_sfx "clink">>
    Mira: Pleasure doing business. You've got {$gold} gold left.
-> I can't afford that right now. <<if $gold < 5>>
    Mira: Come back when your pockets are heavier.
-> Just looking.
    Mira: ...You're still just looking?
===
```

About 30 lines. Playable in under a minute. Every feature fits in a single node, so you can see the whole script without scrolling.

## What you'll need

- A Rust project with the **bubbles-dialogue** dependency in `Cargo.toml` and **`use bubbles::…`** in code (see [Your First Dialogue](../getting-started/first-dialogue.md) if you haven't done this yet)
- The TUI runner open in a second terminal (see [Using the TUI](../getting-started/tui.md))
- A file called `mira.bub` somewhere you can edit and reload

Create the file now. You'll paste into it as each session adds features.

---

> **Next:** [Your First NPC](./first-npc.md)
