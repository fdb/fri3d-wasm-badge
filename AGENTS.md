# Fri3d WASM Badge — Agent Guidelines

See [CLAUDE.md](CLAUDE.md) for the canonical workflow. Short version:

- **Everything is Rust.** Kernel in `fri3d-kernel/` (`no_std`), hosts in
  `hosts/`, apps in `apps/`, bundler in `tools/fri3d-pack/`.
- **Cheapest loop first:** `cargo test -p fri3d-kernel` → headless desktop
  screenshot → browser `test.html` → badge.
- **Design docs:** `design_docs/` explains the limits, the bundle format,
  the lifecycle and why. Read before changing the kernel.

## Common commands

```bash
cargo run -p fri3d-pack                                # build + bundle apps
cargo run --release -p fri3d-host-desktop              # desktop window
cargo run --release -p fri3d-host-desktop -- --headless --app snake \
    --keys ok,down --screenshot out.png                # scripted screenshot
cargo test -p fri3d-kernel                             # kernel tests
hosts/web/build.sh && (cd hosts/web/dist && python3 -m http.server 8091)
hosts/badge/flash.sh                                   # build + flash badge
```

## Commit style

Short imperative titles ("Add X", "Port Y", "Fix Z"), one-line subject + optional
paragraph explaining the *why*. Ship with `/ship` — main branch doesn't need PRs.
