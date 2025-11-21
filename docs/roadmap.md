# Rustact Roadmap

This document captures the current state of the project and highlights the most impactful directions to pursue next. Revisit and adjust as priorities shift.

## 1. Developer Experience & Documentation

- Extend the starter template with the ops dashboard patterns (tabs, overlays, toasts) so new apps can opt in quickly.
- Bundle a troubleshooting appendix into `docs/guide.md` (common tracing filters, style reload flags, terminal quirks).
- Keep the docs in sync with upcoming diagnostics tooling (section 4) once APIs stabilize.

## 2. Runtime Ergonomics & Reliability

- ✅ Abstracted the terminal/tick/shutdown tasks behind the `RuntimeDriver` trait; `App::with_driver` now injects mocks for deterministic tests (see `runtime/tests/app.rs`).
- ✅ Improved shutdown handling by aborting and awaiting runtime tasks, logging ctrl-c detection, and surfacing renderer errors.
- ✅ Added `tracing` instrumentation across the runtime, dispatcher, event bus, and background tasks for structured debugging.
- Next: prototype a headless driver (no terminal) to unlock CI snapshots and in-memory render diffing; evaluate guardrails for runaway tasks (timeouts, panic propagation).

## 3. Feature Depth & Showcase Apps

- ✅ Built the `ops_dashboard` showcase binary that streams logs, simulates deployments, switches tabs, and displays incident modals/toasts.
- ✅ Added missing widgets/components (`TabsNode`, `ModalNode`, `LayeredNode`, `ToastStackNode`) so dashboards can express overlays and notifications without bespoke rendering.
- Next: demonstrate richer inter-component messaging patterns (e.g., global command palette) to highlight the event bus and context primitives.
- Future stretch: add a data-entry heavy showcase (forms + validation + keyboard navigation) to balance the ops dashboard.

## 4. Tooling & Observability

- Integrate an in-app diagnostics panel showing render frequency, hook counts, event throughput, and memory stats.
- Emit performance metrics to logs (render duration, diff counts) to spot regressions.
- ✅ Added optional CSS hot-reload: set `RUSTACT_WATCH_STYLES` to watch `styles/demo.css` and update the runtime without restarting the app.
- Plan: surface tracing spans via `tracing` layers or JSON logs for easier ingestion in external tooling.

## 5. Packaging & Distribution

- ✅ Polished `Cargo.toml` metadata (license, repository, keywords) and started a `CHANGELOG.md` for semantic releases.
- Evaluate publishing prebuilt demo binaries or a `rustact` CLI that wraps `cargo run` plus asset watching.
- Draft a release checklist (tests, fmt, clippy, doc build, CHANGELOG update) and publish guidance before the first crates.io release.
- Decide on dual licensing (MIT/Apache-2.0) and add the corresponding `LICENSE` files if needed for broader adoption.

## 6. Release Readiness

- Establish semantic versioning rules (what constitutes breaking runtime API changes vs. additive widget updates).
- Automate verification: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, `cargo doc --no-deps`.
- Gate main with CI once the publishing process is defined.

---

**Next Review:** Reassess after completing two major items from sections 1–3 or when preparing a public release.
