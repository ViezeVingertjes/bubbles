# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Versions refer to the published [`bubbles-dialogue`](https://crates.io/crates/bubbles-dialogue)
crate. The `bubbles-ffi` and `bubbles-cli` crates share the workspace version and are
publishable, but are not published to crates.io yet.

> **Note on history.** This file was reconstructed in October 2026 from git
> history, release tags, and crates.io publication records. Releases 0.1.0
> through 0.3.0 were published to crates.io but never tagged in git, so they have
> no comparison links below. Entries for those releases are recovered from a
> `CHANGELOG.md` that existed in the repository until 0.5.0 and was then deleted;
> where that file disagreed with the code actually shipped in a tag, the tag wins
> and the discrepancy is noted.

## [Unreleased]

### ⚠ Breaking changes

- **`RunnerSnapshot` and `HashMapStorage` now use ordered collections.**
  `RunnerSnapshot::visits` is a `BTreeMap<String, u32>`, `RunnerSnapshot::once_seen`
  is a `BTreeSet<String>`, and `HashMapStorage` holds a `BTreeMap` internally.

  Nothing was ever corrupted by the old types — restore was correct and
  `PartialEq` compares by value — but `std`'s hashing is seeded per process, so
  serialising the same state produced different bytes on every run, and
  `Runner::all_variables` returned its pairs in that same shifting order despite
  being documented for save editors and tests. Any workflow that treats a save
  as bytes (content-hashing, diffing two saves, golden-file tests,
  deduplication) could not work. Ordered collections make the bytes canonical:
  the same state always serialises identically, regardless of how it was reached.

  *Upgrading:* if you construct a `RunnerSnapshot` by hand, swap `HashMap` /
  `HashSet` for `BTreeMap` / `BTreeSet`; the entries are unchanged. If you read
  `snapshot.visits` or `snapshot.once_seen`, the API you need (`get`,
  `contains`, `iter`, `len`) is the same, and iteration is now sorted. Existing
  serialised saves load unchanged — JSON objects and arrays deserialise into
  either collection. Code that sorted the output of `all_variables` before
  comparing it can stop. `HashMapStorage` keeps its name: it is the default
  store, and renaming it would be a second break for no gain.

- **`RunnerSnapshot` has a new `rng` field**, carrying the position of the
  runner's random stream so a restored save continues it rather than diverging.

  *Upgrading:* struct literals need the extra field. Use
  `runner.rng().state()` to capture a live runner's position, or
  `RngState::default()` for a fixed starting stream. Snapshots written by 2.x
  deserialise without it and fall back to whatever an unseeded runner would
  have started from — a fresh OS-seeded stream with `rand`, the beginning of
  the stream without it — which is exactly what those saves did before.

- **`SaliencyStrategy::select` takes the runner's generator:**
  `fn select(&mut self, candidates: &[Candidate<'_>], rng: &DialogueRng) -> Option<usize>`.

  A strategy that chooses randomly has to draw from the runner's stream, or a
  seeded session is not reproducible. Passing the generator makes that
  structural rather than something a strategy can forget.

  *Upgrading:* add the parameter. Deterministic strategies ignore it and can
  name it `_rng`. Strategies that called `rand::rng()` should call
  `rng.choose_index(len)` instead. The built-in strategies are unchanged at the
  use site: `set_saliency(RandomAvailable)` still works as written.

### Added

- **`RunnerBuilder::with_seed(u64)` — seeded, reproducible dialogue.** Given the
  same program, the same starting variables, the same seed, and the same
  sequence of option selections, a runner produces the same sequence of events,
  in every process and on every platform, including every `random()`,
  `random_range()`, `dice()` and `RandomAvailable` choice. Previously those four
  reached for the OS-seeded thread-local RNG with no way to influence them, so
  replay, lockstep, and server-authoritative dialogue were out of reach for any
  script that rolled dice. Runners built without a seed behave as before and
  draw from the operating system — except with `default-features = false`,
  where there is no entropy source in the crate at all and an unseeded runner
  starts at the beginning of the stream, so that build replays identically
  whether or not you pass a seed. That is the `wasm32-unknown-unknown`
  configuration, so in a browser `with_seed` is the only way to make a session
  unpredictable. The crate deliberately does not improvise a seed there:
  `SystemTime` aborts the module on that target, and `std`'s hash key is a
  compile-time constant on it, so the two obvious substitutes are a crash and
  a fixed number dressed up as entropy.
