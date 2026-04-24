# Releasing

1. Bump **`version`** in **`Cargo.toml`** under **`[workspace.package]`** (workspace members inherit it).
2. Set the first non-blank line of **`VERSION`** to the same value (used by mdBook `{{#include}}` in the guide).
3. Update **`README.md`** quick start so the line `bubbles-dialogue = "<version>"` matches (GitHub does not process the book includes).
4. Run **`bash scripts/check-version-sync.sh`** (runs in **pre-commit** and in **CI**).
5. Tag the release (see **Publish** workflow: the tag must match the crate version).
6. **`cargo publish -p bubbles-dialogue`**

Patch releases (`0.6.0` → `0.6.1`) follow the same steps; the book and README stay on the exact semver shown above.
