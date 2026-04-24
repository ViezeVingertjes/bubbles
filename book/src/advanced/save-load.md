# Save and Load

Saving a mid-conversation means saving two things:

1. The runner's internal state - current node, visit counts, which `<<once>>` blocks have fired.
2. Your variable storage - all the `$variables` the dialogue has touched.

Bubbles makes (1) serialisable through `RunnerSnapshot`. You handle (2) however your game already handles saves.

Enable the `serde` feature to get snapshot serialisation:

```toml
[dependencies]
bubbles-dialogue = { version = "0.6", features = ["serde"] }
```

## Snapshot the runner

```rust,ignore
let snapshot = runner.snapshot();
let json = serde_json::to_string(&snapshot)?;
std::fs::write("save.json", json)?;
```

`snapshot()` returns a `RunnerSnapshot` - a small, cheap copy containing:

- `current_node: Option<String>` - the node the runner was in.
- `visits: HashMap<String, u32>` - how many times each node has completed.
- `once_seen: HashSet<String>` - fired `<<once>>` block ids.

That's everything specific to the runner. Serialise it as part of your save file.

## Save your variables separately

```rust,ignore
let storage_json = serde_json::to_string(runner.storage())?;
std::fs::write("vars.json", storage_json)?;
```

`HashMapStorage` already derives `Serialize`/`Deserialize` under the `serde` feature. If you've written your own storage, implement those traits yourself (or wrap your actual save format around the `VariableStorage` trait).

## Restore

On the other side of a save/load, you're recreating the runner from scratch and then pouring state back in.

```rust,ignore
use bubbles::{compile, HashMapStorage, Runner, RunnerSnapshot};

let program = compile(source)?;

let saved_storage: HashMapStorage = serde_json::from_str(&std::fs::read_to_string("vars.json")?)?;
let snapshot: RunnerSnapshot = serde_json::from_str(&std::fs::read_to_string("save.json")?)?;

let mut runner = Runner::new(program, saved_storage);
runner.restore(snapshot)?;
```

After `restore`, the runner is back at the start of the snapshotted node, with visit counts and `<<once>>` state intact.

> **Note:** Restore resumes at the *beginning* of the saved node, not mid-line. For most games that's perfect - scenes are natural save points. If you need finer-grained resumes, break long scenes into smaller nodes and snapshot more often.

## Handling script updates

What if the dialogue script changed between saving and loading? A node got renamed, a variable got removed?

- Unknown node → `DialogueError::UnknownNode` from `restore`. Catch it and start a safe fallback node (`"MainMenu"`, `"RecoveryScene"`).
- Missing variables → just missing. Your `VariableStorage::get` returns `None`; the script's `<<declare>>` reinitialises it. This is why you should almost always use `<<declare>>` for saved state: it survives script updates gracefully.
- Stale `<<once>>` ids → harmless. Bubbles doesn't re-fire once blocks it didn't save for, but new ones will work normally.

The pattern is: try restore, fall back gracefully, lose as little as possible.

## A complete save/load cycle

```rust,ignore
use bubbles::{compile, DialogueEvent, HashMapStorage, Runner};

fn save(runner: &Runner<HashMapStorage>) -> std::io::Result<()> {
    let snap = runner.snapshot();
    let bundle = SaveBundle {
        snapshot: snap,
        storage: runner.storage().clone(),
    };
    let json = serde_json::to_string(&bundle)?;
    std::fs::write("save.json", json)
}

fn load(program: bubbles::Program) -> std::io::Result<Runner<HashMapStorage>> {
    let json = std::fs::read_to_string("save.json")?;
    let bundle: SaveBundle = serde_json::from_str(&json)?;
    let mut runner = Runner::new(program, bundle.storage);
    runner.restore(bundle.snapshot).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
    })?;
    Ok(runner)
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SaveBundle {
    snapshot: bubbles::RunnerSnapshot,
    storage: HashMapStorage,
}
```

One bundle, round-trips cleanly, easy to extend.

## When *not* to use snapshots

If your game uses Bubbles for quick, self-contained conversations (think: short barks, UI flavour text), you probably don't need snapshotting. The cost of serialising nothing is nothing - but the complexity of adding it might not be worth it.

A good signal: **if a conversation can plausibly be interrupted by the player saving, snapshot it. If it runs to completion every time, don't bother.**

---

> **Next:** [Multi-file Projects](./multi-file.md)
