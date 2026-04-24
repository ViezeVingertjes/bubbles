#!/usr/bin/env bash
# Ensures the published semver, the book's VERSION file, and README stay aligned.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

METADATA_VER=$(
  cargo metadata --no-deps --format-version=1 \
    | jq -r '.packages[] | select(.name=="bubbles-dialogue") | .version'
)
FILE_VER=$(grep -m1 -v '^[[:space:]]*$' VERSION | tr -d '\r\n[:space:]')

if [[ "$METADATA_VER" != "$FILE_VER" ]]; then
  echo "error: VERSION file ('$FILE_VER') != bubbles-dialogue from cargo metadata ('$METADATA_VER')." >&2
  echo "Update root Cargo.toml [workspace.package] version and the VERSION file together." >&2
  exit 1
fi

if ! grep -Fq "bubbles-dialogue = \"$METADATA_VER\"" README.md; then
  echo "error: README quick start must contain: bubbles-dialogue = \"$METADATA_VER\"" >&2
  exit 1
fi

echo "version sync OK ($METADATA_VER)"
