#!/usr/bin/env bash
# Measures the shipped footprint of bubbles-dialogue: the size of a minimal
# WebAssembly module that drives a dialogue, and the size of a native release
# binary. Prints a table and, with --check, fails if an artifact is outside its
# size range.
#
# Why a throwaway crate rather than `cargo build -p bubbles-dialogue`: an rlib is
# not a loadable module and its size says nothing about what a host ships. The
# only honest number is a linked, optimised artifact, so this builds one.
#
# Usage:
#   scripts/measure-footprint.sh            # print the table
#   scripts/measure-footprint.sh --check    # also enforce the ranges below
set -euo pipefail
cd "$(dirname "$0")/.."
REPO="$PWD"

# Budgets are deliberately loose. Module size moves with the Rust toolchain and
# with transitive dependency releases, neither of which is under our control, so
# a tight budget would fail for reasons that have nothing to do with this
# library. These are set to roughly 1.3x the measured values: tight enough to
# catch a dependency being pulled in by accident, loose enough not to fire on a
# compiler upgrade.
BUDGET_WASM_MINIMAL_KIB=235
BUDGET_WASM_SERDE_KIB=300
BUDGET_WASM_DEFAULT_KIB=265
BUDGET_TUI_BINARY_KIB=1850

# Floors, at roughly 0.85x the measured values. A module that suddenly gets
# much smaller has usually lost code it needed rather than got leaner: if
# anything on the probe's path diverges unconditionally on the target - a
# platform intrinsic that panics, say - the optimiser proves everything after
# it unreachable and deletes the library from the artifact. The size check
# passes, because it only looks upward. Lowering a floor is a deliberate
# decision to record a real reduction, which is the review this is asking for.
FLOOR_WASM_MINIMAL_KIB=154
FLOOR_WASM_SERDE_KIB=158
FLOOR_WASM_DEFAULT_KIB=170
FLOOR_TUI_BINARY_KIB=1200

CHECK=0
[[ "${1:-}" == "--check" ]] && CHECK=1

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

FAILED=0

# Reports size in KiB, raw and gzip -9, and compares against a budget and a floor.
report() {
  local label="$1" file="$2" budget="$3" floor="$4"
  local raw gz
  raw=$(( ( $(stat -c %s "$file") + 1023 ) / 1024 ))
  gz=$(( ( $(gzip -9 -c "$file" | wc -c) + 1023 ) / 1024 ))
  printf '%-34s %6s KiB %8s KiB gzip   (%s-%s KiB)\n' "$label" "$raw" "$gz" "$floor" "$budget"
  if [[ "$CHECK" -eq 1 && "$raw" -gt "$budget" ]]; then
    echo "OVER BUDGET: $label is ${raw} KiB, budget is ${budget} KiB" >&2
    FAILED=1
  fi
  if [[ "$CHECK" -eq 1 && "$raw" -lt "$floor" ]]; then
    echo "UNDER FLOOR: $label is ${raw} KiB, floor is ${floor} KiB." >&2
    echo "  A module this much smaller has usually lost code it needed. Check that" >&2
    echo "  nothing on the probe's path panics on the target before lowering it." >&2
    FAILED=1
  fi
}

# Builds a cdylib from scripts/footprint-probe.rs and echoes the artifact path.
#
#   $1 name            package name
#   $2 deps            full [dependencies] body
#   $3 probe_features   full [features] body, may be empty
#   $4 cargo_features   value for --features, may be empty
#   $5 rustflags        extra RUSTFLAGS, may be empty
build_wasm() {
  local name="$1" deps="$2" probe_features="$3" cargo_features="$4" rustflags="${5:-}"
  local dir="$WORK/$name"
  mkdir -p "$dir/src"

  {
    cat <<TOML
[package]
name = "$name"
version = "0.0.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
$deps
TOML
    # Always declare the feature the probe gates `save` on, even for variants
    # that leave it off, so cargo does not warn about an unexpected cfg value.
    printf '\n[features]\n%s\n' "${probe_features:-serde = []}"
    # The profile a host shipping to a browser would actually use. A stock
    # release build would overstate the footprint several times over and would
    # not reflect anyone's deployment.
    cat <<'TOML'

[profile.release]
opt-level     = "z"
lto           = "fat"
codegen-units = 1
panic         = "abort"
strip         = true

[workspace]
TOML
  } > "$dir/Cargo.toml"

  cp "$REPO/rust-toolchain.toml" "$dir/rust-toolchain.toml"
  cp "$REPO/scripts/footprint-probe.rs" "$dir/src/lib.rs"

  (
    cd "$dir"
    local args=(build --release --target wasm32-unknown-unknown --quiet)
    [[ -n "$cargo_features" ]] && args+=(--features "$cargo_features")
    if [[ -n "$rustflags" ]]; then
      RUSTFLAGS="$rustflags" cargo "${args[@]}"
    else
      cargo "${args[@]}"
    fi
  ) >&2
  echo "$dir/target/wasm32-unknown-unknown/release/$name.wasm"
}

rustup target add wasm32-unknown-unknown >/dev/null 2>&1 || true

echo "toolchain: $(rustc -V)"
echo "host:      $(uname -srm)"
echo

BUBBLES="bubbles-dialogue = { path = \"$REPO/crates/bubbles-dialogue\", default-features = false"

MINIMAL=$(build_wasm wasm_minimal "$BUBBLES }" "" "")
report "wasm, no default features" "$MINIMAL" "$BUDGET_WASM_MINIMAL_KIB" "$FLOOR_WASM_MINIMAL_KIB"

# The serde feature only adds derive impls, which the linker strips unless a host
# serialises something - so the probe's `save` entry point is compiled in here
# and the figure includes serde_json. That makes this the cost of save/load for a
# browser host, not the cost of the feature flag on its own.
SERDE=$(build_wasm wasm_serde \
  "$BUBBLES, features = [\"serde\"] }
serde = { version = \"1\", features = [\"derive\"], optional = true }
serde_json = { version = \"1\", optional = true }" \
  'serde = ["dep:serde", "dep:serde_json"]' \
  'serde' )
report "wasm, serde + serde_json" "$SERDE" "$BUDGET_WASM_SERDE_KIB" "$FLOOR_WASM_SERDE_KIB"

# The `rand` feature needs a getrandom backend on wasm32-unknown-unknown: the
# crate feature alone is not enough, the cfg has to be set too. Building it here
# is what keeps the instructions in book/src/advanced/wasm.md honest.
DEFAULT=$(build_wasm wasm_default \
  "$BUBBLES, features = [\"rand\"] }
getrandom = { version = \"0.4\", features = [\"wasm_js\"] }" \
  "" "" '--cfg getrandom_backend="wasm_js"')
report "wasm, default (rand)" "$DEFAULT" "$BUDGET_WASM_DEFAULT_KIB" "$FLOOR_WASM_DEFAULT_KIB"

echo
cargo build --release --quiet -p bubbles-tui >&2
report "bubbles-tui native binary" "target/release/bubbles-tui" "$BUDGET_TUI_BINARY_KIB" "$FLOOR_TUI_BINARY_KIB"

echo
if [[ "$CHECK" -eq 1 ]]; then
  [[ "$FAILED" -eq 0 ]] && echo "Footprint check passed." || exit 1
else
  echo "Reported sizes only; pass --check to enforce budgets."
fi
