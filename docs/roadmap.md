# Rustact Roadmap

This document captures the current state of the project and highlights the most impactful directions to pursue next. Revisit and adjust as priorities shift.

## 1. Developer Experience & Documentation

Completed for this milestone; future documentation or template enhancements will be tracked separately.

## 2. Runtime Ergonomics & Reliability

- ✅ Abstracted the terminal/tick/shutdown tasks behind the `RuntimeDriver` trait; `App::with_driver` now injects mocks for deterministic tests (see `runtime/tests/app.rs`).
- ✅ Improved shutdown handling by aborting and awaiting runtime tasks, logging ctrl-c detection, and surfacing renderer errors.
- ✅ Added `tracing` instrumentation across the runtime, dispatcher, event bus, and background tasks for structured debugging.
- Update: driver injection covers mock needs; revisit feature flags once additional drivers (e.g., headless) land.

## 3. Feature Depth & Showcase Apps

- ✅ Built the `ops_dashboard` showcase binary that streams logs, simulates deployments, switches tabs, and displays incident modals/toasts.
- ✅ Added missing widgets/components (`TabsNode`, `ModalNode`, `LayeredNode`, `ToastStackNode`) so dashboards can express overlays and notifications without bespoke rendering.
- Next: demonstrate richer inter-component messaging patterns (e.g., global command palette) to highlight the event bus and context primitives.

## 4. Tooling & Observability

- Integrate an in-app diagnostics panel showing render frequency, hook counts, event throughput, and memory stats.
- Emit performance metrics to logs (render duration, diff counts) to spot regressions.
- ✅ Added optional CSS hot-reload: set `RUSTACT_WATCH_STYLES` to watch `styles/demo.css` and update the runtime without restarting the app.

## 5. Packaging & Distribution

- Polish `Cargo.toml` metadata (license, repository, keywords) and start a `CHANGELOG.md` for semantic releases.
- Evaluate publishing prebuilt demo binaries or a `rustact` CLI that wraps `cargo run` plus asset watching.
- Plan crate versioning policy and release checklist (tests, fmt, clippy, doc build) before first crates.io publish.

---

**Next Review:** Reassess after completing two major items from sections 1–3 or when preparing a public release.
