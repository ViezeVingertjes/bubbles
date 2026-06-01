# The Event Loop

Bubbles is **pull-based**. You call `next_event()`, you get back one [`DialogueEvent`](https://docs.rs/bubbles-dialogue/latest/bubbles/enum.DialogueEvent.html), you handle it, you call again. That's the entire integration surface.

Here are the events you'll actually handle in a game:

| Event | When | What to do |
|---|---|---|
| `Line` | A speaker has something to say | Render it; wait for the player to advance |
| `Options` | Branching choice | Show the choices; call `select_option(i)` |
| `Command` | A host directive like `<<play_sound bell>>` | Dispatch to your engine |
| `NodeStarted` / `NodeComplete` | A node begins or ends | Analytics, transitions, scene fades |
| `DialogueComplete` | The whole conversation is done | Clean up, return to the game |

## Handling options

Let's finish the dialogue from the previous chapter. We need to handle the `Options` event and tell the runner which one we picked.

```rust,ignore
use bubbles::{compile, DialogueEvent, HashMapStorage, Runner};
use std::io::{self, BufRead};

fn main() -> Result<(), bubbles::DialogueError> {
    let program = compile(r#"
        title: Adventure
        ---
        Narrator: Where do you go?
        -> The forest.
            Narrator: You hear wolves.
        -> The mountain.
            Narrator: The view is worth it.
        ===
    "#)?;

    let mut runner = Runner::new(program, HashMapStorage::new());
    runner.start("Adventure")?;

    let stdin = io::stdin();
    while let Some(event) = runner.next_event()? {
        match event {
            DialogueEvent::Line { speaker, text, .. } => {
                println!("{}: {text}", speaker.as_deref().unwrap_or("*"));
            }
            DialogueEvent::Options(opts) => {
                for (i, opt) in opts.iter().enumerate() {
                    let mark = if opt.available { ' ' } else { 'x' };
                    println!("  {i}{mark}) {}", opt.text);
                }
                let line = stdin.lock().lines().next().unwrap().unwrap();
                let choice: usize = line.trim().parse().unwrap_or(0);
                if opts.get(choice).is_some_and(|o| o.available) {
                    runner.select_option(choice)?;
                }
            }
            _ => {}
        }
    }

    Ok(())
}
```

That's it. The runner pauses on `Options` until you call [`select_option`](https://docs.rs/bubbles-dialogue/latest/bubbles/struct.Runner.html#method.select_option). Miss the call and `next_event` will keep returning the same `Options` event - Bubbles never moves forward without your say-so.

## The full event match

In a real game you probably want to handle every event. Here is the complete set with friendly comments:

```rust,ignore
while let Some(event) = runner.next_event()? {
    match event {
        DialogueEvent::NodeStarted(name) => {
            // good place for a scene transition or analytics ping
        }
        DialogueEvent::Line { speaker, text, spans, tags, line_id } => {
            // render and wait for input
            // spans: Vec<MarkupSpan> - byte ranges for [b], [color …], etc.
        }
        DialogueEvent::Options(opts) => {
            // show a menu; each opt has .text, .available, .spans, .tags, .line_id
            runner.select_option(chosen)?;
        }
        DialogueEvent::Command { name, args, tags } => {
            // <<play_sound bell>> arrives here
            match name.as_str() {
                "play_sound" => play_sound(&args[0]),
                _ => {}
            }
        }
        DialogueEvent::NodeComplete(name) => {
            // node finished - save, fade, whatever
        }
        DialogueEvent::DialogueComplete => {
            // all done
        }
        _ => {} // DialogueEvent is #[non_exhaustive]
    }
}
```

> **Note:** [`DialogueEvent`](https://docs.rs/bubbles-dialogue/latest/bubbles/enum.DialogueEvent.html) is marked `#[non_exhaustive]`. Always include a `_ =>` arm to stay forward-compatible when new event kinds land.

## Why pull-based?

Games run on frames. Dialogue runs on beats. A push-based callback system would either block your frame or force you to spawn threads. Pull-based means:

- You poll the runner when you're ready - usually when the player presses a key.
- The runner never allocates on its own. The event you get back owns its data; the runner moves on.
- Saving and loading is explicit: snapshot the runner and persist your variable storage beside it.

This is also why you won't find `async` anywhere. Bubbles is meant to be called from a fixed-timestep `update()` with no tears.

## A quick self-check

Before moving on, make sure you can answer these:

- Where does `DialogueEvent` come from? *(Your call to `runner.next_event()`.)*
- What stops the loop? *(Returning `None`, usually paired with `DialogueEvent::DialogueComplete` on the step before.)*
- What do you do with an `Options` event? *(Show the choices, then call `select_option(i)`.)*

Good. Now let's zoom into the `.bub` format itself.

---

> **Next:** [Using the TUI](./tui.md)
