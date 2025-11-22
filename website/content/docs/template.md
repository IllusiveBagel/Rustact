+++
title = "Starter Template"
description = "Scaffold a Rustact application with the bundled cargo-generate template."
weight = 60
template = "doc.html"
updated = 2025-11-21
+++

# Starter Template Outline

Goal: provide a `cargo generate` template (or starter repo) that bootstraps a Rustact project with sane defaults. The template now lives in `templates/rustact-app/` inside this repository.

## Target structure

```
{{project-name}}/
├─ Cargo.toml (depends on `rustact`, `tokio`, `anyhow`, `ratatui`)
├─ README.md (quick-start instructions for the generated app)
├─ src/
│  ├─ main.rs (Tokio entrypoint, loads the stylesheet, mounts the root component)
│  └─ components/
│     └─ root.rs (example component using hooks, text inputs, and widgets)
└─ styles/
    └─ app.css (placeholders for :root tokens, buttons, inputs, embedded via `include_str!`)
```

## Template contents

-   `template.toml` describing placeholders (`project-name`, `author_name`, `app_description`).
-   `Cargo.toml` wired to `rustact` (git dependency targeting `main` for now) plus `anyhow`, `tokio`, and `ratatui`.
-   `src/main.rs` bootstrapping Tokio, parsing `styles/app.css` with `Stylesheet::parse(include_str!("../styles/app.css"))`, and running the root component.
-   `src/components/root.rs` demo component showcasing hooks, text inputs, and button interactions via the dispatcher.
-   `styles/app.css` with sensible defaults for buttons/inputs, matching the selectors documented in the styling guide.
-   `README.md` explaining how to run, test, and customize the generated app (no separate `docs/` folder is emitted).

## Usage

Install `cargo-generate` and point it at the template:

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

The generated `Cargo.toml` intentionally points at the GitHub repo so new apps track `main`. Swap it for the crates.io release (e.g., `rustact = "0.1"`) once you want a pinned dependency. Pair the template with the [tutorial](/docs/tutorial/) and [developer guide](/docs/guide/) to keep building.
