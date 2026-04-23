# CLI Runner

[`examples/cli_runner.rs`](https://github.com/ViezeVingertjes/bubbles/blob/main/examples/cli_runner.rs) is the smallest useful Bubbles driver: a command-line dialogue player. Point it at a `.bub` file and it plays the script in your terminal.

```sh
cargo run --example cli_runner -- path/to/script.bub Start
```

Everything you need to understand the pull-based loop lives in this file. Let's walk it top to bottom.

## Parsing CLI arguments

```rust,ignore
let mut args = std::env::args().skip(1);
let path = args.next().unwrap_or_else(|| {
    eprintln!("usage: cli_runner <file.bub> [StartNode]");
    std::process::exit(1);
});
let start = args.next().unwrap_or_else(|| "Start".to_owned());
```

Take the first arg as the script path, the second (optional) as the starting node. Default to `"Start"` — that's the convention the beginner docs use.

## Compile the script

```rust,ignore
let source = std::fs::read_to_string(&path)
    .unwrap_or_else(|e| panic!("cannot read `{path}`: {e}"));

let prog = compile(&source).unwrap_or_else(|e| panic!("compile error: {e}"));
```

One string in, one `Program` out. If the script has syntax errors or references unknown nodes, `compile` returns a `DialogueError` with the file position — the CLI surfaces it by panicking. A real game would catch and log instead.

## Set up the runner

```rust,ignore
let mut runner = Runner::new(prog, HashMapStorage::new());
runner.start(&start).unwrap_or_else(|e| panic!("{e}"));
```

`HashMapStorage` is fine for a CLI — variables live in memory, die with the process. `start` primes the runner on the chosen node.

## The event loop

The body of `main` is one `loop { match runner.next_event() { … } }`:

### `NodeStarted` / `NodeComplete`

```rust,ignore
Some(DialogueEvent::NodeStarted(n)) => println!("[ node: {n} ]"),
// ...
Some(DialogueEvent::NodeComplete(n)) => println!("[ /{n} ]"),
```

Print a bracketed marker. Cheap "I entered this node" debug trail.

### `Line`

```rust,ignore
Some(DialogueEvent::Line { speaker, text, .. }) => {
    if let Some(spk) = speaker {
        println!("{spk}: {text}");
    } else {
        println!("{text}");
    }
    print!("(press enter)");
    io::stdout().flush().ok();
    stdin.lock().lines().next();
}
```

Render the speaker (if any) and the text. Pause for the player to press enter — this is the "wait for advance" step you'd wire to a key press in a real game.

### `Options`

```rust,ignore
Some(DialogueEvent::Options(opts)) => {
    for (i, opt) in opts.iter().enumerate() {
        let marker = if opt.available { "→" } else { "✗" };
        println!("  {marker} [{i}] {}", opt.text);
    }
    let choice = loop {
        print!("choose: ");
        io::stdout().flush().ok();
        let line = stdin.lock().lines().next()...unwrap_or_default();
        if let Ok(n) = line.trim().parse::<usize>()
            && n < opts.len()
            && opts[n].available
        {
            break n;
        }
        println!("invalid choice");
    };
    runner.select_option(choice).expect("select_option failed");
}
```

Three things happening:

1. Render each option with an arrow (`→`) or cross (`✗`) depending on `available`.
2. Read input until it's a valid, available index.
3. Commit the choice with `select_option`.

This is the minimal "refuse locked options in the UI" contract. Any real UI does roughly the same, just with buttons.

### `Command`

```rust,ignore
Some(DialogueEvent::Command { name, args, .. }) => {
    println!("[command] {name} {}", args.join(" "));
}
```

The CLI just echoes commands. A real game would dispatch to its audio / VFX / quest systems.

### `DialogueComplete` and `None`

```rust,ignore
Some(DialogueEvent::DialogueComplete) | None => {
    println!("\n[end]");
    break;
}
```

Both mean "we're done." Break out of the loop.

### The forward-compatible fallback

```rust,ignore
Some(_) => {} // forward-compatible with future event kinds
```

`DialogueEvent` is `#[non_exhaustive]`. That bare `_` arm makes sure this example keeps compiling when Bubbles adds a new event variant.

## Running it

Drop any `.bub` file in the repo and try it:

```sh
cargo run --example cli_runner --all-features -- examples/demo.bub Start
```

You'll see node markers, speaker lines, interactive prompts — an entire playable dialogue in a terminal. Great for sanity-checking scripts before wiring them into a larger game.

## Ideas to extend it

- **Coloured speakers** with [`termcolor`](https://crates.io/crates/termcolor).
- **Typewriter effect** — print text one character at a time with a short sleep.
- **Save/load** — wire up the `serde` feature and write snapshots on Ctrl-Z.
- **Hot reload** — watch the file and recompile on change.
- **Custom functions** — register a few game-style functions (`has_item`, `reputation`) to exercise expression features from the terminal.

The whole example is ~70 lines. Every extension above is another 5–20. It's a good place to prototype.

---

> **Next:** [The Tavern](./tavern.md)
