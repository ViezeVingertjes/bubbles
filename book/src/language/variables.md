# Variables

Bubbles variables start with a `$` and have one of three types:

| Type | Example |
|---|---|
| `Number` | `42`, `3.14`, `-1` |
| `Text` | `"Aria"`, `"Welcome!"` |
| `Bool` | `true`, `false` |

Once a variable has a type, that type is fixed. Trying to store a string into a number variable is a compile- or run-time error, not a silent coercion.

## Declaring a variable

Use `<<declare>>` at the top of a node (usually the starting one). It initialises the variable **once** — subsequent runs leave the existing value alone.

```text
title: Tavern
---
<<declare $gold = 50>>
<<declare $name = "stranger">>
<<declare $greeted = false>>

Barkeep: Evening, {$name}.
===
```

First visit: `$gold = 50`, `$name = "stranger"`, `$greeted = false`.
Second visit (after a save/load or a jump back): values preserved — `<<declare>>` is a no-op if the variable already exists.

> **Tip:** Prefer `<<declare>>` over `<<set>>` for initial values. It makes save/load "just work": your existing saves carry over, new variables get their defaults.

## Assigning

Use `<<set>>` anywhere in a node body:

```text
<<set $gold = 100>>
<<set $gold = $gold + 10>>
<<set $greeted = true>>
<<set $name = "Aria">>
```

The right-hand side is a full [expression](./expressions.md), so arithmetic and function calls are fair game:

```text
<<set $hp = clamp($hp - $dmg, 0, 100)>>
<<set $greeting = "Hello, " + $name>>
```

## Reading variables

You reference a variable by writing its name. In an expression:

```text
<<if $gold >= 10>>
    Merchant: You can afford it.
<<endif>>
```

In line text, via `{…}` [interpolation](./interpolation.md):

```text
Merchant: That'll be 10 gold. You have {$gold}.
```

## Type safety

Variables are typed. Expressions that mix types the wrong way fail at compile time when possible, otherwise at runtime with a clear error.

```text
<<declare $gold = 50>>
<<set $gold = "broke">>   # runtime error: type mismatch
```

Concatenating strings with `+` is fine if both sides are strings:

```text
<<set $name = "Aria">>
<<set $greeting = "Hello, " + $name>>   # OK
<<set $bad = "gold: " + $gold>>         # error: mix of string and number
```

To format numbers into strings, use the built-in `string()` function (see [Expressions](./expressions.md)):

```text
<<set $bad = "gold: " + string($gold)>>   # OK
```

## Introspection

Bubbles exposes declared variables through the `Program` API so you can pre-populate UI, build save migrations, or validate configs:

```rust,ignore
for decl in program.variable_declarations() {
    println!("{} = {}", decl.name, decl.default_src);
}
```

That lists every `<<declare>>`'d variable across the whole program, with its source text.

## Smart variables: when to `declare` vs `set`

Rules of thumb:

- **Declare** values that are *part of the game state*. Gold, HP, faction reputation, quest flags.
- **Set** values when they should *always* reset — for instance, a one-shot counter inside a single scene.

```text
<<declare $reputation = 0>>   # persists across saves
<<set $intro_seen = false>>   # reset each time this node starts
```

> **Note:** Variable storage is pluggable. If `HashMapStorage` doesn't cut it — maybe you want to back it with your game's own save system — implement the [`VariableStorage`](../integration/storage.md) trait. Bubbles never touches your state except through that interface.

---

> **Next:** [Expressions](./expressions.md)
