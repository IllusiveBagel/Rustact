+++
title = "Widget Catalogue"
description = "Visual reference and usage examples for every built-in Rustact widget."
weight = 25
template = "doc.html"
updated = 2025-11-23
+++

# Widget Catalogue

This guide walks through the widgets that ship with Rustact, how to construct each node, where to hang styling hooks, and how to capture clean screenshots for your documentation. Pair these snippets with the [styling reference](/docs/styling/) to reskin each component in your demos.

> Need still images for the marketing site? Launch the demo or your own app with a consistent terminal theme, then use a region screenshot tool (e.g., `grim -g "$(slurp)" widgets/button.png`) to capture a cropped frame. Keep the terminal font and background aligned with the site palette so the screenshots blend seamlessly.

## Quick reference

| Widget          | Builder                                  | Styling selectors               | Notes                                        |
| --------------- | ---------------------------------------- | ------------------------------- | -------------------------------------------- |
| Text            | `Element::text`, `Element::colored_text` | `text`, `text#id`, `text.class` | Lightweight copy blocks or labels.           |
| Flex stacks     | `Element::vstack`, `Element::hstack`     | `hero`, `panel`, etc.           | Compose higher-level layouts (rows/columns). |
| Block           | `Element::block("Title", child)`         | `panel#counter`                 | Adds borders, titles, and padding.           |
| List            | `Element::list(ListNode)`                | `list#stats`                    | Great for log feeds or recent-events panels. |
| Gauge           | `Element::gauge(GaugeNode)`              | `gauge#counter-progress`        | Shows progress toward a target.              |
| Button          | `Element::button(ButtonNode)`            | `button#counter-plus`           | Registers hitboxes for mouse clicks.         |
| Table           | `Element::table(TableNode)`              | `table#services`                | Multi-column data with optional header.      |
| Tree            | `Element::tree(TreeNode)`                | `tree#files`                    | Hierarchical explorations.                   |
| Form            | `Element::form(FormNode)`                | `form#release`                  | Key/value summaries with statuses.           |
| Tabs            | `Element::tabs(TabsNode)`                | `tabs#overview`                 | Tabbed navigation for multiple panes.        |
| Layered layouts | `Element::layers(LayeredNode)`           | `layer#main`                    | Overlay UI like charts + modals.             |
| Modal           | `Element::modal(ModalNode)`              | `modal#incident`                | Centered cards for confirmations.            |
| Toast stack     | `Element::toast_stack(ToastStackNode)`   | `toast-stack#global`            | Growl-style notifications.                   |
| Text input      | `Element::text_input(TextInputNode)`     | `input#feedback-name`           | Focusable, validated fields.                 |

## Text & layout primitives

```rust
Element::block(
    "rustact demo",
    Element::vstack(vec![
        Element::colored_text("Welcome", Color::Cyan),
        Element::hstack(vec![
            Element::text("Left"),
            Element::text("Right"),
        ]),
    ]),
);
```

-   Use blocks to frame larger sections. The child element can be any other widget tree.
-   `Element::fragment` groups multiple siblings without injecting layout hints—handy inside lists or modals.

## Lists & gauges

```rust
use rustact::{Element, GaugeNode, ListItemNode, ListNode};
use rustact::runtime::Color;

let recent = vec![
    ListItemNode::new("Tick").color(Color::Yellow),
    ListItemNode::new("Key: Char('q')").color(Color::Blue),
];

Element::vstack(vec![
    Element::list(
        ListNode::new(recent)
            .title("Recent events")
            .highlight_color(Color::Cyan),
    ),
    Element::gauge(
        GaugeNode::new(0.4)
            .label("Momentum to ±10")
            .color(Color::Blue),
    ),
]);
```

Style hooks:

-   `list#stats` for borders, highlight color, and max items.
-   `gauge#counter-progress` for accent colors and labels.

## Buttons & interactions

```rust
use rustact::{ButtonNode, Element};
use rustact::runtime::Color;

Element::hstack(vec![
    Element::button(ButtonNode::new("counter-minus", "-")
        .accent(Color::Red)
        .filled(true)),
    Element::button(ButtonNode::new("counter-plus", "+")
        .accent(Color::Green)
        .filled(true)),
]);
```

