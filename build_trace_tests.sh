#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "=== Building Rust WASM apps ==="
"${ROOT_DIR}/build_apps.sh"

echo ""
echo "=== Building Rust trace harness ==="
cargo build -p fri3d-trace-harness --release

TRACE_BIN_DIR="${ROOT_DIR}/build/trace/bin"
mkdir -p "${TRACE_BIN_DIR}"
cp "${ROOT_DIR}/target/release/trace_harness" "${TRACE_BIN_DIR}/trace_harness"

echo ""
python3 "${ROOT_DIR}/tests/trace/run_trace_tests.py"
