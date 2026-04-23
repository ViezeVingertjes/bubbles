# Commands

Commands are how your dialogue talks to your engine. Play a sound, trigger a cutscene, give the player an item, switch the music - write `<<command_name args>>` in your script and your game code handles it.

Anything between `<<...>>` that isn't a reserved keyword is a command. Bubbles hands the name and arguments to your game via a `DialogueEvent::Command`:

```text
<<play_sfx "door_creak">>
<<set_portrait "angry">>
<<give_item "health_potion">>
<<start_cutscene "mira_winks">>
```

Each of those emits a `DialogueEvent::Command` with:

- `name: String` - `"play_sfx"`, `"set_portrait"`, etc.
- `args: Vec<String>` - the whitespace-separated tokens after the name
- `tags: Vec<String>` - any trailing `#tags`

## Handling commands in Rust

```rust,ignore
while let Some(event) = runner.next_event()? {
    match event {
        DialogueEvent::Command { name, args, .. } => match name.as_str() {
            "play_sfx" => audio.one_shot(&args[0]),
            "set_portrait" => ui.set_portrait(&args[0]),
            "give_item" => inventory.give(&args[0]),
            "shake_camera" => {
                let strength: f32 = args[0].parse().unwrap_or(0.2);
                camera.shake(strength);
            }
            other => log::warn!("unknown dialogue command: {other}"),
        },
        _ => {}
    }
}
```

That match block is the heart of your engine integration. Add one arm per command name. The `_ =>` arm catches typos early - much easier than debugging a silent miss at runtime.

## Voice-overs

The most common pattern in narrative games: trigger a voice-over clip when a line plays. Two approaches:

**Via command:** explicitly fire a `<<play_voice>>` alongside or before the line:

```text
<<play_voice "aria_greet_01">>
Aria: Evening, friend.
```

**Via `line_id`:** tag the line with `#line:aria_greet_01` and look it up in the `Line` event:

```text
Aria: Evening, friend. #line:aria_greet_01
```

```rust,ignore
DialogueEvent::Line { line_id, text, .. } => {
    if let Some(id) = &line_id {
        audio.play_voice_over(id); // "aria_greet_01"
    }
    ui.show_line(text);
}
```

The `line_id` approach is usually cleaner: one tag instead of two lines, and the id doubles as the localisation key (see [Tags and Metadata](./tags.md) and [Localisation](../integration/localisation.md)).

## Portraits and per-line metadata

Combine commands and tags to drive rich UI state:

```text
Mira: I told you - five gold, no exceptions. #portrait angry #sfx table_slam
```

```rust,ignore
DialogueEvent::Line { tags, .. } => {
    for tag in &tags {
        if let Some(mood) = tag.strip_prefix("portrait ") {
            ui.set_portrait_expression(mood);
        }
        if let Some(sfx) = tag.strip_prefix("sfx ") {
            audio.one_shot(sfx);
        }
    }
}
```

Tags travel passively with lines. Commands are explicit events that pause nothing and fire in order. Use whichever fits the beat.

## Interpolation in arguments

Arguments support `{...}` [interpolation](./interpolation.md):

```text
<<set $pitch = 1.0 + $nervous * 0.3>>
<<play_voice "aria_greet_01" {$pitch}>>
```

By the time your handler runs, `args` is already `["aria_greet_01", "1.3"]`. No parsing of `{$pitch}` in your game code.

## Commands vs functions

Two ways to talk to the host:

- **Commands** (`<<give_item "key">>`) - fire-and-forget side effects. Emit an event, your code reacts. Use for audio, VFX, animations, inventory changes, quest triggers.
- **Functions** (`reputation("thieves_guild")`) - synchronous values you need back in an expression. Use inside `<<if>>`, `<<set>>`, or `{...}`. See [Custom Functions](../integration/functions.md).

If the dialogue needs a result to keep going, use a function. If it's kicking off something elsewhere, use a command.

## Reserved command names

These are built-in directives - they're not dispatched as events:

`set`, `declare`, `if`, `elseif`, `else`, `endif`, `once`, `endonce`, `jump`, `detour`, `return`, `stop`.

Everything else is yours. `<<save_checkpoint>>`, `<<play_cutscene "ending_a">>`, `<<roll 2d6>>` - go wild.

## A worked example

A small ambush scene with audio, camera, and a music change:

```text
title: Ambush
---
Narrator: Rocks clatter down the path.
<<shake_camera 0.4>>
<<play_sfx "rockfall">>

Bandit: Your gold. Now.

-> Hand it over.
    <<play_sfx "sigh">>
    <<jump PeacefulExit>>
-> Fight!
    <<play_music "combat_tense">>
    <<jump Combat>>
===
```

```rust,ignore
DialogueEvent::Command { name, args, .. } => {
    match name.as_str() {
        "shake_camera" => camera.shake(args[0].parse().unwrap_or(0.2)),
        "play_sfx" => audio.one_shot(&args[0]),
        "play_music" => audio.music(&args[0]),
        _ => {}
    }
}
```

Six lines of dialogue drive camera shake, two sound effects, and a music change - with zero coupling between the script and your engine beyond the command names you agree on.

---

> **Try it:** [`examples/snippets/commands.bub`](../../examples/snippets/commands.bub): a treasure chest discovery that fires fanfare, particles, and a curse, with line tags for audio cues.
> ```sh
> cargo run -p bubbles-tui -- examples/snippets/commands.bub
> ```

---

> **Next:** [Line Groups](./line-groups.md)