-   Provide stable IDs so `is_button_click(event, id)` can route mouse events.
-   Use styles like `button#counter-plus { accent-color: #5be7ff; --filled: true; }` to theme them consistently.

## Tables & trees

```rust
use rustact::{Element, TableCellNode, TableNode, TableRowNode, TreeItemNode, TreeNode};
use rustact::runtime::Color;

let header = TableRowNode::new(vec![
    TableCellNode::new("Service").bold(),
    TableCellNode::new("Status").bold(),
]);
let rows = vec![
    TableRowNode::new(vec![
        TableCellNode::new("web"),
        TableCellNode::new("Healthy").color(Color::Green),
    ]),
];

let service_table = Element::table(
    TableNode::new(rows)
        .header(header)
        .title("Service health")
        .widths(vec![40, 60]),
);

let tree = Element::tree(
    TreeNode::new(vec![
        TreeItemNode::new("ops/")
            .child(TreeItemNode::new("incidents.rs"))
            .child(TreeItemNode::new("deploy.rs")),
    ])
    .title("Sources"),
);
```

-   Tables read `--column-widths` from styles; trees can highlight the active node via `.highlight(idx)`.
-   Use monospace fonts in screenshots so columns line up cleanly.

## Forms & status fields

```rust
use rustact::{Element, FormFieldNode, FormFieldStatus, FormNode};

let release = Element::form(
    FormNode::new(vec![
        FormFieldNode::new("Environment", "production"),
        FormFieldNode::new("Smoke tests", "failing")
            .status(FormFieldStatus::Error),
    ])
    .title("Release checklist")
    .label_width(18),
);
```

-   Pair with `form#release { --label-width: 18; }` to align labels and values.
-   Text inputs can supply live statuses through `ctx.use_text_input_validation` for richer forms.

## Tabs, layers, modals, and toasts

```rust
use rustact::{Element, LayeredNode, ModalNode, TabPaneNode, TabsNode, ToastLevel, ToastNode, ToastStackNode};

let tabs = Element::tabs(
    TabsNode::new(vec![
        TabPaneNode::new("overview", "Overview", Element::text("Ops")),
        TabPaneNode::new("incidents", "Incidents", Element::text("⚠")),
    ])
    .active("overview"),
);

let modal = Element::modal(
    ModalNode::new("incident-modal", Element::text("Resolve incident?"))
        .title("Incident #42"),
);

let toasts = Element::toast_stack(
    ToastStackNode::new("notifier", vec![
        ToastNode::new("deploy", ToastLevel::Info, "Deploy started"),
    ]),
);

let layered = Element::layers(
    LayeredNode::new(vec![tabs, toasts, modal]),
);
```

-   Tabs expect stable pane IDs; style them via `tabs#overview`, `tab-pane.incidents`, etc.
-   Layered layouts render children back-to-front—use them for toasts and modals on top of dashboards.

## Text inputs & validation

```rust
use rustact::runtime::{FormFieldStatus, TextInputNode};

let handle = ctx.use_text_input("feedback-email", || String::new());
let status = ctx.use_text_input_validation(&handle, |snapshot| {
    if snapshot.value.contains('@') {
        FormFieldStatus::Success
    } else {
        FormFieldStatus::Warning
    }
});

Element::text_input(
    TextInputNode::new(handle.clone())
        .label("Email")
        .placeholder("ops@example.com")
        .status(status),
);
```

Styling tips:

-   Target selectors like `input#feedback-email` for accent color, cursor color, placeholder tint, and focus background.
-   Secure fields call `.secure(true)` to mask the rendered value.

## Screenshot checklist

1. Launch the widget in a dedicated terminal window with the same background/foreground colors used on the website.
2. Resize the terminal so it wraps the widget tightly (e.g., `tput cols` / `tput lines` to confirm dimensions).
3. Use a region screenshot tool to capture just the widget and its label. Trim extra padding with ImageMagick (`convert shot.png -trim +repage output.png`).
4. Store the captures under `website/static/img/widgets/` (create as needed) and reference them inside this doc once you are ready to publish.

By keeping IDs consistent between code, styles, and screenshots, you can document each widget once and reuse the same capture across guides and release notes.
