# Tags and Metadata

Tags are little bits of metadata you can hang off nodes, lines, and options. They don't change behaviour - they travel with the event so *you* can act on them.

## Line tags

Stick a `#tag` (or several) at the end of a line:

```text
Aria: Halt! #combat #loud
```

Those tags show up on the event:

```rust,ignore
DialogueEvent::Line { text, tags, .. } => {
    // tags = ["combat", "loud"]
    if tags.iter().any(|t| t == "loud") {
        play_sfx("shout");
    }
}
```

Perfect for:

- Triggering animations (`#wave`, `#bow`)
- Routing to audio buses (`#whisper`, `#shout`)
- Flagging lines for translators (`#idiom`, `#pun`)
- Gameplay hooks (`#hostile`, `#quest-complete`)

Options take tags the same way:

```text
-> Sneak past. #stealth
-> Charge in! #combat #loud
```

## Node tags

Nodes can also carry tags via the header:

```text
title: TavernEvening
tags: scene indoor warm
---
Barkeep: Evening.
===
```

Multiple tags, space-separated. Useful when you want to query the program *before* running it - for example, to pre-load the right music:

```rust,ignore
if let Some(tags) = program.node_tags("TavernEvening") {
    if tags.iter().any(|t| t == "indoor") {
        load_ambience("room_tone");
    }
}
```

Node tags are also surfaced via [`node_titles`](https://docs.rs/bubbles-dialogue/latest/bubbles/struct.Program.html) and related introspection APIs.

## Stable line IDs

One tag is special: `#line:something`. It marks a line with a **stable id** for localisation and voice-over.

```text
Aria: Evening, friend. #line:aria_greet_evening
```

That id flows onto every related event:

- `DialogueEvent::Line { line_id: Some("aria_greet_evening"), .. }`
- `DialogueOption { line_id: Some(..), .. }` when options use `#line:`

It's also the key Bubbles uses when looking up translations through a [`LineProvider`](../integration/localisation.md).

> **Tip:** Pick a tag naming convention early and stick to it. Something like `#line:<scene>_<speaker>_<variant>` scales well. The ids never leave the script until you ask for them, so they cost nothing at runtime.

## Getting just the id

If you have a bare `&[String]` of tags (say, in some custom UI code), you can extract the id the same way Bubbles does:

```rust,ignore
use bubbles::line_id_from_tags;

let id = line_id_from_tags(&tags); // Option<String>
```

Same rule as the runtime: first `line:` prefix wins, empty ids are rejected.

## Reserved tags

- `line:<id>` - stable id, described above.

That's it. Every other tag is yours to define. Bubbles promises not to grow a list of reserved names that clashes with `#combat` or `#boss`.

---

> **Next:** [Variables](./variables.md)
