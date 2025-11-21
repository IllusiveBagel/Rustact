+++
title = "Roadmap"
description = "Track adoption goals, runtime ergonomics, showcase apps, tooling, packaging, and governance priorities."
weight = 50
template = "doc.html"
updated = 2025-11-21
+++

# Rustact Roadmap

This document captures the current state of the project and highlights the most impactful directions to pursue next. Revisit and adjust as priorities shift.

## 1. Adoption & Documentation

- Publish an introduction section in the [developer guide](/docs/guide/) that mirrors the crates.io quick start (`cargo add rustact`, minimal component example).
- Expand the starter template to optionally scaffold the ops dashboard widgets so new apps can opt into overlays/toasts immediately.
- Add a troubleshooting appendix covering tracing filters, terminal quirks, and style hot-reload gotchas.
- Keep docs aligned with forthcoming diagnostics tooling (section 4) so new APIs never ship undocumented.

## 2. Runtime Ergonomics & Reliability

- ✅ Abstracted the terminal/tick/shutdown tasks behind the `RuntimeDriver` trait; `App::with_driver` now injects mocks for deterministic tests (see `runtime/tests/app.rs`).
- ✅ Improved shutdown handling by aborting and awaiting runtime tasks, logging Ctrl+C detection, and surfacing renderer errors.
- ✅ Added `tracing` instrumentation across the runtime, dispatcher, event bus, and background tasks for structured debugging.
- ✅ Added a headless renderer mode so tests (and future CLI tools) can render without touching the terminal.
- Next: capture deterministic render snapshots (e.g., JSON diff or ANSI frame dump) to enable golden tests and docs previews.
- Next: add guardrails for runaway background tasks (timeouts, panic bubbling) plus a feature-flagged `tokio::task::Builder` hook for custom error reporting.

## 3. Feature Depth & Showcase Apps

- ✅ Built the `ops_dashboard` showcase binary that streams logs, simulates deployments, switches tabs, and displays incident modals/toasts.
- ✅ Added missing widgets/components (`TabsNode`, `ModalNode`, `LayeredNode`, `ToastStackNode`) so dashboards can express overlays and notifications without bespoke rendering.
- Next: demonstrate richer inter-component messaging patterns (e.g., global command palette) to highlight the event bus and context primitives.
- Future stretch: add a data-entry heavy showcase (forms + validation + keyboard navigation) to balance the ops dashboard.

## 4. Tooling & Observability

- Integrate an in-app diagnostics panel showing render frequency, hook counts, event throughput, and memory stats.
- Emit performance metrics to logs (render duration, diff counts) to spot regressions and feed the diagnostics panel.
- ✅ Added optional CSS hot-reload: set `RUSTACT_WATCH_STYLES` to watch `styles/demo.css` and update the runtime without restarting the app.
- Plan: surface tracing spans via `tracing` layers or JSON logs for easier ingestion in external tooling.

## 5. Packaging & Distribution

- ✅ Polished `Cargo.toml` metadata, LICENSE, and CHANGELOG.
- ✅ Published `rustact` v0.1.0 to crates.io.
- Evaluate publishing prebuilt demo binaries or a lightweight `rustact` CLI (`rustact dev --watch`) to simplify local workflows.
- Add docs.rs badges + feature tables so crate consumers can see API status at a glance.
- Investigate template distribution via `cargo-generate` registry (list `rustact-app` as a template repo).

## 6. Release & Governance

- ✅ Automated verification + release workflow (fmt, clippy, tests, docs, package, optional publish) triggered on `v*` tags.
- Define semantic versioning rules (what breaks `App`/`Element` APIs vs. additive widget changes) and document them in `RELEASE.md`.
- Add CODEOWNERS + branch protections that require review before release tagging.
- Once diagnostics + CLI land, plan a v0.2 roadmap review before the next tag.

---

**Next review:** reassess after shipping the diagnostics panel (section 4) and starter template upgrades (section 1), or ahead of the v0.2 release planning session.
