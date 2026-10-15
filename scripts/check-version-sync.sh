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

# The C header is what native hosts compile against, so its advertised version has to
# track the crate. crates/bubbles-ffi/tests/header.rs checks the same thing at test time;
# this runs in pre-commit, where a release bump actually happens.
FFI_HEADER="crates/bubbles-ffi/include/bubbles_ffi.h"
if ! grep -Fq "#define BUBBLES_FFI_VERSION_STRING \"$METADATA_VER\"" "$FFI_HEADER"; then
  echo "error: $FFI_HEADER must contain:" >&2
  echo "  #define BUBBLES_FFI_VERSION_STRING \"$METADATA_VER\"" >&2
  echo "Update BUBBLES_FFI_VERSION_MAJOR/MINOR/PATCH and the 'Library version:' comment too." >&2
  exit 1
fi
if ! grep -Fq "Library version: $METADATA_VER" "$FFI_HEADER"; then
  echo "error: $FFI_HEADER comment block must say 'Library version: $METADATA_VER'." >&2
  exit 1
fi

# bubbles-ffi and bubbles-cli depend on bubbles-dialogue by path *and* version so that
# they can be published. A caret requirement only stays correct while the majors agree,
# so a major bump must update them.
for CRATE in bubbles-ffi bubbles-cli; do
  DEP_REQ=$(
    cargo metadata --no-deps --format-version=1 \
      | jq -r --arg crate "$CRATE" '.packages[] | select(.name==$crate) | .dependencies[]
               | select(.name=="bubbles-dialogue") | .req'
  )
  if [[ "${DEP_REQ#^}" != "$METADATA_VER" ]]; then
    echo "error: $CRATE depends on bubbles-dialogue '$DEP_REQ', expected '^$METADATA_VER'." >&2
    echo "Update the version in crates/$CRATE/Cargo.toml alongside the workspace bump." >&2
    exit 1
  fi
done

echo "version sync OK ($METADATA_VER)"
