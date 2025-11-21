# Release checklist

This project publishes tagged releases to GitHub (and eventually crates.io). Use the following workflow to keep artifacts consistent.

1. **Create a release branch**
   ```bash
   git checkout -b release/v0.x.y
   ```
2. **Update metadata**
   - Bump `version` in `Cargo.toml` and `Cargo.lock` (if present).
   - Update `CHANGELOG.md`: move entries from `Unreleased` into a new `## [v0.x.y] - YYYY-MM-DD` section.
   - Update docs/README if installation instructions need version bumps.
3. **Verify quality gates**
   ```bash
   cargo fmt --all
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test --all --all-features
   cargo doc --no-deps
   ```
4. **Smoke test binaries**
   - `cargo run` (demo app)
   - `cargo run --bin ops_dashboard`
5. **Commit & push**
   ```bash
   git commit -am "Release v0.x.y"
   git push origin release/v0.x.y
   ```
6. **Open a PR**
   - Request review from another maintainer.
   - Ensure CI passes.
7. **Tag & publish**
   After the PR merges:
   ```bash
   git checkout main
   git pull
   git tag v0.x.y
   git push origin v0.x.y
   ```
   The `release.yml` workflow will build artifacts and attach them to the GitHub Release automatically.
8. **Post-release tasks**
   - Close the milestone.
   - Announce in README/docs if necessary.
   - Create a new `Unreleased` section in `CHANGELOG.md` if it was removed.

Future crates.io publishing steps can be appended once the crate is public.
