# Options

A node that just prints lines is a monologue. A node with **options** is a conversation.

```text
title: Crossroads
---
Aria: Which way do you go?
-> The forest path.
    Aria: You hear wolves. You press on anyway.
-> The mountain pass.
    Aria: Cold, but clear. You can see for miles.
===
```

Every line starting with `->` is an option. When the runner hits a set of options, it pauses and emits a `DialogueEvent::Options(…)`. Your game shows the choices, the player picks one, you call `runner.select_option(index)`, and execution continues into that option's body.

## Indentation matters

The body of an option is everything indented *beneath* it:

```text
-> The forest path.
    Aria: You hear wolves.     <- runs only if this option is chosen
    Aria: You keep walking.    <- also inside the forest branch
-> The mountain pass.
    Aria: Cold, but clear.     <- runs only if the mountain option is chosen
```

Use spaces or tabs - just be consistent inside a block. After the options end, execution continues with whatever comes next in the node:

```text
-> Yes.
    Aria: Good.
-> No.
    Aria: Fine.
Aria: Either way, we leave at dawn.   <- runs after either branch
```

## Guards

Options can have conditions - so the "buy a sword" choice only shows up if you can afford it.

```text
Merchant: What'll it be?
-> A fine sword (10g) <<if $gold >= 10>>
    Merchant: Sharp as a winter morning.
-> A loaf of bread (1g) <<if $gold >= 1>>
    Merchant: Still warm.
-> Nothing for now.
    Merchant: Come back soon.
```

The `<<if>>` after the option text is a **guard**. When the guard evaluates to `false`, the option is still presented - but marked `available: false` in the event:

```rust,ignore
DialogueEvent::Options(opts) => {
    for opt in &opts {
        if opt.available {
            println!("  [ ] {}", opt.text);
        } else {
            println!("  [x] {} (locked)", opt.text);
        }
    }
}
```

Whether you render locked options greyed-out, hidden, or with a tooltip is your call. Bubbles gives you the `available` field and steps aside.

> **Tip:** If the player tries to `select_option` on an unavailable entry, Bubbles returns a runtime error. Filter or disable those choices in your UI.

## Options that don't jump

You don't have to `<<jump>>` out of an option. If the body just runs and ends, execution falls through to whatever comes after the options block:

```text
Aria: Ready?
-> Yes.
-> Actually, wait.
    Aria: Take a breath.
Aria: Alright, let's go.   <- always runs, after either branch
```

Both options land on the final line. That's a handy pattern for "gate" choices where you want a beat before continuing.

## A shopping example

Let's make something practical - a little shop with a running gold total. This uses [variables](./variables.md) (next chapter!) but the shape is all options.

```text
title: Shop
---
<<declare $gold = 20>>
Merchant: Welcome. What would you like?
-> Sword, 15 gold <<if $gold >= 15>>
    <<set $gold = $gold - 15>>
    Merchant: A fine blade.
-> Torch, 3 gold <<if $gold >= 3>>
    <<set $gold = $gold - 3>>
    Merchant: Mind the sparks.
-> Just browsing.
    Merchant: Suit yourself.
Merchant: Anything else? You have {$gold} gold left.
===
```

Three options, each guarded by gold, each doing its own thing. The final line runs regardless - and uses `{$gold}` [interpolation](./interpolation.md) to show the updated balance.

---

> **Try it:** [`examples/snippets/options.bub`](../../examples/snippets/options.bub): an insult sword-fight duel showing guarded options and an `<<elseif>>` skill chain.
> ```sh
> cargo run -p bubbles-tui -- examples/snippets/options.bub
> ```

---

> **Next:** [Tags and Metadata](./tags.md)
