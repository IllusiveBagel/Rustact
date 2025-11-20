# {{ project-name | upper }}

{{ app_description }}

## Prerequisites

- Rust stable toolchain (`rustup show`)
- `cargo`

## Getting started

```bash
cargo run
```

While running:
- Use `+`, `-`, or click the on-screen buttons to adjust the counter.
- Type in the "Display name" field; validation switches from warning to success once non-empty.
- Press `r` to reset the counter.

## Useful commands

```bash
cargo fmt
cargo clippy
cargo test
```

## Customizing

- Update `src/components/root.rs` to tailor the layout, hooks, and interactions.
- Edit `styles/app.css` to change colors, labels, and button fills.
- Rename the root component in `src/main.rs` or split additional components into `src/components/`.
