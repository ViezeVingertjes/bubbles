# Contributing to bubbles

Contributions of every size are welcome: typo fixes, documentation, tests, bug
reports, and code. You do not need to be an experienced Rust developer to help
here, and you do not need permission to start.

## Where to start

- **Issues labelled [`good first issue`](https://github.com/ViezeVingertjes/bubbles/labels/good%20first%20issue)**
  are scoped to be finishable in an evening and name the file to edit.
  [`help wanted`](https://github.com/ViezeVingertjes/bubbles/labels/help%20wanted)
  issues are larger but still self-contained.
- **Found a bug?** Open a [bug report](https://github.com/ViezeVingertjes/bubbles/issues/new/choose).
  A minimal `.bub` script that reproduces it is the most useful thing you can
  provide.
- **Spotted a confusing sentence in the [guide](https://viezevingertjes.github.io/bubbles/)?**
  That is a real contribution. The guide lives in `book/src/`; edit the Markdown
  and open a pull request. No test required.
- **Want a feature?** Open a
  [feature request](https://github.com/ViezeVingertjes/bubbles/issues/new/choose)
  before writing code, so we can agree on the shape before you invest the time.

Small pull requests are easier to review and land faster than large ones. If you
are unsure whether something is worth doing, open an issue and ask. A short
question is never a nuisance.

## How much process applies to your change?

Not every change needs the full checklist. Find your row:

| Your change | What is expected |
|---|---|
| Typo, wording, broken link, guide edit | Just the fix. CI runs `cargo fmt` and the doc build. |
| New or corrected test | The test. |
| Bug fix | A test that fails before your fix and passes after. |
| New feature or public API | A test, doc comments on new public items, and a changelog entry. |

Everything below is reference material for the last two rows.

## Development setup

Use **Rust 1.94 or newer** (matching `rust-version` in `Cargo.toml`). The
repository pins `stable` in `rust-toolchain.toml`, so `rustup` installs a
matching toolchain for you.

```bash
git clone https://github.com/ViezeVingertjes/bubbles
cd bubbles
cargo test --all-features
```

To see a dialogue actually run, play one of the examples in the terminal UI:

```bash
cargo run -p bubbles-cli -- play examples/harbour/harbour.bub examples/harbour/services.bub
```

### Optional: the pre-commit hook

```bash
bash scripts/install-hooks.sh
```

The hook runs most of the gates before each commit: version sync, formatting,
clippy (with and without default features), rustdoc, and file sizes. That
catches a red CI run before you push it. It does not run the rest of CI: the
test suite, the WebAssembly build, the sign-off check, the dependency audit,
the .NET FFI smoke test, and the MSRV build. A clean commit is still not a
promise of green CI. It needs **`jq`** on your `PATH`, and it is genuinely
optional: if you would rather let CI tell you, that is fine.

## Sign your commits (DCO)

This project uses the [Developer Certificate of Origin](DCO). Every commit needs
a sign-off trailer:

```
Signed-off-by: Your Name <your.email@example.com>
```

Git adds it for you when you pass `-s`:

```bash
git commit -s -m "fix(runtime): once block fires twice from a detour"
```

The address must match your commit author address. A PR check verifies this and
tells you how to fix it if it is missing. You can also check before pushing:

```bash
bash scripts/check-dco.sh
```

If you forget, nothing is lost: amend the last commit with
`git commit --amend -s --no-edit`, or sign off a whole branch with
`git rebase --signoff origin/main`, then push with `--force-with-lease`.

**What you are certifying** is that you wrote the patch, or otherwise have the
right to contribute it under this project's MIT OR Apache-2.0 licence. It is not
a copyright assignment, there is no separate agreement to sign, and we do not
collect anything beyond the name and email already in your commit.

## Commit style

We use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat(parser): support <<detour>> inside option bodies
fix(runtime): once block fires twice when called from detour
docs: add doc tests for Program::variable_declarations
```

Types in use: `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `chore`, `ci`.
If you get the prefix wrong, a reviewer will suggest the right one. It is not a
reason to reject a change.

## The quality gates

These ten run in CI and need nothing beyond your Rust toolchain, so you can run
any of them locally. (`check-version-sync.sh` also wants **`jq`** on your
`PATH`; `check-wasm.sh` installs the `wasm32-unknown-unknown` target itself.)

| Gate | Command |
|---|---|
| Release strings in sync | `bash scripts/check-version-sync.sh` |
| Formatting | `cargo fmt --all --check` |
| Lints | `cargo clippy --workspace --all-features --all-targets -- -D warnings` |
| Tests | `cargo test --workspace --all-features` |
| Tests, no default features | `cargo test -p bubbles-dialogue --no-default-features` |
| Checker without the terminal UI | `cargo clippy -p bubbles-cli --no-default-features --all-targets -- -D warnings && cargo test -p bubbles-cli --no-default-features` |
| WebAssembly build | `bash scripts/check-wasm.sh` |
| Doc warnings | `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps` |
| File size | `bash scripts/check-file-sizes.sh` |
| Sign-off | `bash scripts/check-dco.sh` |

Four more gates run in CI and each need something the rest do not. **You are not
expected to run these before opening a pull request**, since CI is the normal
place to find out. If one of them goes red on your PR, this is where to look:

| Gate | Where | Running it yourself |
|---|---|---|
| Publish dry-runs | `Publish dry-run` steps in `ci.yml` | `cargo publish --dry-run -p bubbles-ffi` and `-p bubbles-cli`. Both reach the crates.io index, so they need network access. They catch missing crate metadata and path dependencies without a version. |
| Dependency audit | `cargo-deny` step in `ci.yml` | `cargo install cargo-deny`, then `cargo deny check`. Licences and advisories are configured in `deny.toml`. |
| .NET FFI smoke test | `bubbles-ffi P/Invoke smoke` step in `ci.yml` | Needs the .NET 10 SDK. `cargo build -p bubbles-ffi --release`, then run `dotnet run -c Release` in `crates/bubbles-ffi/tests/dotnet_smoke/` with `LD_LIBRARY_PATH` pointing at `target/release`. CI runs it on Linux only, so the exact invocation differs on macOS and Windows. |
| MSRV build | separate `msrv` job in `ci.yml` | `rustup toolchain install 1.94`, then `cargo +1.94 check --workspace --all-features`. This catches code that compiles on current stable but not on our oldest supported Rust. |

A red CI run is normal and is not a verdict on your work. Push a fix-up commit;
nobody is counting.

### About clippy

`clippy::pedantic` and `clippy::nursery` are enabled workspace-wide in
`Cargo.toml`, so you get exactly the same warnings locally that CI will show
you. These lint sets are opinionated and occasionally wrong.

**If a lint is telling you to make the code worse, say so in the pull request.**
We would rather discuss one lint than have you contort a clean patch around it.
Adding a scoped `#[allow(...)]` with a comment explaining why is a legitimate
outcome, but ask first rather than scattering them. The few already in `src/`
are all narrow: two float-to-integer casts in the random built-ins, each
followed by a checked conversion; `multiple_crate_versions` at the top of the
TUI crate; and one unreachable match arm that keeps the C ABI
forward-compatible.

### About the 300-line file cap

`scripts/check-file-sizes.sh` fails if any file matching `crates/*/src/**/*.rs`
exceeds 300 non-blank lines. **Integration tests under `crates/*/tests/` are not
covered**, so write as much test code there as the change needs. Unit-test
files do count, because they live beside the code in `src/` (for example
`src/library/library_tests/`), so those get split into submodules as they grow.

It is a prompt to split a file along a concept boundary, not a quality bar you
have to hit on the first try. If you are near the limit and the natural split
is not obvious, open the PR anyway and ask; deciding where a module should be cut
is a reasonable thing to want a second opinion on.

The conventions behind it:

- One concept per file. No `utils.rs`, `helpers.rs`, or `common.rs` catch-alls in
  `src/`. (`tests/common/mod.rs` is a deliberate exception: it is the shared
  integration-test harness.)
- `mod.rs` files stay thin: declare submodules, re-export the public surface.

### About tests

New behaviour needs a test. Integration tests live in
`crates/bubbles-dialogue/tests/` and drive the public API the way a game would.

Writing the failing test first is a habit worth having, since it proves the
test actually exercises the bug, but we care about the test existing and failing
without your fix, not the order you typed things in. If you have a fix and are
stuck on how to test it, open a draft PR and ask.

Every new public item needs a doc comment (CI fails on missing docs). Doc tests
are welcome where an example helps.

### Changelog

User-visible changes get an entry under `## [Unreleased]` in
[CHANGELOG.md](CHANGELOG.md), in the matching `Added` / `Changed` / `Fixed` /
`Removed` section. Internal refactors that change nothing for a user do not need
one. If you are unsure, skip it and a reviewer will tell you.

## Fuzzing

Fuzz targets covering the compiler, runtime, and FFI JSON boundaries live in
`fuzz/fuzz_targets/`. See [FUZZING.md](FUZZING.md).

**Fuzzing is not expected on pull requests.** A scheduled workflow runs it
weekly. If you are making a large parser or runtime change, a short local session
is a nice extra:

```bash
cargo +nightly fuzz run compile_bub fuzz/corpus/compile_bub -- -max_total_time=60
```

## Pull requests

All changes reach `main` through a pull request. Open it as a **draft** if you
want early feedback on an unfinished change. That is encouraged, not a sign of
weakness.

- [ ] Commits are signed off (`git commit -s`)
- [ ] CI is green, or you have said what you are stuck on
- [ ] New behaviour has a test in `crates/bubbles-dialogue/tests/`
- [ ] New public items have doc comments
- [ ] User-visible changes have a `CHANGELOG.md` entry under `[Unreleased]`

Review is a conversation, not a gate you have to pass alone. Expect questions,
and feel free to push back on review comments you disagree with.

## Reporting bugs

Please include:

- The `.bub` source that triggers it, as small as you can make it
- What you expected, and what happened instead
- Output of `cargo --version` and `rustc --version`
- The `bubbles-dialogue` version you are using

## Security

Do not report security vulnerabilities in a public issue. See
[SECURITY.md](SECURITY.md) for the private reporting channel.

## Code of Conduct

This project follows the [Contributor Covenant](CODE_OF_CONDUCT.md). Be
respectful and constructive; assume the person on the other end is trying to
help.
