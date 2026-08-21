# Design docs

Lessons learned and decisions for the Rust kernel. Each doc states the
source, the lesson, and how the kernel applies it. Read in order.

| Doc | Topic |
| --- | --- |
| [001](001-performance-lessons-tigerbeetle.md) | Performance engineering rules (from TigerBeetle / Tiger Style) |
| [002](002-micropythonos-app-model.md) | What MicroPythonOS does: manifests, activities, launcher |
| [003](003-badge-2026-hardware.md) | Fri3d 2026 badge hardware facts the firmware depends on |
| [004](004-wasm-runtime-choice.md) | Why wasmi, why esp-hal, and the AOT options |
| [005](005-kernel-architecture.md) | The kernel: state, loop, lifecycle, limits |
| [006](006-app-bundle-format.md) | The `.fab` app bundle and `manifest.toml` |
| [007](007-app-api-guidelines.md) | How the API steers app developers toward fast apps |
| [008](008-launcher-design.md) | Launcher and Settings: Flipper Zero look on a 160x120 canvas |
