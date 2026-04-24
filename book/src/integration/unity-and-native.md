# Unity and native hosts (C ABI)

Most of this guide assumes you call **`bubbles-dialogue` from Rust** (`Runner`, `DialogueEvent`, and so on). If your game is in **C# (Unity, Godot .NET, a custom host)** or another language that can load a shared library and use a C calling convention, use the workspace crate **`bubbles-ffi`** instead.

It is a **`cdylib`** plus a [C header](https://github.com/ViezeVingertjes/bubbles/blob/main/crates/bubbles-ffi/include/bubbles_ffi.h). You compile `.bub` source to a program handle, query the program if you need [node lists or `<<declare>>` data](./runner.md), optionally configure [saliency](../language/node-groups.md) (`FirstAvailable`, `BestLeastRecentlyViewed`, `RandomAvailable`), [localisation](./localisation.md) via a JSON map of line ids to templates, and [host functions](./functions.md) via a C callback. Then you step with `bubbles_runner_next_event` and parse **JSON** for each event. Options use `bubbles_runner_select_option`. [Variable storage](./storage.md) and [save/load](../advanced/save-load.md) are available as JSON snapshots (session + `HashMapStorage`).

## What you need to know

- **Build:** `cargo build -p bubbles-ffi --release` (see the [crate README](https://github.com/ViezeVingertjes/bubbles/blob/main/crates/bubbles-ffi/README.md) for the full surface area, library names, and a .NET smoke test).
- **Threading:** the FFI surface is **not** thread-safe; call it from one thread (Unity's main thread is fine). Do **not** re-enter `bubbles_*` from inside a host function callback.
- **ABI:** check `bubbles_abi_version()` against what your bindings expect.
- **Language:** everything in the C API is **UTF-8**; string inputs use pointer **plus byte length** (not necessarily NUL-terminated).

For stepping through dialogue, **event kinds and script semantics** match [The Runner Lifecycle](./runner.md) and [Handling Events](./events.md); the FFI returns each event as a **JSON** string instead of a Rust `DialogueEvent`.

## Still Rust-only

The C API fixes storage as **`HashMapStorage`**, line lookup as **`HashMapProvider`** (from JSON), and saliency as one of the three strategies above. A fully custom [`VariableStorage`](../api/bubbles/trait.VariableStorage.html), [`LineProvider`](https://docs.rs/bubbles-dialogue/latest/bubbles/trait.LineProvider.html), or [`SaliencyStrategy`](../api/bubbles/trait.SaliencyStrategy.html) still needs a **Rust** façade that configures [`RunnerBuilder`](https://docs.rs/bubbles-dialogue/latest/bubbles/struct.RunnerBuilder.html) and exposes your own FFI.

A **.NET** smoke app lives in the repo at `crates/bubbles-ffi/tests/dotnet_smoke/`; CI builds the release library and runs it on Linux.

---

> **See also:** [WebAssembly](../advanced/wasm.md) if you embed dialogue in the browser without a native plugin.
