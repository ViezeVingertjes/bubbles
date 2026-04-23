# Changelog

All notable changes are documented here (keep-a-changelog format).

## [Unreleased]

### Added

- `VariableStorage::get_ref(&self, name) -> Option<Cow<'_, Value>>`: an
  optional, borrow-friendly read path the expression evaluator prefers.
  The default implementation forwards to `get` (so existing storages keep
  working untouched); override it to return `Cow::Borrowed` and
  `{$text}` interpolation evaluates without cloning the string.
  `HashMapStorage` overrides it accordingly.

### Changed

- **Breaking (AST):** every statement-body field in the AST is now
  `Arc<[Stmt]>` (re-exported as `bubbles::compiler::ast::StmtList`) instead
  of `Vec<Stmt>` / `Arc<Vec<Stmt>>`.  This covers `Node::body`,
  `IfBranch::body`, `Stmt::If { else_body }`, `Stmt::Once { body, else_body }`,
  and `OptionItem::body`.  Anyone constructing AST nodes by hand must wrap
  statement lists in `Arc::from(vec_of_stmt)`.  In return, frame pushes
  (`<<if>>`, `<<once>>`, options, detours, jumps, node-group selection) no
  longer clone statement vectors — they bump an `Arc` refcount.
- `Runner` frames now store `{ node: Arc<str>, body: Arc<[Stmt]>, ip: usize }`
  and advance a program counter instead of popping off a `VecDeque<Stmt>`;
  stepping is a simple indexed read.
- `exec_jump` / `exec_detour` / `Runner::start` share a new
  `enter_node(target, replace_stack)` helper, removing three copies of the
  "resolve body, bump visits, push frame, emit `NodeStarted`" idiom.
- `Runner` now stores visit counts as a plain `HashMap<String, u32>` instead of
  `Arc<Mutex<HashMap<...>>>`.  The `visited()` and `visited_count()` builtins
  are intercepted by the evaluator directly, removing the Mutex, the lock
  surface, and the "panics on poisoned mutex" caveats from `Runner::new`,
  `Runner::start`, `Runner::snapshot`, and `Runner::restore`.  User-registered
  functions named `visited` / `visited_count` are now masked by the builtins
  (before, library lookups for those names were never reached because the
  closures registered by `Runner::new` resolved first, so the observable
  behaviour is unchanged).

### Internal

- Removed `src/runtime/interpolate.rs`, a dead module that only held a duplicate
  copy of the interpolation algorithm used by its own tests.  End-to-end
  behaviour is still covered by `tests/interpolation.rs` and
  `tests/compiled_expr_pipeline.rs`.

## [0.2.0] — 2026-04-23

### Added

- **`plural(count, singular, plural)`** built-in: returns the singular form when `|count| == 1`,
  plural otherwise.  Usable inside any `{expr}` substitution.
- **`select(key, mapping)`** built-in: key-based text dispatch for gendered grammar and similar
  patterns.  Format: `"k1:text1|k2:text2|other:fallback"`.  The first colon per entry is the
  separator so values may contain colons.  The `other` key is required; omitting it returns an error.
- **Translate-then-format provider ordering**: `LineProvider::get()` is now called *before*
  `{expr}` segments are evaluated.  The returned string is itself a template — any `{expr}` it
  contains are evaluated against current variable storage after translation.  This lets translators
  reorder or reshape interpolations freely and use `plural()` / `select()` inside translated strings.
  `exec_line_group` and `exec_options` also gained provider lookup (previously only `exec_line` used it).
- CI and `scripts/check-wasm.sh`: `wasm32-unknown-unknown` clippy for `--no-default-features` and
  `--no-default-features --features serde` (library only; keeps the crate wasm-compatible).
- `autoexamples = false` with explicit `[[example]]` entries so ad-hoc files under `examples/` are
  not picked up by Cargo (local scratch examples stay out of `cargo clippy --all-targets`).
- `line_id_from_tags()` helper and `line_id: Option<String>` on `DialogueEvent::Line` and
  `DialogueOption` when the source has a `#line:<id>` tag (stable key for VO / loc without
  re-parsing `tags`).

### Changed

- **Breaking:** `RunnerSnapshot::visits` changed from `HashMap<String, usize>` to
  `HashMap<String, u32>`.  Existing snapshots serialised as JSON are unaffected on 64-bit targets
  (JSON numbers are untyped), but binary formats may need migration.
- **Breaking:** `LineProvider::get()` contract changed — the returned `String` is now a *template*
  that may contain `{expr}` syntax evaluated after translation.  Plain strings (no braces) continue
  to work unchanged.
- **Breaking:** `DialogueEvent::Line` and `DialogueOption` have a new `line_id` field.  Update
  struct literals and exhaustive matches, or use `..` in patterns.
- `rand` builtins (`random_range`, `dice`) now validate their arguments (non-finite, fractional, or
  out-of-range values produce a `DialogueError::Function` instead of silently wrapping).

### Fixed

- Removed all `#[allow(clippy::...)]` suppressions; the underlying code was restructured so no
  suppression is needed.

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
