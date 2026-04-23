# Variables

Your game has state: gold, health, quest flags, faction reputation, whether the player has already met a character. Variables let your `.bub` scripts read and write that state.

Bubbles variables start with `$` and have one of three types:

| Type | Example values |
|---|---|
| `Number` | `42`, `3.14`, `-1` |
| `Text` | `"Aria"`, `"Welcome!"` |
| `Bool` | `true`, `false` |

Once a variable has a type, it's fixed. Storing a string into a number variable is an error, not a silent coercion.

## Declaring

Use `<<declare>>` (usually at the top of a starting node) to register a variable with its type and default value. It initialises the variable once - subsequent runs leave the existing value alone.

```text
title: Tavern
---
<<declare $gold = 50>>
<<declare $name = "stranger">>
<<declare $greeted = false>>

Barkeep: Evening, {$name}.
===
```

First visit: `$gold = 50`, `$name = "stranger"`, `$greeted = false`. Second visit (after save/load or a jump back): values preserved. `<<declare>>` is a no-op if the variable already exists.

> **Tip:** Use `<<declare>>` for anything that should persist - gold, quest flags, reputation counters. It makes save/load work automatically: old saves carry their values, new variables get their defaults.

## Assigning

`<<set>>` changes a value anywhere in a node body:

```text
<<set $gold = 100>>
<<set $gold = $gold + 10>>
<<set $greeted = true>>
<<set $name = "Aria">>
```

The right side is a full [expression](./expressions.md), so arithmetic, comparisons, and function calls all work:

```text
<<set $hp = clamp($hp - $dmg, 0, 100)>>
<<set $greeting = "Hello, " + $name>>
```

## Reading

Reference a variable by name in any expression:

```text
<<if $gold >= 10>>
    Merchant: You can afford it.
<<endif>>
```

In line text, use `{...}` [interpolation](./interpolation.md):

```text
Merchant: That'll be 10 gold. You have {$gold}.
```

## Type safety

Types are fixed. Mixing them the wrong way fails at compile time when possible, at runtime with a clear error otherwise.

```text
<<declare $gold = 50>>
<<set $gold = "broke">>   # runtime error: type mismatch
```

Concatenating strings with `+` is fine when both sides are strings:

```text
<<set $greeting = "Hello, " + $name>>   # OK
<<set $bad = "gold: " + $gold>>         # error: mix of string and number
```

To format a number into a string, use the built-in `string()`:

```text
<<set $label = "gold: " + string($gold)>>   # OK
```

## Introspection

The `Program` API exposes every declared variable. Useful for pre-populating UIs, building save migrations, or validating external data:

```rust,ignore
for decl in program.variable_declarations() {
    println!("{} = {}", decl.name, decl.default_src);
}
```

That lists every `<<declare>>` across the whole program, with its source text.

## `declare` vs `set` - quick rule

- **`<<declare>>`** for game state that should persist: gold, HP, faction rep, quest flags.
- **`<<set>>`** when the value should always reset - a per-scene counter, a temp calculation result.

```text
<<declare $reputation = 0>>   # persists across saves
<<set $tmp_roll = dice(20, 1) + $luck>>   # scratch value, fine to clobber
```

> **Note:** Storage is pluggable. If `HashMapStorage` isn't enough - maybe you want variables to live in your game's save system - implement the [`VariableStorage`](../integration/storage.md) trait. Bubbles never touches your state except through those two methods.

---

> **Try it:** [`examples/snippets/variables.bub`](../../examples/snippets/variables.bub): a grog shop where the price rises with each purchase, showing `<<declare>>`, `<<set>>`, arithmetic, and interpolation in action.
> ```sh
> cargo run -p bubbles-tui -- examples/snippets/variables.bub
> ```

---

> **Next:** [Expressions](./expressions.md)
