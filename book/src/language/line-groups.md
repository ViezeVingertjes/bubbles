# Line Groups

Players notice when an NPC says the same thing every time. Line groups let you write a handful of variants and let Bubbles pick which one plays - so your dock worker doesn't shout "Oi, watch yer step!" on every single visit.

```text
=> Barkeep: The fire crackles nearby.
=> Barkeep: A minstrel plucks softly in the corner.
=> Barkeep: The smell of roasting meat fills the air.
```

Three lines, all starting with `=>`. When the runner reaches this block, the active **saliency strategy** picks one. That one line emits a `DialogueEvent::Line`. The others stay silent.

With `BestLeastRecentlyViewed` (BLRV), the player hears a different line each visit, cycling through all three before any repeats. Set it once on the runner:

```rust,ignore
use bubbles::saliency::BestLeastRecentlyViewed;

runner.set_saliency(BestLeastRecentlyViewed::new());
```

Your game receives a normal `DialogueEvent::Line` - it doesn't know there were three variants. It just gets the chosen one.

## Saliency strategies

| Strategy | Behaviour |
|---|---|
| `FirstAvailable` | Always the first eligible line (default; deterministic) |
| `RandomAvailable` | Uniformly random (needs the `rand` feature) |
| `BestLeastRecentlyViewed` | Prefers the one seen least recently |

`FirstAvailable` is the default. It's great for node groups where you've sorted variants by priority (see [Node Groups and Saliency](./node-groups.md)). For ambient barks, BLRV is almost always the right pick.

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
