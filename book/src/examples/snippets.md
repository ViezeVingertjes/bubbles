# Snippets

Five focused recipes, each answering a specific "can Bubbles do X?" question. Every snippet is a self-contained 30-50 line `.bub` file built around a pirate scenario, and each one runs independently through the TUI.

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

A duelist challenges you to an insult sword-fight. Whether you can land the devastating final insult depends on `$sword_skill`. Try it with the default value, then change the `<<declare $sword_skill = ...>>` line and press `r` to reload - different skill levels unlock different options and route to different outcome nodes.

Key lines from the script:

```text
-> Deliver the Insulto Magnifico! <<if $sword_skill >= 8>>
    Aria: Your fighting is like a cow having a fit!
    <<jump Victory>>
-> Attempt a lesser insult. <<if $sword_skill >= 4>>
    Aria: You're no match for my blade OR my wit!
    <<jump CloseCall>>
-> Run away.
    <<jump Retreat>>
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
<<declare $grog_price = 2>>

-> Buy a mug ({$grog_price} doubloons). <<if $doubloons >= $grog_price>>
    <<set $doubloons = $doubloons - $grog_price>>
    <<set $grog_price = $grog_price + 1>>
    Griselda: Bottoms up. {$doubloons} doubloons left.
```

`{$doubloons}` in the text is interpolation - evaluated and substituted before the event reaches your game. By the time your code sees the line, it already reads `"9 doubloons left."`.

**Remix ideas:**
- Add a `$grog_consumed` counter and branch on it with `<<if $grog_consumed >= 3>>`
- Show a "running total spent" line using arithmetic in interpolation: `{$grog_price - 2} doubloons wasted`
- Add a `<<declare $tipsy = false>>` flag that flips after two mugs and changes the dialogue options

See [Variables](../language/variables.md), [Expressions](../language/expressions.md), and [Interpolation](../language/interpolation.md).

---

## commands

**You want dialogue to trigger sounds, voice-overs, animations, and other engine events. You also want per-line metadata (portrait cues, audio buses, subtitle styles) to travel with each line.**

```sh
cargo run -p bubbles-tui -- examples/snippets/commands.bub
```

Prying open a cursed chest fires `<<play_fanfare>>`, `<<spawn_particles>>`, and `<<apply_curse>>`. Key lines carry tags like `#dramatic` and `#sfx`. The TUI shows commands in the transcript pane so you can see exactly what your event loop would receive.

Key lines from the script:

```text
<<play_fanfare>>
You found the cursed chest! #dramatic #sfx treasure_sting
<<spawn_particles "gold_burst">>
<<apply_curse "greed">>
A cold feeling settles in your chest. #eerie
```

In your game:

```rust,ignore
DialogueEvent::Command { name, args, .. } => match name.as_str() {
    "play_fanfare" => audio.fanfare(),
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

Old Barnacle Pete has a legendary kraken story. The first visit gets the epic full account. Every return gets a brief acknowledgement. No `$kraken_told` variable in sight.

Key lines:

```text
<<once>>
    Pete: Pull up a chair. It was a night like any other, when the sea itself rose up...
    Pete: Three masts snapped like twigs. The crew? Gone. Every last one.
    Pete: And I, alone, swam home. Took three days.
<<else>>
    Pete: Aye, same old story. Still gives me chills.
<<endonce>>
```

The "once-seen" state is stored with the runner and survives save/load. Press `R` (rerun) to see the second-visit lines without resetting that history. Press `r` (reload) to start completely fresh.

**Remix ideas:**
- Add a third `<<once>>` nested inside the `<<else>>` for a third-visit variant
- Try `<<once if $has_bought_a_drink>>` to delay the story until the player's bought a round
- Add a `<<once>>` for a one-off ambient detail that never repeats

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
=> Dockworker: Oi, mind the ropes!
=> Dockworker: These crates won't unload themselves.
=> Dockworker: Low tide's coming in. Move it!
=> Dockworker: Cap'n wants the hold cleared by noon.
=> Dockworker: Smells like fish and regret out here.
```

**Node groups**: the `Storyteller` NPC has four nodes with the same title. `when: $time_of_day == "..."` picks the right one. Change the `<<declare $time_of_day = "evening">>` line at the top and press `r` to reload - different time, different story.

```text
title: Storyteller
when: $time_of_day == "dawn"
---
Old Salt: Red sky at morning. Sailors take warning.
===

title: Storyteller
---
Old Salt: Another fine day at the docks. Mostly.
===
```

**Remix ideas:**
- Add a sixth bark to the line group and watch BLRV cycle through all six
- Add a `when: visited_count("Storyteller") >= 3` node for a "heard it before" variant
- Swap `BestLeastRecentlyViewed` for `RandomAvailable` in the Rust setup and notice the difference

See [Line Groups](../language/line-groups.md) and [Node Groups and Saliency](../language/node-groups.md).

---

> **Next:** [API Reference](../api-reference.md)
