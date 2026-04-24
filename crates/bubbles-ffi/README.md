# bubbles-ffi

C ABI shared library for driving `.bub` dialogue from Unity, C#, C++, or any native host.

```sh
cargo build -p bubbles-ffi --release
```

| Platform | File |
|----------|------|
| Linux    | `target/release/libbubbles_ffi.so` |
| macOS    | `target/release/libbubbles_ffi.dylib` |
| Windows  | `target/release/bubbles_ffi.dll` |

For everything else - P/Invoke declarations, event JSON format, variables, save/load, host functions, and the .NET smoke test - see the [Unity and native hosts](../../book/src/integration/unity-and-native.md) chapter in the guide.

C header: [`include/bubbles_ffi.h`](include/bubbles_ffi.h).
