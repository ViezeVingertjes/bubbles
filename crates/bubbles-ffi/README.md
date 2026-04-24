# bubbles-ffi

Thin **C ABI** around [`bubbles-dialogue`](../bubbles-dialogue) so you can drive `.bub` scripts from **Unity** (or any host with `DllImport` / `ffi`).

- Events come back as **JSON** strings (parse in C# with `System.Text.Json`, Newtonsoft, etc.).
- All text is **UTF-8** with an explicit **byte length** on inputs (no embedded NUL required).
- **Not thread-safe:** use from your game main thread only (typical for Unity).

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

After `cargo build -p bubbles-ffi --release`, a minimal **.NET 10** console app under [`tests/dotnet_smoke/`](tests/dotnet_smoke/) loads `libbubbles_ffi.so` (Linux), `libbubbles_ffi.dylib` (macOS), or `bubbles_ffi.dll` (Windows) by name `bubbles_ffi`. From the repo root:

```sh
cargo build -p bubbles-ffi --release
cd crates/bubbles-ffi/tests/dotnet_smoke
# Linux / macOS: point the loader at the release output (adjust path if your layout differs).
LD_LIBRARY_PATH=../../../../target/release dotnet run -c Release
```

On Windows, copy `target\release\bubbles_ffi.dll` next to the built executable (or add that directory to `PATH`) and run `dotnet run -c Release` from `tests\dotnet_smoke`.

## C# sketch (Unity)

Pin UTF-8 bytes for `bubbles_compile` / `bubbles_runner_start` (for example `GCHandle.Alloc(bytes, GCHandleType.Pinned)` and `AddrOfPinnedObject()`). Event JSON from `bubbles_runner_next_event` is **NUL-terminated**; free it with `bubbles_string_free`. Length arguments use the **native `usize`** (`nuint` in modern C#).

```csharp
using System;
using System.Runtime.InteropServices;
using System.Text;

public static class BubblesNative
{
    const string Dll = "bubbles_ffi";

    public const int OK = 0;
    public const int Done = 1;
    public const int Err = -1;

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern uint bubbles_abi_version();

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr bubbles_last_error();

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern void bubbles_string_free(IntPtr p);

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern int bubbles_compile(IntPtr textPtr, nuint textLen, out IntPtr outProgram);

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern int bubbles_runner_new(IntPtr program, out IntPtr outRunner);

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern void bubbles_program_free(IntPtr program);

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern void bubbles_runner_free(IntPtr runner);

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern int bubbles_runner_start(IntPtr runner, IntPtr nodePtr, nuint nodeLen);

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern int bubbles_runner_next_event(IntPtr runner, out IntPtr outJson);

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl)]
    public static extern int bubbles_runner_select_option(IntPtr runner, nuint index);

    public static string Utf8Bytes(IntPtr ptr, int byteLen)
    {
        if (ptr == IntPtr.Zero || byteLen == 0) return string.Empty;
        byte[] buf = new byte[byteLen];
        Marshal.Copy(ptr, buf, 0, byteLen);
        return Encoding.UTF8.GetString(buf);
    }
}
```

`bubbles_last_error()` returns a pointer valid until the next native call on the same thread. If your runtime exposes `Marshal.PtrToStringUTF8`, you can use that for NUL-terminated error text.

## ABI version

`bubbles_abi_version()` must match what your bindings expect (currently **1**).
