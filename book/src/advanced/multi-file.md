# Multi-file Projects

Small games fit in one `.bub` file. Real ones don't. Bubbles handles this with [`compile_many`](https://docs.rs/bubbles-dialogue/latest/bubbles/fn.compile_many.html), which stitches multiple files into a single `Program` where every node can see every other node.

## The API

```rust,ignore
use bubbles::compile_many;

let program = compile_many(&[
    ("tavern.bub", tavern_src),
    ("merchants.bub", merchants_src),
    ("intro.bub", intro_src),
])?;
```

Each entry is a `(name, source)` pair. The name only appears in error messages - it's how you'll find the file when something goes wrong. The source is the raw `.bub` text.

`compile_many` produces a single `Program` that sees every node in every file as if they were all written in one big document.

## Loading from disk

```rust,ignore
use std::fs;

fn load_program(dir: &str) -> Result<bubbles::Program, Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().map_or(false, |e| e == "bub") {
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            let src = fs::read_to_string(&path)?;
            files.push((name, src));
        }
    }

    let refs: Vec<(&str, &str)> = files.iter()
        .map(|(n, s)| (n.as_str(), s.as_str()))
        .collect();

    Ok(bubbles::compile_many(&refs)?)
}
```

Point it at `assets/dialogue/`, ship.

## Organising files

Bubbles doesn't care how you split your files. Common patterns:

- **By scene**: `tavern.bub`, `market.bub`, `forest.bub`.
- **By character**: `aria.bub`, `barkeep.bub`, `merchant.bub`.
- **By chapter**: `ch1_arrival.bub`, `ch2_the_king.bub`, `ch3_the_cave.bub`.
- **By concern**: `story.bub`, `barks.bub`, `tutorial.bub`.

Pick the split that makes it easy for writers to find what they need. Bubbles itself is flat - jumps and detours don't care which file the target lives in.

## Duplicate titles

If two files define a node with the same title - and neither has a `when:` header that differentiates them - `compile_many` returns an error. That's the "you probably typed the same name twice" check.

Intentional duplicates (node groups) need distinct `when:` conditions. Bubbles picks that up as a group and everyone is happy.

```text
# greetings.bub
title: Greet
when: $reputation >= 50
---
Baker: Ah, my favourite customer!
===

# greetings.bub
title: Greet
---
Baker: Morning. What'll it be?
===
```

Two "Greet" nodes in the same file is fine - they form a group. Two without `when:` at all is the duplicate that trips the error.

## Validation

`validate(program)` runs the same reference-checking pass `compile` does, but on an existing `Program`. Handy for tools and editors that want to verify a file without throwing away the result.

```rust,ignore
use bubbles::validate;

validate(&program)?;
```

Catches things like:

- `<<jump>>`/`<<detour>>` to an unknown node.
- Duplicate group titles that aren't `when:`-differentiated.

It does not require every variable to be declared. Host-supplied variables are a normal integration pattern, and `<<declare>>` is for script defaults and introspection rather than a compile-time requirement.

## Introspection

Once compiled, the `Program` is queryable:

```rust,ignore
for title in program.node_titles() {
    let tags = program.node_tags(title).unwrap_or_default();
    println!("{title}  [{}]", tags.join(", "));
}

for decl in program.variable_declarations() {
    println!("{} = {}", decl.name, decl.default_src);
}

if program.node_exists("TavernEntry") {
    runner.start("TavernEntry")?;
}
```

Useful for:

- Building editor tooling.
- Generating a scene index.
- Validating against external data (achievements, VO manifests).

## Hot reloading during development

A nice dev-loop trick: rebuild the program whenever a `.bub` file changes, and spin up a fresh runner for the current scene.

```rust,ignore
use notify::{RecursiveMode, Watcher};

let mut watcher = notify::recommended_watcher(move |res| {
    if let Ok(_event) = res {
        match load_program("assets/dialogue/") {
            Ok(prog) => reload_program(prog),
            Err(e) => eprintln!("dialogue compile error: {e}"),
        }
    }
})?;

watcher.watch("assets/dialogue/", RecursiveMode::Recursive)?;
```

Writers edit a file, save, and the game picks up the new script - often without losing their current scene state if you snapshot before reloading.

> **Tip:** In production, compile once at load time and cache the `Program`. Recompiling is fast but not *free*, and writers only need hot reload during development.

---

> **Try it:** The harbour showcase is a two-file project you can run right now:
> ```sh
> cargo run -p bubbles-tui -- examples/harbour/harbour.bub examples/harbour/services.bub
> ```
> `harbour.bub` detours into `MapSeller`, which is defined in `services.bub`, a cross-file jump working exactly as described above. See [The Harbour](../examples/harbour.md) for a full walkthrough.

---

> **Next:** [WebAssembly](./wasm.md)
