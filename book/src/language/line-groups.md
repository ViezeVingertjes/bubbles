# Line Groups

Games talk a lot. And when they repeat themselves, players notice. Line groups are Bubbles' answer: write several variants of a line, and let the runner pick which one plays.

```text
=> Barkeep: The fire crackles nearby.
=> Barkeep: A minstrel plucks softly in the corner.
=> Barkeep: The smell of roasting meat fills the air.
```

Three lines, all starting with `=>`. When the runner reaches this block, it asks the active **saliency strategy** to pick one. That one line emits a `DialogueEvent::Line`. The other two stay quiet.

## Saliency at a glance

A [saliency strategy](../integration/saliency.md) is "the rule for picking." Bubbles ships three:

| Strategy | Behaviour |
|---|---|
| `FirstAvailable` | Always the first eligible line (default; deterministic) |
| `RandomAvailable` | Uniformly random (needs the `rand` feature) |
| `BestLeastRecentlyViewed` | Prefers the one you've heard least recently |

The last one - **BLRV** - is usually what you want for ambient barks. It guarantees variety: the player never hears the same line twice in a row, and every variant eventually comes up.

Pick a strategy once on the runner:

```rust,ignore
use bubbles::saliency::BestLeastRecentlyViewed;

runner.set_saliency(BestLeastRecentlyViewed::new());
```

## Conditions on variants

Each `=>` line can have its own guard:

```text
=> <<if $weather == "rain">> Barkeep: Dreary out, isn't it?
=> <<if $weather == "snow">> Barkeep: Mind the ice on the steps.
=> Barkeep: Another fine evening.
```

The strategy only picks among variants whose guard is true. The last line has no guard, so it's always eligible - a nice "default" to keep the group from going silent.

## Mixing with speaker, tags, and commands

`=>` lines behave like any other line: they can have a speaker, tags, even a `#line:id` for localisation.

```text
=> Barkeep: Welcome back. #line:tavern_greet_01
=> Barkeep: Evening, stranger. #line:tavern_greet_02 #warm
=> Barkeep: Mind the step. #line:tavern_greet_03
```

You can also put commands inside a variant's indented body:

```text
=> Barkeep: The fire snaps louder than usual.
    <<shake_camera 0.05>>
=> Barkeep: The fire settles to embers.
```

## A talkative guard

Line groups shine for NPCs who hang around:

```text
title: Guard
---
Guard: Halt! ... Oh, it's you.

=> Guard: Quiet day, thankfully.
=> Guard: Thought I saw a shadow on the wall. Probably nothing.
=> Guard: My feet are killing me.
=> Guard: Heard anything from the capital?

-> Ask about the road.
    <<jump GuardRoad>>
-> Be on your way.
===
```

Every time you pass the guard, you hear one of four lines - and (with BLRV) probably a different one. No scripting, no counters, no bespoke code.

## Not the same as options

Easy to confuse at first. Quick table:

| | Options (`->`) | Line groups (`=>`) |
|---|---|---|
| Who picks | The player | The saliency strategy |
| Event emitted | `Options` | `Line` |
| Intent | Branching story | Variant / flavour |

`->` means "give the player a choice." `=>` means "you choose one for me."

---

> **Try it:** [`examples/snippets/saliency.bub`](../../examples/snippets/saliency.bub): five ambient dock worker barks cycling with BLRV so the same line never plays twice in a row.
> ```sh
> cargo run -p bubbles-tui -- examples/snippets/saliency.bub
> ```

---

> **Next:** [Node Groups and Saliency](./node-groups.md)
