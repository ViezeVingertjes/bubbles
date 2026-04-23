# Commands

A command is anything between `<<…>>` that isn't a reserved keyword. Bubbles doesn't try to interpret it - it just hands the name and arguments off to your game.

```text
<<play_sound bell>>
<<shake_camera 0.3>>
<<fade_to black 2>>
```

Each of those triggers a `DialogueEvent::Command` with:

- `name: String` - `"play_sound"`, `"shake_camera"`, `"fade_to"`
- `args: Vec<String>` - the whitespace-separated tokens after the name
- `tags: Vec<String>` - any trailing `#tags`

## Handling commands in Rust

```rust,ignore
while let Some(event) = runner.next_event()? {
    match event {
        DialogueEvent::Command { name, args, .. } => match name.as_str() {
            "play_sound" => audio.play(&args[0]),
            "shake_camera" => {
                let strength: f32 = args[0].parse().unwrap_or(0.0);
                camera.shake(strength);
            }
            "fade_to" => renderer.fade(&args[0], args[1].parse().unwrap_or(1.0)),
            other => eprintln!("unknown command: {other}"),
        },
        _ => {}
    }
}
```

That match block is the heart of your engine integration. Add one arm per command name.

> **Tip:** Validate unknown command names with a `_ =>` arm and log them. It'll save you a half-hour debugging a typo in a `.bub` file later.

## Interpolation in arguments

Command arguments are text, but they support `{…}` [interpolation](./interpolation.md):

```text
<<set $pitch = 1.0 + $nervous * 0.5>>
<<play_vo aria_greet_01 {$pitch}>>
```

By the time your handler runs, `args` is `["aria_greet_01", "1.35"]`. No escape artistry in your game code - Bubbles has already done the substitution.

## Commands vs functions

Two ways to talk to the host:

- **Commands** (`<<cast_spell fireball>>`) - fire-and-forget actions. They emit an event, your code reacts. Use this for audio, VFX, animations, quest triggers.
- **Functions** (`reputation("thieves_guild")`) - synchronous values you want back in an expression. Use these inside `<<if>>`, `<<set>>`, or `{…}`. See [Custom Functions](../integration/functions.md).

Rule of thumb: if you need the result in a later expression, use a function. If you're kicking off something that happens elsewhere, use a command.

## Reserved command names

Bubbles reserves the built-in script directives. You can't use them as command names because they aren't dispatched as events:

`set`, `declare`, `if`, `elseif`, `else`, `endif`, `once`, `endonce`, `jump`, `detour`, `return`.

Anything else is yours. `<<save>>`, `<<pray>>`, `<<roll 2d6>>` - go wild.

## A worked example

Let's wire a small combat beat. The script:

```text
title: Ambush
---
Narrator: Rocks clatter down the path.
<<shake_camera 0.4>>
<<play_sound rockfall>>

Bandit: Your gold. Now.

-> Hand it over.
    <<play_sound sigh>>
    <<jump PeacefulExit>>
-> Fight!
    <<play_music combat_tense>>
    <<jump Combat>>
===
```

And the Rust handler:

```rust,ignore
use bubbles::DialogueEvent;

fn handle(event: DialogueEvent, engine: &mut Engine) {
    if let DialogueEvent::Command { name, args, .. } = event {
        match name.as_str() {
            "shake_camera" => {
                let strength: f32 = args.first()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0.2);
                engine.camera.shake(strength);
            }
            "play_sound" => engine.audio.one_shot(&args[0]),
            "play_music" => engine.audio.music(&args[0]),
            _ => {}
        }
    }
}
```

Six lines of dialogue drive camera shake, two sound effects, and a music change - with zero coupling between the script and your engine beyond the command names you agree on.

---

> **Next:** [Line Groups](./line-groups.md)
