# Rustact

Rustact is an experimental, React-inspired framework for building async terminal UIs in Rust. It layers a component/hook system on top of `ratatui`, offering familiar primitives such as components, state, effects, context, and an event bus.

## Features

- **Component tree & hooks** – Define components via `component("Name", |ctx| ...)` and manage state with `use_state`, `use_reducer`, `use_ref`, `use_effect`, `use_memo`, `use_callback`, and `provide_context` / `use_context`.
- **Async runtime** – Built on `tokio`, handling terminal IO, ticks, and effect scheduling without blocking the UI thread.
- **Event bus** – Subscribe to keyboard/mouse/resize/tick events via `ctx.dispatcher().events().subscribe()` and react within hooks or spawned tasks.
- **Ratatui renderer** – Converts the virtual tree into `ratatui` widgets, handling layout primitives like blocks, vertical/horizontal stacks, styled text, list panels, gauges, tables, tree views, forms, and rich text inputs.
- **View diffing** – The runtime caches the previous view tree and skips terminal draws when nothing has changed, keeping renders snappy even with frequent ticks.
- **Mouse interactions** – Define button or input nodes with stable IDs; the renderer automatically maps mouse hitboxes so event handlers can react to clicks and focus changes.
- **Text inputs & validation hooks** – `use_text_input` binds component state to editable fields (with cursor management, tab focus, and secure mode), while `use_text_input_validation` or `handle.set_status(...)` drive contextual border colors and helper text.
- **CSS-inspired styling** – Drop tweaks into `styles/demo.css` to recolor widgets, toggle button fills, rename gauge labels, change list highlight colors, resize form/table columns, or theme inputs without touching Rust code.
- **Demo app** – `src/main.rs` now composes every hook (state, reducer, ref, memo, callback, effect, context) with all widgets (text, flex, gauge, buttons, lists, tables, trees, forms) so you can see each feature in one place.

## Documentation

- `docs/guide.md` – day-to-day developer guide (setup, workflows, hook primer).
- `docs/tutorial.md` – step-by-step tutorial for building a fresh Rustact app.
- `docs/architecture.md` – deep dive into the runtime, hooks, renderer, and events.
- `docs/styling.md` – CSS subset reference and theming tips.
- `docs/roadmap.md` – prioritized initiatives to steer ongoing work.

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
```}