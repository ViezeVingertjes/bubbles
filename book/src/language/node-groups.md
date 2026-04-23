# Node Groups and Saliency

Line groups pick a line. Node groups pick a whole *node*. It's the same idea, one level up: write several nodes with the same title and different `when:` conditions, and Bubbles picks the right one for the current game state.

This is the pattern for NPCs that feel different depending on who you've become. The baker who hates you after you stole from her. The guard who suddenly has respect for you after the quest. The tavern that transforms on festival night.

```text
title: GreetPlayer
when: $reputation >= 50
---
Baker: Ah, my favourite customer! A warm loaf for you, on the house.
===

title: GreetPlayer
when: $reputation < 0
---
Baker: Out. You're not welcome here.
===

title: GreetPlayer
---
Baker: Good morning. What can I get you?
===
```

Three nodes, all called `GreetPlayer`. When you `<<jump GreetPlayer>>`, Bubbles filters down to the ones whose `when:` is currently true (the third node has no `when:`, so it's always eligible) and hands the result to the active saliency strategy.

## The node group rules

- Every node with the same `title:` forms a group.
- Each can have its own `when: <expression>` header.
- A node without `when:` is always eligible - it's the fallback.
- The saliency strategy (see [Saliency Strategies](../integration/saliency.md)) picks one eligible node to run.
- If nothing is eligible - no `when:` matches, no fallback node exists - Bubbles returns a runtime error.

> **Tip:** Always include an unconditional fallback in a group. It's a belt-and-braces guarantee that "we tried to greet the player" never turns into a runtime crash.

## Order of precedence

With the default `FirstAvailable` strategy, the **first declared** eligible node wins. So write your most specific `when:` conditions *first*, and the generic fallback *last*:

```text
title: Entrance
when: $quest_complete && $hero_level >= 10
---
Guard: Hero! The captain wants to see you.
===

title: Entrance
when: $quest_complete
---
Guard: Well done out there.
===

title: Entrance
---
Guard: Move along.
===
```

Read top to bottom, it reads like a priority list: "if all the big stuff is true, take that branch; else if some of it is true, take that one; else the plain one."

## Variety via BLRV

Swap `FirstAvailable` for `BestLeastRecentlyViewed` and node groups become a proper "pick something fresh" mechanism. If two or more nodes are eligible, BLRV prefers the one you've seen least recently - great for re-usable vignettes, daily-life scenes, or barks that should feel alive without a full state machine.

```rust,ignore
runner.set_saliency(bubbles::BestLeastRecentlyViewed::new());
```

## When to use node groups

If the variation is a **single line**, reach for a line group. If it's a whole scene with its own options and branches, use a node group.

Great for:

- Different greetings based on reputation, quest state, or time of day
- Randomised vignettes in a hub scene - a handful of mini-scenes the player might trigger
- Gated content where the same entry point has wildly different beats depending on who the player has become

## A hub scene

Let's do a tavern entrance that feels different every time.

```text
title: TavernEntry
when: $time == "night" && $festival
---
Barkeep: You made it for the feast! Get in here.
<<play_music tavern_festive>>
<<jump TavernFestival>>
===

title: TavernEntry
when: $time == "night"
---
Barkeep: Evening. Fire's warm.
<<jump TavernNight>>
===

title: TavernEntry
when: $time == "day"
---
Barkeep: Early one today, are we?
<<jump TavernDay>>
===

title: TavernEntry
---
Barkeep: Welcome.
===
```

Four variants, ranked from most specific to fallback. Every `<<jump TavernEntry>>` lands on the right scene for the moment.

---

> **Try it:** [`examples/snippets/saliency.bub`](../../examples/snippets/saliency.bub): a storyteller NPC with four `Storyteller` nodes where `when: $time_of_day == "..."` picks the right one. Change the `<<declare>>` value and press `r` to reload (re-reads from disk).
> ```sh
> cargo run -p bubbles-tui -- examples/snippets/saliency.bub
> ```

---

> **Next:** [The Runner Lifecycle](../integration/runner.md)
