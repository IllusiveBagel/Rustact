# Rustact Architecture & Usage Guide

Rustact is a React-inspired framework for building asynchronous terminal UIs on top of [`ratatui`](https://github.com/ratatui-org/ratatui). This document explains every moving piece—components, hooks, context, runtime, renderer, and events—so you can confidently extend the framework or build your own apps.

## Quick start

```bash
cargo run
```

The demo (`src/main.rs`) launches a multi-pane dashboard:
- `Hero` component displays instructions and shared theme colors.
- `Counter` shows state updates, memoized summaries, and a progress gauge for ±10.
- `Stats` keeps a scrolling list of the five most recent framework events.
- `Events` logs the latest framework event and tick counter.
- `Ctrl+C` exits immediately (handled inside the runtime loop).

## Architectural overview

```
┌─────────────┐        render()        ┌─────────────────┐        draw()        ┌─────────────┐
│ Components  │ ─────────────────────▶ │ Virtual Elements │ ───────────────────▶ │ Renderer     │
└─────────────┘        hooks           └─────────────────┘        View tree      └─────────────┘
       ▲                                                        │
       │ framework events                                       ▼
┌─────────────┐  publish/subscribe   ┌─────────────┐   AppMessage loop   ┌──────────────┐
│ Dispatcher  │ ◀──────────────────▶ │ Event Bus   │ ◀──────────────────▶ │ Runtime (App)│
└─────────────┘                     └─────────────┘                      └──────────────┘
```

### Source layout

| File | Responsibility |
| --- | --- |
| `src/runtime/mod.rs` | App lifecycle, reconciler, dispatcher, terminal/tick/shutdown tasks, element definitions. |
| `src/hooks/mod.rs` | Hook registry, `Scope`, `use_state`, `use_effect`, context access. |
| `src/context/mod.rs` | Type-safe provider stack (push/pop via guards). |
| `src/events/mod.rs` | Framework event definitions, broadcast bus, Ctrl+C detection. |
| `src/renderer/mod.rs` | `ratatui` adapter that turns `View` structs into widgets. |
| `src/main.rs` | Demo app wiring components together. |

## Components and elements

Rustact components are plain functions that receive a mutable [`Scope`](../src/hooks/mod.rs) and return an [`Element`](../src/runtime/mod.rs).

```rust
use rustact::{component, Element, Scope};
use rustact::runtime::Color;

fn hero(ctx: &mut Scope) -> Element {
    let accent = ctx
        .use_context::<Theme>()
        .map(|theme| theme.accent)
        .unwrap_or(Color::White);

    Element::vstack(vec![
        Element::colored_text("Welcome", accent),
        Element::text("Press '+' / '-' to change the counter"),
    ])
}

let app = App::new("HeroDemo", component("Hero", hero));
```

`Element` is the virtual tree description. Built‑ins include:
- `Element::text / colored_text` for plain lines.
- `Element::vstack / hstack` to arrange children in rows/columns (`FlexDirection`).
- `Element::block` for bordered panels (mapped to `ratatui::widgets::Block`).
- `Element::list(ListNode)` for highlighted feeds built on `ratatui::widgets::List`.
- `Element::gauge(GaugeNode)` for progress bars backed by `ratatui::widgets::Gauge`.
- `Element::button(ButtonNode)` for clickable controls; the renderer records mouse hitboxes so handlers can react to button presses.
- `Element::table(TableNode)` for multi-column data grids rendered through `ratatui::widgets::Table`.
- `Element::tree(TreeNode)` for hierarchical explorers that render as indented lists.
- `Element::form(FormNode)` for key/value summaries with validation-aware styling.
- `Element::text_input(TextInputNode)` for focusable, styled, and optionally secure fields bound to component state.
- `Element::fragment` for lightweight wrappers without their own view node.
- `component("Name", render_fn)` to embed another component in the tree.

```rust
use rustact::{Element, GaugeNode, ListItemNode, ListNode};
use rustact::runtime::Color;

let recent = vec![
    ListItemNode::new("Tick").color(Color::Yellow),
    ListItemNode::new("Key: Char('q')").color(Color::Blue),
];

Element::vstack(vec![
    Element::list(ListNode::new(recent).title("Recent events")),
    Element::gauge(GaugeNode::new(0.42).label("Workload").color(Color::Green)),
]);
```

### Advanced builders

Tables, trees, and forms share a fluent builder API so you can compose them quickly:

```rust
use rustact::{
    Element, FormFieldNode, FormFieldStatus, FormNode, TableCellNode, TableNode, TableRowNode,
};
use rustact::runtime::Color;

let services = TableNode::new(vec![
    TableRowNode::new(vec![
        TableCellNode::new("api").bold(),
        TableCellNode::new("Healthy").color(Color::Green),
    ]),
])
.header(TableRowNode::new(vec![
    TableCellNode::new("Service").bold(),
    TableCellNode::new("Status").bold(),
]))
.widths(vec![40, 60]);

let release = FormNode::new(vec![
    FormFieldNode::new("Environment", "production"),
    FormFieldNode::new("Smoke tests", "failing").status(FormFieldStatus::Error),
]);

Element::fragment(vec![
    Element::table(services),
    Element::form(release),
]);
```

## Hooks & scope

Hooks live in `src/hooks/mod.rs` and are tracked per component via `HookRegistry`. The ordering rules mirror React: call hooks in the same order every render.

### `use_state`

```rust
let (count, set_count) = ctx.use_state(|| 0);
set_count.update(|value| *value += 1);
set_count.set(0);
```

`StateHandle` clones cheaply and schedules a render whenever you call `set` or `update`.

### `use_effect`

```rust
ctx.use_effect((), move |dispatcher| {
    let mut events = dispatcher.events().subscribe();
    let handle = tokio::spawn(async move {
        while let Ok(event) = events.recv().await {
            // respond to FrameworkEvent values
        }
    });
    Some(Box::new(move || handle.abort()))
});
```

- The first argument is the dependency payload (`impl PartialEq`).
- Returning `Some(cleanup)` lets you tear down async tasks before re-running the effect or when the component unmounts.

### Context helpers

`ContextStack` stores type-erased values keyed by `TypeId`.

```rust
let _theme_guard = ctx.provide_context(Theme { accent: Color::Cyan });
let theme = ctx.use_context::<Theme>();
```

The guard drops automatically at the end of the component render, ensuring providers unwind in LIFO order.

### `use_memo`

Cache expensive computations behind dependency keys. The hook only recomputes when the provided dependency payload changes (based on `PartialEq`). It returns an `Arc<T>` so you can cheaply clone pointers to large data.

```rust
use std::sync::Arc;

let palette = ctx.use_memo(theme_version, || generate_palette(theme_version));
let colors: Arc<Palette> = palette.clone();
```

### `use_callback`

`use_callback` is a convenience wrapper over `use_memo` for function values. It keeps a stable `Arc<dyn Fn>` reference unless its dependencies change, which is handy when passing handlers down the tree.

```rust
use crossterm::event::KeyCode;
use rustact::FrameworkEvent;

let key_char = props.key;
let on_key = ctx.use_callback(key_char, move || {
    move |event: &FrameworkEvent| {
        if matches!(event, FrameworkEvent::Key(key) if key.code == KeyCode::Char(key_char)) {
            dispatch_action();
        }
    }
});

child_component::props(on_key.clone());
```

### `use_reducer`

`use_reducer` mirrors React’s API: provide an initializer and a reducer function, and it returns the current value plus a dispatch handle that schedules renders when you send actions.

```rust
#[derive(Clone, Copy)]
enum Action {
    Increment,
    Decrement,
    Reset,
}

let (count, dispatch) = ctx.use_reducer(|| 0i32, |state, action: Action| match action {
    Action::Increment => *state += 1,
    Action::Decrement => *state -= 1,
    Action::Reset => *state = 0,
});

dispatch.dispatch(Action::Increment);
```

### `use_ref`

`use_ref` stores a mutable value without triggering re-renders. Think of it as an imperative handle—perfect for counters, cached layouts, or interop with external APIs.

```rust
let total_events = ctx.use_ref(|| 0usize);

ctx.use_effect((), move |dispatcher| {
    let tracker = total_events.clone();
    // ...subscribe to events...
    tracker.with_mut(|count| *count += 1);
    None
});

let seen = total_events.with(|count| *count);
Element::text(format!("Events processed: {seen}"));
```

### `use_text_input`

`use_text_input` registers a focusable binding that keeps text input state outside of component re-renders. The returned `TextInputHandle` exposes helpers such as `value()`, `set_value()`, `cursor()`, `set_cursor()`, `focus()`, and `snapshot()` for downstream hooks or effects.

```rust
let token = ctx.use_text_input("feedback:token", String::new);
let token_field = TextInputNode::new(token.clone())
    .label("API token")
    .placeholder("Optional secret")
    .secure(true)
    .width(36)
    .accent(Color::Yellow);

Element::text_input(token_field);
```

The shared `TextInputs` registry tracks hitboxes, so clicking anywhere inside the field focuses it, and Tab/Shift+Tab move through inputs in declaration order. Focused fields display a blinking caret, and secure inputs render placeholder glyphs instead of the raw value.

### `use_text_input_validation`

Pair `use_text_input` with `use_text_input_validation` to derive `FormFieldStatus` values that drive border and helper-text styling.

```rust
let email = ctx.use_text_input("feedback:email", String::new);
let email_status = ctx.use_text_input_validation(&email, |snapshot| {
    let trimmed = snapshot.value.trim();
    if trimmed.is_empty() {
        FormFieldStatus::Normal
    } else if trimmed.contains('@') {
        FormFieldStatus::Success
    } else {
        FormFieldStatus::Error
    }
});

Element::text_input(TextInputNode::new(email).status(email_status));
```

The hook stores the computed status on the handle so the runtime and renderer prefer it over any static `.status(...)` assigned to the node. You can also call `handle.set_status(...)` or `handle.clear_status()` manually—for example, after an async availability check completes.

### Text input lifecycle

- **Rendering**: `TextInputNode` carries styling (accent/border/text/placeholder/focus colors), layout (`width`, labels), and secure mode flags. During reconciliation the runtime clones a `TextInputSnapshot` so validation logic can read the value, cursor offset, and latest status.
- **Focus & cursor**: The `TextInputs` singleton stores hitboxes each frame. Mouse clicks toggle focus, Tab cycles between registered IDs, and a 250ms tick flips a shared `cursor_visible` flag to create a blinking caret. Blurs hide the caret.
- **Status coloring**: The renderer calls `status_to_color` to map `FormFieldStatus::{Normal,Warning,Error,Success}` into accent colors used for the border, label, and cursor. Live statuses from validation hooks immediately change those colors without rebuilding the node.
- **Secure mode**: `.secure(true)` masks the value when painting, but snapshots still expose the underlying text so validation or submission logic can operate on the same data.

## Events & dispatcher

`EventBus` is a Tokio `broadcast::channel` shared across the runtime. Every keyboard, mouse, resize, and tick event is published as a `FrameworkEvent`. `Ctrl+C` is detected in `events::is_ctrl_c` and triggers an app shutdown.

The [`Dispatcher`](../src/runtime/mod.rs) offers:
- `request_render()` – schedule a render without waiting for the main loop.
- `events()` – clone of the `EventBus` for hook/effect code.

The demo’s counter listens for `KeyCode::Char('+')`, `'-'`, and `r`, updating its state handles accordingly.

## Runtime pipeline

`App::run` (in `src/runtime/mod.rs`):

1. Spawns three async tasks (via the pluggable `RuntimeDriver`—swap in mocks with `App::with_driver` when testing):
    - `spawn_terminal_events` – wraps `crossterm::event::EventStream`, converts to `FrameworkEvent`, and issues `AppMessage::ExternalEvent`. Detects Ctrl+C, routes mouse clicks into the button/input hitbox registries, and requests shutdown.
   - `spawn_tick_loop` – emits `FrameworkEvent::Tick` at `AppConfig::tick_rate` (default 250ms).
   - `spawn_shutdown_watcher` – listens for OS-level `tokio::signal::ctrl_c` as a fallback.
2. Enters an `mpsc::Receiver<AppMessage>` loop. On `RequestRender`:
   - Clears the `live_components` set and builds a fresh `ContextStack`.
   - Recursively traverses the `Element` tree, building `View` structs (`TextView`, `FlexView`, `BlockView`).
    - Collects hook `EffectInvocation`s per component.
    - Compares the `View` tree with the previous frame; only invokes the renderer when the tree changed.
   - Flushes pending effects (`run_effects`).
   - Prunes hook stores for unmounted components.
3. On `ExternalEvent`, publishes it on the `EventBus` so subscribers react (e.g., the Stats panel updates its list whenever a new `FrameworkEvent` arrives).
4. On `Shutdown`, breaks the loop, drops the renderer (restoring the terminal), and aborts the helper tasks.

## Renderer

`src/renderer/mod.rs` adapts a `View` tree to `ratatui` widgets:
- Enters alternate screen, hides cursor, enables mouse capture.
- Recursively renders views using `Layout` for flex nodes, `Paragraph` for text, `List`/`ListState` for feeds, `Gauge` for progress bars, `Table`/`Tree` widgets for structured data, and a custom `TextInputWidget` that draws labels, placeholders, secure glyphs, focus backgrounds, borders, and the blinking caret.
- Registers hitboxes for every button and input so later mouse events can be matched back to view IDs.
- `Drop` impl restores the terminal (disables raw mode, leaves alt screen).

This layer is intentionally tiny so you can swap in richer widgets or adopt another backend later.

## Writing your own app

1. **Create components** in any module, returning `Element` values.
2. **Compose them** inside a top-level component and pass to `App::new`.
3. **Run the app** inside `#[tokio::main]` (or `tokio::runtime::Builder`) so hooks can spawn async work.

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = App::new("MyApp", component("Root", root));
    app.run().await
}
```

### Handling input

Subscribe to the dispatcher’s event bus from an effect:

```rust
ctx.use_effect((), move |dispatcher| {
    let mut events = dispatcher.events().subscribe();
    let handle = tokio::spawn(async move {
        while let Ok(FrameworkEvent::Key(key)) = events.recv().await {
            if key.code == KeyCode::Char('x') {
                // do something
            }
        }
    });
    Some(Box::new(move || handle.abort()))
});
```

Need to wire mouse clicks to a specific button? Give the button a stable ID and test for it inside your handler:

```rust
const SAVE_BUTTON_ID: &str = "toolbar:save";

