# bubbles

A lightweight, engine-agnostic dialogue runtime for Rust games.

Write branching dialogue in `.bub` scripts, compile them at startup, then drive
them from any game loop with a simple pull-based event API.

## Quick start

```toml
[dependencies]
bubbles = "0.1"
```

```rust
use bubbles::{compile, DialogueEvent, HashMapStorage, Runner};

let source = r#"
title: Start
---
Alice: Hello! How are you today?
-> Great!
    Glad to hear it.
-> Terrible.
    I'm sorry to hear that.
===
"#;

let prog = compile(source)?;
let mut runner = Runner::new(prog, HashMapStorage::new());
runner.start("Start")?;

loop {
    match runner.next_event()? {
        Some(DialogueEvent::Line { speaker, text, .. }) => {
            println!("{}: {}", speaker.unwrap_or_default(), text);
            // advance immediately (no player input for lines)
        }
        Some(DialogueEvent::Options(opts)) => {
            // Show opts to the player, then:
            runner.select_option(0)?;
        }
        Some(DialogueEvent::DialogueComplete) | None => break,
        _ => {}
    }
}
```

## Script syntax

| Construct | Syntax |
|-----------|--------|
| Node | `title: Name` … `---` … `===` |
| Line | `Optional speaker: text {expr}` |
| Shortcut option | `-> text <<if cond>>` / indented body |
| Line group | `=> text <<if cond>>` |
| Set variable | `<<set $var = expr>>` |
| Declare variable | `<<declare $var = expr>>` |
| Conditional | `<<if cond>>` … `<<elseif cond>>` … `<<else>>` … `<<endif>>` |
| Jump | `<<jump NodeTitle>>` |
| Detour | `<<detour NodeTitle>>` … `<<return>>` |
| Once block | `<<once>>` … `<<else>>` … `<<endonce>>` |
| Host command | `<<commandName args>>` |
| Metadata | `text #tag1 #tag2` |
| Node group | Multiple nodes with the same title and `when: <expr>` headers |

## Built-in functions

`round`, `floor`, `ceil`, `min`, `max`, `abs`, `clamp`, `string`, `int`,  
`random` *(rand feature)*, `random_range` *(rand feature)*, `dice` *(rand feature)*,  
`visited`, `visited_count`

## Feature flags

| Flag | Default | Effect |
|------|---------|--------|
| `rand` | ✓ | Enables `random`, `random_range`, `dice`, and `RandomAvailable` saliency |
| `serde` | — | Derive `Serialize` / `Deserialize` on `Value` and `HashMapStorage` |

## Licence

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at your option.
