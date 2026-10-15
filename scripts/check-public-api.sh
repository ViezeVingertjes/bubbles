#!/usr/bin/env bash
# The public API of bubbles-dialogue is a contract, so it is checked in:
# crates/bubbles-dialogue/public-api.txt lists every public entry. This
# regenerates that listing and fails if it differs, which means growing or
# changing the surface takes a deliberate edit to the file, in the same pull
# request, where it is visible in review.
#
# Half the v2.0.0 surface was published by accident - four compiler modules that
# existed only because the fuzz harness is a separate crate - and nothing would
# have told us. This is what tells us.
#
# When the diff is intentional: run this script, copy the generated listing over
# the checked-in one, and say in the commit message why the surface moved.
#
# rustdoc JSON is nightly-only and its format changes, so the toolchain and the
# tool are both pinned. Bumping them is a deliberate commit that regenerates the
# listing in the same breath.
set -euo pipefail
cd "$(dirname "$0")/.."

TOOLCHAIN="${PUBLIC_API_TOOLCHAIN:-nightly-2026-10-15}"
BASELINE="crates/bubbles-dialogue/public-api.txt"

if ! rustup toolchain list | grep -q "^${TOOLCHAIN}"; then
    echo "check-public-api: $TOOLCHAIN is not installed (rustup toolchain install $TOOLCHAIN)" >&2
    exit 1
fi

if ! command -v cargo-public-api >/dev/null 2>&1; then
    echo "check-public-api: cargo-public-api is not installed (cargo install cargo-public-api)" >&2
    exit 1
fi

actual="$(mktemp)"
trap 'rm -f "$actual"' EXIT

# `full`, not `--all-features`: the `fuzzing` feature exposes compiler internals
# to the fuzz harness and is documented as outside semver, so it is not part of
# the contract this file records.
RUSTUP_TOOLCHAIN="$TOOLCHAIN" cargo public-api \
    --manifest-path crates/bubbles-dialogue/Cargo.toml \
    --features full \
    --omit blanket-impls,auto-trait-impls,auto-derived-impls \
    --color never \
    >"$actual"

if ! diff -u "$BASELINE" "$actual"; then
    cat >&2 <<'MSG'

The public API of bubbles-dialogue no longer matches crates/bubbles-dialogue/public-api.txt.
Lines marked - were removed from the API, lines marked + were added.

If that was the point, regenerate the file:

    RUSTUP_TOOLCHAIN=nightly-2026-10-15 cargo public-api \
        --manifest-path crates/bubbles-dialogue/Cargo.toml --features full \
        --omit blanket-impls,auto-trait-impls,auto-derived-impls \
        > crates/bubbles-dialogue/public-api.txt

and commit it alongside the change that moved the surface.
MSG
    exit 1
fi

echo "Public API check passed ($(grep -c . "$BASELINE") entries)."
