+++
title = "Hands-on Tutorial"
description = "Build a mini Rustact-powered terminal dashboard from scratch in ~20 minutes."
weight = 20
template = "doc.html"
updated = 2025-11-21
+++

# Tutorial: Build a Mini Rustact App

This walkthrough creates a fresh Rustact-powered terminal dashboard from scratch. Expect to spend about 20 minutes.

## 0. Prerequisites

-   Rust stable toolchain (`rustup show`)
-   `cargo` for building/running
-   A terminal with mouse support (optional but recommended)

## 1. Scaffold a new binary crate

```bash
cargo new hello-rustact --bin
cd hello-rustact
```

Add dependencies to `Cargo.toml`:

```toml
[dependencies]
rustact = "0.1"
anyhow = "1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

> Working from a local clone of the Rustact repo? Temporarily swap in `rustact = { path = "../rustact" }` so your sample app uses the checked-out sources.

## 2. Boot the runtime

Update `src/main.rs`:

````rust
use anyhow::Result;
use rustact::{component, App};
use rustact::runtime::Element;

++
title = "Hands-on Tutorial"
description = "Use the Rustact template to spin up a working app, then extend it with state, events, inputs, and styles."
weight = 20
template = "doc.html"
updated = 2025-11-25
+++

# Template-Driven Quickstart

This consolidated guide replaces the separate template and tutorial docs. Follow it end-to-end to scaffold a new Rustact project, understand the generated files, and build a simple counter dashboard with buttons, text inputs, validation, and styling.

## 0. Prerequisites

- Rust toolchain 1.85+ (`rustup show`, `rustup update`, `rustup component add rustfmt clippy`).
- `cargo-generate` for bootstrapping from the template (`cargo install cargo-generate`).
- A terminal with ANSI + mouse support for the live demo captures.

## 1. Generate a project from the template

```bash
cargo install cargo-generate # skip if already installed
cargo generate \
    --git https://github.com/IllusiveBagel/rustact \
    --branch main \
    --path templates/rustact-app \
    --name hello-rustact
cd hello-rustact
````

The generated `Cargo.toml` points at the `main` branch via a git dependency so you always get the freshest APIs. Once a crates.io release fits your needs, swap that line for `rustact = "0.1"` (or higher).

## 2. Explore the layout

```
hello-rustact/
├─ Cargo.toml           # rustact + tokio + anyhow + ratatui
├─ README.md            # quick-start instructions for the generated app
├─ src/
│  ├─ main.rs           # Tokio entrypoint that loads the stylesheet
│  └─ components/
│     └─ root.rs        # primary component with hooks, inputs, buttons
└─ styles/
   └─ app.css           # :root tokens + widget selectors (embedded via include_str!)
```

Key files:

| Path                     | Purpose                                                                          |
| ------------------------ | -------------------------------------------------------------------------------- |
| `src/main.rs`            | Parses `styles/app.css`, builds the component tree, and runs `App`.              |
| `src/components/root.rs` | Holds the counter/text-input example using hooks, events, and inputs.            |
| `styles/app.css`         | CSS-inspired theme consumed by `ctx.styles()` queries or direct builder methods. |

## 3. Run the starter

```bash
cargo run
```

Expect to see a greeting, counter gauge, text input, and +/- buttons. Set `RUSTACT_WATCH_STYLES=1 cargo run` to hot-reload `styles/app.css` while the app runs.

## 4. Main entrypoint tweaks

`src/main.rs` ships with a minimal runner. Add stylesheet watching if you plan to iterate on CSS frequently:

```rust
use rustact::styles::Stylesheet;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let stylesheet = Stylesheet::parse(include_str!("../styles/app.css"))?;
    let mut app = App::new("HelloRustact", component("Root", root))
        .with_stylesheet(stylesheet);

    if std::env::var("RUSTACT_WATCH_STYLES").is_ok() {
        app = app.watch_stylesheet("styles/app.css");
    }

    app.run().await
}
```

## 5. Build the root component

Open `src/components/root.rs`. The template already includes a counter, text input, and gauge. Customize it—or recreate it from scratch—using the snippet below:

