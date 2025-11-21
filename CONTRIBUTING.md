# Contributing to Rustact

Thanks for helping improve Rustact! This document summarizes the expectations, tooling, and workflow we use so contributions land smoothly.

## Prerequisites

- **Rust toolchain**: Install the latest stable toolchain via [`rustup`](https://rustup.rs/). The project targets Rust 1.85+ (edition 2024).
- **Cargo utilities**: Ensure you have `cargo fmt`, `cargo clippy`, and `cargo test` available (installed with the toolchain). Optional helpers such as [`cargo-nextest`](https://nexte.st/) or [`just`](https://github.com/casey/just) are welcome but not required.
- **Node-free build**: The repo is pure Rust; no other runtime is necessary unless you are editing the static marketing site under `website/`.

## Development workflow

1. **Fork & branch**: Fork the repo (or use a feature branch if you have push rights). Create a descriptive branch such as `feature/hot-reload-config`.
2. **Set up the workspace**:
   ```bash
   cargo fetch
   cargo fmt --all
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test --all --all-features
   ```
   Running everything once ensures your environment matches CI.
3. **Make changes**: Keep commits focused. Update docs/tests alongside code.
4. **Self-check**: Before opening a PR, run the quality gates locally:
   ```bash
   cargo fmt --all
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test --all --all-features
   ```
   - If you touch CSS or docs, ensure `cargo doc --no-deps` still builds.
   - Headless rendering is available via `.headless()` helpers to keep tests terminal-safe.
5. **Open a pull request**: Fill out the PR template, note any follow-up work, and link issues as appropriate. Please keep PRs <~400 LOC when possible to ease review.
6. **Review cycle**: Discuss feedback inline, keep the conversation respectful (see the Code of Conduct), and re-run checks before requesting another review.

## Coding standards

- **Edition & style**: Rust 2024, formatted with `cargo fmt`. Apply clippy suggestions unless they conflict with readability; otherwise add explicit `#[allow(...)]` explaining why.
- **Testing**: Prefer unit tests near the code under test. Integration/UI changes should update the demo in `src/main.rs` or add coverage in `src/runtime/tests`.
- **Docs**: Update README/docs when you add features, flags, or behavior. Include changelog entries in `CHANGELOG.md` under the `Unreleased` section.
- **Error handling**: Use `anyhow::Context` for fallible operations at the app boundary and `thiserror`-style enums inside libraries. Avoid `unwrap()`/`expect()` unless the condition truly cannot fail.
- **Logging**: Favor `tracing` macros (`trace!`, `info!`, etc.) over `println!`.

## Git & review etiquette

- Keep commits logical and squashed when necessary. Use imperative commit subjects ("Add headless renderer" not "Added...").
- Reference issues using `Fixes #123` or `Closes #123` when applicable.
- Avoid force-pushes during active review unless you coordinate with reviewers.
- Follow the [Code of Conduct](CODE_OF_CONDUCT.md) in all interactions.

## Opening issues

If you found a bug or have a feature idea, check existing issues first. When opening a new one, include:

- Steps to reproduce (or expected behavior) with terminal output if relevant.
- Environment details (`rustc -V`, OS, terminal emulator).
- Screenshots or recordings for visual regressions.

The issue templates in `.github/ISSUE_TEMPLATE/` will guide you through these fields.

## Need help?

- Use GitHub Discussions (if enabled) or the issue tracker for questions.
- Tag issues with `help wanted` / `good first issue` if you spot approachable work.
- Ping `@IllusiveBagel` or the maintainers listed in `MAINTAINERS.md` for clarifications.

We’re excited to see what you build—thank you for contributing!
