# Node Groups and Saliency

Line groups (from the last chapter) pick a line. Node groups pick a whole *node*. Write several nodes with the same title and different `when:` conditions, and let Bubbles choose the most appropriate one for the current game state.

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

- **Different greetings based on reputation, quest state, or time of day.**
- **Randomised vignettes** in a hub scene - mini-scenes the player might trigger a handful of times.
- **Gated content** where the same entry point has wildly different beats depending on who the player has become.

If the variation is a single line, use a line group. If it's a whole scene with its own options and branches, use a node group.

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

> **Next:** [The Runner Lifecycle](../integration/runner.md)
