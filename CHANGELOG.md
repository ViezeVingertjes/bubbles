# Changelog

All notable changes are documented here (keep-a-changelog format).

## [0.1.0] — 2026-04-22

### Added

- `compile` / `compile_many` and `Program` with node map and merge-time duplicate detection
- `Runner` with pull-based `next_event()` / `select_option()` and call stack for detours
- `DialogueEvent`: `NodeStarted`, `Line`, `Options`, `Command`, `NodeComplete`, `DialogueComplete`
- `DialogueOption` with guard-driven `available` flag
- `.bub` syntax: nodes (`title:` / `---` / `===`), lines, speaker prefixes, shortcut options (`->`),
  line groups (`=>`), node groups (`when:`)
- `<<if>>` / `<<elseif>>` / `<<else>>` / `<<endif>>`
- `<<set>>` (including `to` form) and `<<declare>>` smart variables
- `<<jump>>`, `<<detour>>`, `<<return>>`
- `<<once>>` / `<<once if expr>>` / `<<endonce>>` with optional `<<else>>`
- Generic `<<command>>` events with args and `#tag` metadata
- `{expr}` interpolation; `#tag` and `#line:<id>` on lines and commands
- `Value` (`Number`, `Text`, `Bool`), `VariableStorage`, `HashMapStorage`
- `FunctionLibrary` with `round`, `floor`, `ceil`, `min`, `max`, `abs`, `clamp`, `string`, `int`,
  `visited`, `visited_count`, and (with `rand`) `random`, `random_range`, `dice`
- `SaliencyStrategy`: `FirstAvailable`, `RandomAvailable`, `BestLeastRecentlyViewed`, or custom
- `LineProvider`: `PassthroughProvider`, `HashMapProvider`
- `validate()` for jump/detour targets across merged programs
- Introspection: `node_exists`, `node_titles`, `node_tags`, `variable_declarations` / `VariableDecl`
- `serde` feature: `Value`, `HashMapStorage`, `RunnerSnapshot`; `Runner::snapshot` / `restore`
- `rand` feature: random builtins and `RandomAvailable`
- Parse-time expression checks for control statements and assignments; clearer structural errors
- Runtime errors for division and modulo by zero
- Examples: `cli_runner`, `tavern`
- Broad integration tests and property-based checks on the expression evaluator

### Fixed

- `<<once if …>>` conditions are recognised (parser strips the `once` prefix before reading `if`).
