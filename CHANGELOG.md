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
- Parse errors in `<<if>>`, `<<once if>>`, missing `---`, missing `===`, and missing
  `title:` headers now report the actual source line number instead of an internal
  buffer index (which previously drifted past blank lines and comments).
- Resolved blocking `clippy::approx_constant` errors that caused CI to fail on
  the `--all-features` test build.

### Changed

- `helpers.rs` files in `src/compiler/parser/` and `src/runtime/runner/` split
  into concept-focused modules (`text`, `command`, `assignments`, `body` /
  `evaluation`, `node_body`) to satisfy the one-concept-per-file rule.
- Extracted a shared `Runner::push_inline_frame` helper used by `<<if>>`,
  `<<once>>`, and option-body execution, removing three copies of the same
  frame-push idiom.
- `parse_if` / `parse_once` now delegate to the shared `validate_expr` helper
  instead of open-coding the same check.
- Lint configuration is now owned exclusively by `Cargo.toml`; `src/lib.rs`
  no longer duplicates the `deny` / `warn` attributes.
