# WebAssembly

Bubbles runs on `wasm32-unknown-unknown` out of the box. No async, no `std::thread`, no OS-specific dependencies - just the parts of `std` that WebAssembly supports.

Every release is built against wasm in CI (see [`scripts/check-wasm.sh`](https://github.com/ViezeVingertjes/bubbles/blob/main/scripts/check-wasm.sh)). If it ever breaks, it breaks the build.

## Minimum setup

For a browser or embedded wasm host, use the library in `no_default_features` mode or turn on only what you need:

```toml
[dependencies]
bubbles-dialogue = { version = "0.3", default-features = false }

# or with serde for save/load:
bubbles-dialogue = { version = "0.3", default-features = false, features = ["serde"] }
```

The `rand` feature is on by default and works on wasm (via `rand`'s `getrandom` dependency), but you may want to turn it off if you're shipping a tiny build or using a custom RNG.

## Building a wasm library

```sh
cargo build --target wasm32-unknown-unknown --release
```

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

The `rand` feature uses `rand` which, on `wasm32-unknown-unknown`, delegates to `getrandom`. You'll need:

```toml
getrandom = { version = "0.2", features = ["js"] }
```

If you're shipping a standalone wasm host (not browser), provide a custom `getrandom` source or replace the RNG-using builtins yourself.

## What's *not* supported

- **Threads.** Bubbles is single-threaded by design, so this is never an issue.
- **Filesystem.** Compile from a `&str` you loaded through your host's IO path (fetch, bundler, etc).
- **System clock.** Bubbles doesn't touch it. If you need time-of-day logic, feed it in via a custom function or a variable.

## Build-size tips

If binary size matters:

- `default-features = false` drops `rand` (~20 KB).
- Run `wasm-opt -Oz` as a post-build step (included with `wasm-pack`).
- Compile with `lto = "fat"` and `codegen-units = 1` in your release profile.

On a stripped, optimised build, a Bubbles runtime plus a small `.bub` script tends to weigh well under 100 KB gzipped - comfortable for a web game.

---

> **Next:** [TUI Runner](../examples/tui-runner.md)
