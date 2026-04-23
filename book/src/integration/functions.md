# Custom Functions

Bubbles ships a small built-in library (see [Expressions](../language/expressions.md)), but the real power is adding your own. Any closure that takes `Vec<Value>` and returns `Result<Value>` plugs straight in - faction checks, inventory queries, distance calculations, cooldown lookups, anything your game tracks.

## Registering a function

```rust,ignore
use bubbles::{DialogueError, Value};

runner.library_mut().register("double", |args| {
    let Some(Value::Number(n)) = args.first() else {
        return Err(DialogueError::Function {
            name: "double".into(),
            message: "expected one number argument".into(),
        });
    };
    Ok(Value::Number(n * 2.0))
});
```

After that, your script can use it anywhere an expression is allowed:

```text
<<set $reward = double($base)>>
Aria: Your haul doubles to {double($coins)}.
```

## Why functions vs commands?

Quick reminder:

- **Function** - synchronous, returns a value, usable in `<<if>>`, `<<set>>`, `{…}`.
- **Command** - fire-and-forget event for your game to react to.

If the dialogue needs the answer *before it can continue*, register a function. If it's dispatching a side effect (sound, VFX, saving), use a command.

## Reading game state

The most common case: expose your game's state to dialogue.

```rust,ignore
let health = game_state.player_health;

runner.library_mut().register("player_hp", move |_args| {
    Ok(Value::Number(health as f64))
});
```

Then:

```text
<<if player_hp() < 20>>
    Aria: You don't look well.
<<endif>>
```

> **Note:** Closures are `Send + Sync + 'static`. If you need to read shared mutable state, wrap it in `Arc<Mutex<…>>` or `Arc<RwLock<…>>` and clone the handle into the closure.

```rust,ignore
use std::sync::{Arc, Mutex};

let state = Arc::new(Mutex::new(GameState::default()));
let state_for_fn = Arc::clone(&state);

runner.library_mut().register("reputation", move |args| {
    let Some(Value::Text(faction)) = args.first() else {
        return Err(DialogueError::Function {
            name: "reputation".into(),
            message: "expected one string argument (faction name)".into(),
        });
    };
    let s = state_for_fn.lock().unwrap();
    Ok(Value::Number(s.reputation(faction) as f64))
});
```

Dialogue calls `reputation("thieves_guild")` and gets a live number back.

## Validating arguments

Bubbles gives you the arguments as a `Vec<Value>`. Validate them up front and fail clearly:

```rust,ignore
runner.library_mut().register("distance", |args| {
    let [Value::Number(x1), Value::Number(y1),
         Value::Number(x2), Value::Number(y2)] = args.as_slice() else {
        return Err(DialogueError::Function {
            name: "distance".into(),
            message: "expected 4 numbers (x1, y1, x2, y2)".into(),
        });
    };
    let dx = x2 - x1;
    let dy = y2 - y1;
    Ok(Value::Number((dx * dx + dy * dy).sqrt()))
});
```

The error message surfaces in `DialogueError`, so `cargo run` shows a precise complaint when someone typos the arguments.

## Shadowing built-ins

You can re-register any name - `register` replaces an existing function. Handy if you want deterministic dice in tests:

```rust,ignore
#[cfg(test)]
runner.library_mut().register("dice", |_args| Ok(Value::Number(4.0)));
```

Your dialogue still writes `dice(6, 3)`, but now it always yields `4`. The script doesn't change; the test becomes deterministic.

## A game-sized example

Here's a `has_item` function that checks the player's inventory. The dialogue author doesn't need to know anything about how items work - they just ask.

```rust,ignore
let inventory = Arc::clone(&inventory_handle);

runner.library_mut().register("has_item", move |args| {
    let Some(Value::Text(item)) = args.first() else {
        return Err(DialogueError::Function {
            name: "has_item".into(),
            message: "expected one string argument (item id)".into(),
        });
    };
    Ok(Value::Bool(inventory.lock().unwrap().contains(item)))
});

runner.library_mut().register("count", {
    let inventory = Arc::clone(&inventory_handle);
    move |args| {
        let Some(Value::Text(item)) = args.first() else {
            return Err(DialogueError::Function {
                name: "count".into(),
                message: "expected one string argument (item id)".into(),
            });
        };
        Ok(Value::Number(inventory.lock().unwrap().count(item) as f64))
    }
});
```

And in the dialogue:

```text
<<if has_item("rusty_key")>>
    Aria: The old key might fit this lock.
<<elseif has_item("lockpicks")>>
    Aria: We could pick it.
<<else>>
    Aria: Dead end. No way in.
<<endif>>

Merchant: {count("apple")} apples, is it? That's {count("apple") * 2} gold.
```

The story branches, counts, and does arithmetic on live game state - all from a `.bub` file a writer can edit without touching Rust.

---

> **Next:** [Saliency Strategies](./saliency.md)
