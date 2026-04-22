# Changelog

All notable changes are documented here (keep-a-changelog format).

## [0.1.0] — unreleased

### Added

- `compile` / `compile_many` functions and `Program` struct
- `Runner` with pull-based `next_event()` / `select_option()` API
- `DialogueEvent` enum: `NodeStarted`, `Line`, `Options`, `Command`, `NodeComplete`, `DialogueComplete`
- `DialogueOption` with `available` guard evaluation
- `.bub` script syntax: nodes, lines, shortcut options, line groups (`=>`), node groups (`when:`)
- `<<if>>` / `<<elseif>>` / `<<else>>` / `<<endif>>` conditional blocks
- `<<set>>` and `<<declare>>` variable statements
- `<<jump>>` node transitions
- `<<detour>>` / `<<return>>` call-stack subroutines
- `<<once>>` / `<<endonce>>` single-run blocks with optional `<<else>>`
- `<<command>>` generic host command events
- `{expr}` inline expression interpolation in line text
- `#tag` trailing metadata on lines and commands
- `#line:<id>` tag for `LineProvider` localisation lookup
- `Value` enum: `Number(f64)`, `Text(String)`, `Bool(bool)`
- `VariableStorage` trait and `HashMapStorage` implementation
- `FunctionLibrary` with built-ins: `round`, `floor`, `ceil`, `min`, `max`, `abs`, `clamp`, `string`, `int`, `random`, `random_range`, `dice`, `visited`, `visited_count`
- `SaliencyStrategy` trait with `FirstAvailable` and `RandomAvailable` strategies
- `LineProvider` trait with `PassthroughProvider` and `HashMapProvider`
- `validate()` function for compile-time cross-node reference checking
- `Program::node_exists`, `node_titles`, `node_tags` introspection accessors
- `serde` feature gate for `Value` and `HashMapStorage`
- `rand` feature gate for random built-ins and `RandomAvailable`
- Minimal CLI runner example (`examples/cli_runner.rs`)
