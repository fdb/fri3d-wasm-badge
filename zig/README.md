# Zig Migration Sandbox

This directory contains the first Zig-native pieces of the badge stack.
The goal is to replace the current C/C++ code with a Zig-only toolchain
while keeping the platform interface stable for the existing runtimes.

## Layout

- `sdk/platform.zig` — Platform ABI contracts the ports must expose to WASM apps.
- `apps/circles/main.zig` — A minimal WASM app written in Zig that exercises the
  platform drawing surface and exports `render()`/`on_input()`.

## Building

These targets are designed to compile to `wasm32-wasi`.
Once Zig is available locally, build the Zig WASM apps with:

```bash
zig build apps
```

Artifacts will be placed under `zig-out/bin/` by default (the standard Zig
install location). Ports can then pick up the resulting `.wasm` files.
