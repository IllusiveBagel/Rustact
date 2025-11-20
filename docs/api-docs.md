# API Documentation Publishing Guide

Rustact already builds local API docs via `cargo doc --open`. This guide records the extra steps required to share them publicly (e.g., GitHub Pages).

## 1. Generate docs locally

```bash
cargo doc --no-deps
```

Artifacts land in `target/doc/`. Add `--open` while iterating.

## 2. Automated GitHub Pages workflow

Rustact ships `.github/workflows/publish-docs.yml`, which:

1. Runs on every push to `main` (plus manual dispatch).
2. Builds docs with `cargo doc --no-deps` on Ubuntu.
3. Uploads `target/doc` as a Pages artifact.
4. Deploys via `actions/deploy-pages`, exposing them at `https://<owner>.github.io/rustact/`.

First-time setup: in GitHub → Settings → Pages, set the source to "GitHub Actions". Subsequent pushes redeploy automatically.

## 3. Manual fallback (optional)

If you ever need to publish by hand:

1. Ensure the repo has a `gh-pages` branch (or enable Pages to use `docs/`).
2. Run `cargo doc --no-deps`.
3. Copy `target/doc` to a staging folder (e.g., `docs/book`).
4. Commit the generated files to the Pages branch and push.
5. Configure Pages to serve from that branch/folder.

## 4. Track updates

- Whenever the public API changes, rerun the workflow to keep docs current.
- Add a README badge or link pointing to the hosted docs (`https://illusivebagel.github.io/rustact/rustact/`).
