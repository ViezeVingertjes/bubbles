# The Harbour

`examples/harbour/` is the main example. Two files, `harbour.bub` and `services.bub`, compile together into a single programme and cover most of Bubbles' language features in one playable scene.

Run it:

```sh
cargo run -p bubbles-tui -- examples/harbour/harbour.bub examples/harbour/services.bub
```

You arrive at Barnacle Bay and need a travel permit from the cantankerous harbormaster Stumpy McGee. A shady map seller lurks nearby. The map seller lives in `services.bub`, called via `<<detour>>` and returning cleanly to the harbour scene.

## Two files, one programme

```text
examples/harbour/
  harbour.bub   - main dockside scene
  services.bub  - shared beats (MapSeller)
```

Real games split scripts by concern: one file per scene, one per character, one for shared utility nodes. `compile_many` stitches them into a single `Program` where every node is visible from every other file. See [Multi-file Projects](../advanced/multi-file.md) for the Rust API.

## Walking through the features

### Node tags

```text
title: Harbour
tags: scene docks outdoor
---
```

Node tags travel with the `NodeStarted` event. Games use them to pre-load music, trigger ambient systems, or build scene lists for editors and save migrations.

### Variables and a first-visit check

```text
<<declare $gold = 25>>
<<declare $met_stumpy = false>>

<<if $met_stumpy>>
    Stumpy: You again. Permit still costs ten doubloons.
<<else>>
    Stumpy: Name's McGee. Stumpy McGee, Harbormaster.
    <<set $met_stumpy = true>>
<<endif>>
```

`<<declare>>` registers the variables a script owns. They are typed from their default values and persist across visits and across save/load cycles when you use `RunnerSnapshot`. See [Save and Load](../advanced/save-load.md).

### Line groups for ambient flavour

```text
=> Dockworker: Oi, watch yer step!
=> Dockworker: These crates won't unload themselves, ye know.
=> Dockworker: Smells like low tide and regret out here.
=> Dockworker: Cap'n said the tide turns at noon. Better hurry.
```

Each time this node runs, `BestLeastRecentlyViewed` picks whichever bark the player has seen least recently. Cycle through all four and the oldest one replays. Four lines of script, an NPC that does not repeat itself immediately.

### `<<once>>` for one-shot content

```text
<<once>>
    Stumpy: Word is there's a three-headed sea creature lurking past Dead Man's Reef.
    Stumpy: Whole fleet turned back. Brave sailors, every one of 'em.
<<else>>
    Stumpy: Aye, same tale about the sea creature. Nothing new to report.
<<endonce>>
```

The first visit gets the full reveal. Every later visit gets the short acknowledgement. No flag variable needed. The once-seen state is stored automatically and survives `RunnerSnapshot` round-trips.

### Guarded options

```text
-> Pay ten doubloons for a permit. <<if $gold >= 10>>
    <<set $gold = $gold - 10>>
    <<stamp_permit>>
    Stumpy: You have {$gold} doubloons left.
    <<jump Depart>>
-> Bribe him with everything. <<if $gold >= 1 && $gold < 10>>
```

Options with an `<<if>>` guard are shown but marked unavailable when the condition is false. The TUI renders them with a `o` marker. Your game sees the same thing via the `available` field on `DialogueOption`.

### Commands

```text
<<stamp_permit>>
<<give_map "sunken_galleon">>
```

Commands emit `DialogueEvent::Command { name, args, tags }`. In a real game you dispatch on `name` to play audio, trigger animations, or update inventory. The TUI shows commands in the transcript pane.

### Cross-file detour

```text
-> Ask about the map seller.
    <<detour MapSeller>>
    Stumpy: That old rogue sell ye anything useful?
```

`MapSeller` is defined in `services.bub`. `<<detour>>` jumps there and `<<return>>` brings execution back to the next line in the calling node - right back to Stumpy's follow-up. `services.bub` could be called from any number of other scenes without duplicating the script.

## Your turn

The harbour is under 80 lines of `.bub` script across two files. Some things to try:

- Add a dockmaster's assistant in a third file with her own greeting using a node group.
- Register a `has_item` custom function so an option unlocks when the player holds a rope.
- Change the declared gold to `0` and reload with `r` to see all guards lock at once.
- Add a fourth ambient bark and watch BLRV cycle through all five before repeating.

---

> **Next:** [Snippets](./snippets.md)
