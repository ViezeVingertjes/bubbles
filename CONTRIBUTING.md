# Contributing to bubbles

Thank you for your interest! Contributions of all kinds are welcome - bug reports, documentation improvements, new tests, and pull requests.

## Before you start

- Check the [issue tracker](https://github.com/ViezeVingertjes/bubbles/issues) to see if your topic is already being discussed.
- For large changes (new features, architectural refactors) please open an issue first so we can align before you invest time coding.

## Development setup

Use **Rust 1.94+** (matches `rust-version` in `Cargo.toml`). The repository pins `stable` in
`rust-toolchain.toml` so `rustup` can install a matching toolchain automatically.

```bash
git clone https://github.com/ViezeVingertjes/bubbles
cd bubbles
cargo build --all-features
cargo test --all-features
```

Install the pre-commit hook so local quality gates match CI:

```bash
bash scripts/install-hooks.sh
```

The hook runs `cargo fmt --check`, the same `cargo clippy` invocations as CI, `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps`, and `scripts/check-file-sizes.sh` before every commit.

## Commit style

We use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat(parser): support <<detour>> inside option bodies
fix(runtime): once block fires twice when called from detour
docs: add doc tests for Program::variable_declarations
chore: bump rand to 0.9
```

Types: `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `chore`.

## Code quality expectations

| Gate | Tool | Target |
|---|---|---|
| Format | `cargo fmt` | no diff |
| Lints | `cargo clippy --all-features -D warnings` | zero warnings |
| Tests | `cargo test --all-features` | all pass |
| Doc warnings | `cargo doc --no-deps -D warnings` | zero warnings |
| File size | `scripts/check-file-sizes.sh` | ≤ 300 non-blank LOC per file |

All of these are enforced in CI and by the pre-commit hook.

## Adding a feature

1. Write a failing integration test in `tests/` first (red).
2. Implement the minimal code to pass it (green).
3. Clean it up before you open the PR - one concept per file, no tangled helpers.
4. Run all gates locally, then open a PR.
5. Every new public item needs a doc comment; doc tests are strongly encouraged.

## File size discipline

- Hard cap: **300 non-blank LOC per file** (enforced by
  `scripts/check-file-sizes.sh` in CI and the pre-commit hook).
- Aim for ~250 LOC; hitting 300 is a refactor signal.
- One concept per file. No `utils.rs`, `helpers.rs`, or `common.rs` catch-alls
  in `src/`. (`tests/common/mod.rs` is the one intentional exception: an
  integration-test harness shared across binaries.)
- `mod.rs` files are thin: declare submodules and re-export the public surface only.

## Pull request checklist

- [ ] All CI gates pass
- [ ] New behaviour has integration test(s) in `tests/`
- [ ] New public items have doc comments (and doc tests where sensible)
- [ ] No file exceeds ~300 LOC
## Reporting bugs

Please include:
- The `.bub` source that triggers the bug (minimal reproduction)
- Expected vs actual behaviour
- `cargo --version` and `rustc --version`

## Code of Conduct

This project follows the [Contributor Covenant](CODE_OF_CONDUCT.md).
All participants are expected to be respectful and constructive.
