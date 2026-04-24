# Interpolation

Hard-coding values into lines breaks the moment anything changes. `{...}` lets you drop any expression directly into text - variables, math, function calls, all of it substituted before the event reaches your game.

```text
Aria: You have {$gold} gold and {$potions} potions.
```

The expression is evaluated when the line is produced. By the time `DialogueEvent::Line` arrives in your game code, the text is already resolved:

```rust,ignore
DialogueEvent::Line { text, .. } => {
    // text = "You have 42 gold and 3 potions."
}
```

## What can go inside `{…}`?

Anything that's a valid [expression](./expressions.md).

```text
Aria: Half of your gold is {$gold / 2}.
Aria: You are {plural($lives, "life", "lives")} from a new record.
Aria: The roll was {dice(6, 2)}.
Aria: {select($mood, "happy:Lovely|sad:Sigh|other:Hm")}.
```

Strings, numbers, booleans - all formatted with sensible defaults:

- Numbers: integers as `42`, floats as `3.14` (no trailing zeros).
- Booleans: `true` / `false`.
- Strings: inserted verbatim.

For finer formatting, do it in an expression or a custom function:

```text
Aria: You have {round($hp)} HP.
Aria: Gold: {string($gold)}.
```

## Escaping

If you need a literal `{` or `}` in your text, double them:

```text
Aria: She said "{{one}}" - I don't know what it meant.
```

That line arrives as: `She said "{one}" - I don't know what it meant.`

> **Tip:** Curly braces outside of interpolation are rare in natural prose, so you'll almost never need this. But when you *do* (code, JSON, serialised data), `{{` and `}}` have your back.

## Interpolation in options and commands

`{…}` works anywhere text flows:

```text
-> Give the Barkeep {$gold} gold.
    Barkeep: Much obliged.
<<play_sound "coin_{$coin_type}">>
```

The option text substitutes before the `Options` event, so your menu shows real numbers. The command argument substitutes before the `Command` event, so your host code sees `coin_gold`, not `coin_{$coin_type}`.

## A worked example

```text
title: LevelUp
---
<<declare $lvl = 1>>
<<declare $xp = 0>>
<<declare $to_next = 100>>

<<set $xp = $xp + 40>>

<<if $xp >= $to_next>>
    <<set $lvl = $lvl + 1>>
    <<set $xp = $xp - $to_next>>
    Narrator: Level up! You are now level {$lvl}.
    Narrator: {$to_next - $xp} XP until level {$lvl + 1}.
<<else>>
    Narrator: {$xp}/{$to_next} XP - keep going.
<<endif>>
===
```

`$lvl`, arithmetic expressions, and a full sentence composed of three interpolations. Readable, fast, and it sits squarely in the territory you'd expect a scripting language to cover.

## Localisation-aware interpolation

When a line carries a `#line:id` tag and a [`LineProvider`](../integration/localisation.md) returns a translation, `{…}` expressions in the translation are evaluated against the current variable storage. Translators can reorder or reshape interpolations for their language:

```text
# English source
You found {$n} {plural($n, "gem", "gems")}.

# German translation (stored in the provider)
Du hast {$n} {plural($n, "Edelstein", "Edelsteine")} gefunden.
```

Same variables, same expressions, grammatically correct output.

## Markup and interpolation together

`{expr}` and `[markup]` syntax both live in the same text and are processed in one pass. Byte offsets in the returned `MarkupSpan`s are always relative to the final, expression-substituted string:

```text
[b]{$name}[/b] found the key.
```

If `$name` is `"Alice"`, the text arrives as `"Alice found the key."` and the span is `{ name: "b", start: 0, length: 5 }`. The span points at the right characters regardless of how long the expression result turns out to be.

See [Markup](./markup.md) for the full tag syntax and span reference.

---

> **Next:** [Markup](./markup.md)
