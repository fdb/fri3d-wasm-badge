#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

ZIG_GLOBAL_CACHE_DIR="${ROOT_DIR}/build/zig-cache/global"
ZIG_LOCAL_CACHE_DIR="${ROOT_DIR}/build/zig-cache/local"

mkdir -p "${ZIG_GLOBAL_CACHE_DIR}" "${ZIG_LOCAL_CACHE_DIR}"

cmake -B "${ROOT_DIR}/build/trace" -DBUILD_TRACE_TESTS=ON -DBUILD_EMULATOR=OFF
cmake --build "${ROOT_DIR}/build/trace"

for app in circles test_drawing mandelbrot test_ui; do
  echo "Building ${app}..."
  ZIG_GLOBAL_CACHE_DIR="${ZIG_GLOBAL_CACHE_DIR}" ZIG_LOCAL_CACHE_DIR="${ZIG_LOCAL_CACHE_DIR}" \
    cmake -B "${ROOT_DIR}/build/apps/${app}" \
      -DCMAKE_TOOLCHAIN_FILE="${ROOT_DIR}/cmake/toolchain-wasm.cmake" \
      -S "${ROOT_DIR}/src/apps/${app}"
  ZIG_GLOBAL_CACHE_DIR="${ZIG_GLOBAL_CACHE_DIR}" ZIG_LOCAL_CACHE_DIR="${ZIG_LOCAL_CACHE_DIR}" \
    cmake --build "${ROOT_DIR}/build/apps/${app}"
done

python3 "${ROOT_DIR}/tests/trace/run_trace_tests.py"
