# Changelog

All notable changes are documented here (keep-a-changelog format).

## [Unreleased]

### Added

- `line_id_from_tags()` helper and `line_id: Option<String>` on `DialogueEvent::Line` and
  `DialogueOption` when the source has a `#line:<id>` tag (stable key for VO / loc without re-parsing `tags`).

### Changed

- **Breaking:** `DialogueEvent::Line` and `DialogueOption` have a new `line_id` field. Update struct
  literals and exhaustive matches, or use `..` in patterns.

## [0.1.0] — 2026-04-22

### Added

- `full` Cargo feature: shorthand for `rand` and `serde` together.
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

- **MSRV** is **1.95** (was 1.85); CI uses `actions/checkout@v6` and read-only
  `permissions: contents: read` for the default `GITHUB_TOKEN`.
- `helpers.rs` files in `src/compiler/parser/` and `src/runtime/runner/` split
  into concept-focused modules (`text`, `command`, `assignments`, `body` /
  `evaluation`, `node_body`) to satisfy the one-concept-per-file rule.
- Extracted a shared `Runner::push_inline_frame` helper used by `<<if>>`,
  `<<once>>`, and option-body execution, removing three copies of the same
  frame-push idiom.
- `parse_if` / `<<once if>>` / option and line-group guards / node `when:` now
  build a shared `Arc<Expr>` at compile time via `parse_expr_arc` instead of
  storing only source strings and re-parsing on every evaluation.
- `Node.body` is now `Arc<Vec<Stmt>>` so `pick_node_body` can clone the
  statement list without re-allocating shared expression trees (each
  `Stmt` holds `Arc<Expr>` where applicable, so `Stmt` clone is cheap for
  hot paths like detours and `<<if>>` branches).
- `Runner` visit counts use `Arc<Mutex<HashMap<…>>>` instead of `RwLock` —
  the previous design only needed re-entrancy for `visited()` / `visited_count()`
  builtins; a mutex matches single-threaded game-loop use and is simpler to
  reason about.
- `SaliencyStrategy` implementations live in separate files under `saliency/`
  (`candidate`, `first`, `random`, `blrv`, `tests`) instead of a single
  `mod.rs` over 200 lines.
- Lint configuration is now owned exclusively by `Cargo.toml`; `src/lib.rs`
  no longer duplicates the `deny` / `warn` attributes.
- `{expr}` fragments in line text, option text, line-group text, and command
  argument strings are now parsed at compile time (via `parse_interpolated`
  into `Vec<TextSegment>`) and stored as `TextSegment::Literal` /
  `TextSegment::Expr(Arc<Expr>)`. At runtime, `Runner::eval_segments` evaluates
  each segment in a single pass with no re-parsing, removing the `interpolate`
  runtime-parse path entirely. Invalid `{expr}` fragments in any text field are
  now caught by `compile()` / `compile_many()` with a `Parse` error.