```rust
use rustact::interactions::is_button_click;
use rustact::runtime::{ButtonNode, Element, FlexDirection, FormFieldStatus, GaugeNode, TextInputNode};
use rustact::{FrameworkEvent, Scope};

pub fn root(ctx: &mut Scope) -> Element {
    let (count, set_count) = ctx.use_state(|| 0i32);
    let name = ctx.use_text_input("profile:name", || String::new());
    let name_status = ctx.use_text_input_validation(&name, |snapshot| {
        if snapshot.value.trim().is_empty() {
            FormFieldStatus::Warning
        } else {
            FormFieldStatus::Success
        }
    });

    ctx.use_effect((), move |dispatcher| {
        let mut events = dispatcher.events().subscribe();
        let decrement = set_count.clone();
        let increment = set_count.clone();
        let handle = tokio::spawn(async move {
            while let Ok(event) = events.recv().await {
                if is_button_click(&event, "counter-minus") {
                    decrement.update(|value| *value -= 1);
                } else if is_button_click(&event, "counter-plus") {
                    increment.update(|value| *value += 1);
                }
            }
        });
        Some(Box::new(move || handle.abort()))
    });

    Element::Flex(rustact::runtime::FlexNode {
        direction: FlexDirection::Column,
        children: vec![
            Element::text(format!("Hello, {}!", name.snapshot().value.trim())),
            Element::gauge(
                GaugeNode::new((count.abs() as f64) / 10.0)
                    .label(format!("Progress to ±10 ({count})")),
            ),
            Element::text_input(
                TextInputNode::new(name)
                    .label("Display name")
                    .placeholder("Rustacean")
                    .status(name_status),
            ),
            Element::Flex(rustact::runtime::FlexNode {
                direction: FlexDirection::Row,
                children: vec![
                    Element::button(ButtonNode::new("counter-minus", "-")),
                    Element::button(ButtonNode::new("counter-plus", "+")),
                ],
            }),
            Element::text(format!("Counter: {count}")),
        ],
    })
}
```

-   State & validation: `use_state` drives the counter; `use_text_input` + `use_text_input_validation` power the input.
-   Layout: `FlexDirection::Column` stacks the widgets; inner `FlexDirection::Row` aligns the +/- buttons.

## 6. Keyboard & mouse interactions

Extend the effect to react to key presses alongside mouse clicks:

```rust
use crossterm::event::KeyCode;
use rustact::FrameworkEvent;

ctx.use_effect((), move |dispatcher| {
    let mut events = dispatcher.events().subscribe();
    let decrement = set_count.clone();
    let increment = set_count.clone();
    let handle = tokio::spawn(async move {
        while let Ok(event) = events.recv().await {
            match event {
                FrameworkEvent::Key(key) => match key.code {
                    KeyCode::Char('-') => decrement.update(|value| *value -= 1),
                    KeyCode::Char('+') | KeyCode::Char('=') => increment.update(|value| *value += 1),
                    KeyCode::Char('r') => decrement.set(0),
                    _ => {}
                },
                _ => {
                    if is_button_click(&event, "counter-minus") {
                        decrement.update(|value| *value -= 1);
                    } else if is_button_click(&event, "counter-plus") {
                        increment.update(|value| *value += 1);
                    }
                }
            }
        }
    });
    Some(Box::new(move || handle.abort()))
});
```

## 7. Style the widgets

`styles/app.css` ships with placeholder selectors. Adjust them—or start from scratch:

```css
:root {
    --accent-color: #00e5ff;
    --warning-color: #f7b801;
}

button#counter-plus {
    accent-color: #5be7ff;
    --filled: true;
}
button#counter-minus {
    accent-color: #ff6b6b;
}
input#profile\:name {
    accent-color: #f5f5f5;
    --border-color: #00e5ff;
    --background-color: #001f2f;
}
```

-   Escaping the colon (`profile\:name`) matches the generated selector.
-   Keep `RUSTACT_WATCH_STYLES=1 cargo run` active to see changes instantly.

## 8. Verify & iterate

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
```

From here you can add lists, tables, modals, or toast stacks using the [widget catalogue](/docs/widgets/), wire async effects via the [architecture guide](/docs/architecture/), and keep styling consistent using the [styling reference](/docs/styling/).

Need distribution-ready screenshots? Follow the capture checklist in the widget catalogue, or script renders with `App::headless()` to dump ANSI frames for later conversion.
