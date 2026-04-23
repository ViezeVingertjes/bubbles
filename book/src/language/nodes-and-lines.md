# Nodes and Lines

Every `.bub` script is made of **nodes**. A node is a named scene - a chunk of dialogue your game can jump into. Lines live inside nodes.

Here is the smallest possible node:

```text
title: Start
---
Hello.
===
```

That's a valid program. It has one node named `Start`, containing one narrator line.

## Anatomy of a node

```text
title: Start        <- header: must start with title
tags: intro safe    <- optional: space-separated tags
---                 <- header/body separator
Hello, traveller.   <- body: lines, commands, options, etc.
===                 <- node terminator
```

Three rules to remember:

1. Every node starts with `title: SomeName`.
2. `---` separates the header from the body.
3. `===` ends the node.

Node titles must start with a letter and can only contain letters, numbers, and underscores.

> **Tip:** Multiple nodes can live in the same file. Bubbles doesn't care where node boundaries sit, only that each one has its header, body, and terminator.

## Lines

A line is any non-empty body text that isn't an option, command, or header. There are two flavours:

```text
Aria: Evening, friend.      <- speaker line
The wind howls outside.     <- narrator line (no speaker)
```

A speaker line starts with `Name:`. The name is passed through to you in the `DialogueEvent::Line` event:

```rust,ignore
DialogueEvent::Line { speaker, text, .. } => {
    // speaker = Some("Aria"), text = "Evening, friend."
}
```

A narrator line has no colon-prefix and `speaker` comes through as `None`. Use whichever fits the moment - Bubbles doesn't judge.

> **Note:** The speaker is everything before the *first* colon on the line. If your dialogue needs literal colons in the middle, put them after the first one and you're fine: `Aria: Tip: always carry salt.`

## A richer example

Let's write a short scene with three nodes. Don't worry about the `<<jump>>` yet - we'll cover it soon.

```text
title: CampfireIntro
---
Aria: You made it.
Aria: Sit down. It's cold tonight.
<<jump CampfireTalk>>
===

title: CampfireTalk
---
Aria: So. What brings you out this far?
The fire crackles while she waits.
<<jump CampfireEnd>>
===

title: CampfireEnd
---
Aria: Well. Rest up. We ride at dawn.
===
```

Three nodes, six lines, one scene. Bubbles will walk them in order because each one jumps to the next.

## Blank lines, comments, and whitespace

Blank lines inside a node are ignored. Leading indentation is meaningful only for options (which we'll cover next) - otherwise it's cosmetic.

Bubbles does not have line-level comments. For scene notes, use **tags** on the node header (covered in [Tags and Metadata](./tags.md)) or a convention like `Narrator: [TODO: rewrite this]`.

## A preview of what's next

Here's a node using features from later chapters. Don't panic - each piece gets its own page.

```text
title: Crossroads
tags: outdoor day
---
Aria: Which path?
-> The forest.
    Aria: Wolves live there. You keep walking anyway.
    <<jump Forest>>
-> The mountain.
    Aria: Cold, but the view is worth it.
    <<jump Mountain>>
-> Wait here a moment.
    Aria: Take your time.
===
```

Options, indented bodies, jumps - we'll unpack them one by one.

---

> **Next:** [Options](./options.md)
