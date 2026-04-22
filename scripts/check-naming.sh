#!/usr/bin/env bash
# Ensures no forbidden third-party engine names appear in non-script source files.
set -euo pipefail

FAILED=0

# Prefer ripgrep if available, fall back to grep -r.
if command -v rg >/dev/null 2>&1; then
  if rg -n -i -e 'yarnspinner' -e 'yarn\.spinner' \
        --glob '!.git/**' \
        --glob '!target/**' \
        --glob '!Cargo.lock' \
        --glob '!scripts/**' \
        .; then
    FAILED=1
  fi
else
  if grep -rn -iE 'yarnspinner|yarn\.spinner' \
        --exclude-dir=.git \
        --exclude-dir=target \
        --exclude-dir=scripts \
        --exclude="Cargo.lock" \
        .; then
    FAILED=1
  fi
fi

if [[ "$FAILED" -ne 0 ]]; then
  echo "ERROR: forbidden name found (see matches above)" >&2
  exit 1
fi

echo "Naming check passed."
