# Rustact

[![CI](https://github.com/IllusiveBagel/rustact/actions/workflows/ci.yml/badge.svg)](https://github.com/IllusiveBagel/rustact/actions/workflows/ci.yml)
[![Latest Release](https://img.shields.io/github/v/release/IllusiveBagel/rustact?include_prereleases&label=release)](https://github.com/IllusiveBagel/rustact/releases)
[![Crates.io](https://img.shields.io/crates/v/rustact.svg)](https://crates.io/crates/rustact)

Rustact is an experimental, React-inspired framework for building async terminal UIs in Rust. It layers a component/hook system on top of `ratatui`, offering familiar primitives such as components, state, effects, context, and an event bus.

Install from crates.io:

```bash
cargo add rustact
```

## Community & contributions

- Read the [Contributing guide](CONTRIBUTING.md) for setup, workflows, and coding standards.
- Review the [Code of Conduct](CODE_OF_CONDUCT.md); we follow the Contributor Covenant.
- Maintainer contacts and responsibilities live in [MAINTAINERS.md](MAINTAINERS.md).
- Release duties and checklists are captured in [RELEASE.md](RELEASE.md).
- New issues/PRs should use the provided GitHub templates for consistent triage.

## Features

- **Component tree & hooks** – Define components via `component("Name", |ctx| ...)` and manage state with `use_state`, `use_reducer`, `use_ref`, `use_effect`, `use_memo`, `use_callback`, and `provide_context` / `use_context`.
- **Async runtime** – Built on `tokio`, handling terminal IO, ticks, and effect scheduling without blocking the UI thread.
- **Injectable runtime drivers** – Swap the event/tick/shutdown drivers with `App::with_driver(...)` for deterministic tests or custom IO sources.
- **Structured tracing** – The runtime emits `tracing` spans/logs around renders, events, and shutdown, so you can inspect behavior with any `tracing` subscriber.
- **Event bus** – Subscribe to keyboard/mouse/resize/tick events via `ctx.dispatcher().events().subscribe()` and react within hooks or spawned tasks.
- **Ratatui renderer** – Converts the virtual tree into `ratatui` widgets, handling layout primitives like blocks, vertical/horizontal stacks, styled text, list panels, gauges, tables, tree views, forms, and rich text inputs.
- **View diffing** – The runtime caches the previous view tree and skips terminal draws when nothing has changed, keeping renders snappy even with frequent ticks.
- **Mouse interactions** – Define button or input nodes with stable IDs; the renderer automatically maps mouse hitboxes so event handlers can react to clicks and focus changes.
- **Text inputs & validation hooks** – `use_text_input` binds component state to editable fields (with cursor management, tab focus, and secure mode), while `use_text_input_validation` or `handle.set_status(...)` drive contextual border colors and helper text.
- **Tabs, overlays, and toasts** – Compose `TabsNode`, `ModalNode`, `LayeredNode`, and `ToastStackNode` to build multi-pane dashboards with modal dialogs and notification stacks without wiring bespoke renderer code.
- **CSS-inspired styling** – Drop tweaks into `styles/demo.css` to recolor widgets, toggle button fills, rename gauge labels, change list highlight colors, resize form/table columns, or theme inputs without touching Rust code.
- **Optional style hot reload** – Set `RUSTACT_WATCH_STYLES=1` (or `true`/`on`) to have the runtime poll `styles/demo.css` and live-reload the stylesheet without restarting the app.
- **Demo app** – `src/main.rs` now composes every hook (state, reducer, ref, memo, callback, effect, context) with all widgets (text, flex, gauge, buttons, lists, tables, trees, forms) so you can see each feature in one place.

## Documentation

- `docs/guide.md` – day-to-day developer guide (setup, workflows, hook primer).
- `docs/tutorial.md` – step-by-step tutorial for building a fresh Rustact app.
- `docs/architecture.md` – deep dive into the runtime, hooks, renderer, and events.
- `docs/styling.md` – CSS subset reference and theming tips.
- `docs/roadmap.md` – prioritized initiatives to steer ongoing work.
- `docs/api-docs.md` – publishing instructions for hosting `cargo doc` output.
- `docs/template.md` – outline for the upcoming `cargo generate` starter template.
- `templates/rustact-app/` – ready-to-use project scaffold consumable via `cargo generate`.

## Starter template

Spin up a fresh app via [`cargo-generate`](https://github.com/cargo-generate/cargo-generate):

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

The template mirrors the demo’s patterns (hooks, dispatcher events, text inputs) plus a default stylesheet and README.

## API docs hosting

The workflow `.github/workflows/publish-docs.yml` builds `cargo doc --no-deps` on every push to `main` and deploys the result via GitHub Pages. Enable Pages → "GitHub Actions" in repo settings to activate it. Manual steps and customization tips live in `docs/api-docs.md`.

## Tracing logs

Rustact emits `tracing` spans throughout the runtime. Enable them in your binary (or the demo) by adding a subscriber:

```rust
use tracing_subscriber::EnvFilter;

fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("rustact=info"))
        .try_init();
}
```

Then set `RUST_LOG=info` (or more specific filters) before running the app to see render requests, external events, and shutdown diagnostics.

## Running the demo

```bash
cargo run
```

While running:
- Press `+`, `-`, or `r`, or click the on-screen `-` / `+` buttons to interact with the counter (watch the progress gauge update as the count approaches ±10).
- Observe the stats panel and event log streaming the five most recent framework events.
- Tab between the name/email/token inputs, click to focus, and type to see live validation statuses plus CSS-driven border and background colors.
- Check the framework overview banner and tips column to see keyed components, fragments, and shared context in action.
- Exit with `Ctrl+C`.

### Watching styles without restarting

```bash
RUSTACT_WATCH_STYLES=1 cargo run
```

When the env var is set (accepts `1`, `true`, or `on`) and `styles/demo.css` exists next to the binary, Rustact will poll the file, re-parse it on change, and trigger a render so you can see your CSS edits immediately. The demo and ops dashboard both honor the flag; when the file is missing, the runtime logs a warning and keeps using the embedded stylesheet.

### Ops dashboard showcase

```bash
cargo run --bin ops_dashboard
```

While running:
- Press `1`/`2` to switch between the overview and streaming logs tabs.
- Press `i` to pop open the incident modal, `Esc` to close it.
- Let the app run to watch deployment toasts bubble up; press `c` to dismiss the oldest toast.
- All overlays are composed via the new `LayeredNode`, `ModalNode`, and `ToastStackNode`, so they are reusable in your own apps.

### Styling via CSS

Rustact loads `styles/demo.css` on startup. The CSS parser understands simple selectors (`element`, `element#id`, `element.class`) plus custom properties, so you can retheme the terminal UI without recompiling:

- `:root` defines palette tokens such as `--accent-color`, `--warning-color`, and friends that the demo injects into its `Theme` context.
- `button#counter-plus` (and `#counter-minus`) set `accent-color` and `--filled` to control button appearance.
- `gauge#counter-progress` can override the bar color and `--label` text.
- `list#stats` exposes `--highlight-color` and `--max-items`, while `table#services` reads `--column-widths` and `form#release` reads `--label-width`.
- `input`, `input#feedback-name`, etc. configure accent/border/text/background colors, while `tip.keyboard` / `.mouse` / `.context` use the standard `color` property to tint each info card.

Save the file and rerun `cargo run` to see your tweaks reflected immediately.
See `docs/styling.md` for a deeper dive into selectors, property types, and integration tips.

## Creating components

```rust
use rustact::{component, Element, Scope};
use rustact::runtime::Color;

fn greeting(ctx: &mut Scope) -> Element {
    let color = ctx
        .use_context::<Theme>()
        .map(|theme| theme.accent)
        .unwrap_or(Color::Green);

    Element::colored_text("Hello from a component", color)
}

let app = App::new("Demo", component("Greeting", greeting));
```

Each render receives a `Scope`, giving access to hooks, the dispatcher, and context. Components can compose other components with `.into()`:

```rust
Element::vstack(vec![
    component("Greeting", greeting).into(),
    Element::text("Static text"),
])
```

### Advanced widgets

Tables, trees, and forms ship with builder-style APIs so you can fluently describe structured layouts:

```rust
use rustact::{Element, TableCellNode, TableNode, TableRowNode};
use rustact::runtime::Color;

let table = TableNode::new(vec![
    TableRowNode::new(vec![
        TableCellNode::new("api").bold(),
        TableCellNode::new("Healthy").color(Color::Green),
    ]),
])
.header(TableRowNode::new(vec![
    TableCellNode::new("Service").bold(),
    TableCellNode::new("Status").bold(),
]))
.widths(vec![40, 60])
.highlight(0);

Element::table(table);
```

`TreeNode`/`TreeItemNode` let you model recursive hierarchies, while `FormNode` + `FormFieldNode` render labeled key/value pairs with validation state highlighting.

### Text inputs & validation

```rust
use rustact::{Element, Scope, TextInputNode};
use rustact::runtime::FormFieldStatus;

fn feedback(ctx: &mut Scope) -> Element {
    let name = ctx.use_text_input("feedback:name", || String::new());
    let name_status = ctx.use_text_input_validation(&name, |snapshot| {
        if snapshot.value.trim().is_empty() {
            FormFieldStatus::Warning
        } else {
            FormFieldStatus::Success
        }
    });

    Element::text_input(
        TextInputNode::new(name)
            .label("Display name")
            .placeholder("Rustacean in Residence")
            .width(32)
            .status(name_status),
    )
}
```

`use_text_input` registers a focusable binding with the shared registry. The runtime tracks hitboxes, so clicking anywhere in the input focuses it, while Tab/Shift+Tab move between inputs in declaration order. A blinking cursor shows when the field is focused, and `.secure(true)` masks sensitive values. Use `ctx.use_text_input_validation` or `handle.set_status(FormFieldStatus::Error)` to tint the field, then read helper text from the same status to explain errors or warnings.

### Reducer & ref hooks

```rust
use rustact::{Element, Scope};

#[derive(Clone, Copy)]
enum Action {
    Increment,
    Reset,
}

fn counter(ctx: &mut Scope) -> Element {
    let (value, dispatch) = ctx.use_reducer(|| 0i32, |state, action: Action| match action {
        Action::Increment => *state += 1,
        Action::Reset => *state = 0,
    });

    let last_event = ctx.use_ref(|| None::<String>);

    Element::text(format!(
        "Value: {value} (last event: {:?})",
        last_event.with(|evt| evt.clone())
    ))
}
```

`use_reducer` returns the current value plus a `dispatch` handle that batches state updates through your reducer; `use_ref` stores mutable data without triggering re-renders (handy for metrics or imperative handles).

### View diffing

Every render builds a `View` tree, compares it with the previous frame, and only asks the renderer to draw when something actually changed. Hooks, effects, and state updates still run, but redundant terminal flushes are avoided—mirroring the virtual DOM approach.

## Next steps

- Package reusable components and publish to crates.io.

## License

Rustact is distributed under the [MIT License](LICENSE). By contributing, you agree that your contributions will be licensed under the same terms.