+++
title = "Developer Guide"
description = "Setup, workflows, and the core concepts you'll touch when working inside the Rustact codebase."
weight = 10
template = "doc.html"
updated = 2025-11-21
+++

# Rustact Developer Guide

A practical reference for working inside the Rustact codebase: setup, workflows, and the core concepts you will touch most frequently.

> **Using Rustact in your own app?** You only need `cargo add rustact` (plus `tokio` in your binary) to embed the runtime. Cloning the repository is optional unless you want to browse the examples or contribute to the core crate.

## 1. Prerequisites

-   Rust toolchain 1.85+ (matches the crate’s `rust-version`) with `cargo`, `rustfmt`, and `clippy` (`rustup update && rustup component add rustfmt clippy`).
-   `tokio` runtime knowledge is helpful but not required; async entrypoints are already wired up.
-   A terminal that supports ANSI escape codes (most modern terminals do) and mouse capture if you want to interact with buttons/inputs.

Verify your toolchain:

```bash
rustup show active-toolchain
cargo --version
```

## 2. Cloning & running the demo

```bash
git clone https://github.com/IllusiveBagel/rustact.git
cd rustact/examples/rustact-demo
cargo run
```

While the demo is running:

-   `+` / `-` / `r` keys or the on-screen buttons drive the counter.
-   Tabs cycle between text inputs; mouse clicks focus individual fields.
-   `Ctrl+C` exits immediately.

Run `cd ../ops-dashboard && cargo run` for the ops showcase. See `examples/README.md` for more details on both apps.

## 3. Project layout

| Path                                    | Overview                                                                                                       |
| --------------------------------------- | -------------------------------------------------------------------------------------------------------------- |
| `src/runtime/`                          | App lifecycle, dispatcher, reconciler, element/view definitions, renderer bridge.                              |
| `src/hooks/`                            | Hook registry, `Scope`, and all built-in hooks (`use_state`, `use_effect`, etc.).                              |
| `src/context/`                          | Type-safe provider stack for passing data down the tree.                                                       |
| `src/events/`                           | `FrameworkEvent`, broadcast bus, helpers for Ctrl+C and mouse detection.                                       |
| `src/interactions.rs`                   | Global hitbox registry for buttons and inputs.                                                                 |
| `src/text_input/`                       | Text input bindings, state machine, and validation hooks.                                                      |
| `examples/rustact-demo/styles/demo.css` | Runtime stylesheet loaded by the main demo app.                                                                |
| `website/content/docs/`                 | Source for this guide, the tutorial, styling reference, roadmap, and architecture docs that power the website. |

## 4. Everyday workflows

### Running & iterating

```bash
cd examples/rustact-demo
cargo run                        # launch the main demo
RUSTACT_WATCH_STYLES=1 cargo run  # optional: live reload demo styles/demo.css
cd ../ops-dashboard
cargo run                        # launch the ops showcase
cargo fmt && cargo clippy  # format + lint changes
cargo test                 # run the growing unit-test suite
```

Tracing spans already wrap renders, external events, and shutdown; add a `tracing_subscriber` and run with `RUST_LOG=rustact=trace` (or an `EnvFilter`) to inspect the runtime in action.

### Hot-edit cycle

1. Modify a component or hook.
2. `cargo run` inside the example you're iterating (`examples/rustact-demo` or `examples/ops-dashboard`).
3. Watch the stats panel to confirm framework events are emitted as expected.
4. Keep `cargo test` handy—many modules (hooks, events, text input) already ship with unit tests.

## 5. Building components

Components are functions `fn(&mut Scope) -> Element`. Builder methods (e.g., `Element::list`, `TextInputNode::new`) describe the virtual tree, and the runtime converts that into `View` structs before hitting `ratatui`.

Key patterns:

-   Use `component("Name", render_fn)` to wrap a function so it can be nested inside other elements.
-   Access the dispatcher via `ctx.dispatcher()` to request renders or subscribe to events.
-   Provide stable keys when rendering lists or conditionally showing fragments so hook ordering remains deterministic.

## 6. Hook cheat sheet

