# Embedding and Determinism

Bubbles is built around two guarantees. Almost every design decision in the library follows from one of them, and if neither matters to your project, another tool is probably a better fit. This chapter explains both, and where each one stops.

## Embedding: the runtime never asks for your main loop

Most dialogue systems expect to be in charge. They want you to register callbacks, or hand them the frame loop, or run them on top of an async executor. That is a reasonable trade if dialogue is the centre of your application. It is a bad trade if dialogue is one feature inside software that already has its own control flow: a simulator, a course, a kiosk, an exhibit.

Bubbles inverts nothing. The API is pull-based:

```rust,ignore
// Called from wherever you already are: a frame callback, an input handler,
// a `requestAnimationFrame` closure, a C host's tick function.
if ready_for_next_line {
    match runner.next_event()? {
        Some(DialogueEvent::Line { text, .. }) => self.show(text),
        Some(DialogueEvent::Options(opts))    => self.present_choices(opts),
        Some(_) => {}
        None => self.close_dialogue(),
    }
}
```

`next_event()` advances the state machine one step and returns. Between calls, the runtime does nothing at all: it is not waiting on a timer, polling a channel, or holding a lock. Concretely, this means:

- **No `async`.** There are no `async fn`s in the library, so you do not inherit an executor or a runtime choice. Bubbles does not care whether you use Tokio, `async-std`, or nothing.
- **No global or thread-local state.** A `Runner` owns everything it needs. You can run several independently (one per NPC, one per lesson, one per browser tab) and they cannot interfere with each other.
- **No callbacks you must register.** Host commands (`<<play_sound bell>>`) arrive as `DialogueEvent::Command` values in the same event stream as everything else, so you handle them in the same `match` as lines and options, on your own stack.
- **Nothing to load.** `compile()` returns a `Program`: an ordinary Rust struct. There is no bytecode, no binary format, no asset to version or ship. A script can be a `&'static str` baked into the binary, a file you read, or bytes fetched over the network.

In practice the integration is the same handful of lines wherever you are: a Bevy system, a `_process` call in Godot-rust, a Macroquad or Fyrox frame, or the main loop of a plain Rust binary. There is no engine adapter to choose because there is no engine assumption to undo.

