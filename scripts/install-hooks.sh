#!/usr/bin/env bash
# Points Git at the tracked hooks directory so the pre-commit hook stays
# up-to-date automatically whenever .githooks/pre-commit changes.
set -euo pipefail

git config core.hooksPath .githooks
echo "Git hooks configured. The pre-commit hook will now run on every commit."
