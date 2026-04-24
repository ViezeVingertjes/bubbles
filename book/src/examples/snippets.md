# Snippets

Six focused recipes, each answering a specific "can Bubbles do X?" question. Every snippet is a self-contained 30-50 line `.bub` file built around a pirate scenario, and each one runs independently through the TUI.

```sh
cargo run -p bubbles-tui -- examples/snippets/<name>.bub
```

Press `r` to reload after editing (re-reads from disk, full reset), `R` to rerun from the start keeping variable values and `<<once>>` history, `b` to step back through individual events.

---

## options

**You want branching choices with conditions, and some options that lock out when the player doesn't qualify.**

```sh
cargo run -p bubbles-tui -- examples/snippets/options.bub
```

A duelist challenges you to an insult sword-fight. Whether you can land the devastating insult depends on `$sword_skill`. Try it with the default value, then change the `<<declare $sword_skill = ...>>` line and press `r` to reload - different skill levels unlock different options and route to different outcome nodes.

Key lines from the script:

```text
-> Deliver a devastating insult. <<if $sword_skill >= 3>>
    You: Your sword arm is as weak as a soggy biscuit!
    Duelist: ...
    <<jump Victory>>
-> Stall for time.
    <<jump Escaped>>
-> Surrender immediately.
    <<jump Defeated>>
```

The `<<if>>` after an option text is a guard. The option stays visible but `available: false` when the guard is false. Your game sees this in `DialogueOption.available`.

**Remix ideas:**
- Add a fourth option locked behind `$has_special_item`
- Add an `<<elseif>>` chain inside a branch for tiered outcomes
- Change the guard to `visited("Victory")` to lock the re-challenge on return

See [Options](../language/options.md) and [Conditionals](../language/conditionals.md).

---

## variables

**You want persistent state that changes during a conversation, with the current values shown in the dialogue text.**

```sh
cargo run -p bubbles-tui -- examples/snippets/variables.bub
```

Griselda's Grog Shack charges more for each mug you buy. The price climbs and your doubloon count drops, both reflected live in the dialogue. The buy option locks itself when you run dry.

Key lines:

```text
<<declare $doubloons = 12>>
<<declare $grogs_bought = 0>>
<<declare $price = 3>>

-> Buy a mug. <<if $doubloons >= $price>>
    <<set $doubloons = $doubloons - $price>>
    <<set $grogs_bought = $grogs_bought + 1>>
    <<set $price = $price + 2>>
    GrogVendor: Pleasure! That'll be {$price} for the next one.
```

`{$price}` in the text is interpolation - evaluated and substituted before the event reaches your game. By the time your code sees the line, it already reads `"That'll be 5 for the next one."`.

**Remix ideas:**
- Branch on `$grogs_bought >= 3` for a "you've had enough" path
- Show a "running total spent" line using arithmetic in interpolation: `{$price - 2} doubloons wasted`
- Add a `<<declare $tipsy = false>>` flag that flips after two mugs and changes the dialogue options

See [Variables](../language/variables.md), [Expressions](../language/expressions.md), and [Interpolation](../language/interpolation.md).

---

## commands

**You want dialogue to trigger sounds, voice-overs, animations, and other engine events. You also want per-line metadata (portrait cues, audio buses, subtitle styles) to travel with each line.**

```sh
cargo run -p bubbles-tui -- examples/snippets/commands.bub
```

Prying open a cursed chest fires `<<creak_sound>>`, `<<play_fanfare>>`, `<<spawn_particles>>`, and `<<apply_curse>>`. Key lines carry tags like `#dramatic` and `#sfx`. The TUI shows commands in the transcript pane so you can see exactly what your event loop would receive.

Key lines from the script:

```text
<<play_fanfare "treasure_found">>
<<spawn_particles "gold_coins" 50>>
Gold coins spill across the sand, glittering in the torchlight. #dramatic
<<apply_curse "greedy_pirate">>
Ghostly Voice: That treasure is CURSED, ye fool! #eerie
```

In your game:

```rust,ignore
DialogueEvent::Command { name, args, .. } => match name.as_str() {
    "play_fanfare" => audio.play(&args[0]),
    "spawn_particles" => vfx.spawn(&args[0]),
    "apply_curse" => player.add_curse(&args[0]),
    _ => {}
},
DialogueEvent::Line { line_id, tags, .. } => {
    // Voice-over lookup
    if let Some(id) = &line_id {
        audio.play_voice_over(id);
    }
    // Per-line metadata
    for tag in &tags {
        if tag == "dramatic" { ui.set_subtitle_style("dramatic"); }
        if let Some(sfx) = tag.strip_prefix("sfx ") { audio.one_shot(sfx); }
    }
},
```

**Remix ideas:**
- Add `#line:chest_open_01` to the fanfare line and look up a VO clip from `line_id`
- Add `#portrait shocked` and wire it to a portrait swap in your `Line` handler
- Add a `<<save_checkpoint>>` command after the discovery and handle it in Rust

See [Commands](../language/commands.md) and [Tags and Metadata](../language/tags.md).

