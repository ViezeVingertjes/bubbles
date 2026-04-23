# Add State

Right now Mira will sell to anyone, even a broke traveller with zero gold. Let's fix that.

Update `mira.bub`:

```text
title: Start
---
<<declare $gold = 8>>

Mira: Hello, traveller. Potions? I've got potions.
Mira: Five gold each. Take it or leave it.
-> Buy a health potion (5 gold) <<if $gold >= 5>>
    <<set $gold = $gold - 5>>
    Mira: Pleasure doing business. You've got {$gold} gold left.
-> I can't afford that right now. <<if $gold < 5>>
    Mira: Come back when your pockets are heavier.
-> Just looking.
    Mira: ...Still just looking?
===
```

Reload with `r`. Try buying. Try changing the `<<declare $gold = 8>>` to `<<declare $gold = 3>>`, reload, and notice the buy option becomes locked.

## What's new

**`<<declare>>`** registers the variable with its type and starting value. Bubbles infers the type from the literal: `8` is a number, `"hello"` is text, `true` is a bool. It only sets the value on the first run - subsequent runs (and save/load cycles) leave the existing value alone.

```text
<<declare $gold = 8>>   <- first run: $gold = 8
                        <- second run: $gold unchanged (already exists)
```

Prefer `<<declare>>` for anything that should persist between sessions. See [Variables](../language/variables.md) for the full picture.

**`<<set>>`** assigns a new value. The right side is a full expression, so arithmetic works:

```text
<<set $gold = $gold - 5>>
```

**Option guards.** `<<if $gold >= 5>>` after an option text is a guard. When the guard is false, the option stays visible in the list but `available: false` in the event - your UI can grey it out or hide it. Bubbles won't let the player select a locked option.

```rust,ignore
DialogueEvent::Options(opts) => {
    for opt in &opts {
        // opt.available tells you whether to render it as selectable
    }
}
```

**`{$gold}` interpolation.** Variables (and any expression) inside `{...}` get substituted into the line text before the event reaches you. By the time your game sees `"You've got {$gold} gold left."`, it already reads `"You've got 3 gold left."` No parsing on your end.

## Self-check

Before moving on:

- What happens if you set `$gold = 5` and buy once? What does `$gold` become?
- What's the difference between a guard (`<<if>>` on an option) and a conditional block (`<<if>>` in the body)?
- Why use `<<declare>>` instead of just `<<set>>`?

*(Answers: 0; guard = option is shown but locked, conditional = lines only run at all when true; `<<declare>>` preserves values across save/load and is a no-op on revisit.)*

## Where this is going

State is working. Now let's make the scene feel like a real place - ambient sounds, a first-visit greeting that doesn't repeat, and a command to let your engine play a sound effect.

---

> **Next:** [Add Polish](./add-polish.md)
