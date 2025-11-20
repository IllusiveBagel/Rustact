# Rustact Developer Guide

A practical reference for working inside the Rustact codebase: setup, workflows, and the core concepts you will touch most frequently.

## 1. Prerequisites

- Rust toolchain 1.76+ with `cargo` and `rustfmt` (`rustup update` to grab the latest stable channel).
- `tokio` runtime knowledge is helpful but not required; async entrypoints are already wired up.
- A terminal that supports ANSI escape codes (most modern terminals do) and mouse capture if you want to interact with buttons/inputs.

Verify your toolchain:

```bash
rustup show active-toolchain
cargo --version
```

## 2. Cloning & running the demo

```bash
git clone https://github.com/IllusiveBagel/rustact.git
cd rustact
cargo run
```

While the demo is running:
- `+` / `-` / `r` keys or the on-screen buttons drive the counter.
- Tabs cycle between text inputs; mouse clicks focus individual fields.
- `Ctrl+C` exits immediately.

## 3. Project layout

| Path | Overview |
| --- | --- |
| `src/runtime/` | App lifecycle, dispatcher, reconciler, element/view definitions, renderer bridge. |
| `src/hooks/` | Hook registry, `Scope`, and all built-in hooks (`use_state`, `use_effect`, etc.). |
| `src/context/` | Type-safe provider stack for passing data down the tree. |
| `src/events/` | `FrameworkEvent`, broadcast bus, helpers for Ctrl+C and mouse detection. |
| `src/interactions.rs` | Global hitbox registry for buttons and inputs. |
| `src/text_input/` | Text input bindings, state machine, and validation hooks. |
| `styles/demo.css` | Runtime stylesheet loaded by the demo app. |
| `docs/` | Architecture notes, styling reference, roadmap, and this guide. |

## 4. Everyday workflows

### Running & iterating

```bash
cargo run                  # launch the demo
RUSTACT_WATCH_STYLES=1 cargo run  # optional: live reload styles/demo.css
cargo fmt && cargo clippy  # format + lint changes
cargo test                 # run the growing unit-test suite
```

Use `RUST_LOG=debug` once tracing is added (see roadmap) to inspect runtime events.

### Hot-edit cycle

1. Modify a component or hook.
2. `cargo run` to preview terminal output.
3. Watch the stats panel to confirm framework events are emitted as expected.
4. Keep `cargo test` handy—many modules (hooks, events, text input) already ship with unit tests.

## 5. Building components

Components are functions `fn(&mut Scope) -> Element`. Builder methods (e.g., `Element::list`, `TextInputNode::new`) describe the virtual tree, and the runtime converts that into `View` structs before hitting `ratatui`.

Key patterns:
- Use `component("Name", render_fn)` to wrap a function so it can be nested inside other elements.
- Access the dispatcher via `ctx.dispatcher()` to request renders or subscribe to events.
- Provide stable keys when rendering lists or conditionally showing fragments so hook ordering remains deterministic.

## 6. Hook cheat sheet

| Hook | Purpose | Tips |
| --- | --- | --- |
| `use_state` | Local state with render scheduling. | Call `set`/`update` to trigger renders. |
| `use_reducer` | Structured state transitions via actions. | Keep reducers pure; they run synchronously during render. |
| `use_effect` | Side effects that can spawn async work. | Return `Some(cleanup)` to tear down tasks or subscriptions. |
| `use_ref` | Mutable data that does not cause re-renders. | Great for metrics or imperative handles. |
| `use_memo` / `use_callback` | Cache expensive computations or function values. | Dependencies must implement `PartialEq`. |
| `use_context` / `provide_context` | Share data down the component tree. | Providers unwind automatically when their guard drops. |
| `use_text_input` | Register focusable inputs that track cursor/focus state outside renders. | Pair with `use_text_input_validation` for live statuses. |

Scope exposes additional helpers (`dispatcher`, `styles`, `use_text_input_validation`, etc.). Explore `docs/architecture.md` for deeper internals.

