#!/usr/bin/env bash
# Ensure the bubbles-dialogue library stays buildable on wasm32-unknown-unknown.
# We use --lib only: dev-dependencies (e.g. proptest) are not wasm-clean, and
# the bubbles-tui binary deliberately targets native terminals, not wasm.
set -euo pipefail
cd "$(dirname "$0")/.."

rustup target add wasm32-unknown-unknown

cargo clippy -p bubbles-dialogue --target wasm32-unknown-unknown --no-default-features --lib -- -D warnings
cargo clippy -p bubbles-dialogue --target wasm32-unknown-unknown --no-default-features --features serde --lib -- -D warnings