| Hook                              | Purpose                                                                  | Tips                                                        |
| --------------------------------- | ------------------------------------------------------------------------ | ----------------------------------------------------------- |
| `use_state`                       | Local state with render scheduling.                                      | Call `set`/`update` to trigger renders.                     |
| `use_reducer`                     | Structured state transitions via actions.                                | Keep reducers pure; they run synchronously during render.   |
| `use_effect`                      | Side effects that can spawn async work.                                  | Return `Some(cleanup)` to tear down tasks or subscriptions. |
| `use_ref`                         | Mutable data that does not cause re-renders.                             | Great for metrics or imperative handles.                    |
| `use_memo` / `use_callback`       | Cache expensive computations or function values.                         | Dependencies must implement `PartialEq`.                    |
| `use_context` / `provide_context` | Share data down the component tree.                                      | Providers unwind automatically when their guard drops.      |
| `use_text_input`                  | Register focusable inputs that track cursor/focus state outside renders. | Pair with `use_text_input_validation` for live statuses.    |

Scope exposes additional helpers (`dispatcher`, `styles`, `use_text_input_validation`, etc.). Explore the [architecture doc](/docs/architecture/) for deeper internals.

## 7. Styling & theming

-   Stylesheets use a compact CSS subset (type/id/class selectors plus `:root`).
-   Load them from disk with `Stylesheet::from_file("styles/demo.css")` inside each example crate (the helper `load_demo_stylesheet` does this) and fall back to `Stylesheet::parse(include_str!("../styles/demo.css"))` if the file is missing, then pass the result to `App::with_stylesheet(...)`.
-   Toggle hot reload by setting `RUSTACT_WATCH_STYLES=1` (or `true`/`on`); the runtime will poll the sibling `styles/demo.css`, re-parse on change, and schedule a redraw without restarting the process.
-   Query inside components with `ctx.styles().query(StyleQuery::element("button").with_id("counter-plus"))`.
-   See the [styling reference](/docs/styling/) for supported selectors, properties, and examples.

## 8. Project template

Bootstrap a fresh app using the bundled `cargo generate` template once published:

```bash
cargo install cargo-generate
cargo generate \
    --git https://github.com/IllusiveBagel/rustact \
    --branch main \
    --path templates/rustact-app \
    --name my-rustact-app
cd my-rustact-app
cargo run
```

The template lives under `templates/rustact-app/` and includes a sample component, stylesheet, and README. Adjust the [template guide](/docs/template/) as you evolve the scaffold.

## 9. Testing & troubleshooting

-   Use `cargo test module::tests::name` to focus on a failing spec.
-   Many modules support deterministic testing (events, hooks, text inputs, runtime tree helpers).
-   Runtime tasks currently depend on `tokio::test`; future work will abstract terminal IO behind traits for deeper coverage (see the [roadmap](/docs/roadmap/)).
-   If the terminal becomes garbled after a panic, run `reset` or simply `stty sane`.

## 10. Where to go next

-   Deep dive: the [architecture guide](/docs/architecture/) for a block-by-block walkthrough of the runtime.
-   Styling reference: the [styling guide](/docs/styling/) to master theming.
-   Roadmap: follow upcoming initiatives inside the [roadmap doc](/docs/roadmap/).
-   Tutorial: walk through the [hands-on tutorial](/docs/tutorial/) to build a fresh app from scratch.

Keep this guide nearby while contributing; update it whenever you add new workflows, commands, or conceptual primitives.

## 11. Custom runtime drivers (internal seam)

`App::with_driver` accepts any `RuntimeDriver`, and the built-in tests (`src/runtime/tests/app.rs`) already use the hook to provide a deterministic driver. Because the trait signature still refers to the internal `AppMessage` type, only code inside this crate can currently implement it; consumers should stick with `DefaultRuntimeDriver` for now. When you do need a mock driver (e.g., for tests), mirror the pattern below:

```rust
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::runtime::{component, App, Element, RuntimeDriver};
use crate::runtime::dispatcher::AppMessage;

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

The seam keeps `App::run()` deterministic during tests (pair it with `.headless()` if you do not want to touch the terminal) while production builds continue to use `DefaultRuntimeDriver`.

## 12. Tracing & diagnostics

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