---

## once

**You want lines that only play on the first visit, with different content on every return - without managing a flag variable.**

```sh
cargo run -p bubbles-tui -- examples/snippets/once.bub
```

Old Barnacle Pete has several legendary tales. Each `<<once>>` block plays in full on the first visit; every return gets a brief acknowledgement. No `$kraken_told` variable in sight.

Key lines:

```text
<<once>>
    Pete: Thirty years ago I wrestled a kraken with me bare hands.
    Pete: Eight tentacles, each one thicker than a ship's mast.
    Pete: The kraken wept. Haven't seen one since.
<<else>>
    Pete: As I told ye before. Wrestled a kraken, bit its tentacle, it wept.
<<endonce>>
```

Multiple `<<once>>` blocks can be chained in sequence - each tracks its own "seen" state independently. The "once-seen" state is stored with the runner and survives save/load. Press `R` (rerun) to see the second-visit lines without resetting that history. Press `r` (reload) to start completely fresh.

**Remix ideas:**
- Try `<<once if $has_bought_a_drink>>` to delay a story until the player's bought a round
- Add a `<<once>>` for a one-off ambient detail that never repeats
- Add a fourth tale in `AnotherTale` to extend the chain

See [Once Blocks](../language/once.md).

---

## saliency

**You want ambient dialogue that never repeats immediately, and an NPC whose behaviour changes based on the time of day (or any game state).**

```sh
cargo run -p bubbles-tui -- examples/snippets/saliency.bub
```

The dockside has two layers:

**Line groups** (`=>`): five dock worker barks cycle with `BestLeastRecentlyViewed`. Each visit picks the one seen least recently, so the same line never plays twice in a row.

```text
=> Dockworker: Watch yer step on those wet planks!
=> Dockworker: Tide's coming in. Move them crates!
=> Dockworker: I've unloaded seventeen ships today. Seventeen!
=> Dockworker: Cap'n said we're done at sundown. Sundown was an hour ago.
=> Dockworker: Me back's killing me. Should've been a blacksmith.
```

**Node groups**: the `Storyteller` NPC has four nodes with the same title. `when: $time_of_day == "..."` picks the right one. Change the `<<declare $time_of_day = "evening">>` line at the top and press `r` to reload - different time, different story.

```text
title: Storyteller
when: $time_of_day == "morning"
---
Storyteller: Ah, a fresh morning at the docks! Best time for a tale.
===

title: Storyteller
---
Storyteller: Hmm. Can't think of a story right now. Come back later.
===
```

**Remix ideas:**
- Add a sixth bark to the line group and watch BLRV cycle through all six
- Add a `when: visited_count("Storyteller") >= 3` node for a "heard it before" variant
- Swap `BestLeastRecentlyViewed` for `RandomAvailable` in the Rust setup and notice the difference

See [Line Groups](../language/line-groups.md) and [Node Groups and Saliency](../language/node-groups.md).

---

## markup

**You want to annotate a range of text inside a line without changing the display text: bold, italic, coloured, animated, or anything else your renderer can do.**

```sh
cargo run -p bubbles-tui -- examples/snippets/markup.bub
```

Tags are stripped from the text before the event arrives. Alongside the clean text you get a `spans` list that names each annotation and gives its byte range. The terminal UI renders the five built-in tag names it knows: `[b]`, `[i]`, `[u]`, `[dim]`, and `[color value=NAME]`.

Key lines from the script:

```text
Narrator: [b]Bold[/b], [i]italic[/i], and [u]underline[/u] — the essentials.
Narrator: [color value=red]Danger![/color] [color value=green]All clear.[/color]
-> [b]Excited![/b] Let's go.
-> [dim]Cautiously optimistic...[/dim]
```

In your game:

```rust,ignore
DialogueEvent::Line { text, spans, .. } => {
    // text  = "Bold, italic, and underline — the essentials."
    // spans = [
    //   MarkupSpan { name: "b",         start: 0,  length: 4  },
    //   MarkupSpan { name: "italic",    start: 6,  length: 6  },
    //   MarkupSpan { name: "underline", start: 18, length: 9  },
    // ]
    for span in &spans {
        match span.name.as_str() {
            "b" => renderer.bold(span.start, span.length),
            "i" => renderer.italic(span.start, span.length),
            "color" => {
                let color = span.properties.iter()
                    .find(|(k, _)| k == "value")
                    .map(|(_, v)| v.as_str())
                    .unwrap_or("white");
                renderer.color(span.start, span.length, color);
            }
            _ => {}
        }
    }
}
```

The TUI preview shows the markup applied live: bold options, coloured lines, underlined phrases.

**Remix ideas:**
- Add `[wave]` or `[shake]` tags and handle them in a custom renderer
- Add `[sfx name=coin_drop /]` (self-closing) and dispatch audio from the span properties
- Combine markup with `{$variable}` interpolation: `[b]{$name}[/b]` works as expected

See [Markup](../language/markup.md).

---

> **Next:** [API Reference](../api-reference.md)
