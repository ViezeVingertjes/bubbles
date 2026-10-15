# Native hosts (C ABI)

The Rust API is the primary way to use Bubbles, and the one the rest of this guide describes. This chapter covers the other case: a host that cannot link Rust directly.

`bubbles-ffi` is a shared library with a C ABI for driving `.bub` dialogue from C, C++, C#, or any language that can load a native plugin. Unity's `DllImport`, .NET P/Invoke, and Godot .NET all work.

Every symbol is declared in [`include/bubbles_ffi.h`](https://github.com/ViezeVingertjes/bubbles/blob/main/crates/bubbles-ffi/include/bubbles_ffi.h), which carries its own ABI and library version. Events come back as JSON strings; everything else uses UTF-8 byte slices.

Two documents back this chapter up:

- **[`bubbles_ffi.h`](https://github.com/ViezeVingertjes/bubbles/blob/main/crates/bubbles-ffi/include/bubbles_ffi.h)** - the authoritative declarations. If this chapter and the header disagree, the header wins.
- **[`ABI.md`](https://github.com/ViezeVingertjes/bubbles/blob/main/crates/bubbles-ffi/ABI.md)** - the stability policy: what you may rely on across releases, the full event JSON schema, who frees what, and the threading rules. Read it before you ship.

## Get the library

### Download a prebuilt binary

You do **not** need a Rust toolchain. Grab the archive for your platform from a [release](https://github.com/ViezeVingertjes/bubbles/releases):

| Platform | Archive | Contents |
|----------|---------|----------|
| Windows x64     | `*-x86_64-pc-windows-msvc.zip`      | `bubbles_ffi.dll` + `bubbles_ffi.dll.lib` |
| macOS universal | `*-universal-apple-darwin.tar.gz`   | `libbubbles_ffi.dylib` (x64 + arm64) |
| macOS x64       | `*-x86_64-apple-darwin.tar.gz`      | `libbubbles_ffi.dylib` |
| macOS arm64     | `*-aarch64-apple-darwin.tar.gz`     | `libbubbles_ffi.dylib` |
| Linux x64       | `*-x86_64-unknown-linux-gnu.tar.gz` | `libbubbles_ffi.so` (glibc 2.35+) |

Every archive also contains `bubbles_ffi.h`, `ABI.md`, and both licences, so one download is everything you need to compile against. Verify it against `SHA256SUMS.txt` on the release.

For Unity on macOS, take the **universal** archive - Unity expects one dylib covering both architectures.

### Or build from source

```sh
cargo build -p bubbles-ffi --release
```

| Platform | File |
|----------|------|
| Linux    | `target/release/libbubbles_ffi.so` |
| macOS    | `target/release/libbubbles_ffi.dylib` |
| Windows  | `target/release/bubbles_ffi.dll` |

Copy the output into your Unity project's `Assets/Plugins/` folder (platform subfolders as needed).

## Fastest way to see it work

If you just want to confirm the ABI does what this chapter claims, the repository has a complete host in one dependency-free C file:

```sh
cd examples/c-host
make run
```

It loads a `.bub` file, pumps the event loop, prints lines, and selects an option in about 300 lines of C with no JSON library and no Rust. [`examples/c-host/README.md`](https://github.com/ViezeVingertjes/bubbles/blob/main/examples/c-host/README.md) walks through it step by step, including how to build it against a downloaded prebuilt binary instead of a local cargo build.

## The shape of it

Compile a `.bub` source to a program handle, create a runner from it, then pull events in a loop.

```csharp
// P/Invoke declarations (mirror whatever you need from the header)
[DllImport("bubbles_ffi")] static extern int  bubbles_compile(nint textPtr, nuint textLen, out nint outProgram);
[DllImport("bubbles_ffi")] static extern int  bubbles_runner_new(nint program, out nint outRunner);
[DllImport("bubbles_ffi")] static extern int  bubbles_runner_start(nint runner, nint nodePtr, nuint nodeLen);
[DllImport("bubbles_ffi")] static extern int  bubbles_runner_next_event(nint runner, out nint outEventJson);
[DllImport("bubbles_ffi")] static extern void bubbles_string_free(nint p);
[DllImport("bubbles_ffi")] static extern void bubbles_runner_free(nint runner);

const int BUBBLES_OK   =  0;
const int BUBBLES_DONE =  1;
const int BUBBLES_ERR  = -1;
```

```csharp
byte[] src  = Encoding.UTF8.GetBytes(File.ReadAllText("dialogue.bub"));
byte[] node = Encoding.UTF8.GetBytes("Start");

var srcHandle  = GCHandle.Alloc(src,  GCHandleType.Pinned);
var nodeHandle = GCHandle.Alloc(node, GCHandleType.Pinned);
try {
    if (bubbles_compile(srcHandle.AddrOfPinnedObject(), (nuint)src.Length, out nint program) != BUBBLES_OK)
        throw new Exception(LastError());

    // bubbles_runner_new consumes the program handle; don't free it separately.
    if (bubbles_runner_new(program, out nint runner) != BUBBLES_OK)
        throw new Exception(LastError());

    if (bubbles_runner_start(runner, nodeHandle.AddrOfPinnedObject(), (nuint)node.Length) != BUBBLES_OK)
        throw new Exception(LastError());

    while (true) {
        int rc = bubbles_runner_next_event(runner, out nint eventPtr);
        if (rc == BUBBLES_DONE) break;
        if (rc != BUBBLES_OK)   throw new Exception(LastError());

        string json = Marshal.PtrToStringUTF8(eventPtr)!;
        bubbles_string_free(eventPtr);
        HandleEvent(runner, json);
    }
} finally {
    srcHandle.Free();
    nodeHandle.Free();
}
```

## Parsing events

Each event is a JSON object with a `"kind"` field:

```json
{ "kind": "Line",     "speaker": "Aria", "text": "Evening, friend.", "tags": [] }
{ "kind": "Options",  "options": [{ "text": "Hello", "available": true, "tags": [] }] }
{ "kind": "Command",  "name": "play_music", "args": ["tavern_theme"] }
{ "kind": "NodeStarted",     "node": "Intro" }
{ "kind": "NodeComplete",    "node": "Intro" }
{ "kind": "DialogueComplete" }
```

These are abridged. The [C ABI stability policy](https://github.com/ViezeVingertjes/bubbles/blob/main/crates/bubbles-ffi/ABI.md#event-json-schema)
has the complete schema with every field and its type, plus the two details most likely to
bite a C# host: markup span offsets are **byte** offsets into the UTF-8 text (not indices
into a .NET `string`), and `properties` is an array of `[key, value]` pairs rather than an
object.

Your JSON parser must ignore unknown keys - new fields are added in patch releases.

A typical handler:

```csharp
[DllImport("bubbles_ffi")] static extern int bubbles_runner_select_option(nint runner, nuint index);

void HandleEvent(nint runner, string json) {
    using var doc = JsonDocument.Parse(json);
    switch (doc.RootElement.GetProperty("kind").GetString()) {
        case "Line":
            string? speaker = doc.RootElement.GetProperty("speaker").GetString();
            string? text    = doc.RootElement.GetProperty("text").GetString();
            ShowLine(speaker, text);
            break;
        case "Options":
            JsonElement opts = doc.RootElement.GetProperty("options");
            int choice = ShowOptions(opts);   // your UI picks an index
            bubbles_runner_select_option(runner, (nuint)choice);
            break;
        case "Command":
            DispatchCommand(doc.RootElement);
            break;
        // "NodeStarted", "NodeComplete", "DialogueComplete", "Unknown": ignore or handle as needed
    }
}
```

`DialogueEvent` is `#[non_exhaustive]` on the Rust side. Unknown future variants arrive as `{ "kind": "Unknown" }` - ignore them and keep looping.

## Configuring the runner

### Saliency

```csharp
[DllImport("bubbles_ffi")]
static extern int bubbles_runner_new_with_saliency(nint program, int saliencyKind, out nint outRunner);

const int BUBBLES_SALIENCY_FIRST_AVAILABLE  = 0;  // default
const int BUBBLES_SALIENCY_BLRV             = 1;  // best least-recently viewed
const int BUBBLES_SALIENCY_RANDOM_AVAILABLE = 2;
```

```csharp
bubbles_runner_new_with_saliency(program, BUBBLES_SALIENCY_BLRV, out nint runner);
```

See [Saliency Strategies](./saliency.md) for what each strategy does.

### Reproducible sessions

`bubbles_runner_new` and `bubbles_runner_new_with_saliency` seed randomness from the operating system, so a script that rolls dice plays differently every run. Pass a seed instead and the session replays exactly:

```csharp
[DllImport("bubbles_ffi")]
static extern int bubbles_runner_new_seeded(nint program, int saliencyKind, ulong seed, out nint outRunner);
```

```csharp
bubbles_runner_new_seeded(program, BUBBLES_SALIENCY_FIRST_AVAILABLE, 20260501, out nint runner);
```

Same program, same starting variables, same seed, same option selections gives the same sequence of events - every `random()`, `random_range()` and `dice()` result and every `BUBBLES_SALIENCY_RANDOM_AVAILABLE` choice - in every process and on every platform. That is what makes replays, lockstep multiplayer, and golden-file tests of a conversation reachable from a native host.

Note `ulong`, not `long`: the seed is a `uint64_t`. There is no `set_seed` counterpart, because a seed belongs to a session rather than to a moment in one - choose it at construction. A snapshot records where the random stream had reached, so a restored save continues it rather than replaying from the top.

### Locale

Pass a JSON object of `line_id -> template` strings before `bubbles_runner_start`:

```csharp
[DllImport("bubbles_ffi")]
static extern int bubbles_runner_set_locale_json(nint runner, nint jsonPtr, nuint jsonLen);
```

```csharp
byte[] locale = Encoding.UTF8.GetBytes("""{"aria_greet_01":"Buenas tardes, amigo."}""");
var h = GCHandle.Alloc(locale, GCHandleType.Pinned);
bubbles_runner_set_locale_json(runner, h.AddrOfPinnedObject(), (nuint)locale.Length);
h.Free();
```

See [Localisation](./localisation.md) for how `#line:` ids and templates work.

### Host functions

Register a C callback for any function called from a `.bub` script:

```csharp
[DllImport("bubbles_ffi")]
static extern int bubbles_runner_register_function(
    nint runner, nint namePtr, nuint nameLen,
    delegate* unmanaged[Cdecl]<nint, nint, nuint, nint*, int> cb,
    nint userdata);

[DllImport("bubbles_ffi")] static extern nint bubbles_copy_utf8(nint ptr, nuint len);
```

```csharp
[UnmanagedCallersOnly(CallConvs = [typeof(CallConvCdecl)])]
static unsafe int HostAddOne(nint userdata, nint argsPtr, nuint argsLen, nint* outResult) {
    // args arrive as a JSON array of dialogue Values
    string argsJson = Marshal.PtrToStringUTF8(argsPtr, (int)argsLen)!;
    using var doc = JsonDocument.Parse(argsJson);
    double n = doc.RootElement[0].GetDouble();

    // return result as a JSON scalar allocated with bubbles_copy_utf8
    byte[] result = Encoding.UTF8.GetBytes(n + 1);
    var h = GCHandle.Alloc(result, GCHandleType.Pinned);
    *outResult = bubbles_copy_utf8(h.AddrOfPinnedObject(), (nuint)result.Length);
    h.Free();
    return BUBBLES_OK;
}
```

The runtime frees the result string with `bubbles_string_free`, so always allocate it with `bubbles_copy_utf8`. See [Custom Functions](./functions.md) for the argument and return value conventions.

## Variables

```csharp
[DllImport("bubbles_ffi")]
static extern int bubbles_runner_variable_get_json(
    nint runner, nint namePtr, nuint nameLen, out nint outJson);

[DllImport("bubbles_ffi")]
static extern int bubbles_runner_variable_set_json(
    nint runner, nint namePtr, nuint nameLen, nint valuePtr, nuint valueLen);
```

```csharp
// Read $hp - returns a JSON scalar ("100", "true", etc.) or "null" if unset
byte[] name = Encoding.UTF8.GetBytes("$hp");
var h = GCHandle.Alloc(name, GCHandleType.Pinned);
bubbles_runner_variable_get_json(runner, h.AddrOfPinnedObject(), (nuint)name.Length, out nint json);
h.Free();
string valueJson = Marshal.PtrToStringUTF8(json)!;
bubbles_string_free(json);
```

To write, pin a JSON scalar the same way and pass it to `bubbles_runner_variable_set_json`. Plain JSON types (`42`, `"sword"`, `true`) and tagged objects (`{"Number":42}`, `{"Text":"sword"}`, `{"Bool":true}`) are both accepted.

## Save and load

Take two snapshots - session state and variable storage - then restore them in order:

```csharp
[DllImport("bubbles_ffi")] static extern int bubbles_runner_snapshot_session_json(nint runner, out nint outJson);
[DllImport("bubbles_ffi")] static extern int bubbles_runner_snapshot_storage_json(nint runner, out nint outJson);
[DllImport("bubbles_ffi")] static extern int bubbles_runner_restore_storage_json(nint runner, nint jsonPtr, nuint jsonLen);
[DllImport("bubbles_ffi")] static extern int bubbles_runner_restore_session_json(nint runner, nint jsonPtr, nuint jsonLen);
```

```csharp
// Save
bubbles_runner_snapshot_storage_json(runner, out nint storagePtr);
bubbles_runner_snapshot_session_json(runner, out nint sessionPtr);
string storageJson = Marshal.PtrToStringUTF8(storagePtr)!;
string sessionJson = Marshal.PtrToStringUTF8(sessionPtr)!;
bubbles_string_free(storagePtr);
bubbles_string_free(sessionPtr);
WriteSave(storageJson, sessionJson);

// Load: restore storage first, then session
byte[] storageSrc = Encoding.UTF8.GetBytes(storageJson);
byte[] sessionSrc = Encoding.UTF8.GetBytes(sessionJson);
var sh = GCHandle.Alloc(storageSrc, GCHandleType.Pinned);
var eh = GCHandle.Alloc(sessionSrc, GCHandleType.Pinned);
bubbles_runner_restore_storage_json(runner, sh.AddrOfPinnedObject(), (nuint)storageSrc.Length);
bubbles_runner_restore_session_json(runner, eh.AddrOfPinnedObject(), (nuint)sessionSrc.Length);
sh.Free();
eh.Free();
```

See [Save and Load](../advanced/save-load.md) for why order matters and what each snapshot captures.

## Practical notes

- **String ownership** - strings returned by the library (event JSON, variable JSON, snapshots) are library-owned. Free each one with `bubbles_string_free`. Do not pass them to `Marshal.FreeHGlobal` or C `free()`.
- **UTF-8 byte lengths** - all string inputs take a pointer plus a byte count as `nuint`. Use `Encoding.UTF8.GetBytes` and pass `.Length`. No trailing NUL required.
- **Threading** - the FFI surface is not thread-safe. Call it from one thread; Unity's main thread is fine.
- **ABI version** - call `bubbles_abi_version()` at startup and assert it equals `BUBBLES_ABI_VERSION` from the header your bindings were written against (currently `1`). It is bumped only for a breaking change, independently of the library version (`BUBBLES_FFI_VERSION_STRING`). A clear startup failure beats discovering a mismatch as memory corruption.
- **Save files** - treat both snapshot strings as opaque blobs. They round-trip within one library version; across versions a restore may fail, so handle a failed load as a recoverable condition rather than a crash.
- **Errors** - on `BUBBLES_ERR`, call `bubbles_last_error()` for a NUL-terminated UTF-8 message valid until the next `bubbles_*` call on the same thread. `Marshal.PtrToStringUTF8` reads it without copying.

## Platform reach

Releases carry Windows x64, macOS x64, macOS arm64, and Linux x64. Nothing in this crate is platform-specific, so other targets are expected to build from source with no code or ABI changes - they are simply not built or tested by CI. That is usually just:

```sh
cargo build -p bubbles-ffi --release --target <triple>
```

Android is the straightforward case: every Android ABI has a tier-2 Rust target and each produces an ordinary `.so` that a native plugin loader picks up, with your existing declarations unchanged. iOS is the awkward one, because pipelines that link native code into the app binary need a **static** library rather than a `cdylib`, which means a `staticlib` crate type and a change to how your bindings name the library. Neither is shipped today. If you need one, open an issue describing the host - it is CI and packaging work, not an ABI change.

### WebAssembly goes somewhere else

The dialogue runtime runs on `wasm32-unknown-unknown` today and CI keeps it that way, but it does not arrive through *this* header. `wasm32-unknown-unknown` has no notion of a C shared library to load: you get a module whose exports JavaScript calls directly. And this ABI's shape - opaque handles, caller-frees-strings, thread-local error slots, raw function-pointer callbacks - maps badly onto JS, where `wasm-bindgen` and a JS-idiomatic surface are the right design. The [WebAssembly chapter](../advanced/wasm.md) covers that path, including the short `wasm-bindgen` façade to write today.

## Still Rust-only

The C API fixes storage as `HashMapStorage`, line lookup as `HashMapProvider` (from JSON), and saliency as one of the three strategies above. A fully custom [`VariableStorage`](../api/bubbles/trait.VariableStorage.html), [`LineProvider`](https://docs.rs/bubbles-dialogue/latest/bubbles/trait.LineProvider.html), or [`SaliencyStrategy`](../api/bubbles/saliency/trait.SaliencyStrategy.html) still needs a Rust shim that configures [`RunnerBuilder`](https://docs.rs/bubbles-dialogue/latest/bubbles/struct.RunnerBuilder.html) and exposes its own FFI.

## Worked examples in the repository

Both are built and run by CI on every commit, so neither can rot:

| Example | Language | Covers |
|---------|----------|--------|
| [`examples/c-host/`](https://github.com/ViezeVingertjes/bubbles/blob/main/examples/c-host/README.md) | C99, no dependencies | Loading a `.bub` file, the event loop, selecting an option, the ownership rules |
| `crates/bubbles-ffi/tests/dotnet_smoke/` | C# / .NET 10 | The above plus host functions, localisation, variables, save/load, markup |

---

> **See also:** the [C ABI stability policy](https://github.com/ViezeVingertjes/bubbles/blob/main/crates/bubbles-ffi/ABI.md) before you ship, and [WebAssembly](../advanced/wasm.md) if you embed dialogue in the browser without a native plugin.
