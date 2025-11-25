<p align="center">
    <img src="website/static/rustact-logo.svg" alt="Rustact logo" width="640" />
</p>

[![CI](https://github.com/IllusiveBagel/rustact/actions/workflows/ci.yml/badge.svg)](https://github.com/IllusiveBagel/rustact/actions/workflows/ci.yml)
[![Latest Release](https://img.shields.io/github/v/release/IllusiveBagel/rustact?include_prereleases&label=release)](https://github.com/IllusiveBagel/rustact/releases)
[![Crates.io](https://img.shields.io/crates/v/rustact.svg)](https://crates.io/crates/rustact)

Rustact is a React-inspired, async terminal UI framework built on top of `ratatui`, `crossterm`, and `tokio`. Components render into a virtual tree, hooks manage state, side effects, and context, and the runtime diff-patches frames to keep redraws quick even when ticks fire continuously.

Install from crates.io:

```bash
cargo add rustact
```

## Quick start

### Build a minimal app

```rust
use rustact::{component, App, Element, Scope};
use rustact::{is_button_click, ButtonNode};

fn counter(ctx: &mut Scope) -> Element {
    let (count, set_count) = ctx.use_state(|| 0i32);

    ctx.use_effect((), move |dispatcher| {
        let mut events = dispatcher.events().subscribe();
        let handle = tokio::spawn(async move {
            while let Ok(event) = events.recv().await {
                if is_button_click(&event, "counter:inc") {
                    set_count.update(|value| *value += 1);
                }
            }
        });
        Some(Box::new(move || handle.abort()))
    });

    Element::vstack(vec![
        Element::text(format!("Count: {count}")),
        Element::button(ButtonNode::new("counter:inc", "+").filled(true)),
    ])
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let root = component("Counter", counter);
    App::new("counter-demo", root)
        .watch_stylesheet("styles/app.css")
        .run()
        .await
}
```

### Run the kitchen-sink demo

```bash
cd examples/rustact-demo
RUSTACT_WATCH_STYLES=1 cargo run
```

-   Counter, gauges, tables, trees, forms, toasts, and tips live in one app under `examples/rustact-demo/src/main.rs`.
-   Set `RUSTACT_WATCH_STYLES` to `1`, `true`, or `on` to hot-reload `styles/demo.css` while the example is running.

### Ops dashboard showcase

```bash
cd examples/ops-dashboard
cargo run
```

Explore layered overlays (`LayeredNode`), toasts (`ToastStackNode`), tabs, and modal dialogs, all powered by hooks and shared context.

### Scaffold a new project

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

The template mirrors the demo’s structure (components, dispatcher usage, stylesheet, README) so you can immediately iterate on your own TUI.

## Why Rustact?

-   **Component + hook model** – Declare components with `component("Name", handler)` and manage state via `use_state`, `use_reducer`, `use_ref`, `use_memo`, `use_callback`, `use_effect`, `provide_context`, `use_context`, and the dedicated `use_text_input` / `use_text_input_validation` hooks.
-   **Async runtime + event bus** – `tokio` drives terminal IO, ticks, shutdown, and external signals; subscribe to `FrameworkEvent`s through `Dispatcher::events()` for keyboard, mouse, resize, and timer events.
-   **Injectable drivers & headless mode** – Use `App::with_driver` to plug deterministic drivers for tests or simulations, or call `App::headless()` to render without a terminal for snapshots.
-   **Rich widget set** – `Element` builders cover flex layouts, blocks, lists, tables, trees, forms, gauges, buttons, inputs, tabs, layered overlays, modals, and toast stacks.
-   **Text input system** – Handles focus rings, cursor placement, secure mode, validation state, and shared registries so inputs behave like native controls (including mouse hits and Tab cycling).
-   **CSS-inspired styling** – The `Stylesheet` parser understands `:root`, element/id/class selectors, and custom properties so you can recolor UI, resize columns, rename labels, or toggle fills without recompiling.
-   **Hot-reloadable themes** – `App::watch_stylesheet` plus the `RUSTACT_WATCH_STYLES` env var reload styles whenever `styles/demo.css` (or any configured path) changes.
-   **View diffing + tracing** – Frames are diffed before drawing, and every render/event/shutdown emits `tracing` spans so you can profile and debug behavior.

## Styling workflow

Rustact looks for a sibling stylesheet (for the demos that is `styles/demo.css`). You can define palette tokens in `:root`, override widget-specific properties, and reload on every save.

```css
:root {
    --accent-color: #5af78e;
    --warning-color: #ffb86c;
}

button#counter-plus {
    accent-color: var(--accent-color);
    --filled: true;
}

input.feedback-name {
    border-color: #8be9fd;
    placeholder: "Display name";
}
```

Run any example with `RUSTACT_WATCH_STYLES=1 cargo run` to live-reload the stylesheet. For custom apps, chain `.watch_stylesheet("styles/app.css")` on `App` and keep the env var enabled while iterating.

Read `website/content/docs/styling.md` for the full selector/property reference and integration tips.

## Documentation & resources

-   **Docs site** – https://illusivebagel.github.io/rustact (generated from `website/content/docs/**`).
-   **website/content/docs/guide.md** – high-level overview, concepts, and workflows.
-   **website/content/docs/tutorial.md** – template-powered quickstart and step-by-step tutorial.
-   **website/content/docs/architecture.md** – runtime, hooks, renderer, and event bus deep dive.
-   **website/content/docs/api-docs.md** – how the GitHub Pages workflow publishes `cargo doc`.
-   **website/content/docs/roadmap.md** – prioritized features and milestones.
-   **examples/README.md** – describes each example crate and how to run it.
-   **templates/rustact-app/** – starter project used by `cargo generate`.

## Repository layout

-   `src/` – core runtime, renderer, hooks, context, styles, text input system, and integration tests.
-   `examples/` – standalone apps (`rustact-demo`, `ops-dashboard`) that depend on the local crate via path dependencies.
-   `templates/` – reusable scaffolds; currently `templates/rustact-app`.
-   `website/` – documentation source consumed by GitHub Pages.
-   `CHANGELOG.md`, `RELEASE.md`, `MAINTAINERS.md`, `CONTRIBUTING.md` – project process, release, and ownership docs.

## Community & contributions

-   Follow the [Code of Conduct](CODE_OF_CONDUCT.md) (Contributor Covenant).
-   Read [CONTRIBUTING.md](CONTRIBUTING.md) for tooling, test, and review expectations.
-   Maintainer responsibilities live in [MAINTAINERS.md](MAINTAINERS.md); release steps in [RELEASE.md](RELEASE.md); history in [CHANGELOG.md](CHANGELOG.md).

## License

Rustact is distributed under the [MIT License](LICENSE). By contributing, you agree that your contributions will be licensed under the same terms.
