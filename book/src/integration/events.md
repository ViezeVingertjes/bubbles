# Handling Events

The runner hands you one event at a time. You handle it, call `next_event` again, and repeat. That's the whole integration surface.

Here's what each event looks like in practice, and what you'll typically do with it.

## `NodeStarted(String)`

Fires when execution enters a node - from `start()`, a `<<jump>>`, or a `<<detour>>`.

```rust,ignore
DialogueEvent::NodeStarted(name) => {
    analytics.track("dialogue.node.started", &name);
    renderer.fade_in(0.3);
}
```

Good for scene transitions, analytics, or music changes tied to entering a location. Pair it with `NodeComplete` for symmetric fade in / fade out.

## `Line { speaker, text, line_id, tags, line_mode, spans }`

A line ready to display. Interpolation is done, tags are parsed, and the localised template (if any) has already been resolved.

```rust,ignore
use bubbles::LineMode;

DialogueEvent::Line {
    speaker,
    text,
    line_id,
    tags,
    line_mode,
    ..
} => {
    if line_mode == LineMode::Debug && !cfg!(debug_assertions) {
        return; // skip QA lines in release builds
    }

    // Voice-over lookup via stable id
    if let Some(id) = &line_id {
        audio.play_voice_over(id);
    }

    // Per-line metadata via tags
    for tag in &tags {
        if let Some(mood) = tag.strip_prefix("portrait ") {
            ui.set_portrait_expression(mood);
        }
    }

    ui.show_line(speaker.as_deref(), &text);
    wait_for_advance();
}
```

Fields:

- `speaker: Option<String>` - the `Speaker:` prefix, or `None` for narrator lines.
- `text: String` - already interpolated, already localised, markup tags stripped.
- `line_id: Option<String>` - the `#line:...` stable id if present. Use this for VO lookups and localisation.
- `tags: Vec<String>` - every other `#tag` on the line. Portrait cues, audio buses, subtitle styles, plus `#narration` / `#debug` if you used them for `line_mode`.
- `line_mode: LineMode` - `Normal`, `Narration` (from a `#narration` tag), or `Debug` (from `#debug`; wins if both tags are present). Same tags still appear in `tags` if you need them.
- `spans: Vec<MarkupSpan>` - inline markup spans over `text`, in source order. Empty when the line has no markup tags. Each span carries a `name`, byte `start`, byte `length`, and optional `properties`. See [Markup](../language/markup.md).

## `Options(Vec<DialogueOption>)`

A branching choice. Execution halts until you call `select_option(i)`. Don't call `next_event` again before that - you'll get the same `Options` event back.

```rust,ignore
DialogueEvent::Options(opts) => {
    let choice = ui.show_choice_menu(&opts);
    runner.select_option(choice)?;
}
```

Each option has:

- `text: String` - pre-interpolated, markup tags stripped.
- `available: bool` - result of the `<<if>>` guard. `false` means locked.
- `line_id: Option<String>` - stable id if `#line:...` is on the option.
- `tags: Vec<String>` - any other tags.
- `spans: Vec<MarkupSpan>` - inline markup spans over `text`, same shape as on `Line`.

> **Note:** Trying to select an unavailable option returns an error. Filter or disable those choices in your UI before the player can pick them.

## `Command { name, args, tags }`

Everything between `<<...>>` that isn't a reserved keyword. Your code decides what it means.

```rust,ignore
DialogueEvent::Command { name, args, tags } => {
    match name.as_str() {
        "play_sfx" => audio.one_shot(&args[0]),
        "give_item" => inventory.add(&args[0]),
        "shake" => camera.shake(args[0].parse().unwrap_or(0.1)),
        "save_checkpoint" => save::checkpoint(),
        other => log::warn!("unknown dialogue command: {other}"),
    }
}
```

Arguments are already interpolated - no raw `{$pitch}` surviving into your handler. Commands don't pause the dialogue, so keep pulling events after handling one.

## `NodeComplete(String)`

Fires when a node finishes - either runs off the end, or `<<return>>`s from a detour. Pair with `NodeStarted` for symmetric transitions.

```rust,ignore
DialogueEvent::NodeComplete(name) => {
    analytics.track("dialogue.node.complete", &name);
    save::maybe_autosave();
}
```

## `DialogueComplete`

The whole conversation is over. `next_event` returns `None` after this.

```rust,ignore
DialogueEvent::DialogueComplete => {
    ui.hide_dialogue_panel();
    gameplay.resume();
}
```

## `#[non_exhaustive]`

`DialogueEvent` is marked `#[non_exhaustive]`. Always include a `_ =>` arm so new variants added in a minor version don't break your match:

```rust,ignore
match event {
    DialogueEvent::Line { .. } => { /* ... */ }
    DialogueEvent::Options(_) => { /* ... */ }
    DialogueEvent::Command { .. } => { /* ... */ }
    DialogueEvent::NodeStarted(_) | DialogueEvent::NodeComplete(_) => {}
    DialogueEvent::DialogueComplete => break,
    _ => {} // forward-compatible
}
```

## A realistic game loop

Putting it together - a minimal but honest frame-tick handler:

```rust,ignore
fn tick_dialogue(runner: &mut Runner<HashMapStorage>, engine: &mut Engine) -> bool {
    loop {
        match runner.next_event() {
            Ok(Some(DialogueEvent::Line {
                speaker,
                text,
                line_id,
                tags,
                line_mode,
                ..
            })) => {
                if line_mode == bubbles::LineMode::Debug && !cfg!(debug_assertions) {
                    continue;
                }
                if let Some(id) = &line_id {
                    engine.audio.play_voice_over(id);
                }
                engine.ui.show_line(speaker.as_deref(), &text, &tags);
                return true; // wait for player input next frame
            }
            Ok(Some(DialogueEvent::Options(opts))) => {
                engine.ui.show_options(&opts);
                return true;
            }
            Ok(Some(DialogueEvent::Command { name, args, .. })) => {
                engine.dispatch_command(&name, &args);
                // no yield - commands don't need player input
            }
            Ok(Some(DialogueEvent::NodeStarted(n))) => {
                engine.analytics.node_started(&n);
            }
            Ok(Some(DialogueEvent::NodeComplete(n))) => {
                engine.analytics.node_complete(&n);
            }
            Ok(Some(DialogueEvent::DialogueComplete)) | Ok(None) => return false,
            Ok(Some(_)) => {} // forward-compatible
            Err(e) => {
                log::error!("dialogue error: {e}");
                return false;
            }
        }
    }
}
```

Returns `true` when the dialogue is waiting on the player, `false` when it's done. Call from your frame tick; only yield back when there's something for the player to respond to.

---

> **Next:** [Variable Storage](./storage.md)
