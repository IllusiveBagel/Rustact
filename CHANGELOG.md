# Changelog

All notable changes to this project will be documented in this file. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Placeholder entries for upcoming releases.

## [0.1.0] - 2025-11-21

### Added
- React-inspired component tree with hooks (`use_state`, `use_effect`, `use_reducer`, `use_context`, etc.) layered on top of `ratatui`.
- Async runtime built on `tokio`, including injectable drivers for deterministic tests and structured `tracing` instrumentation.
- Comprehensive demo plus the new `examples/ops-dashboard` showcase that highlights tabs, modals, layered overlays, streaming logs, and toast notifications.
- Optional CSS hot reload: set `RUSTACT_WATCH_STYLES=1` to live-reload the stylesheet that ships with each example (e.g., `examples/rustact-demo/styles/demo.css`) without restarting the app.
- Documentation suite (guide, tutorial, styling reference, roadmap, API docs workflow) and a starter template under `templates/rustact-app/`.

### Changed
- Improved runtime ergonomics with better shutdown handling, effect cleanup, and renderer error reporting.
- Added richer styling nodes (tabs, modal, layered, toast stack) and renderer support to cover real-world dashboard scenarios.

### Fixed
- Ensured stylesheet reloads are detected when saving from different working directories by fingerprinting the file contents and logging watcher activity.
