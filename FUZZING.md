# Fuzzing

Bubbles uses [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz) (libFuzzer) to test
the `.bub` compiler, the markup and interpolation scanners, the runtime event loop, and
the host-facing JSON/state boundaries in `bubbles-ffi`.

All fuzz targets live in `fuzz/fuzz_targets/`. Seed corpora are in `fuzz/corpus/<target>/`.

## Requirements

- **nightly Rust** — libFuzzer instrumentation requires nightly. cargo-fuzz handles
  this automatically when called with `+nightly`.
- **cargo-fuzz**

```bash
cargo install cargo-fuzz
```

## Running a target

```bash
cargo +nightly fuzz run <target> [corpus_dir] [-- libfuzzer_flags]
```

### Targets

| Target | What it covers |
|---|---|
| `compile_bub` | Full `.bub` pipeline: lexer → parser → AST → Program → validate |
| `compile_many_bub` | Multi-file compilation: duplicate nodes, cross-file jump/detour resolution |
| `lexer_expr` | Expression lexer (`tokenise`) and recursive-descent expression parser |
| `markup_text` | Markup scanner (`scan_text_segments`) and brace interpolation (`scan_brace_segments`) |
| `runtime_bounded` | Bounded compile → Runner → event loop (max 64 events, always picks option 0) |
| `serde_state_json` | `Value`, `HashMapStorage`, and `RunnerSnapshot` serde deserialization |
| `ffi_public_json` | Public C ABI JSON entry points in `bubbles-ffi` (variable set, locale, save/load) |

### Examples

Run the compiler target indefinitely from the seed corpus:

```bash
cargo +nightly fuzz run compile_bub fuzz/corpus/compile_bub
```

Run the expression parser target with a 2 KB input cap (recommended for the recursive parser):

```bash
cargo +nightly fuzz run lexer_expr fuzz/corpus/lexer_expr -- -max_len=2048
```

Run a quick 30-second smoke session:

```bash
cargo +nightly fuzz run markup_text fuzz/corpus/markup_text -- -max_total_time=30
```

```bash
for t in compile_bub compile_many_bub lexer_expr markup_text runtime_bounded serde_state_json ffi_public_json; do
  cargo +nightly fuzz run "$t" "fuzz/corpus/$t" -- -max_total_time=60
done
```

## Reproducing a crash

cargo-fuzz writes crash inputs to `fuzz/artifacts/<target>/`. To reproduce:

```bash
cargo +nightly fuzz run <target> fuzz/artifacts/<target>/crash-<hash>
```

## Minimizing a crash input

```bash
cargo +nightly fuzz tmin <target> fuzz/artifacts/<target>/crash-<hash>
```

## Input size guidelines

These limits balance thoroughness with runtime speed. Adjust after triaging crashes.

| Target | Recommended `max_len` |
|---|---|
| `compile_bub` | 8192 |
| `compile_many_bub` | 8192 |
| `lexer_expr` | 2048 |
| `markup_text` | 4096 |
| `runtime_bounded` | 4096 |
| `serde_state_json` | 4096 |
| `ffi_public_json` | 4096 |

## CI

Fuzzing is not part of the required PR gate. A scheduled workflow runs weekly
and can also be triggered manually via GitHub Actions:

- **Schedule** — every Monday at 02:00 UTC (`.github/workflows/fuzz.yml`).
- **Manual** — `Actions → Fuzz → Run workflow`, with an optional `duration`
  input (seconds per target, default 300).

Each target runs in a separate matrix job. On failure the crash input is
uploaded as a `fuzz-crash-<target>` artifact.

To run a quick smoke session locally before a release:

## Corpus

`fuzz/corpus/` is gitignored. The CI workflow seeds each target's corpus
directory from existing repo content before fuzzing starts:

| Targets | Seeds from |
|---|---|
| `compile_bub`, `compile_many_bub`, `runtime_bounded` | `tests/fixtures/*.bub`, `examples/**/*.bub` |
| `lexer_expr` | Inline expression strings written by the workflow |
| `markup_text` | Inline markup/interpolation strings written by the workflow |
| `serde_state_json`, `ffi_public_json` | Inline JSON values and maps written by the workflow |

For local runs, populate the corpus the same way before the first session:

```bash
bash .github/workflows/fuzz.yml  # not executable — copy the "Seed corpus" step manually, or:
cargo +nightly fuzz run compile_bub fuzz/corpus/compile_bub  # libFuzzer starts from scratch if empty
```

After a long local run, save any interesting minimized inputs under
`fuzz/corpus/<target>/` and add them to the seeding step if they cover a new
code path worth preserving across machines.
