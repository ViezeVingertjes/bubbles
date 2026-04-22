#!/usr/bin/env bash
# Ensures no forbidden third-party engine names appear in the codebase.
set -euo pipefail

forbidden="yarnspinner|yarn.spinner|yarn spinner"

if rg -n -i -e "$forbidden" \
      --glob '!.git/**' \
      --glob '!target/**' \
      --glob '!Cargo.lock' \
      .; then
  echo "ERROR: forbidden name found (see matches above)" >&2
  exit 1
fi

echo "Naming check passed."
