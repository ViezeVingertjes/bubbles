# Handling Events

Every event variant, what it looks like, and how to hook it into a real game.

## `NodeStarted(String)`

Fired when execution enters a node - either from `start()`, a `<<jump>>`, or a `<<detour>>`.

```rust,ignore
DialogueEvent::NodeStarted(name) => {
    analytics.track("dialogue.node.started", &name);
    renderer.fade_in(0.3);
}
```

Use it for scene transitions, analytics, or logging.

## `Line { speaker, text, line_id, tags }`

A line ready to display. Everything's pre-substituted: interpolation is done, tags are parsed, the localised template (if any) has been resolved.

```rust,ignore
DialogueEvent::Line { speaker, text, line_id, tags } => {
    let voice = line_id.as_deref().and_then(|id| audio.clip(id));
    ui.show_line(speaker.as_deref(), &text, voice, &tags);
    wait_for_advance();
}
```

Fields:

- `speaker: Option<String>` - the `Speaker:` prefix if present.
- `text: String` - already interpolated, already localised.
- `line_id: Option<String>` - the `#line:…` stable id if present, empty otherwise.
- `tags: Vec<String>` - every other `#tag` on the line.

## `Options(Vec<DialogueOption>)`

A branching choice. Execution halts until you call `select_option(i)`.

```rust,ignore
DialogueEvent::Options(opts) => {
    let available: Vec<_> = opts.iter().enumerate()
        .filter(|(_, o)| o.available)
        .collect();

    let choice = ui.ask("Pick one:", &available);
    runner.select_option(choice)?;
}
```

Each option has:

- `text: String` - pre-interpolated option label.
- `available: bool` - `<<if>>` guard result. `false` = locked.
- `line_id: Option<String>` - stable id if `#line:…` is present.
- `tags: Vec<String>` - any other tags.

> **Note:** If the player tries to pick an unavailable option, Bubbles returns an error. Make your UI reject the input before calling `select_option`.

## `Command { name, args, tags }`

Everything between `<<…>>` that isn't a reserved keyword. Your code decides what it means.

```rust,ignore
DialogueEvent::Command { name, args, tags } => {
    match name.as_str() {
        "play_sound" => audio.one_shot(&args[0]),
        "shake" => camera.shake(args[0].parse().unwrap_or(0.1)),
        "save_checkpoint" => save::checkpoint(),
        other => log::warn!("unknown dialogue command: {other}"),
    }
}
```

Arguments are already interpolated - no `{$pitch}` surviving into your handler.

## `NodeComplete(String)`

Fired when a node finishes (either runs off the end, or `<<return>>`s from a detour). Pair with `NodeStarted` for symmetric transitions.

```rust,ignore
DialogueEvent::NodeComplete(name) => {
    analytics.track("dialogue.node.complete", &name);
    save::maybe_autosave();
}
```

## `DialogueComplete`

The whole conversation is over. `next_event` will return `None` after this.

```rust,ignore
DialogueEvent::DialogueComplete => {
    ui.hide_dialogue_panel();
    gameplay.resume();
}
```

## `#[non_exhaustive]`

The enum is marked `#[non_exhaustive]`. Always include a `_ =>` arm so new variants (added in a minor version) won't break your match:

```rust,ignore
match event {
    DialogueEvent::Line { .. } => { /* … */ }
    DialogueEvent::Options(_) => { /* … */ }
    DialogueEvent::Command { .. } => { /* … */ }
    DialogueEvent::NodeStarted(_) | DialogueEvent::NodeComplete(_) => {}
    DialogueEvent::DialogueComplete => break,
    _ => {} // future-proof
}
```

## A realistic match

Putting it all together - a minimal but honest game-loop handler:

```rust,ignore
fn tick_dialogue(runner: &mut Runner<HashMapStorage>, engine: &mut Engine) -> bool {
    loop {
        match runner.next_event() {
            Ok(Some(DialogueEvent::Line { speaker, text, line_id, tags })) => {
                engine.ui.show_line(speaker.as_deref(), &text, line_id.as_deref(), &tags);
                return true; // wait for input next frame
            }
            Ok(Some(DialogueEvent::Options(opts))) => {
                engine.ui.show_options(&opts);
                return true;
            }
            Ok(Some(DialogueEvent::Command { name, args, .. })) => {
                engine.dispatch_command(&name, &args);
                // don't yield; keep pulling events
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

Returns `true` when the dialogue is waiting on the player, `false` when it's done. Call it from your frame tick; only yield back to the caller when you've got something the player needs to respond to.

---

> **Next:** [Variable Storage](./storage.md)
