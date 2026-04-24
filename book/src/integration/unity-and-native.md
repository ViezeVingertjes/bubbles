# Unity and native hosts (C ABI)

`bubbles-ffi` is a shared library with a C ABI for driving `.bub` dialogue from C#, C++, or any language that can load a native plugin. Unity's `DllImport`, .NET P/Invoke, and Godot .NET all work.

Every symbol is declared in [`include/bubbles_ffi.h`](https://github.com/ViezeVingertjes/bubbles/blob/main/crates/bubbles-ffi/include/bubbles_ffi.h). Events come back as JSON strings; everything else uses UTF-8 byte slices.

## Build

```sh
cargo build -p bubbles-ffi --release
```

| Platform | File |
|----------|------|
| Linux    | `target/release/libbubbles_ffi.so` |
| macOS    | `target/release/libbubbles_ffi.dylib` |
| Windows  | `target/release/bubbles_ffi.dll` |

Copy the output into your Unity project's `Assets/Plugins/` folder (platform subfolders as needed).

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
{ "kind": "Options",  "options": [{ "text": "Hello", "tags": [] }, ...] }
{ "kind": "Command",  "name": "play_music", "args": ["tavern_theme"] }
{ "kind": "NodeStarted",     "title": "Intro" }
{ "kind": "NodeComplete",    "title": "Intro" }
{ "kind": "DialogueComplete" }
```

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
- **ABI version** - call `bubbles_abi_version()` at startup and assert it equals the version your bindings target (currently `1`).
- **Errors** - on `BUBBLES_ERR`, call `bubbles_last_error()` for a NUL-terminated UTF-8 message valid until the next `bubbles_*` call on the same thread. `Marshal.PtrToStringUTF8` reads it without copying.

## Still Rust-only

The C API fixes storage as `HashMapStorage`, line lookup as `HashMapProvider` (from JSON), and saliency as one of the three strategies above. A fully custom [`VariableStorage`](../api/bubbles/trait.VariableStorage.html), [`LineProvider`](https://docs.rs/bubbles-dialogue/latest/bubbles/trait.LineProvider.html), or [`SaliencyStrategy`](../api/bubbles/trait.SaliencyStrategy.html) still needs a Rust shim that configures [`RunnerBuilder`](https://docs.rs/bubbles-dialogue/latest/bubbles/struct.RunnerBuilder.html) and exposes its own FFI.

A .NET smoke app lives in the repo at `crates/bubbles-ffi/tests/dotnet_smoke/`; CI builds the release library and runs it on Linux.

---

> **See also:** [WebAssembly](../advanced/wasm.md) if you embed dialogue in the browser without a native plugin.