Element::button(ButtonNode::new(SAVE_BUTTON_ID, "Save").accent(Color::Green));

if is_button_click(&event, SAVE_BUTTON_ID) {
    persist_state();
}
```

### Custom tick rate

```rust
use rustact::{App, AppConfig};
use std::time::Duration;

let config = AppConfig { tick_rate: Duration::from_millis(100) };
let app = App::new("FastTicks", root_component).with_config(config);
```

## Extending the framework

- **New hooks**: add storage variants to `HookSlot` and expose convenience methods on `Scope` (e.g., `use_memo`).
- **Advanced layout**: enrich `View` with additional widgets and implement them in the renderer.
- **Testing**: because components are pure functions, you can call them with a fake `Scope` or snapshot the `Element` tree for assertions.
- **Performance**: introduce diffing between old/new `View` trees to avoid redrawing the entire screen every render.

## Useful references

- [`src/main.rs`](../src/main.rs) – concrete example tying everything together.
- [`src/hooks/mod.rs`](../src/hooks/mod.rs) – hook lifecycle implementation details.
- [`ratatui` docs](https://docs.rs/ratatui/latest/ratatui/) – widget and layout primitives.
- [`crossterm` docs](https://docs.rs/crossterm/latest/crossterm/) – keyboard/mouse event reference.

Feel free to copy this guide into your own project wiki or extend it with additional recipes as the framework evolves.
