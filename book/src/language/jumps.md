# Jumps and Detours

Nodes don't flow into each other automatically. You decide where execution goes next — either by **jumping** (a one-way move) or **detouring** (a call that returns).

## `<<jump>>`: one-way

```text
title: Start
---
Narrator: Welcome.
<<jump Adventure>>
===

title: Adventure
---
Narrator: Here we go.
===
```

`<<jump>>` is exactly what it sounds like. Execution leaves the current node and begins the named one. Anything after the `<<jump>>` in the original node never runs.

Jumps clear the call stack, so you can't "return" from a jump. Use this for scene changes and story transitions.

> **Tip:** If you reference a node that doesn't exist, Bubbles catches it at compile time. `compile()` returns an error pointing at the bad name — no more typos surfacing at runtime.

## `<<detour>>` and `<<return>>`: call and come back

Sometimes you want to call into a subroutine and resume where you left off.

```text
title: Tavern
---
Barkeep: What'll it be?
-> A mug of ale.
    <<detour PourAle>>
    Barkeep: Anything else?
===

title: PourAle
---
<<play_pour_sound>>
Barkeep: Here you are.
<<return>>
===
```

The flow:

1. `<<detour PourAle>>` pushes the current position and jumps to `PourAle`.
2. `PourAle` runs, finishing with `<<return>>`.
3. Execution picks up exactly where `<<detour>>` left off — the "Anything else?" line runs next.

You can detour from within a detour. The stack handles any depth.

If a detour node runs off the end without a `<<return>>`, Bubbles returns anyway. `<<return>>` is just the explicit form.

## `<<stop>>`: end the dialogue right here

When you want to exit the whole dialogue — not just the current node, and not return to the caller — use `<<stop>>`.

```text
title: Guard
---
Guard: State your business.

-> I'm just passing through.
    Guard: Move along.
-> ...
    Guard: That's enough. You're done here.
    <<stop>>

Guard: Anything else?
===
```

Unlike `<<return>>`, which pops one frame, `<<stop>>` clears the entire call stack and emits a single `DialogueComplete` event. Nothing after it in the current node runs, and no caller resumes.

Reach for `<<stop>>` for bad-ending branches, timed interruptions, or any beat where the conversation is simply over — regardless of how deep the detour stack is.

## When to jump, when to detour

- **Jump** for scene changes: `<<jump StreetAtNight>>`, `<<jump Ending>>`.
- **Detour** for reusable bits: buying an item, telling a joke, a shared "she nods and looks away" beat you want to play from several places.

A simple way to tell: if the current conversation should continue after this bit, detour. If the current conversation is over, jump.

## A richer example

```text
title: Tavern
---
<<declare $gold = 10>>

Barkeep: What'll it be?

-> Ale, 5 gold <<if $gold >= 5>>
    <<set $gold = $gold - 5>>
    <<detour PourDrink>>
-> Wine, 10 gold <<if $gold >= 10>>
    <<set $gold = $gold - 10>>
    <<detour PourDrink>>
-> Nothing. <<jump Leave>>

Barkeep: Anything else?
-> Another round. <<jump Tavern>>
-> I'm done. <<jump Leave>>
===

title: PourDrink
---
<<play_pour_sound>>
Barkeep: Here you go.
<<return>>
===

title: Leave
---
Barkeep: Safe travels.
===
```

`PourDrink` is a shared beat — both the ale and wine branches detour into it, and both come back to the "Anything else?" line. No duplication, clean flow.

## Infinite loops

Bubbles won't save you from an infinite jump cycle:

```text
title: A
---
<<jump B>>
===

title: B
---
<<jump A>>
===
```

That will spin your game until something else stops it. If you need a loop, make sure something inside the loop consumes an event (a `Line`, `Options`, or `Command`) so `next_event` actually yields.

> **Note:** Every `Line` or `Options` event is a natural yield point. A loop with at least one dialogue beat per lap is always safe.

---

> **Next:** [Once Blocks](./once.md)
