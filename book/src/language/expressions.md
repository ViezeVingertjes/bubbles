# Expressions

Wherever Bubbles expects a value - the right side of `<<set>>`, the condition of `<<if>>`, an option guard, or a `{...}` interpolation - you can write a full expression. Arithmetic, comparisons, logic, function calls, all of it.

```text
<<set $hp = clamp($hp - $dmg * 2, 0, 100)>>
<<if $gold >= 10 && !$banned>>
You have {$gold + 5} gold after the tip.
```

## Operators

In rough order of precedence (lowest first):

| Category | Operators |
|---|---|
| Logical OR | `\|\|` |
| Logical AND | `&&` |
| Equality | `==`, `!=` |
| Comparison | `<`, `<=`, `>`, `>=` |
| Additive | `+`, `-` |
| Multiplicative | `*`, `/`, `%` |
| Unary | `-`, `!` |
| Grouping | `( ... )` |

They work the way you'd expect from any C-family language. Use parentheses to override precedence:

```text
<<if ($hp < 20 || $poisoned) && !$invulnerable>>
    Aria: You don't look well.
<<endif>>
```

## Literals

```text
42            <- Number
3.14          <- Number
-7            <- Number
"Aria"        <- Text
"It's cold."  <- Text (escapes: \", \\, \n)
true          <- Bool
false         <- Bool
```

String concatenation uses `+` - both sides must be strings. See [Variables](./variables.md) for how to format a number into a string first.

## Built-in functions

### Numeric

| Function | Returns |
|---|---|
| `round(x)` | nearest integer |
| `floor(x)` | largest integer `<= x` |
| `ceil(x)` | smallest integer `>= x` |
| `abs(x)` | absolute value |
| `min(a, b, ...)` | smallest |
| `max(a, b, ...)` | largest |
| `clamp(x, lo, hi)` | `x` clamped to `[lo, hi]` |

### Conversions

| Function | Returns |
|---|---|
| `int(x)` | number truncated to integer |
| `string(x)` | value formatted as `Text` |

### Random (the `rand` feature, on by default)

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

`plural` and `select` are handy for localisation and gender-aware dialogue (see [Localisation](../integration/localisation.md)):

```text
You found {$n} {plural($n, "gem", "gems")}.
{select($gender, "m:He|f:She|other:They")} nods.
```

## Your own functions

Your game probably has things Bubbles can't know about - inventory checks, faction reputation, proximity queries. Register closures with the runner's [`FunctionLibrary`](../integration/functions.md):

```rust,ignore
runner.library_mut().register("faction_at_least", |args| {
    let Some(bubbles::Value::Text(name)) = args.first() else {
        return Err(bubbles::DialogueError::Function {
            name: "faction_at_least".into(),
            message: "expected string argument (faction name)".into(),
        });
    };
    let Some(bubbles::Value::Number(thresh)) = args.get(1) else {
        return Err(bubbles::DialogueError::Function {
            name: "faction_at_least".into(),
            message: "expected number argument (threshold)".into(),
        });
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

## A fuller example

Here's a skill check that uses several features together:

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

`dice`, `plural`, `<<if>>`, variables, interpolation - all in one readable node. You can follow what it does without looking anything up.

---

> **Next:** [Conditionals](./conditionals.md)
