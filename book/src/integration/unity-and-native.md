# Unity and native hosts (C ABI)

Most of this guide assumes you call **`bubbles-dialogue` from Rust** (`Runner`, `DialogueEvent`, and so on). If your game is in **C# (Unity, Godot .NET, a custom host)** or another language that can load a shared library and use a C calling convention, use the workspace crate **`bubbles-ffi`** instead.

It is a small **`cdylib`** plus a [C header](https://github.com/ViezeVingertjes/bubbles/blob/main/crates/bubbles-ffi/include/bubbles_ffi.h). You compile `.bub` source to a program handle, build a runner, step with `bubbles_runner_next_event`, and parse **JSON** for each event in your host (for example `System.Text.Json` in .NET). Options are committed with `bubbles_runner_select_option`.

## What you need to know

- **Build:** `cargo build -p bubbles-ffi --release` (see the [crate README](https://github.com/ViezeVingertjes/bubbles/blob/main/crates/bubbles-ffi/README.md) for library names per platform and a P/Invoke sketch).
- **Threading:** the FFI surface is **not** thread-safe; call it from one thread (Unity's main thread is fine).
- **ABI:** check `bubbles_abi_version()` against what your bindings expect.
- **Language:** everything in the C API is **UTF-8**; string inputs use pointer **plus byte length** (not necessarily NUL-terminated).

The `.bub` language, event kinds, and runtime behaviour are the same as in [The Runner Lifecycle](./runner.md) and [Handling Events](./events.md). Only the **packaging** differs: JSON event strings instead of typed `DialogueEvent` in Rust.

A minimal **.NET** smoke app lives in the repo at `crates/bubbles-ffi/tests/dotnet_smoke/`; CI builds the release library and runs it on Linux.

---

> **See also:** [WebAssembly](../advanced/wasm.md) if you embed dialogue in the browser without a native plugin.
