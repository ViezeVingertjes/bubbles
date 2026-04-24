# Your First Dialogue

Let's get something talking. We will go from an empty project to a narrator greeting the player, in about ten lines of Rust.

## Install Bubbles

In a fresh Rust project, add Bubbles to your `Cargo.toml`:

```sh
cargo add bubbles-dialogue
```

Or by hand:

```toml
[dependencies]
bubbles-dialogue = "0.6"
```

> **Requires Rust 1.94 or later.** Bubbles targets a recent stable toolchain so it can lean on modern features without `unsafe`.

## Write a dialogue

Paste this into `src/main.rs`:

```rust,ignore
use bubbles::{compile, DialogueEvent, HashMapStorage, Runner};

fn main() -> Result<(), bubbles::DialogueError> {
    let program = compile(r#"
        title: Start
        ---
        Narrator: Hi. I'm the narrator.
        ===
    "#)?;

    let mut runner = Runner::new(program, HashMapStorage::new());
    runner.start("Start")?;

    while let Some(event) = runner.next_event()? {
        if let DialogueEvent::Line { speaker, text, .. } = event {
            println!("{}: {text}", speaker.as_deref().unwrap_or("*"));
        }
    }

    Ok(())
}
```

Run it:

```sh
cargo run
```

You should see:

```text
Narrator: Hi. I'm the narrator.
```

That is a full Bubbles loop. Take a second to appreciate how little is happening.

## What just happened?

Three moving parts:

1. **The script.** A `.bub` node called `Start`, with one line of narration. Every node starts with `title:`, a `---` header separator, the body, and a closing `===`.
2. **The compile step.** [`compile`](https://docs.rs/bubbles-dialogue/latest/bubbles/fn.compile.html) parses and validates the source, returning a [`Program`](https://docs.rs/bubbles-dialogue/latest/bubbles/struct.Program.html). Do this once at load time.
3. **The runner.** [`Runner::new`](https://docs.rs/bubbles-dialogue/latest/bubbles/struct.Runner.html#method.new) takes the `Program` and a variable storage backend. `HashMapStorage` is the default in-memory one. We call [`start`](https://docs.rs/bubbles-dialogue/latest/bubbles/struct.Runner.html#method.start) with the node name, then pull events out with [`next_event`](https://docs.rs/bubbles-dialogue/latest/bubbles/struct.Runner.html#method.next_event) until it hands back `None`.

## Add another node

Let's make the narrator actually *go* somewhere. Update the script:

```text
title: Start
---
Narrator: Hi. I'm the narrator.
<<jump Adventure>>
===

title: Adventure
---
Narrator: Let's go on an adventure.
===
```

Run it again. Now you'll see both lines.

```text
Narrator: Hi. I'm the narrator.
Narrator: Let's go on an adventure.
```

`<<jump Adventure>>` is a **command** - anything between `<<` and `>>` is a built-in directive. We'll meet more of them throughout the guide.

> **Tip:** You do not need newlines between nodes, but they help readability. Bubbles happily parses them smushed together.

## Give the player a choice

Dialogue without branching is just narration. Let's add options:

```text
title: Adventure
---
Narrator: Where do you go?
-> The forest.
    Narrator: You hear wolves.
-> The mountain.
    Narrator: The air is thin, but the view is worth it.
===
```

Running it now prints the prompt, then... nothing happens. Bubbles is politely waiting for you to pick an option. Our Rust loop only handles `Line` events - we ignore the `Options` event entirely.

Fixing that is the next chapter.

---

> **Next:** [The Event Loop](./event-loop.md)
