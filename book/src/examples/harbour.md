# The Harbour

`examples/harbour/` is the main example: a two-file pirate dockside scene that puts most of Bubbles' language features together in one playable run. Go through it first, then come back and read this walkthrough.

```sh
cargo run -p bubbles-tui -- examples/harbour/harbour.bub examples/harbour/services.bub
```

You arrive at Barnacle Bay and need a travel permit from the cantankerous harbormaster Stumpy McGee. A shady map seller lurks nearby. Try to get out with a permit - or without one.

---

## Follow along

The source for this walkthrough is exactly what you just played. Open `examples/harbour/harbour.bub` in your editor and follow each section below. When a section suggests a change, make it, press `r` to reload, and see what happens.

---

## Node tags

The starting node declares what kind of place this is:

```text
title: Start
tags: scene docks outdoor
---
```

Node tags travel with the `NodeStarted` event. Your game uses them to pre-load music, trigger ambient systems, or build an editor scene list. Try adding `foggy` to the tags list, reload, and watch it appear in the TUI transcript when the node starts.

In Rust:

```rust,ignore
DialogueEvent::NodeStarted(name) => {
    if let Some(tags) = program.node_tags(&name) {
        if tags.iter().any(|t| t == "outdoor") {
            audio.load_ambience("wind_harbour");
        }
    }
}
```

---

## Variables

```text
<<declare $gold = 25>>
```

`<<declare>>` registers the variable once. The second time through (after a bribe that fails or a detour to the map seller), `$gold` keeps whatever value it has - `<<declare>>` won't overwrite it.

**Try it:** Change `25` to `9`. Press `r`. Now you can't afford the ten-doubloon permit, but you're in range for the bribe option. Then change it to `0` - all the money-gated options lock at once.

---

## Line groups for ambient flavour

```text
=> Dockworker: Oi, watch yer step!
=> Dockworker: These crates won't unload themselves, ye know.
=> Dockworker: Smells like low tide and regret out here.
=> Dockworker: Cap'n said the tide turns at noon. Better hurry.
```

Four lines starting with `=>`. The runner (using `BestLeastRecentlyViewed`) picks whichever one you've heard least recently. Cycle through the Options node a few times - you'll hear all four before any repeats.

**Try it:** Add a fifth bark. Reload. Now you've got five lines cycling, still with no repeat until all five have played.

---

## `<<once>>` for first-visit content

```text
<<once>>
    Stumpy: Word is there's a three-headed sea creature lurking past Dead Man's Reef.
    Stumpy: Whole fleet turned back. Brave sailors, every one of 'em.
<<else>>
    Stumpy: Aye, same tale about the sea creature. Nothing new to report.
<<endonce>>
```

First time you reach the Options node: Stumpy tells the full sea creature story. After that: the short acknowledgement. No flag variable, no manual tracking.

**Try it:** Press `R` (rerun) after completing the scene once. Stumpy's now in "second visit" mode - you'll hear the short version immediately. Press `r` instead to reload and confirm the long version comes back.

---

## Guarded options

```text
-> Pay ten doubloons for a permit. <<if $gold >= 10>>
    <<set $gold = $gold - 10>>
    <<stamp_permit>>
    Stumpy: There she is. Official seal and everything.
    Stumpy: You have {$gold} doubloons left. Don't spend 'em all at once.
    <<jump Depart>>
-> Bribe him with everything you've got. <<if $gold >= 1 && $gold < 10>>
    Stumpy: Hah! {$gold} doubloons? That barely covers the ink.
    <<jump Options>>
```

Guards on options: `<<if $gold >= 10>>` after the option text. When the condition is false, the option shows with `available: false`. The TUI renders locked options with a distinct style. In your game:

```rust,ignore
DialogueEvent::Options(opts) => {
    for opt in &opts {
        if opt.available {
            ui.show_option(&opt.text);
        } else {
            ui.show_locked_option(&opt.text);
        }
    }
}
```

`{$gold}` in the option and line text is interpolation - evaluated and substituted before the event reaches you.

---

## Commands

```text
<<stamp_permit>>
```

`<<stamp_permit>>` emits a `DialogueEvent::Command { name: "stamp_permit", args: [], .. }`. Your game dispatches on the name:

```rust,ignore
DialogueEvent::Command { name, args, .. } => match name.as_str() {
    "stamp_permit" => {
        player.inventory.add("travel_permit");
        audio.play_sfx("stamp");
    }
    _ => {}
},
```

The map seller in `services.bub` fires `<<give_map "sunken_galleon">>` or `<<give_map "rough_sketch">>` depending on your gold. Same pattern - `args[0]` is the map id.

---

## Cross-file detour

```text
-> Ask about the map seller.
    <<detour MapSeller>>
    Stumpy: That old rogue sell ye anything useful?
    <<jump Options>>
```

`MapSeller` is defined in `services.bub`, not `harbour.bub`. `<<detour>>` jumps there, `services.bub`'s node runs and calls `<<return>>`, and execution comes back to the very next line - Stumpy's follow-up question.

This is how you split dialogue by concern: one file per scene or character, cross-file detours for shared beats, no duplication.

**Try it:** Open `services.bub`. Notice the `<<return>>` at the bottom of `MapSeller`. Try removing it - the detour will still return, because a node that runs off the end without `<<return>>` returns implicitly. `<<return>>` is just the explicit form.

---

## Your turn

The harbour is under 80 lines across two files. A few ideas to extend it:

- Add a dockmaster's assistant in a `dockmaster.bub` with her own `when:` node group that changes based on `$gold`.
- Register a `has_item("rope")` custom function and add an option that only appears if the player carries a rope.
- Add a `<<play_voice "stumpy_greet_01">>` command to the opening node and handle it in the `Command` event.
- Add a fourth dock worker bark and watch BLRV cycle through all five before repeating.

---

> **Next:** [Snippets](./snippets.md)
