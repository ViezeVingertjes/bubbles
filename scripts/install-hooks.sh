#!/usr/bin/env bash
# Installs a pre-commit hook that mirrors CI quality gates locally.
set -euo pipefail

mkdir -p .git/hooks
cat > .git/hooks/pre-commit <<'HOOK'
#!/usr/bin/env bash
set -euo pipefail
cargo fmt --check
cargo clippy --all-features -- -D warnings
cargo test --all-features
bash scripts/check-naming.sh
bash scripts/check-file-sizes.sh
HOOK

chmod +x .git/hooks/pre-commit
echo "pre-commit hook installed."
