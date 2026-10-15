#!/usr/bin/env bash
# Ensure the bubbles-dialogue library stays usable on wasm32-unknown-unknown.
# We use --lib only: dev-dependencies (e.g. proptest) are not wasm-clean, and
# the bubbles-cli binary deliberately targets native terminals, not wasm.
#
# Compiling is not enough. A call that diverges on this target - `SystemTime`
# is the obvious one, since std lowers it to an `unreachable` here - compiles
# cleanly, and the optimiser is then entitled to delete everything downstream
# of it. The build goes green while the module traps on the first line of
# dialogue, and the artifact gets *smaller*, so a size check does not catch it
# either. After the lint pass we therefore instantiate a real module and play a
# script inside it.
set -euo pipefail
cd "$(dirname "$0")/.."
repo="$PWD"

rustup target add wasm32-unknown-unknown

cargo clippy -p bubbles-dialogue --target wasm32-unknown-unknown --no-default-features --lib -- -D warnings
cargo clippy -p bubbles-dialogue --target wasm32-unknown-unknown --no-default-features --features serde --lib -- -D warnings

# The default feature set is deliberately absent, and cannot be added here.
# `rand` reaches `getrandom`, which refuses to build for wasm32-unknown-unknown
# until a backend is selected, and selecting one is the final binary's job: it
# needs `getrandom`'s `wasm_js` feature *and* a matching `getrandom_backend` cfg,
# neither of which a library in the middle of the graph can turn on for its
# dependent. Checking `--all-features` from here fails on getrandom's own
# compile_error, which says nothing about this crate.
#
# That is why the default feature set on wasm is covered by
# scripts/measure-footprint.sh instead: it builds a real cdylib host that
# supplies the backend, which is the configuration a browser host actually ships
# and the one the guide documents.

if ! command -v node >/dev/null 2>&1; then
    echo "check-wasm: node is needed to run the wasm smoke test" >&2
    exit 1
fi

# The probe lives in a temp directory rather than the repository: it is a
# fixture for this gate, not something anyone should depend on or maintain.
probe="$(mktemp -d)"
trap 'rm -rf "$probe"' EXIT
mkdir -p "$probe/src"
cp rust-toolchain.toml "$probe/"

cat >"$probe/Cargo.toml" <<EOF
[package]
name = "wasm-smoke"
version = "0.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
bubbles = { package = "bubbles-dialogue", path = "$repo/crates/bubbles-dialogue", default-features = false, features = ["serde"] }
serde_json = "1"

[profile.release]
opt-level = "z"
panic = "abort"

[workspace]
EOF

cat >"$probe/src/lib.rs" <<'PROBE'
use bubbles::{HashMapStorage, RunnerBuilder, RunnerSnapshot, compile};

const SRC: &str = "title: A\n---\n<<set $n = 1>>\nhello {$n}\n===\n";

/// Plays a script to the end and round-trips the save, returning the number of
/// events. A module that traps, or that quietly stops producing events, cannot
/// return a plausible number.
#[no_mangle]
pub extern "C" fn play() -> u32 {
    let Ok(program) = compile(SRC) else { return 0 };
    let mut runner = RunnerBuilder::new(program, HashMapStorage::new())
        .with_seed(7)
        .build();
    if runner.start("A").is_err() {
        return 0;
    }
    let mut events = 0;
    while let Ok(Some(_)) = runner.next_event() {
        events += 1;
        if events > 64 {
            break;
        }
    }
    let json = match serde_json::to_string(&runner.snapshot()) {
        Ok(json) => json,
        Err(_) => return 0,
    };
    if serde_json::from_str::<RunnerSnapshot>(&json).is_err() {
        return 0;
    }
    events
}
PROBE

cat >"$probe/run.mjs" <<'RUNNER'
import { readFileSync } from 'node:fs';

const mod = await WebAssembly.compile(readFileSync(process.argv[2]));

const imports = WebAssembly.Module.imports(mod);
if (imports.length > 0) {
  console.error(`wasm smoke: module expects host imports: ${JSON.stringify(imports)}`);
  process.exit(1);
}

const instance = await WebAssembly.instantiate(mod, {});
let events;
try {
  events = instance.exports.play();
} catch (err) {
  console.error(`wasm smoke: trapped while playing a script: ${err}`);
  console.error('A call that diverges on this target compiles cleanly and traps');
  console.error('at runtime. SystemTime is the usual one. Find it before merging.');
  process.exit(1);
}

if (events < 2) {
  console.error(`wasm smoke: expected a played script, got ${events} events`);
  process.exit(1);
}
console.log(`wasm smoke: played a script and round-tripped a save, ${events} events`);
RUNNER

cargo build --manifest-path "$probe/Cargo.toml" --target wasm32-unknown-unknown --release
node "$probe/run.mjs" "$probe/target/wasm32-unknown-unknown/release/wasm_smoke.wasm"
