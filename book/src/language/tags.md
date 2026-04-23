# Tags and Metadata

Your game needs to know more than just what a line says. It needs to know *how* to say it: which portrait to show, whether to play a voice-over clip, how to style the subtitle, whether a line should trigger a camera cut. Tags are how you attach that information to a line without hardcoding it into your Rust.

A tag is just a `#word` at the end of a line, option, or node header. It doesn't change dialogue behaviour - it travels with the event so your game code can act on it.

## Line tags

```text
Aria: Halt! #combat #loud
```

Those tags arrive on the event:

```rust,ignore
DialogueEvent::Line { text, tags, .. } => {
    // tags = ["combat", "loud"]
    if tags.iter().any(|t| t == "loud") {
        audio.play_on_bus("sfx_shout", "loud_bus");
    }
}
```

Some things you can do with this:

- Trigger audio: `#sfx door_creak`, `#voice_bus ambient`
- Swap a portrait or expression: `#portrait angry`, `#portrait relieved`
- Style the subtitle: `#subtitle_style whisper`, `#subtitle_style urgent`
- Camera direction: `#camera closeup`, `#camera wide`
- Gameplay hooks: `#hostile`, `#quest_complete`, `#gift`

```text
Mira: You won't find better prices in three ports. #portrait smug #sfx coin_jingle
```

```rust,ignore
DialogueEvent::Line { tags, .. } => {
    for tag in &tags {
        if let Some(mood) = tag.strip_prefix("portrait ") {
            ui.set_portrait(mood); // "smug"
        }
        if let Some(sfx) = tag.strip_prefix("sfx ") {
            audio.one_shot(sfx); // "coin_jingle"
        }
    }
}
```

Options take tags the same way:

```text
-> Sneak past. #stealth
-> Charge in! #combat #loud
```

## Node tags

Nodes can carry tags in their header:

```text
title: TavernEvening
tags: scene indoor warm
---
Barkeep: Evening.
===
```

Multiple tags, space-separated. Query them before running to pre-load music, prepare ambient systems, or build an editor scene list:

```rust,ignore
if let Some(tags) = program.node_tags("TavernEvening") {
    if tags.iter().any(|t| t == "indoor") {
        audio.load_ambience("room_tone");
    }
}
```

Node tags also surface via `node_titles` and related introspection APIs.

## Stable line IDs for voice-over

One tag is special: `#line:some_id`. It marks a line with a stable id for localisation and voice-over lookup.

```text
Aria: Evening, friend. #line:aria_greet_evening
```

That id flows onto every related event:

```rust,ignore
DialogueEvent::Line { line_id, text, .. } => {
    // line_id = Some("aria_greet_evening")
    if let Some(id) = &line_id {
        audio.play_voice_over(id);
    }
}
```

The id is also the key Bubbles uses when looking up translations through a [`LineProvider`](../integration/localisation.md). One tag, two jobs.

> **Tip:** Pick a naming convention early. Something like `scene_speaker_variant` (`tavern_aria_greet_first`) scales well and makes VO manifests easy to audit. The ids are never sent anywhere until you ask for them - they cost nothing at runtime.

## Getting just the id

If you have a bare `&[String]` of tags and need to extract the id:

```rust,ignore
use bubbles::line_id_from_tags;

let id = line_id_from_tags(&tags); // Option<String>
```

Same rule as the runtime: the first `line:` prefix wins, empty ids are rejected.

## Reserved tags

Only one:

- `line:<id>` - stable id, described above.

Every other tag is yours. Bubbles won't grow a reserved list that clashes with `#combat` or `#boss`. Go ahead and use whatever makes sense for your engine.

---

> **Try it:** [`examples/snippets/commands.bub`](../../examples/snippets/commands.bub): see line tags (`#dramatic`, `#sfx`, `#eerie`) alongside commands in the transcript pane.
> ```sh
> cargo run -p bubbles-tui -- examples/snippets/commands.bub
> ```

---

> **Next:** [Variables](./variables.md)
