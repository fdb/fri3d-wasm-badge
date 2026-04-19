#!/usr/bin/env bash
# Build the native WASM app runner. Links firmware's canvas/random/wasm_host
# against wasm3 and stb_image_write, producing a standalone CLI.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT"

WASM3="firmware/lib/wasm3/src"

c++ -std=c++17 -O2 \
    -I "$WASM3" \
    -I firmware/src \
    -Wno-unused-variable \
    -Wno-unused-parameter \
    -Wno-unused-function \
    "$WASM3"/m3_api_libc.c \
    "$WASM3"/m3_bind.c \
    "$WASM3"/m3_code.c \
    "$WASM3"/m3_compile.c \
    "$WASM3"/m3_core.c \
    "$WASM3"/m3_emit.c \
    "$WASM3"/m3_env.c \
    "$WASM3"/m3_exec.c \
    "$WASM3"/m3_function.c \
    "$WASM3"/m3_info.c \
    "$WASM3"/m3_module.c \
    "$WASM3"/m3_optimize.c \
    "$WASM3"/m3_parse.c \
    firmware/src/canvas.cpp \
    firmware/src/font.cpp \
    firmware/src/random.cpp \
    firmware/src/screen.cpp \
    firmware/src/wasm_host.cpp \
    firmware/tools/app-runner/app_runner.cpp \
    -o firmware/tools/app-runner/app_runner

echo "[app-runner] built firmware/tools/app-runner/app_runner"
