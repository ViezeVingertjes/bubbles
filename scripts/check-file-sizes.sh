#!/usr/bin/env bash
# Flags any crates/*/src/**/*.rs file that exceeds 300 non-blank lines.
set -euo pipefail

LIMIT=300
FAILED=0

while IFS= read -r -d '' file; do
  lines=$(grep -c . "$file" || true)
  if [[ "$lines" -gt "$LIMIT" ]]; then
    echo "OVERSIZE: $file ($lines lines, limit $LIMIT)" >&2
    FAILED=1
  fi
done < <(find crates -type f -name '*.rs' -path '*/src/*' -print0)

[[ "$FAILED" -eq 0 ]] && echo "File-size check passed." || exit 1
