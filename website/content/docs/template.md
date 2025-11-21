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
├─ Cargo.toml (depends on `rustact`, `tokio`, `anyhow`)
├─ src/
│  ├─ main.rs (tokio entrypoint, loads stylesheet, mounts root component)
│  └─ components/
│     └─ root.rs (example component using hooks + inputs)
├─ styles/
│  └─ app.css (placeholders for :root tokens, buttons, inputs)
├─ docs/
│  └─ README.md (quick start instructions for the generated app)
└─ .cargo/config.toml (optional: set `rustflags` or strip settings)
```

## Template contents

- `template.toml` describing placeholders (`project-name`, `author_name`, `app_description`).
- `Cargo.toml` wired to `rustact` (git dependency for now) plus `anyhow`, `tokio`, `ratatui`.
- `src/components/root.rs` demo component showcasing hooks, text inputs, and button interactions via the dispatcher.
- `styles/app.css` with sensible defaults for buttons/inputs.
- `README.md` explaining how to run, test, and customize the generated app.

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

After the crate is published to crates.io you can swap the dependency in the generated `Cargo.toml` to the released version. Pair the template with the [tutorial](/docs/tutorial/) and [developer guide](/docs/guide/) to keep building.
