# Rustact Examples

These standalone apps used to live inside the main `rustact` crate. They now sit under `examples/` so you can copy them into a separate repository (for example `rustact-examples`) without mixing demo code into the core runtime.

## Layout

- `rustact-demo/` – the original kitchen-sink showcase with counters, tables, trees, forms, and text inputs. Run it with `cargo run` from inside the folder.
- `ops-dashboard/` – the incident-response dashboard featuring tabs, overlays, modals, and toast stacks.

Each example is its own Cargo package that depends on the local `rustact` crate via a path dependency. If you plan to publish them in another repository, update the dependency to `rustact = "<version>"` once the crate is released, or keep the path dependency if you develop both repos side by side.

## Running an example

```bash
cd examples/rustact-demo
cargo run
```

Set `RUSTACT_WATCH_STYLES=1` before running to hot-reload the example stylesheet in `styles/demo.css`. The same command works for the ops dashboard example.