## 7. Styling & theming

- Stylesheets use a compact CSS subset (type/id/class selectors plus `:root`).
- Load them from disk with `Stylesheet::from_file("styles/demo.css")` (falls back to embedded CSS when missing) and pass them to `App::with_stylesheet(...)`.
- Toggle hot reload by setting `RUSTACT_WATCH_STYLES=1` (or `true`/`on`); the runtime will poll `styles/demo.css`, re-parse on change, and schedule a redraw without restarting the process.
- Query inside components with `ctx.styles().query(StyleQuery::element("button").with_id("counter-plus"))`.
- See `docs/styling.md` for supported selectors, properties, and examples.

## 8. Project template

Bootstrap a fresh app using the bundled `cargo generate` template once published:

```bash
cargo install cargo-generate
cargo generate --git https://github.com/IllusiveBagel/rustact --name my-app --template rustact-app
cd my-app
cargo run
```

The template lives under `templates/rustact-app/` and includes a sample component, stylesheet, and README. Adjust `docs/template.md` as you evolve the scaffold.

## 8. Testing & troubleshooting

- Use `cargo test module::tests::name` to focus on a failing spec.
- Many modules support deterministic testing (events, hooks, text inputs, runtime tree helpers).
- Runtime tasks currently depend on `tokio::test`; future work will abstract terminal IO behind traits for deeper coverage (see roadmap §2).
- If the terminal becomes garbled after a panic, run `reset` or simply `stty sane`.

## 9. Where to go next

- Deep dive: `docs/architecture.md` for a block-by-block walkthrough of the runtime.
- Styling reference: `docs/styling.md` to master theming.
- Roadmap: `docs/roadmap.md` for upcoming initiatives.
- Tutorial: `docs/tutorial.md` (below) to build a fresh app from scratch.

Keep this guide nearby while contributing; update it whenever you add new workflows, commands, or conceptual primitives.

## 10. Custom runtime drivers

The runtime now accepts pluggable drivers for the terminal/tick/shutdown tasks. Implement the `RuntimeDriver` trait (from `rustact::runtime`) and pass it to `App::with_driver(...)` to swap in mocks or alternate IO sources.

```rust
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use rustact::runtime::{component, App, Element, RuntimeDriver};
use rustact::runtime::dispatcher::AppMessage;

#[derive(Clone)]
struct TestDriver;

impl RuntimeDriver for TestDriver {
	fn spawn_terminal_events(&self, tx: mpsc::Sender<AppMessage>) -> JoinHandle<()> {
		tokio::spawn(async move {
			// Immediately request shutdown in tests.
			let _ = tx.send(AppMessage::Shutdown).await;
		})
	}

	fn spawn_tick_loop(&self, _tx: mpsc::Sender<AppMessage>, _rate: Duration) -> JoinHandle<()> {
		tokio::spawn(async {})
	}

	fn spawn_shutdown_watcher(&self, _tx: mpsc::Sender<AppMessage>) -> JoinHandle<()> {
		tokio::spawn(async {})
	}
}

let app = App::new("Testable", component("Unit", |_ctx| Element::Empty))
	.with_driver(TestDriver);
```

This makes `App::run()` deterministic under `cargo test`, unlocks headless integration drivers, and keeps the production behavior intact via the default driver (`DefaultRuntimeDriver`).

## 11. Tracing & diagnostics

Rustact emits `tracing` spans around render requests, external events, effect scheduling, and shutdown flow. To see the logs, add a subscriber in your binary (or the demo app) before calling `App::run`:

```rust
use tracing_subscriber::EnvFilter;

fn init_tracing() {
	let _ = tracing_subscriber::fmt()
		.with_env_filter(EnvFilter::new("rustact=info"))
		.try_init();
}
```

Run with `RUST_LOG=rustact=trace` (or any filter) to inspect the lifecycle. This is especially handy when debugging shutdown behavior, effect churn, or event floods.
