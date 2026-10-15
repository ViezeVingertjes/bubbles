# WebAssembly

Bubbles runs on `wasm32-unknown-unknown` with `default-features = false`. No async, no `std::thread`, no OS-specific deps - just the parts of `std` that WebAssembly supports. The default feature set needs one extra step, covered under [Randomness on wasm](#randomness-on-wasm).

CI builds the library for wasm without default features and with `serde`, and builds a complete wasm host with the default features, so both paths break the build if they stop working.

## Minimum setup

Turn default features off, then add back only what you need:

```toml
[dependencies]
bubbles-dialogue = { version = "{{#include ../../../VERSION}}", default-features = false }

# or with serde for save/load:
bubbles-dialogue = { version = "{{#include ../../../VERSION}}", default-features = false, features = ["serde"] }
```

This is the recommended configuration for a browser or embedded host. The `rand` feature is on by default, and on wasm it needs a `getrandom` backend you have to select yourself - leaving it off avoids that and is also the smaller build.

With `rand` off there is no entropy source in the crate, so an unseeded runner starts its random stream at the beginning rather than somewhere unpredictable. The build is reproducible by default, which is usually what you want on the web. If a session should differ between page loads, take a number from the host - `crypto.getRandomValues`, a server, a save file - and pass it to [`with_seed`](https://docs.rs/bubbles-dialogue/latest/bubbles/struct.RunnerBuilder.html#method.with_seed). Bubbles deliberately does not improvise one for you: `SystemTime` aborts the module on `wasm32-unknown-unknown`, and `std`'s hash key is a compile-time constant there, so anything the crate could reach for on its own would be either a crash or a fixed number dressed up as entropy.

## Building a wasm library

```sh
cargo build --target wasm32-unknown-unknown --release
```

With `default-features = false` that is the whole command. If you keep `rand`, read [Randomness on wasm](#randomness-on-wasm) first - the build fails without the extra configuration there.

That gives you a `.wasm` module. For browser work you'll usually wrap it with [`wasm-bindgen`](https://rustwasm.github.io/docs/wasm-bindgen/) or [`wasm-pack`](https://rustwasm.github.io/wasm-pack/):

```sh
wasm-pack build --target web --release
```

## A wasm-friendly driver

Here's a minimal `wasm-bindgen` façade. Compile `.bub` in JS, drive events one at a time from the browser's main thread.

```rust,ignore
use wasm_bindgen::prelude::*;
use bubbles::{compile, DialogueEvent, HashMapStorage, Runner};

#[wasm_bindgen]
pub struct WebRunner {
    runner: Runner<HashMapStorage>,
}

#[wasm_bindgen]
impl WebRunner {
    #[wasm_bindgen(constructor)]
    pub fn new(source: &str, start: &str) -> Result<WebRunner, JsValue> {
        let program = compile(source).map_err(|e| JsValue::from_str(&e.to_string()))?;
        let mut runner = Runner::new(program, HashMapStorage::new());
        runner.start(start).map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(Self { runner })
    }

    /// Returns the next event as JSON, or null when done.
    pub fn next(&mut self) -> Result<JsValue, JsValue> {
        match self.runner.next_event().map_err(|e| JsValue::from_str(&e.to_string()))? {
            Some(event) => Ok(serde_wasm_bindgen::to_value(&event)?),
            None => Ok(JsValue::NULL),
        }
    }

    pub fn select(&mut self, index: usize) -> Result<(), JsValue> {
        self.runner.select_option(index).map_err(|e| JsValue::from_str(&e.to_string()))
    }
}
```

That's under 50 lines, and it's everything a JS front-end needs to drive a dialogue loop.

> **Note:** `DialogueEvent` is `#[non_exhaustive]`. Serialising it with `serde_wasm_bindgen` requires the `serde` feature.

## Rendering in the browser

From JS:

```js
import init, { WebRunner } from "./pkg/my_game.js";

await init();

const runner = new WebRunner(source, "Start");

function step() {
  const event = runner.next();
  if (!event) {
    console.log("dialogue complete");
    return;
  }
  if (event.Line) {
    render(event.Line.speaker, event.Line.text);
    waitForAdvance().then(step);
  } else if (event.Options) {
    renderOptions(event.Options, (i) => {
      runner.select(i);
      step();
    });
  } else if (event.Command) {
    handleCommand(event.Command);
    step(); // commands don't pause the flow
  } else {
    step();
  }
}

step();
```

Identical in shape to the Rust version - just a different presentation layer.

## Randomness on wasm

The `rand` feature delegates to `getrandom`, which has no default backend on `wasm32-unknown-unknown` and stops the build with a `compile_error!` until you pick one. A library cannot pick it for you, so this is on your crate even though nothing in your code mentions `getrandom`.

For a browser host, take a direct dependency on it and set the matching cfg:

```toml
[dependencies]
getrandom = { version = "0.4", features = ["wasm_js"] }
```

```sh
RUSTFLAGS='--cfg getrandom_backend="wasm_js"' cargo build --target wasm32-unknown-unknown --release
```

Both halves are required: the feature alone still fails to build, and so does the cfg alone. Put the flag in `.cargo/config.toml` under `[target.wasm32-unknown-unknown] rustflags` if you would rather not repeat it.

If you are shipping a standalone wasm host outside a browser, there is no `wasm_js` to reach for - register a custom `getrandom` backend, or drop the `rand` feature and register your own `random`/`dice` functions, which also makes those calls reproducible.

> **On reproducibility:** `random()`, `random_range()`, `dice()`, and `RandomAvailable` read a process-global RNG and take no seed, so a run that uses them cannot be replayed. If you need deterministic playback, leave `rand` off and register your own functions with an RNG you control. See [Custom Functions](../integration/functions.md).

## What's *not* supported

- **Threads.** Bubbles is single-threaded by design, so this is never an issue.
- **Filesystem.** Compile from a `&str` you loaded through your host's IO path (fetch, bundler, etc).
- **System clock.** Bubbles doesn't touch it. If you need time-of-day logic, feed it in via a custom function or a variable.

## Build-size tips

If binary size matters:

- `default-features = false` drops `rand`, worth 20 KiB.
- Run `wasm-opt -Oz` as a post-build step (included with `wasm-pack`).
- Compile with `opt-level = "z"`, `lto = "fat"`, `codegen-units = 1`, `panic = "abort"` and `strip = true` in your release profile.

Measured with those profile settings, on a cdylib that compiles a script and pumps every event:

| Features | `.wasm` | gzipped |
|---|---|---|
| `default-features = false` | 181 KiB | 76 KiB |
| `serde`, plus `serde_json` to use it | 187 KiB | 78 KiB |
| default (`rand`) | 201 KiB | 83 KiB |

So a browser host carrying the runtime and a small script lands around 80 KiB over the wire, before `wasm-opt`. Reproduce the table with `scripts/measure-footprint.sh`; the figures come from rustc 1.99.0 on x86_64 Linux and drift a little with the toolchain.

For scale: the same cdylib with this library removed, keeping only `std` string formatting, is 16 KiB raw and 6 KiB gzipped. The runtime is therefore the bulk of the module, not `std` overhead. A host pays for the compiler whether or not it compiles at runtime, because `compile` is reachable from the entry point - if you only ever play a program you built elsewhere, `wasm-opt -Oz` and dead-code elimination are what recover that, and the figures above do not include them.

---

> **Next:** [The Harbour](../examples/harbour.md)