The last point is what makes the WebAssembly story work rather than merely compile. The library builds for `wasm32-unknown-unknown` and CI enforces it, but more usefully, a pull-based runtime with no async and no loader is one you can call directly from inside an existing frame callback, which is exactly what [SiliSim](https://silisim.com/) does.

One practical caveat: build with `default-features = false` for that target. The default feature set pulls in `getrandom` via `rand`, and `getrandom` does not support `wasm32-unknown-unknown` unless you opt into a JavaScript backend. Conveniently, this is the same configuration you want for determinism (see below). [WebAssembly](./advanced/wasm.md) has the full setup.

A host that is not written in Rust can link the `bubbles-ffi` crate instead, a C ABI wrapper that surfaces the same event stream as JSON.

### Where embedding stops

Bubbles has no opinion about rendering, input, audio, or timing, and provides nothing for them. Typewriter effects, text layout, option highlighting, voice-over synchronisation, and save-file formats are all yours to write. The library gives you byte-precise [markup spans](./language/markup.md) and an event stream; it does not give you a dialogue UI. If you wanted a drop-in dialogue box, this is not it.

## Determinism: nondeterminism is opt-in and retractable

Determinism here means something specific and checkable: **given the same program, the same starting variable state, the same seed, and the same sequence of option selections, you get the same sequence of events.**

This holds because the runtime has no independent access to anything that varies. It does not read the system clock, consult the environment, depend on thread scheduling, or reach for a global RNG. Node lookup is backed by an `IndexMap`, so iteration follows insertion order rather than hash order. The library denies `unsafe_code`, so there is no uninitialised memory to behave unpredictably. The one crate that opts out is the optional `bubbles-ffi` C wrapper, which cannot be written in safe Rust; nothing in it is reachable from a pure Rust host.

Saved state is canonical too: `RunnerSnapshot` and `HashMapStorage` hold their entries in sorted order, so the same state always serialises to the same bytes, and `all_variables()` comes back sorted by name. A save can be content-hashed, diffed against another save, or compared to a committed golden file.

That leaves exactly three places where nondeterminism can enter, all of them under your control.

**1. The `rand` feature.** It is on by default, and it is what supplies `random()`, `random_range()`, `dice()`, and the `RandomAvailable` saliency strategy. Turn it off and none of them are compiled in:

```toml
[dependencies]
bubbles-dialogue = { version = "2.0.0", default-features = false }
```

This is also the configuration you need to target `wasm32-unknown-unknown`, so for browser-hosted software the deterministic build is simply the build.

**2. The seed.** This is the option you want if you need *reproducible* randomness rather than *no* randomness. Give the runner a seed and the built-ins and `RandomAvailable` all draw from one generator you control:

```rust,ignore
use bubbles::{HashMapStorage, RunnerBuilder, compile};

let mut runner = RunnerBuilder::new(compile(script)?, HashMapStorage::new())
    .with_seed(20_260_815)
    .build();
```

The same seed produces the same rolls and the same random line choices, in every process and on every platform. `Runner::snapshot()` records where the stream had reached, so a save resumes it rather than starting a new one, and `Runner::rng()` hands out a handle your own host functions can draw from to stay inside the same stream. Without a seed the generator is seeded from the operating system, which is the old behaviour and still the default.

The generator is `SplitMix64` implemented inside this crate rather than borrowed from `rand`, because the stream is part of the save format: `rand`'s `SmallRng` is documented as not reproducible across platforms or versions, which is not a foundation a replay guarantee can stand on.

**The function library remains a seam** if you want the rolls themselves to come from somewhere else - a recorded table, a server, your engine's own generator. `random`, `random_range`, and `dice` are ordinary entries in a `FunctionLibrary`, and registering a function under an existing name replaces it:

```rust,ignore
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use bubbles::{HashMapStorage, RunnerBuilder, Value, compile};

let program = compile(script)?;

// A fixed, replayable sequence standing in for the built-in RNG.
let rolls = Arc::new(vec![3.0, 1.0, 4.0, 1.0, 5.0]);
let cursor = Arc::new(AtomicUsize::new(0));

let mut runner = RunnerBuilder::new(program, HashMapStorage::new())
    .with_function("random_range", move |_args| {
        let i = cursor.fetch_add(1, Ordering::Relaxed) % rolls.len();
        Ok(Value::Number(rolls[i]))
    })
    .build();
```

The point is that the seam exists: the runtime calls `random_range` through the registry, so whatever you put there is what the script gets. For ordinary reproducibility, reach for `with_seed` first - it covers the saliency strategy too, which a replacement function cannot.

**3. The saliency strategy.** Line groups and node groups pick among candidates through an injected `SaliencyStrategy`, so the choice is never hidden. `FirstAvailable` (the default) always takes the first eligible candidate. `BestLeastRecentlyViewed` picks the least recently shown, breaking ties toward the earliest candidate, and is deterministic given the same sequence of calls. `RandomAvailable` is the only random one, you have to ask for it by name, and it draws from the runner's seeded generator like everything else. A strategy you write yourself is handed that generator as an argument, so there is no quiet way to opt out of the seed. See [Saliency Strategies](./integration/saliency.md).

### Where determinism stops

Two limits are worth knowing before you build a replay system on this.

**Snapshots are node-granular bookmarks, not continuations.** `Runner::snapshot()` records the active node title, visit counts, and the set of `<<once>>` blocks that have fired. `Runner::snapshot()` also records where the random stream had reached, so rolls continue rather than restart. It deliberately does not serialise the in-progress statement list, because that would mean round-tripping the whole AST. So `restore()` re-enters the recorded node *from its beginning*, not from the line you were on. Two further pieces of state do not survive the round trip at all: the `<<detour>>` return stack, and any history held inside a stateful saliency strategy such as `BestLeastRecentlyViewed`. Variable storage is separate and must be persisted yourself. This is enough to implement "resume this lesson" or a conversation bookmark; it is not a time-travel debugger. [Save and Load](./advanced/save-load.md) has the specifics.

**Determinism is a property of the event stream, not of floating-point results across platforms.** Variables are `f64`, and expression evaluation uses ordinary Rust float arithmetic. Identical inputs on the same binary and target give identical results; the library makes no claim about bit-identical float behaviour across different architectures.

## Deciding whether this fits

Bubbles is likely a good fit if you are an engineer embedding scripted conversation into software that already exists, you cannot or will not adopt a game engine, and you would rather read a small library than configure a large one. It is a particularly good fit if reproducible behaviour is a correctness requirement: in assessment, training, simulation, or any system with replay.

It is likely a poor fit if the person writing your dialogue is not a programmer and needs to author, preview, and iterate without you. [Yarn Spinner](https://yarnspinner.dev) and [Ink](https://github.com/inkle/ink) are more mature, have far better writer-facing tooling, ship editor integrations and engine plugins, and have much larger communities. That is the job they are built for and they do it well. Bubbles is built for the program doing the running, not the person doing the writing.

---

> **Next:** [Your First Dialogue](./getting-started/first-dialogue.md)
