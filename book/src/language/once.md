# Once Blocks

"Say something different the first time" is a thing you'll write a hundred times in any game. Bubbles has a dedicated form for it.

```text
<<once>>
    Aria: Welcome to the ruins. They say the old king's crown is still down there.
<<else>>
    Aria: Back again, I see. Find anything this time?
<<endonce>>
```

First time the runner reaches that block: the first branch runs. Every time after: the `<<else>>` branch runs. If there's no `<<else>>`, subsequent visits skip the block entirely.

## Three shapes

**Run once, then silent:**

```text
<<once>>
    Narrator: The door creaks open for the first time.
<<endonce>>
```

**Run once, then something else forever:**

```text
<<once>>
    Narrator: A bell tolls in the distance.
<<else>>
    Narrator: The bell is silent now.
<<endonce>>
```

**Run once, but only if a condition is true:**

```text
<<once if $has_amulet>>
    Narrator: The amulet glows. You feel warmer.
<<endonce>>
```

With `<<once if>>`, the block "consumes" its one shot the first time the condition is true. After that it never runs again - even if the condition becomes true later. If you want the block to wait for its moment, use the conditional guard, not a plain `<<if>>`.

## How "once" is tracked

Each once block gets a stable id derived from the node and its position. That id lives in the runner's "seen" set, which is part of the [`RunnerSnapshot`](../advanced/save-load.md). Save and load pick up right where they left off - including which once blocks have fired.

> **Tip:** Because once-ness is stored with the runner, it resets if you make a fresh `Runner`. That's usually what you want (new game, fresh first-times) - but if you're scripting a cutscene you want to replay, remember to save/restore the snapshot.

## A little NPC

Everyone writes this NPC at some point. With `<<once>>`, it's three blocks of dialogue.

```text
title: Merchant
---
<<once>>
    Merchant: Oh! A new face. Welcome to my shop.
<<else>>
    <<once>>
        Merchant: Back again? Good memory - I'm glad you came.
    <<else>>
        Merchant: Hello again.
    <<endonce>>
<<endonce>>

Merchant: What can I get you?
-> Show me your wares.
    <<detour Wares>>
-> Nothing today.
    Merchant: Always welcome.
===
```

Three distinct greetings: first visit, second visit, all subsequent visits - without a single variable. The [harbour example](../examples/harbour.md) uses the same pattern for rumours that play out in full on first ask and are briefly acknowledged on return.

## `<<once>>` vs a flag variable

You *could* write:

```text
<<declare $greeted = false>>
<<if !$greeted>>
    Aria: Welcome.
    <<set $greeted = true>>
<<else>>
    Aria: Back again.
<<endif>>
```

...and it works. But `<<once>>` is shorter, harder to mess up (no forgetting the `<<set>>`), and the state is saved for you automatically. Reach for a variable when you need to *read* the flag elsewhere. Reach for `<<once>>` when you just need a one-shot.

---

> **Try it:** [`examples/snippets/once.bub`](../../examples/snippets/once.bub): Barnacle Pete's kraken tale, epic on first visit and acknowledged on every repeat. Run to the end, then press `R` (rerun) to see the second-visit lines without losing the once history. Press `b` to step back through individual events.
> ```sh
> cargo run -p bubbles-tui -- examples/snippets/once.bub
> ```

---

> **Next:** [Interpolation](./interpolation.md)
