# Add Polish

Two problems with Mira so far: she says the same opening line every time, and the scene feels silent and static. Let's fix both, and add a sound cue so your engine can react to the sale.

Update `mira.bub` to the complete version:

```text
title: Start
---
<<declare $gold = 8>>

=> Seagull: Squawk!
=> A cart rumbles past with barrels of fish.
=> The salt air stings your eyes.

<<once>>
    Mira: Oh! A new customer. Fresh stock just arrived, as luck would have it.
<<else>>
    Mira: Back again? The price hasn't changed.
<<endonce>>

Mira: Five gold for a health potion. Best deal in the harbour district.
-> Buy a health potion (5 gold) <<if $gold >= 5>>
    <<set $gold = $gold - 5>>
    <<play_sfx "clink">>
    Mira: Pleasure doing business. You've got {$gold} gold left.
-> I can't afford that right now. <<if $gold < 5>>
    Mira: Come back when your pockets are heavier.
-> Just looking.
    Mira: ...You're still just looking?
===
```

Reload with `r`. Play through. Then press `R` (rerun) - Mira now gives her second-visit greeting. Play through again. The ambient line at the top changes each time.

## `<<once>>`

`<<once>>` runs the first branch exactly once. Every visit after fires the `<<else>>` branch. No variable needed.

```text
<<once>>
    Mira: Oh! A new customer. Fresh stock just arrived.
<<else>>
    Mira: Back again? The price hasn't changed.
<<endonce>>
```

The "once-seen" state is saved with the runner and survives save/load. Press `r` (full reload) to reset it, `R` (rerun) to keep it.

You can also make a block that runs once and goes completely silent after:

```text
<<once>>
    Narrator: Something shifts in the air as you approach.
<<endonce>>
```

See [Once Blocks](../language/once.md) for the full forms, including `<<once if condition>>`.

## Line groups (`=>`)

The three `=>` lines at the top are a **line group**. Every time the runner reaches them, it picks one. With the default strategy, it picks whichever one you've seen least recently - so you never hear the same ambient line twice in a row.

```text
=> Seagull: Squawk!
=> A cart rumbles past with barrels of fish.
=> The salt air stings your eyes.
```

Your game receives a normal `DialogueEvent::Line` for whichever one got picked. It doesn't know there were three variants - it just gets the chosen one.

Tell the runner which strategy to use. `BestLeastRecentlyViewed` is the right pick for ambient barks:

```rust,ignore
use bubbles::saliency::BestLeastRecentlyViewed;

runner.set_saliency(BestLeastRecentlyViewed::new());
```

See [Line Groups](../language/line-groups.md) for guarded variants and mixing in tags.

## Commands: talking to your engine

`<<play_sfx "clink">>` is a **command**. Bubbles doesn't know what it means - it just emits a `DialogueEvent::Command` with the name and arguments, and your game handles it.

```rust,ignore
DialogueEvent::Command { name, args, .. } => {
    match name.as_str() {
        "play_sfx" => audio.play_one_shot(&args[0]),
        _ => {}
    }
}
```

Commands are the bridge between your dialogue and your engine. Some common patterns:

**Audio cues:**
```text
<<play_sfx "door_creak">>
<<play_music "tavern_night">>
<<stop_music>>
```

**Voice-over by line id.** Tag a line with `#line:some_id` and use `line_id` in the event to look up the VO clip:
```text
Mira: Fresh stock just arrived. #line:mira_greet_first
```
```rust,ignore
DialogueEvent::Line { line_id, .. } => {
    if let Some(id) = &line_id {
        audio.play_voice(id); // "mira_greet_first"
    }
}
```

**Per-line metadata with tags.** Tags travel with every `Line` event and are yours to interpret:
```text
Mira: Five gold each. #portrait_mood angry #subtitle_style bold
```
```rust,ignore
DialogueEvent::Line { tags, .. } => {
    for tag in &tags {
        if let Some(mood) = tag.strip_prefix("portrait_mood ") {
            ui.set_portrait_mood(mood); // "angry"
        }
    }
}
```

**Animations, VFX, game state:**
```text
<<trigger_cutscene "mira_winks">>
<<give_item "health_potion">>
<<set_camera_target "mira">>
```

The rule: if your dialogue needs a **value back** (a reputation check, an inventory count), use a custom function. If it's a **side effect** (play a sound, trigger an animation, give an item), use a command. See [Commands](../language/commands.md) and [Custom Functions](../integration/functions.md) for the full story on each.

## What's next

That's the tutorial. You've got a working NPC with state, variety, first-visit behaviour, and engine signals. The language reference chapters cover everything in depth - head there for the full forms of anything you want to extend.

---

> **Next:** [Nodes and Lines](../language/nodes-and-lines.md) - the full language reference starts here
