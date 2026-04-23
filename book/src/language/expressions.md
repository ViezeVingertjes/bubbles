# Expressions

Anywhere Bubbles expects a value — the right side of `<<set>>`, the condition of an `<<if>>`, an option guard, an interpolation like `{…}` — you can use a full expression.

```text
<<set $hp = clamp($hp - $dmg * 2, 0, 100)>>
<<if $gold >= 10 && !$banned>>
You have {$gold + 5} gold after the tip.
```

## Operators

In rough order of precedence (lowest to highest):

| Category | Operators |
|---|---|
| Logical OR | `\|\|` |
| Logical AND | `&&` |
| Equality | `==`, `!=` |
| Comparison | `<`, `<=`, `>`, `>=` |
| Additive | `+`, `-` |
| Multiplicative | `*`, `/`, `%` |
| Unary | `-`, `!` |
| Grouping | `( … )` |

They behave how you'd expect from a language in the C family. Parentheses override precedence when you need them.

```text
<<if ($hp < 20 || $poisoned) && !$invulnerable>>
    Aria: You don't look well.
<<endif>>
```

## Literals

Numbers, strings, and booleans:

```text
42            <- Number
3.14          <- Number
-7            <- Number
"Aria"        <- Text
"It's cold."  <- Text (escapes: \", \\, \n)
true          <- Bool
false         <- Bool
```

String concatenation uses `+`:

```text
"Hello, " + $name + "!"
```

Both sides must be strings. See [Variables](./variables.md) for how to format a number into a string first.

## Built-in functions

Bubbles ships with a small library of functions you can call anywhere an expression is allowed:

### Numeric

| Function | Returns |
|---|---|
| `round(x)` | nearest integer |
| `floor(x)` | largest integer `<= x` |
| `ceil(x)` | smallest integer `>= x` |
| `abs(x)` | absolute value |
| `min(a, b, …)` | smallest |
| `max(a, b, …)` | largest |
| `clamp(x, lo, hi)` | `x` clamped to the `[lo, hi]` range |

### Conversions

| Function | Returns |
|---|---|
| `int(x)` | number truncated to integer (`Number`) |
| `string(x)` | value formatted as `Text` |

### Random (`rand` feature, on by default)

| Function | Returns |
|---|---|
| `random()` | uniform float in `[0, 1)` |
| `random_range(lo, hi)` | uniform int in `[lo, hi]` inclusive |
| `dice(sides, count)` | sum of `count` rolls of a `sides`-sided die |

### Narrative helpers

| Function | Returns |
|---|---|
| `visited(node)` | `true` once you've run the named node at least once |
| `visited_count(node)` | how many times the node has completed |
| `plural(n, sing, plur)` | `sing` if `|n| == 1`, else `plur` |
| `select(key, "k1:text\|k2:text\|other:fallback")` | picks a branch by key |

`plural` and `select` are designed for localisation (see [Localisation](../integration/localisation.md)):

```text
You found {$n} {plural($n, "gem", "gems")}.
{select($gender, "m:He|f:She|other:They")} nods.
```

## Registering your own functions

Your game probably has its own notions — a faction reputation check, a distance calculation, a cooldown lookup. Register closures with the runner's [`FunctionLibrary`](../integration/functions.md):

```rust,ignore
runner.library_mut().register("faction_at_least", |args| {
    let Some(bubbles::Value::Text(name)) = args.first() else {
        return Err(bubbles::DialogueError::Runtime("name required".into()));
    };
    let Some(bubbles::Value::Number(thresh)) = args.get(1) else {
        return Err(bubbles::DialogueError::Runtime("threshold required".into()));
    };
    let score = game::faction_score(name);
    Ok(bubbles::Value::Bool(score >= *thresh))
});
```

Then in your dialogue:

```text
<<if faction_at_least("thieves_guild", 50)>>
    Aria: One of us, are you?
<<endif>>
```

## A longer example

Let's put a handful of these together. Imagine a skill check.

```text
title: SkillCheck
---
<<declare $luck = 4>>
<<declare $attempts = 0>>

<<set $attempts = $attempts + 1>>
<<set $roll = dice(20, 1) + $luck>>

<<if $roll >= 15>>
    Narrator: You thread the needle. Attempt {$attempts}, roll {$roll}.
<<elseif $roll >= 10>>
    Narrator: Close. Try again? Attempt {$attempts}, roll {$roll}.
<<else>>
    Narrator: Nope. ({$roll}) {plural($attempts, "attempt", "attempts")} and counting.
<<endif>>
===
```

`dice`, `plural`, `<<if>>`, variables, interpolation — all working together. You can read what this node does end-to-end without once looking up an API.

---

> **Next:** [Conditionals](./conditionals.md)
