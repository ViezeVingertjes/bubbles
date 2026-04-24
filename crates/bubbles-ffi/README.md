# bubbles-ffi

Thin **C ABI** around [`bubbles-dialogue`](../bubbles-dialogue) so you can drive `.bub` scripts from **Unity** (or any host with `DllImport` / `ffi`).

- Events come back as **JSON** strings (parse in C# with `System.Text.Json`, Newtonsoft, etc.).
- All text is **UTF-8** with an explicit **byte length** on inputs (no embedded NUL required).
- **Not thread-safe:** use from your game main thread only (typical for Unity).
- The crate links **`bubbles-dialogue` with the `full` feature** (`rand` + `serde`) so saliency, save/load, and JSON value encoding match the Rust API.

## Surface area (summary)

| Area | C entry points |
|------|----------------|
| Compile | `bubbles_compile`, `bubbles_compile_files`, `bubbles_program_free` |
| Program (before `bubbles_runner_*` consumes the handle) | `bubbles_program_node_exists`, `bubbles_program_node_titles_json`, `bubbles_program_node_tags_json`, `bubbles_program_variable_declarations_json` |
| Runner | `bubbles_runner_new`, `bubbles_runner_new_with_saliency` (`BUBBLES_SALIENCY_*`), `bubbles_runner_set_saliency`, `bubbles_runner_free` |
| Localisation | `bubbles_runner_set_locale_json` (object: line id → template). Prefer before `bubbles_runner_start`. |
| Host functions | `bubbles_runner_register_function` + `BubblesHostFn` in the header. Args are a JSON array of dialogue `Value`s (each element may be a JSON number/string/bool **or** `{"Number":…}`, `{"Text":…}`, `{"Bool":…}`). Return a JSON **scalar** or tagged object; allocate the returned string with `bubbles_copy_utf8`. |
| Dialogue loop | `bubbles_runner_start`, `bubbles_runner_next_event`, `bubbles_runner_select_option` |
| Variables | `bubbles_runner_variable_get_json`, `bubbles_runner_variable_set_json` (serde JSON for `Value`) |
| Save / load | `bubbles_runner_snapshot_session_json`, `bubbles_runner_snapshot_storage_json`, `bubbles_runner_restore_storage_json` (first), `bubbles_runner_restore_session_json` |
| Helpers | `bubbles_copy_utf8`, `bubbles_string_free`, `bubbles_last_error` |

See [`include/bubbles_ffi.h`](include/bubbles_ffi.h) for full signatures.

**Not exposed:** pluggable storage other than `HashMapStorage`, custom `LineProvider` implementations beyond the locale JSON map, arbitrary `SaliencyStrategy` types beyond First / BLRV / Random (add a Rust shim if you need more).

## Build

```sh
cargo build -p bubbles-ffi --release
```

Copy the library into your Unity project (often under `Assets/Plugins/` with platform subfolders):

| Platform | File |
|----------|------|
| Linux | `target/release/libbubbles_ffi.so` |
| macOS | `target/release/libbubbles_ffi.dylib` |
| Windows | `target/release/bubbles_ffi.dll` |

C header: [`include/bubbles_ffi.h`](include/bubbles_ffi.h).

## .NET smoke test

After `cargo build -p bubbles-ffi --release`, a **.NET 10** console app under [`tests/dotnet_smoke/`](tests/dotnet_smoke/) exercises program introspection, BLRV, locale JSON, a native host callback, and variable reads. From the repo root:

```sh
cargo build -p bubbles-ffi --release
cd crates/bubbles-ffi/tests/dotnet_smoke
LD_LIBRARY_PATH=../../../../target/release dotnet run -c Release
```

On Windows, copy `target\release\bubbles_ffi.dll` next to the built executable (or add that directory to `PATH`) and run `dotnet run -c Release` from `tests\dotnet_smoke`.

## C# sketch (Unity)

Pin UTF-8 bytes for `bubbles_compile` / `bubbles_runner_start` (for example `GCHandle.Alloc(bytes, GCHandleType.Pinned)` and `AddrOfPinnedObject()`). Event JSON from `bubbles_runner_next_event` is **NUL-terminated**; free it with `bubbles_string_free`. Length arguments use the **native `usize`** (`nuint` in modern C#).

Host callbacks can use `[UnmanagedCallersOnly(CallConvs = [typeof(CallConvCdecl)])]` and pass a `delegate* unmanaged[Cdecl]<…>` to `bubbles_runner_register_function`. Copy result strings with `bubbles_copy_utf8` so the runtime can free them with `bubbles_string_free`.

The header lists every symbol; generate bindings or mirror the declarations you need.

`bubbles_last_error()` returns a pointer valid until the next native call on the same thread. If your runtime exposes `Marshal.PtrToStringUTF8`, you can use that for NUL-terminated error text.

## ABI version

`bubbles_abi_version()` must match what your bindings expect (currently **1**).
