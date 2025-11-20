# Styling Terminal UIs

Rustact can reskin a terminal UI without recompilation by loading a lightweight CSS-inspired stylesheet at runtime. The demo app (see `src/main.rs`) calls `Stylesheet::parse(include_str!("../styles/demo.css"))` and passes the result into `App::with_stylesheet()`. Every component receives a `Scope` that exposes the shared stylesheet, so widgets can query CSS rules as they render.

## Selector model

Only a small subset of CSS is implemented today, optimized for predictable terminal styling:

| Feature | Supported? | Notes |
| --- | --- | --- |
| Type selectors (`hero`) | ✅ | Matches by element name supplied when querying. |
| ID selectors (`button#counter-plus`) | ✅ | Each selector may include at most one `#id`. |
| Class selectors (`tip.context`) | ✅ | Each selector may include at most one `.class`. |
| Combined selectors (`hero.highlighted`) | ✅ | Element + optional ID + optional class. |
| Descendant / combinators | ❌ | Not yet supported. |
| Pseudo selectors | ❌ | Not supported. |

Rules follow standard CSS precedence: IDs outrank classes, which outrank type selectors. When specificity ties, later rules win. The special `:root` selector is also supported; values defined there are merged into every computed style.

## Supported property types

The parser normalizes property names to lowercase and keeps values as strings, but the `ComputedStyle` helper exposes typed accessors:

- `color("name")` parses named colors (`red`, `cyan`, `gray`), hex codes (`#04b5ff`, `#0bf`), or `rgb(r,g,b)`. Used for foreground/text colors and accent fills.
- `bool("name")` interprets `true/false`, `yes/no`, `on/off`, `1/0`.
- `u16("name")`, `f64("name")` parse numeric values for sizing.
- `list_u16("name")` accepts whitespace- or comma-separated integers, handy for table column widths.
- `text("name")` returns the raw string (useful for labels).

Properties that begin with `--` are treated exactly like regular keys—the prefix simply makes the CSS more idiomatic and avoids clashing with built-in color names.

## Built-in selectors and properties

The demo stylesheet (`styles/demo.css`) illustrates the selectors Rustact currently consumes:

| Selector | Purpose | Properties read by the code |
| --- | --- | --- |
| `:root` | Global theme tokens shared via context. | `--accent-color`, `--warning-color`, `--success-color`, `--danger-color`, `--info-color` |
| `hero` | Splash text block. | `color`, `--subtitle-color` |
| `panel#counter` | Counter instructions. | `color` |
| `button#counter-plus`, `button#counter-minus` | Counter buttons. | `accent-color`, `--filled` |
| `gauge#counter-progress` | Counter progress bar. | `color`, `--label` |
| `list#stats` | Recent events list. | `color`, `--highlight-color`, `--max-items` |
| `table#services` | Service health table. | `--column-widths` |
| `form#release` | Release checklist form. | `--label-width` |
| `input`, `input#feedback-name` | Text inputs (global + per-field overrides). | `accent-color`, `--border-color`, `color`, `--placeholder-color`, `--background-color`, `--focus-background` |
| `tip.keyboard`, `tip.mouse`, `tip.context` | Tip cards keyed by class. | `color` |

Add your own selectors and query them inside components by calling `ctx.styles().query(StyleQuery::element("element").with_id("id").with_classes(&classes))`.

Text inputs follow the same pattern as other widgets: query `input` selectors (optionally with an `#id`) and feed the computed colors into `TextInputNode` builder methods like `.accent(...)`, `.border_color(...)`, `.background_color(...)`, or `.placeholder_color(...)`. The renderer consumes those values to drive focus borders, cursor color, and placeholder contrast.

Validation logic can tint those inputs by pushing a [`FormFieldStatus`](../src/runtime/mod.rs) into the binding. Call `ctx.use_text_input_validation(&handle, |snapshot| { ... })` to derive a status from the current value, or invoke `handle.set_status(FormFieldStatus::Error)` directly when performing asynchronous checks. The renderer prefers the dynamic status over the static `.status(...)` builder setting, so validation hooks immediately impact border and label colors.

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

Save your changes and rerun `cargo run` to see the new palette, copy, or sizing reflected in the terminal.
