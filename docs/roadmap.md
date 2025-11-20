# Rustact Roadmap

This document captures the current state of the project and highlights the most impactful directions to pursue next. Revisit and adjust as priorities shift.

## 1. Developer Experience & Documentation

- Expand `README.md` into a structured guide under `docs/`, covering setup, component patterns, hook reference, and architecture overview.
- Add a "Quick Start" tutorial (e.g., `docs/tutorial.md`) that walks through building and styling a minimal app end-to-end.
- Publish comprehensive API docs via `cargo doc --open` guidance, and consider hosting them with GitHub Pages.
- Explore a `cargo generate` template or starter repo to bootstrap new Rustact apps with sensible defaults.

## 2. Runtime Ergonomics & Reliability

- Abstract the terminal/tick/shutdown tasks behind traits so the event loop is unit-testable without `crossterm`.
- Improve shutdown handling: propagate errors from renderer/effect tasks, surface panics, and ensure terminal state is restored on exit.
- Provide structured logging (e.g., `tracing`) around renders, effects, and dispatched events for easier debugging.
- Offer feature flags to opt into mock drivers for deterministic integration tests.

## 3. Feature Depth & Showcase Apps

- Build a second demo (log monitor, dashboard, etc.) to stress real-world flows: streaming data, background jobs, navigation, and modals.
- Add missing widgets/components (tabs, modal overlays, status toasts) discovered while building the showcase.
- Demonstrate inter-component messaging patterns (e.g., global command palette) to highlight the event bus and context primitives.

## 4. Tooling & Observability

- Integrate an in-app diagnostics panel showing render frequency, hook counts, event throughput, and memory stats.
- Emit performance metrics to logs (render duration, diff counts) to spot regressions.
- Add optional hot-reload for styles/theme files or watch mode that reloads CSS without restarting the app.

## 5. Packaging & Distribution

- Polish `Cargo.toml` metadata (license, repository, keywords) and start a `CHANGELOG.md` for semantic releases.
- Evaluate publishing prebuilt demo binaries or a `rustact` CLI that wraps `cargo run` plus asset watching.
- Plan crate versioning policy and release checklist (tests, fmt, clippy, doc build) before first crates.io publish.

---

**Next Review:** Reassess after completing two major items from sections 1–3 or when preparing a public release.
