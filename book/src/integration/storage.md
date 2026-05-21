# Variable Storage

Every `<<declare>>` and `<<set>>` in a `.bub` script goes through the runner's storage. The default - `HashMapStorage` - works fine for most games. When you need variables to live inside your own save system, ECS component, or database row, implement the [`VariableStorage`](https://docs.rs/bubbles-dialogue/latest/bubbles/trait.VariableStorage.html) trait.

## The trait

The core is two methods, plus an optional borrow-friendly variant the runner prefers on the hot expression-evaluation path:

```rust,ignore
use std::borrow::Cow;

pub trait VariableStorage {
    fn get(&self, name: &str) -> Option<Value>;
    fn set(&mut self, name: &str, value: Value);

    /// Default: forwards to `get` and wraps the result in `Cow::Owned`.
    /// Override to return `Cow::Borrowed` when you already own the value,
    /// so `{$text}` reads don't clone on every evaluation.
    fn get_ref(&self, name: &str) -> Option<Cow<'_, Value>> {
        self.get(name).map(Cow::Owned)
    }
}
```

`name` is the variable name as written in the script, including the leading `$`. `Value` is a tagged enum: `Number(f64)`, `Text(String)`, or `Bool(bool)`.

The expression evaluator reads variables through `get_ref`; `get` is kept as the ergonomic API hosts reach for when they actually want ownership (e.g. `runner.storage().get("$hp")`). If you can cheaply hand back a reference - most in-memory stores can - override `get_ref` and you get allocation-free `{$var}` interpolation for free.

## The default: `HashMapStorage`

```rust,ignore
use bubbles::{HashMapStorage, Runner, Value, VariableStorage};

let mut storage = HashMapStorage::new();
storage.set("$player_name", Value::Text("Aria".into()));
storage.set("$hp", Value::Number(100.0));

let mut runner = Runner::new(program, storage);
```

You can access it later:

```rust,ignore
if let Some(Value::Number(hp)) = runner.storage().get("$hp") {
    hud.update_health(hp);
}

runner.storage_mut().set("$hp", Value::Number(50.0));
```

With the `serde` feature, `HashMapStorage` derives `Serialize`/`Deserialize`, so you can serialise it alongside your main save file.

## Sharing between runners

[`HashMapStorage`](https://docs.rs/bubbles-dialogue/latest/bubbles/struct.HashMapStorage.html) implements [`Clone`](https://doc.rust-lang.org/std/clone/trait.Clone.html) by copying the map. Cloning storage and passing each copy to a different [`Runner`](https://docs.rs/bubbles-dialogue/latest/bubbles/struct.Runner.html) gives each runner its own variable set.

To share game state across concurrent or sequential conversations, either:

- keep one storage behind every runner that should see the same `$variables`, or
- implement [`VariableStorage`](https://docs.rs/bubbles-dialogue/latest/bubbles/trait.VariableStorage.html) over your existing save model (ECS component, database row, and so on).

## Writing your own storage

When you want Bubbles variables to live inside *your* data model - say, a component in your ECS, or a row in a save database - implement the trait yourself.

```rust,ignore
use bubbles::{Value, VariableStorage};

pub struct GameSaveStorage<'a> {
    save: &'a mut GameSave,
}

impl VariableStorage for GameSaveStorage<'_> {
    fn get(&self, name: &str) -> Option<Value> {
        self.save.flags.get(name).cloned().map(|v| match v {
            SaveValue::Int(n) => Value::Number(n as f64),
            SaveValue::Str(s) => Value::Text(s),
            SaveValue::Bool(b) => Value::Bool(b),
        })
    }

    fn set(&mut self, name: &str, value: Value) {
        self.save.flags.insert(name.to_owned(), match value {
            Value::Number(n) => SaveValue::Int(n as i64),
            Value::Text(s) => SaveValue::Str(s),
            Value::Bool(b) => SaveValue::Bool(b),
        });
    }
}
```

Now dialogue writes land straight in the save file. No synchronisation step, no import/export round-trip.

> **Tip:** You're free to filter, rename, or project variables in `get`/`set`. Want only variables starting with `$quest_` to persist? Check the name in `set` and ignore the rest. Bubbles never peeks at storage outside these two methods.

## Overriding `get_ref` for zero-copy reads

The runner calls `get_ref` on every `{$var}` interpolation and every expression that reads a variable. The default implementation calls `get` and wraps the result in `Cow::Owned`, which means a `clone()` on every read.

If your store already owns `Value` objects you can hand back a borrow instead:

```rust,ignore
use std::borrow::Cow;
use std::collections::HashMap;
use bubbles::{Value, VariableStorage};

pub struct MyStorage {
    vars: HashMap<String, Value>,
}

impl VariableStorage for MyStorage {
    fn get(&self, name: &str) -> Option<Value> {
        self.vars.get(name).cloned()
    }

    fn set(&mut self, name: &str, value: Value) {
        self.vars.insert(name.to_owned(), value);
    }

    // Override: hand back a reference so the evaluator never clones.
    fn get_ref(&self, name: &str) -> Option<Cow<'_, Value>> {
        self.vars.get(name).map(Cow::Borrowed)
    }
}
```

For a `HashMap<String, Value>` this is a one-liner. The payoff is that `{$long_text}` in a line of dialogue never copies the string - it borrows from your map for the duration of the interpolation.

> **When the default is fine:** If your store does a lookup that already produces owned `Value`s (e.g. a database row, a deserialized field) there is nothing to borrow - keep the default and only override if profiling shows the allocations matter.

## Seeding storage from the outside

Before (or during) a conversation, push values in from your game:

```rust,ignore
runner.storage_mut().set("$time_of_day", Value::Text("evening".into()));
runner.storage_mut().set("$gold", Value::Number(player.gold as f64));
```

Scripts can then branch on them:

```text
<<if $time_of_day == "evening">>
    Innkeeper: Getting late. One for the road?
<<else>>
    Innkeeper: Morning!
<<endif>>
```

This is how you bridge dialogue and gameplay: set the state, start the conversation, read the state back when it ends.

## Checking declared variables at load time

When building UIs - settings screens, debug inspectors, save-file migrations - it's often handy to know every variable the script *could* touch:

```rust,ignore
for decl in program.variable_declarations() {
    println!("{} (default source: {})", decl.name, decl.default_src);
}
```

Every `<<declare>>` across the whole program shows up here, including the textual form of its default. Great for generating a "fresh game" save without running any dialogue.

## Things the storage never sees

Some script-internal state lives on the runner, not in storage:

- Visit counts (`visited`, `visited_count`)
- `<<once>>` block exhaustion

These are part of the [`RunnerSnapshot`](../advanced/save-load.md). If you need them to persist, include that snapshot in your save file alongside your storage.

---

> **Next:** [Localisation](./localisation.md)