- `DialogueRng` and `RngState`: the generator a runner draws from, and the
  stream position stored in a snapshot. `Runner::rng()` hands out a handle, so
  host functions registered through `library_mut()` can draw from the same
  stream and stay part of the replay. The generator is `SplitMix64`, implemented
  in this crate rather than taken from a dependency, because the stream is part
  of the save format and has to be identical on every target and in every build
  — `rand`'s own `SmallRng` is explicitly not reproducible across platforms or
  versions.
- **`bubbles_runner_new_seeded` in the C ABI** — the same reproducible session
  for a native host:
  `int32_t bubbles_runner_new_seeded(void *program, int32_t saliency_kind, uint64_t seed, void **out_runner)`.
  The existing constructors are OS-seeded, so a C or C# host had no way to
  replay a script that rolls dice. It is a constructor rather than a `set_seed`
  setter because a seed belongs to a session; pass
  `BUBBLES_SALIENCY_FIRST_AVAILABLE` for the default strategy. Additive — the
  ABI version stays at 1 and the header now declares 27 symbols, so a host
  built against the previous header links and runs unchanged.
- `CHANGELOG.md` (this file), backfilled across all 12 published releases.
- Developer Certificate of Origin (DCO) sign-off requirement for contributions,
  documented in [CONTRIBUTING.md](CONTRIBUTING.md) and enforced by a PR check.
- A versioned C header for `bubbles-ffi` at `crates/bubbles-ffi/include/bubbles_ffi.h`,
  carrying `BUBBLES_ABI_VERSION` alongside the library version, with every
  declaration documented. A test and `scripts/check-ffi-abi.sh` keep it from
  drifting from the built library.
- `crates/bubbles-ffi/ABI.md`, the C ABI stability policy: linkage, integer
  widths, status codes, struct layout, the event JSON schema, who frees what,
  threading rules, and an explicit list of what not to rely on.
- Prebuilt `bubbles-ffi` shared libraries for Windows x64, macOS x64, macOS
  arm64, macOS universal, and Linux x64, built by a workflow that attaches them
  to a release draft. Each archive carries the library, the header, `ABI.md`,
  and both licences, so a host needs no Rust toolchain.
- `examples/c-host/`, a dependency-free C99 host that loads a `.bub` file, pumps
  the event loop, and selects an option. CI builds and runs it.
- The WebAssembly gate in CI now instantiates a module and plays a script
  through it, instead of only compiling the library for the target. A call that
  diverges on `wasm32-unknown-unknown` compiles cleanly, and the optimiser is
  then free to delete everything downstream of it — so the build goes green,
  the artifact gets *smaller*, and the module traps on the first line of
  dialogue. Compiling could not see that; running can.
- `bubbles-ffi` is now publishable: it carries the metadata crates.io requires
  and `cargo publish --dry-run --no-verify` runs in CI. Verification is a release
  step rather than a CI one, because it builds the packaged tarball against the
  registry copy of `bubbles-dialogue`, which between releases does not yet have
  the API this crate calls; see [RELEASING.md](RELEASING.md). It remains
  unpublished for now.
- **`bubbles check <file.bub>...`**, a script checker with no Rust in sight. It
  compiles the given files as one programme, prints one diagnostic per line on
  stdout, and exits non-zero on failure. `--quiet` drops the success summary.
- A default-on `tui` feature on `bubbles-cli`. With
  `cargo install bubbles-cli --no-default-features` the terminal player is left
  out and the checker adds nothing to the library's own three direct
  dependencies - `indexmap`, `logos`, and `thiserror` - which is small enough to
  add to someone else's CI.
- `?` opens a key list in the player. The footer fits six hints and the tool
  binds thirteen verbs, so the rest had been unreachable without reading the
  source.

### Changed

- **`bubbles-tui` is now `bubbles-cli`, and its binary is `bubbles`.** The crate
  is publishable, with a description and keywords aimed at checking and
  embedding rather than at narrative authoring. `cargo run -p bubbles-tui -- X`
  becomes `cargo run -p bubbles-cli -- play X`; a bare path list with no verb
  still means `play`.
