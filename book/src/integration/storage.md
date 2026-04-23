# Variable Storage

Every `<<declare>>` and `<<set>>` in a `.bub` script reads and writes through the runner's storage. The default — `HashMapStorage` — is fine for a lot of games. When it isn't, you implement the [`VariableStorage`](https://docs.rs/bubbles-dialogue/latest/bubbles/trait.VariableStorage.html) trait.

## The trait

It's two methods:

```rust,ignore
pub trait VariableStorage {
    fn get(&self, name: &str) -> Option<Value>;
    fn set(&mut self, name: &str, value: Value);
}
```

`name` is the variable name as written in the script, including the leading `$`. `Value` is a tagged enum: `Number(f64)`, `Text(String)`, or `Bool(bool)`.

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

## Writing your own storage

When you want Bubbles variables to live inside *your* data model — say, a component in your ECS, or a row in a save database — implement the trait yourself.

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

When building UIs — settings screens, debug inspectors, save-file migrations — it's often handy to know every variable the script *could* touch:

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
