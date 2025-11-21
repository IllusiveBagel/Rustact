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

- Rust stable toolchain (`rustup show`)
- `cargo` for building/running
- A terminal with mouse support (optional but recommended)

## 1. Scaffold a new binary crate

```bash
cargo new hello-rustact --bin
cd hello-rustact
```

Add dependencies to `Cargo.toml`:

```toml
[dependencies]
rustact = { path = "../rustact" } # use the local checkout while developing
anyhow = "1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

> Replace the `path` dependency with `rustact = "0.x"` once the crate is published.

## 2. Boot the runtime

Update `src/main.rs`:

```rust
use anyhow::Result;
use rustact::{component, App};
use rustact::runtime::Element;

fn root(_ctx: &mut rustact::Scope) -> Element {
    Element::text("Hello, Rustact!")
}

#[tokio::main]
async fn main() -> Result<()> {
    App::new("HelloRustact", component("Root", root)).run().await
}
```

Run it:

```bash
cargo run
```

You should see a centered "Hello, Rustact!" message in the terminal. Exit with `Ctrl+C`.

## 3. Add state and buttons

Enhance `root` with buttons and a counter gauge:

```rust
use rustact::runtime::{ButtonNode, Element, GaugeNode};
use rustact::runtime::Color;

fn root(ctx: &mut rustact::Scope) -> Element {
    let (count, set_count) = ctx.use_state(|| 0i32);

    let decrement = {
        let set_count = set_count.clone();
        ctx.use_callback((), move || {
            let set_count = set_count.clone();
            move |_| set_count.update(|value| *value -= 1)
        })
    };
    let increment = {
        let set_count = set_count.clone();
        ctx.use_callback((), move || {
            let set_count = set_count.clone();
            move |_| set_count.update(|value| *value += 1)
        })
    };

    Element::vstack(vec![
        Element::text(format!("Count: {count}")),
        Element::gauge(
            GaugeNode::new((count.abs() as f64) / 10.0)
                .label(format!("Target ±10 ({count})"))
                .color(Color::Cyan),
        ),
        Element::button(ButtonNode::new("counter-minus", "-")),
        Element::button(ButtonNode::new("counter-plus", "+")),
    ])
}
```

Hook the callbacks up via an effect that watches the event bus (or handle key presses with `FrameworkEvent::Key`). For brevity, you can increment/decrement inside the button hitbox handler later.

## 4. Capture button clicks

Use the dispatcher’s event bus plus the `is_button_click` helper:

```rust
use rustact::{Dispatcher, FrameworkEvent};
use rustact::interactions::is_button_click;

ctx.use_effect((), move |dispatcher: Dispatcher| {
    let mut events = dispatcher.events().subscribe();
    let decrement = set_count.clone();
    let increment = set_count.clone();
    tokio::spawn(async move {
        while let Ok(event) = events.recv().await {
            if is_button_click(&event, "counter-minus") {
                decrement.update(|value| *value -= 1);
            } else if is_button_click(&event, "counter-plus") {
                increment.update(|value| *value += 1);
            }
        }
    });
    None
});
```

Now the rendered buttons respond to mouse clicks.

## 5. Add a text input with validation

```rust
use rustact::runtime::{FormFieldStatus, TextInputNode};

let name = ctx.use_text_input("profile:name", || String::new());
let name_status = ctx.use_text_input_validation(&name, |snapshot| {
    if snapshot.value.trim().is_empty() {
        FormFieldStatus::Warning
    } else {
        FormFieldStatus::Success
    }
});

let input = Element::text_input(
    TextInputNode::new(name.clone())
        .label("Display name")
        .placeholder("Rustacean")
        .status(name_status)
);
```

Append `input` to the VStack. Tab between the field and buttons, or click directly to focus.

## 6. Style it

Create `styles/app.css`:

```css
:root {
  --accent-color: #00e5ff;
}

button#counter-plus { accent-color: #5be7ff; --filled: true; }
button#counter-minus { accent-color: #ff6b6b; }
input#profile:name {
  accent-color: #f5f5f5;
  --border-color: #00e5ff;
  --background-color: #001f2f;
}
```

Load it in `main`:

```rust
use rustact::styles::Stylesheet;

let stylesheet = Stylesheet::parse(include_str!("../styles/app.css"))?;
App::new("HelloRustact", component("Root", root))
    .with_stylesheet(stylesheet)
    .run()
    .await
```

## 7. Wire keyboard shortcuts

Augment the earlier effect to listen for `FrameworkEvent::Key` events and adjust the counter when users press `+`, `-`, or `r`.

```rust
use crossterm::event::KeyCode;

if let FrameworkEvent::Key(key) = event {
    match key.code {
        KeyCode::Char('+') => increment.update(|value| *value += 1),
        KeyCode::Char('-') => decrement.update(|value| *value -= 1),
        KeyCode::Char('r') => set_count.set(0),
        _ => {}
    }
}
```

## 8. Test & iterate

Run the full suite to ensure upstream changes stay healthy:

```bash
cargo test
```

From here you can:
- Add a `ListView` that logs every event.
- Persist state to disk using `tokio::fs` inside an effect.
- Break components into separate modules and reuse them across multiple screens.

Congrats—you now have a custom Rustact-powered TUI! Expand it following the [roadmap](/docs/roadmap/) and reference documents like the [developer guide](/docs/guide/), [architecture walkthrough](/docs/architecture/), and [styling reference](/docs/styling/) whenever you need deeper detail.
