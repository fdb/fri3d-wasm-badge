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
| [008](008-launcher-design.md) | Launcher and Settings: the home grid and the settings menu |
| [009](009-app-abi.md) | App ABI reference: every import, export, limit and rule |
| [010](010-host-contract.md) | Host contract, and what desktop / browser / badge taught us |
| [011](011-storage-and-ota.md) | Partition layout, settings persistence, OTA status, file-system decision |
| [012](012-bring-up-lessons.md) | Hardware bring-up lessons, chronological, with a checklist |
| [015](015-wifi.md) | Wi-Fi: kernel-owned model and auto-connect, host radio primitives, settings flow |
| [016](016-network-ops.md) | Network operations (probe, download) and the Speed Test app; IP stack on the badge |
| [017](017-color-ui.md) | Colour UI: 320×240 DB32 framebuffer, Pixelify fonts, tokens, icons, artwork pipeline |