- **Reload moved off `r` and onto `Ctrl+R` (or `F5`), and `r` now means rerun.**
  Reload discards every variable with no confirmation and no undo, and it used
  to sit on the lowercase half of the only case-sensitive binding in a tool
  where `q/Q`, `j/J`, `k/K`, `b/B` and `x/X` are all interchangeable. The footer
  now spells it out as `Ctrl+R: reload (wipes variables)`.
- The guide's two TUI pages merged into one
  [CLI chapter](https://viezevingertjes.github.io/bubbles/getting-started/cli.html)
  that documents `check` first and `play` second. The old URLs redirect.
- The `plural` and `select` built-ins are no longer documented as
  internationalisation. `plural` is described as the two-form rule it actually
  implements - the singular form when `|n|` is 1, the plural form otherwise -
  and as incorrect for languages with other plural categories, with
  `FunctionLibrary::register` named as the way to install a correct rule.
  `select` now records that its `|` and `:` separators cannot be escaped. The
  guide moves both out of the Localisation chapter into Expressions; the
  `LineProvider` seam that does localise is unchanged. Behaviour, the public
  API, and the `.bub` language are untouched.

### Fixed

- `bubbles_copy_utf8` returned `NULL` without setting the thread-local error
  when the bytes it was given contained an interior NUL. `ABI.md` tells a host
  to read `bubbles_last_error()` after a failure, so the host read either
  nothing or a stale message from an earlier call. Reachable from a host
  function that builds its result out of player-entered or externally sourced
  text: the host saw `NULL`, returned `BUBBLES_ERR` with a null result pointer,
  and the dialogue error read "host callback left null result pointer", which
  points at the host's control flow when the problem was its data. The Rust
  docs also only mentioned invalid UTF-8, though the C header listed both.
- The guide described the `NodeStarted` and `NodeComplete` FFI events as
  carrying a `"title"` field; the runtime emits `"node"`. The header also
  claimed `bubbles_runner_set_locale_json` merges into the existing locale
  table, when it replaces it. Both would have misled a host author.
- The guide's API index listed only two of the three shipped saliency
  strategies; `RandomAvailable` is now linked alongside them.
- Deep links into the rustdoc pointed at pages that were never generated.
  Items re-exported at the crate root from public modules are documented only
  at their module path by default, so `bubbles/struct.Runner.html` did not
  exist and all 55 links written in that short form — across the guide, the
  README and the crate docs — returned 404, including 17 of the 18 on the
  guide's API index. The root re-exports now carry `#[doc(inline)]`, which
  makes `bubbles::Runner` the canonical path callers are already told to
  import. No links were rewritten.
- `LICENSE-APACHE` now contains the full Apache License 2.0 text. It previously
  held only the short "how to apply the license" boilerplate notice, so the
  repository referenced Apache-2.0 without shipping its terms, and GitHub's
  licence detection reported `NOASSERTION` ("Other") for the repository. The
  dual `MIT OR Apache-2.0` licensing is unchanged.

## [2.0.0] - 2026-08-21

A cleanup-driven major release. The runtime, FFI layer, and TUI were simplified
with a net reduction of roughly 340 lines; only one public item was removed.

### ⚠ Breaking changes

- **Removed the public `bubbles::compiler::interpolation` module**, including
  `BraceSegment` and `scan_brace_segments`. The module was a leftover from the
  pre-0.1 runtime interpolation path: `{expr}` fragments have been parsed at
  compile time into `TextSegment` since 0.1.0, so nothing in the crate consumed
  it. Brace scanning now lives in the private
  `compiler::markup` / parser internals.

  *Upgrading:* if you called `scan_brace_segments` to find `{…}` spans in raw
  `.bub` text, you were relying on an internal helper that no longer reflects how
  the compiler works. Parse the source with `compile()` and read
  `TextSegment::Expr` from the resulting AST instead.

- **String escapes in `.bub` string literals are resolved differently.** See
  *Fixed* below. A literal such as `"C:\\new"` previously produced a newline;
  it now produces a backslash followed by `n`, as documented. Scripts that
  depended on the old double-unescaping behaviour change output.

`Runner`, `DialogueEvent`, `DialogueOption`, the saliency traits, and every other
public type kept the same signatures across this release.

### Fixed

- **String escapes are resolved in a single pass.** The lexer previously applied
  three chained `replace()` calls, each re-scanning the output of the last, so an
  escaped backslash followed by `n` (for example `"C:\\new"`) was turned into a
  newline instead of a literal `\` + `n`. Escapes `\"`, `\\`, and `\n` are now
  handled in one pass, unknown escapes are preserved verbatim, and the string
  allocates once instead of three times.

### Changed

- The host-facing `bubbles-ffi` C ABI implementation was substantially
  reorganised (error handling, runner state, and JSON conversion). **The exported
  C symbol set is unchanged**, so existing P/Invoke declarations and the Unity
  integration continue to link and work.
- `RunnerBuilder::build` constructs the `Runner` in one step via an internal
  `Runner::with_parts` rather than building and then mutating it.
- `line_id_from_tags` and `option_group_from_tags` share a single internal
  `tag_value` helper.
- The TUI's markup rendering, error overlay, and session handling were
  simplified; the saliency and markup integration tests were tightened.

## [1.0.1] - 2026-06-01

### Fixed

- Dialogue error messages now identify the offending construct and location more
  precisely.
- **Unterminated dialogue blocks are now rejected at compile time.** A node body
  that never closes its block terminator is a compile error instead of parsing
  into something surprising. Scripts that compiled by accident may now fail —
  which is the point, but it is a behaviour change in a patch release.
- `<<once>>` options are exhausted after selection, so a one-shot option cannot
  be chosen twice.
- The TUI skips locked (guard-failed) options when moving focus, instead of
  letting the cursor land on an unselectable row.

### Added

- Documentation for one-shot options in the guide's snippets cookbook.

### Changed

- The guide's validation chapter was aligned with the compiler's actual
  behaviour.

## [1.0.0] - 2026-05-21

The first release with a stability commitment: the public API of
`bubbles-dialogue` is considered stable, and breaking changes from here require a
major version bump.

### ⚠ Breaking changes

Both breaking changes in this release are validation that was previously missing.
No type or function signature changed, so most projects upgrade from 0.8.0 by
bumping the version — but code that relied on the permissive behaviour below will
now see errors.

- **`Runner::select_option` rejects options whose guard is false.** Selecting an
  unavailable option previously executed its body anyway. It now returns
  `DialogueError::ProtocolViolation`.

  *Upgrading:* check `DialogueOption::available` before offering an option to the
  player — which the guide already recommended. UIs that grey out unavailable
  options need no change.

- **`compile` and `compile_many` reject empty input.** An empty or
  whitespace-only source, or a set of sources that yields no nodes at all, now
  returns `DialogueError::Validation` instead of an empty `Program` that failed
  later at `Runner::start`.

  *Upgrading:* guard against empty strings before compiling if your content
  pipeline can legitimately produce them (for example an unwritten localisation
  file).

### Added

- `cargo-fuzz` harnesses covering the compiler, runtime, and the host-facing FFI
  JSON boundaries, plus a scheduled weekly fuzzing workflow that seeds its corpus
  from repository content. See [FUZZING.md](FUZZING.md).
- Guide: a "sharing between runners" section in the storage chapter, and an
  explanation of save/restore behaviour at option prompts.
- Tests that compile every dialogue snippet in the getting-started guide, and a
  test that drains the harbour example to `DialogueComplete`.

### Changed

- `Runner::snapshot` and `Runner::restore` moved into an internal `session`
  module. Both remain available at the same public paths with the same
  signatures.
- `VariableStorage` gained documentation clarifying that cloning a
  `HashMapStorage` does **not** share state with the original.

### Removed

- The PullPact governance configuration and its README references, added earlier
  in the same release cycle and then withdrawn.

## [0.8.0] - 2026-04-24

### Added

- **Option groups.** A `#group:<name>` tag on an option sets
  `DialogueOption::group`, letting a UI enforce constraints such as radio-button
  semantics or mutually exclusive choice sets. Read `group` for constraint logic
  and `tags` for styling. New helper `option_group_from_tags`.

### Changed

- Text formatting made consistent across the documentation and code comments.

### Fixed

- The README feature table rendered incorrectly.

## [0.7.0] - 2026-04-24

### Added

- **Line modes.** `#narration` and `#debug` tags on a line set
  `DialogueEvent::Line::line_mode`, so hosts can filter or route narration and
  debug lines without re-parsing tags. New helper `line_mode_from_tags`.
- **Variable inspection on `Runner`:** `all_variables`, `variable`, and
  `variable_ref`, backed by a new `VariableStorage::all_variables` method.
  Custom storage implementations should override `all_variables`; the default
  implementation returns nothing.

### Changed

- **`Runner::snapshot` and `Runner::restore` are always available.** They were
  previously gated behind the `serde` feature. The `serde` feature is now only
  required to *serialise* a `RunnerSnapshot` (or `Value` / `HashMapStorage`) to
  disk, not to create one. In-memory bookmarks and rewind no longer pull in
  `serde`.

## [0.6.1] - 2026-04-24

### Added

- `Runner::program()` and `Runner::phase()` accessors.
- `scripts/check-version-sync.sh` and a tracked `VERSION` file, keeping the
  workspace version, `VERSION`, and the README quick-start snippet in agreement.
  Wired into CI and the pre-commit hook.

### Fixed

- **`Runner::start` clears pending event and option queues.** Restarting a runner
  (or starting a different node on a runner that had already run) could otherwise
  emit events left over from the previous session.

### Changed

- Documentation clarifies that the crates.io package is `bubbles-dialogue` while
  the Rust import path is `bubbles`.
- Deduplicated an FFI helper and resynchronised the guide's code snippets with
  the `.bub` files they are drawn from.

## [0.6.0] - 2026-04-24

### Added

- **`bubbles-ffi`: a C ABI for Unity and other native hosts.** A `cdylib`
  exposing the compiler and runner across the C boundary with JSON-encoded
  events, suitable for P/Invoke. Covered by a .NET smoke test in CI that runs
  against the release `cdylib`. Not published to crates.io; build it from the
  workspace.
- **Inline markup.** `[b]text[/b]`, `[color value=red]…[/color]`, and
  self-closing `[pause /]` are stripped from the delivered text and returned as
  byte-precise `MarkupSpan`s on lines and options, so a UI can apply styling
  without string surgery. The TUI renders markup spans with terminal styling.
- Guide: a "Unity and native hosts" chapter covering the C ABI, and a note on
  what the FFI surface covers relative to the full Rust API.

### Changed

- The README was rewritten for first-time visitors.
- All inline `#[cfg(test)]` blocks were extracted into sibling `*_tests.rs`
  files, applying one naming rule across the crate.

## [0.5.0] - 2026-04-23

The largest pre-1.0 API cleanup. Several public items were removed outright.

### ⚠ Breaking changes

- **Removed `compile_validated` and `compile_many_validated`.** `compile` and
  `compile_many` now always validate jump and detour targets.

  *Upgrading:* replace `compile_validated(src)` with `compile(src)` and
  `compile_many_validated(srcs)` with `compile_many(srcs)`. If you deliberately
  wanted the old non-validating behaviour, there is no longer an option for it —
  invalid jump targets are a compile error.

- **Removed the `DialogueError::Runtime` and `DialogueError::Type` variants**
  (deprecated earlier in this same release cycle). The structured
  `ProtocolViolation` and `TypeMismatch` variants added in 0.4.0 replace them.

  *Upgrading:* match on `ProtocolViolation` and `TypeMismatch`. Exhaustive
  `match` statements over `DialogueError` must be updated.

- **Removed test-only compiler surface from the public API**, including
  `parse_expr` and the public `tokenise` entry point (`tokenise_strict` was
  renamed).

- **MSRV lowered to 1.94** (from 1.95). Lowering an MSRV is not breaking for
  consumers; it is listed here because the matching `rust-version` and CI floor
  moved.

### Added

- Multi-file compilation in `bubbles-tui`: pass several `.bub` paths and they are
  compiled together with cross-file jump validation.
- `Debug` implementation for `FunctionLibrary`.
- `line_id` and `tags` on the TUI's `DisplayedLine`.
- Guide: a four-page tutorial building a complete NPC scene from scratch, a
  dedicated "Using the TUI" page, and a rewritten snippets cookbook.

### Changed

- `rand` updated from 0.9 to 0.10.
- `FunctionLibrary` built-ins split into `math`, `i18n`, and `rand` submodules;
  the expression evaluator split into `eval/mod`, `binary`, and `unary`.
- `execute_stmt` takes `&Stmt` instead of an owned `Stmt`, removing a clone from
  the hot path.
- The `tavern` example was replaced by the `harbour` showcase, which is the
  example the README and guide now follow.
- The TUI dropped its separate dialogue pane, stacking the transcript and options
  instead, and gained a "rerun" key.

### Fixed

- Out-of-range saliency indices return `ProtocolViolation` instead of panicking.
- Parser `.expect()` panics replaced with `DialogueError::Parse`.
- `compile_validated` is used by the TUI so bad jump targets fail at load time
  rather than mid-scene.
- Several corrections to the `harbour` and `once` examples: a repeated price
  announcement, a repeated greeting on return, an incorrect guard operator, a
  mis-named entry node, and dead code.

### Removed

- `CHANGELOG.md` was deleted in this release, with the rationale "git log is the
  history". Reinstated in October 2026 as the file you are reading.

## [0.4.0] - 2026-04-23

### Added

- **`bubbles-tui`: a ratatui-based terminal UI for writers**, driving the same
  `Runner` a game would use. Shows node markers, speaker lines, option prompts,
  command emissions, and runtime errors, with a scrollable transcript pane, an
  error overlay carrying file/line excerpts, and step-back across session
  history. Run it with
  `cargo run -p bubbles-tui -- file.bub`.
- `compile_validated` and `compile_many_validated`, which validate jump and
  detour targets at compile time. (Folded into `compile` / `compile_many` in
  0.5.0.)
- `RunnerBuilder` for configuring a `Runner`'s saliency strategy, line provider,
  and function library in one expression.
- Structured `DialogueError::ProtocolViolation` and `DialogueError::TypeMismatch`
  variants.
- `line_id` and `tags` preserved on the TUI's `DisplayedOption`.
- A publish workflow triggered by a GitHub release.

### Changed

- **The repository became a Cargo workspace.** `bubbles-dialogue` moved to
  `crates/bubbles-dialogue/`, keeping the same crate name, features, and
  semantics. Scripts and CI switched to workspace-aware invocations.
- A shared brace-scanner now backs `{expr}` interpolation parsing.

### Fixed

- **The lexer surfaces unknown characters as parse errors** instead of silently
  dropping them, so a typo in a `.bub` file is reported rather than ignored.

### Removed

- The `cli_runner` example, superseded by `bubbles-tui`.

> **Attribution note.** The repository's old `CHANGELOG.md` recorded the
> workspace conversion, `bubbles-tui`, and the `cli_runner` removal under 0.5.0.
> The v0.4.0 tag already contains `crates/bubbles-tui` and the workspace layout,
> so they shipped in 0.4.0 and are listed here.

## [0.3.0] - 2026-04-23

### Added

- **`<<stop>>` statement:** ends the dialogue immediately regardless of
  detour/jump stack depth, emitting a single `DialogueEvent::DialogueComplete`.
  Previously this needed a dedicated empty "end" node or a host command.
- **`VariableStorage::get_ref`**, returning `Option<Cow<'_, Value>>`: an
  optional borrow-friendly read path the expression evaluator prefers. The
  default implementation forwards to `get`, so existing storages keep working;
  override it to return `Cow::Borrowed` and `{$text}` interpolation evaluates
  without cloning the string. `HashMapStorage` overrides it.

### ⚠ Breaking changes

- **Every statement-body field in the AST is now `Arc<[Stmt]>`** (re-exported as
  `bubbles::compiler::ast::StmtList`) instead of `Vec<Stmt>` or
  `Arc<Vec<Stmt>>`. This covers `Node::body`, `IfBranch::body`,
  `Stmt::If { else_body }`, `Stmt::Once { body, else_body }`, and
  `OptionItem::body`.

  *Upgrading:* code that constructs AST nodes by hand must wrap statement lists
  with `Arc::from(vec_of_stmt)`. In exchange, frame pushes for `<<if>>`,
  `<<once>>`, options, detours, jumps, and node-group selection bump an `Arc`
  refcount instead of cloning a vector.

### Changed

- `Runner` frames store `{ node: Arc<str>, body: Arc<[Stmt]>, ip: usize }` and
  advance a program counter instead of popping from a `VecDeque<Stmt>`.
- `exec_jump`, `exec_detour`, and `Runner::start` share a new
  `enter_node(target, replace_stack)` helper.
- `Runner` stores visit counts as a plain `HashMap<String, u32>` rather than
  `Arc<Mutex<…>>`. The `visited()` and `visited_count()` built-ins are
  intercepted by the evaluator directly, removing the lock surface and the
  "panics on a poisoned mutex" caveats from `Runner::new`, `start`, `snapshot`,
  and `restore`. User-registered functions named `visited` or `visited_count`
  are now masked by the built-ins; previously they were unreachable for the same
  reason, so observable behaviour is unchanged.

### Fixed

- **Expression parse errors report the enclosing `.bub` file and line.**
  Failures previously surfaced as `<expr>:0` behind a generic wrapper
  (`invalid expression in <<set>>: …`). The parser now threads `(file, line)`
  through and reports the specific reason, such as
  `unexpected end of expression`.

### Removed

- `src/runtime/interpolate.rs`, a dead module holding a duplicate copy of the
  interpolation algorithm used only by its own tests.
- The `first_word` helper in `compiler::parser::command`, in favour of the
  existing `split_first_word`.

## [0.2.0] - 2026-04-23

### Added

- **`plural(count, singular, plural)`** built-in: the singular form when
  `|count| == 1`, the plural otherwise. Usable in any `{expr}`.
- **`select(key, mapping)`** built-in: key-based text dispatch for gendered
  grammar and similar patterns, formatted as
  `"k1:text1|k2:text2|other:fallback"`. The first colon per entry separates key
  from value, so values may contain colons. The `other` key is required.
- `line_id_from_tags()` and `line_id: Option<String>` on `DialogueEvent::Line`
  and `DialogueOption`, populated from a `#line:<id>` tag — a stable key for
  voice-over and localisation without re-parsing `tags`.
- `wasm32-unknown-unknown` clippy checks in CI and `scripts/check-wasm.sh`,
  covering `--no-default-features` with and without `serde`, keeping the library
  WebAssembly-compatible.
- `autoexamples = false` with explicit `[[example]]` entries, so scratch files
  under `examples/` are not swept into `cargo clippy --all-targets`.

### ⚠ Breaking changes

- **Translate-then-format provider ordering.** `LineProvider::get()` is now
  called *before* `{expr}` segments are evaluated, and its returned `String` is
  itself a template whose `{expr}` fragments are evaluated after translation.
  This lets translators reorder interpolations and use `plural()` / `select()`
  inside translated strings. `exec_line_group` and `exec_options` also gained
  provider lookup, which previously only `exec_line` performed.

  *Upgrading:* plain translations without braces are unaffected. A translation
  that contains a literal `{` must now escape it, and providers are consulted in
  more places than before.

- **`RunnerSnapshot::visits` changed from `HashMap<String, usize>` to
  `HashMap<String, u32>`.** Snapshots serialised as JSON are unaffected on
  64-bit targets because JSON numbers are untyped; binary formats may need
  migration.

- **`DialogueEvent::Line` and `DialogueOption` gained a `line_id` field.**
  Struct literals and exhaustive patterns must be updated, or use `..`.

### Changed

- The `rand` built-ins `random_range` and `dice` validate their arguments.
  Non-finite, fractional, or out-of-range values produce a
  `DialogueError::Function` instead of silently wrapping.

### Fixed

- All `#[allow(clippy::…)]` suppressions were removed, with the underlying code
  restructured so none is needed.

## [0.1.0] - 2026-04-22

Initial release.

### Added

- **Compiler:** `compile` / `compile_many` producing a `Program` with a node map
  and merge-time duplicate-title detection; `validate()` for jump and detour
  targets across merged programs.
- **Runtime:** `Runner` with a pull-based `next_event()` / `select_option()` API
  and a call stack for detours. No async, no global state.
- **Events:** `DialogueEvent::NodeStarted`, `Line`, `Options`, `Command`,
  `NodeComplete`, `DialogueComplete`; `DialogueOption` with a guard-driven
  `available` flag.
- **`.bub` syntax:** nodes (`title:` / `---` / `===`), lines, `Speaker:`
  prefixes, shortcut options (`->`), line groups (`=>`), node groups (`when:`),
  `{expr}` interpolation, and `#tag` / `#line:<id>` metadata on lines and
  commands.
- **Statements:** `<<if>>` / `<<elseif>>` / `<<else>>` / `<<endif>>`, `<<set>>`
  (including the `to` form), `<<declare>>` smart variables, `<<jump>>`,
  `<<detour>>`, `<<return>>`, `<<once>>` / `<<once if expr>>` / `<<endonce>>`
  with an optional `<<else>>`, and generic `<<command>>` events with arguments.
- **Values and storage:** `Value` (`Number`, `Text`, `Bool`),
  the `VariableStorage` trait, and `HashMapStorage`.
- **Functions:** `FunctionLibrary` with `round`, `floor`, `ceil`, `min`, `max`,
  `abs`, `clamp`, `string`, `int`, `visited`, `visited_count`, and — with the
  `rand` feature — `random`, `random_range`, `dice`. Host closures can be
  registered and called from any expression.
- **Saliency:** the `SaliencyStrategy` trait with `FirstAvailable`,
  `RandomAvailable`, and `BestLeastRecentlyViewed` implementations, or your own.
- **Localisation:** the `LineProvider` trait with `PassthroughProvider` and
  `HashMapProvider`.
- **Introspection:** `node_exists`, `node_titles`, `node_tags`, and
  `variable_declarations` / `VariableDecl`.
- **Features:** `serde` (`Value`, `HashMapStorage`, `RunnerSnapshot`, plus
  `Runner::snapshot` / `restore`), `rand` (random built-ins and
  `RandomAvailable`), and `full` as shorthand for both.
- Parse-time expression checks for control statements and assignments; runtime
  errors for division and modulo by zero.
- Examples: `cli_runner` and `tavern`. Broad integration tests plus
  property-based tests over the expression evaluator.

### Changed

- `{expr}` fragments in line text, option text, line-group text, and command
  arguments are parsed **at compile time** into
  `TextSegment::Literal` / `TextSegment::Expr(Arc<Expr>)`, and evaluated in a
  single pass at runtime with no re-parsing. Guards and `when:` conditions
  likewise build a shared `Arc<Expr>` at compile time instead of re-parsing on
  every evaluation.
- MSRV set to 1.94.

### Fixed

- `<<once if …>>` conditions are recognised; the parser strips the `once` prefix
  before reading `if`.
- Parse errors in `<<if>>`, `<<once if>>`, and missing `---`, `===`, or `title:`
  headers report the real source line number instead of an internal buffer index
  that drifted past blank lines and comments.

[Unreleased]: https://github.com/ViezeVingertjes/bubbles/compare/v2.0.0...HEAD
[2.0.0]: https://github.com/ViezeVingertjes/bubbles/compare/v1.0.1...v2.0.0
[1.0.1]: https://github.com/ViezeVingertjes/bubbles/compare/v1.0.0...v1.0.1
[1.0.0]: https://github.com/ViezeVingertjes/bubbles/compare/v0.8.0...v1.0.0
[0.8.0]: https://github.com/ViezeVingertjes/bubbles/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/ViezeVingertjes/bubbles/compare/v0.6.1...v0.7.0
[0.6.1]: https://github.com/ViezeVingertjes/bubbles/compare/v0.6.0...v0.6.1
[0.6.0]: https://github.com/ViezeVingertjes/bubbles/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/ViezeVingertjes/bubbles/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/ViezeVingertjes/bubbles/releases/tag/v0.4.0
[0.3.0]: https://crates.io/crates/bubbles-dialogue/0.3.0
[0.2.0]: https://crates.io/crates/bubbles-dialogue/0.2.0
[0.1.0]: https://crates.io/crates/bubbles-dialogue/0.1.0
