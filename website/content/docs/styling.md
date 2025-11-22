+++
title = "Styling Reference"
description = "Reskin Rustact TUIs at runtime with the lightweight CSS-inspired stylesheet system."
weight = 40
template = "doc.html"
updated = 2025-11-21
+++

# Styling Terminal UIs

Rustact can reskin a terminal UI without recompilation by loading a lightweight CSS-inspired stylesheet at runtime. The demo app (`examples/rustact-demo/src/main.rs`) runs `load_demo_stylesheet()`, which prefers `Stylesheet::from_file("styles/demo.css")` and falls back to `Stylesheet::parse(include_str!("../styles/demo.css"))`, then feeds the result to `App::with_stylesheet()`. Every component receives a `Scope` that exposes the shared stylesheet, so widgets can query CSS rules as they render. Set `RUSTACT_WATCH_STYLES=1` to have `App::watch_stylesheet("styles/demo.css")` reload the file automatically while the example is running.

## Selector model

Only a small subset of CSS is implemented today, optimized for predictable terminal styling:

| Feature                                 | Supported? | Notes                                           |
| --------------------------------------- | ---------- | ----------------------------------------------- |
| Type selectors (`hero`)                 | ✅         | Matches by element name supplied when querying. |
| ID selectors (`button#counter-plus`)    | ✅         | Each selector may include at most one `#id`.    |
| Class selectors (`tip.context`)         | ✅         | Each selector may include at most one `.class`. |
| Combined selectors (`hero.highlighted`) | ✅         | Element + optional ID + optional class.         |
| Descendant / combinators                | ❌         | Not yet supported.                              |
| Pseudo selectors                        | ❌         | Not supported.                                  |

Rules follow standard CSS precedence: IDs outrank classes, which outrank type selectors. When specificity ties, later rules win. The special `:root` selector is also supported; values defined there are merged into every computed style.

## Supported property types

The parser normalizes property names to lowercase and keeps values as strings, but the `ComputedStyle` helper exposes typed accessors:

-   `color("name")` parses named colors (`red`, `cyan`, `gray`), hex codes (`#04b5ff`, `#0bf`), or `rgb(r,g,b)`. Used for foreground/text colors and accent fills.
-   `bool("name")` interprets `true/false`, `yes/no`, `on/off`, `1/0`.
-   `u16("name")`, `f64("name")` parse numeric values for sizing.
-   `list_u16("name")` accepts whitespace- or comma-separated integers, handy for table column widths.
-   `text("name")` returns the raw string (useful for labels).

Properties that begin with `--` are treated exactly like regular keys—the prefix simply keeps the CSS idiomatic and avoids clashing with built-in color names.

## Built-in selectors and properties

The demo stylesheet (`examples/rustact-demo/styles/demo.css`) illustrates the selectors Rustact currently consumes:

| Selector                                      | Purpose                                     | Properties read by the code                                                                                  |
| --------------------------------------------- | ------------------------------------------- | ------------------------------------------------------------------------------------------------------------ |
| `:root`                                       | Global theme tokens shared via context.     | `--accent-color`, `--warning-color`, `--success-color`, `--danger-color`, `--info-color`                     |
| `hero`                                        | Splash text block.                          | `color`, `--subtitle-color`                                                                                  |
| `panel#counter`                               | Counter instructions.                       | `color`                                                                                                      |
| `button#counter-plus`, `button#counter-minus` | Counter buttons.                            | `accent-color`, `--filled`                                                                                   |
| `gauge#counter-progress`                      | Counter progress bar.                       | `color`, `--label`                                                                                           |
| `list#stats`                                  | Recent events list.                         | `color`, `--highlight-color`, `--max-items`                                                                  |
| `table#services`                              | Service health table.                       | `--column-widths`                                                                                            |
| `form#release`                                | Release checklist form.                     | `--label-width`                                                                                              |
| `input`, `input#feedback-name`                | Text inputs (global + per-field overrides). | `accent-color`, `--border-color`, `color`, `--placeholder-color`, `--background-color`, `--focus-background` |
| `tip.keyboard`, `tip.mouse`, `tip.context`    | Tip cards keyed by class.                   | `color`                                                                                                      |

Add your own selectors and query them inside components by calling:

```rust
use rustact::styles::StyleQuery;

let style = ctx
    .styles()
    .query(StyleQuery::element("button").with_id("counter-plus"));
let accent = style.color("accent-color").unwrap_or(Color::Cyan);
```

Text inputs follow the same pattern as other widgets: query `input` selectors (optionally with an `#id`) and feed the computed colors into `TextInputNode` builder methods like `.accent(...)`, `.border_color(...)`, `.background_color(...)`, or `.placeholder_color(...)`. The renderer consumes those values to drive focus borders, cursor color, and placeholder contrast.

Validation logic can tint those inputs by pushing a [`FormFieldStatus`](https://docs.rs/rustact/latest/rustact/runtime/enum.FormFieldStatus.html) into the binding. Call `ctx.use_text_input_validation(&handle, |snapshot| { ... })` to derive a status from the current value, or invoke `handle.set_status(FormFieldStatus::Error)` directly when performing asynchronous checks. The renderer prefers the dynamic status over the static `.status(...)` builder setting, so validation hooks immediately impact border and label colors.

## Example stylesheet

```css
:root {
    --accent-color: #00e8ff;
    --warning-color: #f7b801;
    --success-color: #7bd88f;
    --danger-color: #ff6b6b;
    --info-color: #7dd3fc;
}

button#counter-plus {
    accent-color: #5be7ff;
    --filled: true;
}

gauge#counter-progress {
    color: #5be7ff;
    --label: "Momentum to ±10";
}
```

Save your changes and rerun `cargo run` from inside the example (`cd examples/rustact-demo && cargo run`)—or keep `RUSTACT_WATCH_STYLES=1 cargo run` active to watch the stylesheet live—for instant feedback on palette, copy, or sizing tweaks. Pair this reference with the [developer guide](/docs/guide/) and [tutorial](/docs/tutorial/) when styling your own applications.
