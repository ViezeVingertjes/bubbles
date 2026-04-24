# API Reference

Every public type, trait, and function has full rustdoc. Two places to read it:

- **This site, under [`/api/bubbles/`](./api/bubbles/index.html)** - built from the same commit as the guide you're reading.
- **[docs.rs](https://docs.rs/bubbles-dialogue)** - rebuilt on every crate release.

Start there for the authoritative signatures, trait definitions, and error types. The guide points into specific pages as you go.

## High-traffic items

A quick index of what you'll look up most often:

| What you want | Where to look |
|---|---|
| Compile a script | [`compile`](./api/bubbles/fn.compile.html), [`compile_many`](./api/bubbles/fn.compile_many.html) |
| Drive a dialogue | [`Runner`](./api/bubbles/struct.Runner.html), [`DialogueEvent`](./api/bubbles/enum.DialogueEvent.html) |
| Store variables | [`VariableStorage`](./api/bubbles/trait.VariableStorage.html), [`HashMapStorage`](./api/bubbles/struct.HashMapStorage.html) |
| Localise lines | [`LineProvider`](./api/bubbles/trait.LineProvider.html), [`HashMapProvider`](./api/bubbles/struct.HashMapProvider.html) |
| Register host functions | [`FunctionLibrary`](./api/bubbles/struct.FunctionLibrary.html) |
| Pick variants | [`SaliencyStrategy`](./api/bubbles/trait.SaliencyStrategy.html), [`FirstAvailable`](./api/bubbles/struct.FirstAvailable.html), [`BestLeastRecentlyViewed`](./api/bubbles/struct.BestLeastRecentlyViewed.html) |
| Save / load | [`RunnerSnapshot`](./api/bubbles/struct.RunnerSnapshot.html) (requires `serde`) |
| Handle errors | [`DialogueError`](./api/bubbles/enum.DialogueError.html) |
| Unity / C# / C shared library | Not on docs.rs: see the [guide chapter](./integration/unity-and-native.md) and the [C header](https://github.com/ViezeVingertjes/bubbles/blob/main/crates/bubbles-ffi/include/bubbles_ffi.h) |

## Feature flags

| Flag | Default | Enables |
|---|---|---|
| `rand` | **on** | `random`, `random_range`, `dice`, `RandomAvailable` |
| `serde` | off | `Serialize`/`Deserialize` on `Value`, `HashMapStorage`, `RunnerSnapshot` |
| `full` | off | Both `rand` and `serde` together |

## Still not sure?

- Search the guide (top-right) for keywords like "once", "option", "localisation".
- Jump into the [examples](./examples/tui-runner.md) - they cover most of the API in under 200 lines each.
- [Open an issue](https://github.com/ViezeVingertjes/bubbles/issues) if something's unclear. Documentation gaps are bugs.
