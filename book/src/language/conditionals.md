# Conditionals

Branch on anything that evaluates to a `Bool`.

```text
<<if $gold >= 10>>
    Merchant: A pleasure doing business.
<<elseif $gold >= 5>>
    Merchant: Every coin counts, friend.
<<else>>
    Merchant: Come back when you're flush.
<<endif>>
```

Four directives, all required to balance:

- `<<if condition>>` — opens a block.
- `<<elseif condition>>` — any number of these.
- `<<else>>` — at most one, always last.
- `<<endif>>` — closes the block.

Each branch is a full body — lines, options, commands, nested `<<if>>`s, all fair game.

## Nested conditionals

You can nest as deep as the story needs:

```text
<<if $has_key>>
    <<if $door_locked>>
        Aria: The key fits. The door groans open.
        <<set $door_locked = false>>
    <<else>>
        Aria: The door's already open.
    <<endif>>
<<else>>
    Aria: Locked, and no key. We'll find another way.
<<endif>>
```

Keep it readable. If a block starts looking like a pyramid, consider splitting it into separate nodes.

## Option guards vs `<<if>>`

Two different tools. Know when to reach for which.

**`<<if>>`** changes what gets run. The lines inside only execute when the condition is true.

```text
<<if $hp < 20>>
    Aria: You're bleeding.
<<endif>>
```

**Option guards** change whether an option is *available*. The player sees the option either way; it's locked if the guard is false.

```text
-> Drink a potion <<if $potions > 0>>
    Aria: That's better.
```

Rule of thumb: if the player should *know* the option exists but isn't accessible, use a guard. If they shouldn't even see that branch, use `<<if>>` around the `->`.

## `<<once>>`: the special case

If you want "run this the first time only," you don't need to manage a variable — Bubbles has a dedicated form:

```text
<<once>>
    Aria: Welcome to the ruins.
<<else>>
    Aria: Here again, I see.
<<endonce>>
```

It's covered in full in [Once Blocks](./once.md).

## A practical scene

```text
title: BakeryDoor
---
<<declare $knocked = false>>
<<declare $baker_awake = false>>

<<if $baker_awake>>
    Baker: We're open. Come in.
<<elseif $knocked>>
    Baker: (grumbling from upstairs) One moment!
    <<set $baker_awake = true>>
<<else>>
    Narrator: The door is shut. A sign says "Back at dawn."
    -> Knock anyway.
        <<set $knocked = true>>
        <<jump BakeryDoor>>
    -> Come back later.
        <<jump Street>>
<<endif>>
===
```

Three states, one node, one `<<jump>>` back to itself to re-run the logic after the knock flips the flag. That's a very common Bubbles pattern.

---

> **Next:** [Jumps and Detours](./jumps.md)
