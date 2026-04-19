#!/usr/bin/env bash
# Rebuild a Rust WASM app in release mode, shrink it with wasm-opt, and
# emit firmware/src/embedded_app.h for inclusion in the Arduino sketch.
#
# Usage: firmware/scripts/embed_app.sh [app-name]
#   app-name defaults to "fri3d-app-circles".
#
# Requires: cargo, rustup target wasm32-unknown-unknown, wasm-opt (binaryen).

set -euo pipefail

APP="${1:-fri3d-app-circles}"
# Cargo emits targets named with underscores, not hyphens.
APP_UNDERSCORE="${APP//-/_}"

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO_ROOT"

echo "[embed] cargo build --release -p $APP --target wasm32-unknown-unknown"
cargo build --release -p "$APP" --target wasm32-unknown-unknown

RAW_WASM="target/wasm32-unknown-unknown/release/${APP_UNDERSCORE}.wasm"
if [[ ! -f "$RAW_WASM" ]]; then
    echo "[embed] expected output not found: $RAW_WASM" >&2
    exit 1
fi

mkdir -p firmware/build
OPT_WASM="firmware/build/${APP_UNDERSCORE}_opt.wasm"

echo "[embed] wasm-opt -Oz --strip-debug $RAW_WASM -> $OPT_WASM"
wasm-opt -Oz --strip-debug -o "$OPT_WASM" "$RAW_WASM"

HEADER="firmware/src/embedded_app.h"
echo "[embed] emit $HEADER"
python3 - "$OPT_WASM" "$HEADER" <<'PY'
import pathlib, sys
src = pathlib.Path(sys.argv[1])
dst = pathlib.Path(sys.argv[2])
data = src.read_bytes()
rows = []
per_row = 12
for i in range(0, len(data), per_row):
    chunk = ', '.join(f'0x{b:02x}' for b in data[i:i+per_row])
    rows.append('    ' + chunk + ',')
if rows:
    rows[-1] = rows[-1].rstrip(',')
body = '\n'.join(rows)
dst.write_text(
    '// Auto-generated from ' + str(src) + ' by embed_app.sh.\n'
    '// Do not edit by hand.\n\n'
    '#pragma once\n\n'
    '#include <stdint.h>\n\n'
    f'static const uint8_t embedded_app_wasm[] = {{\n{body}\n}};\n'
    f'static const uint32_t embedded_app_wasm_len = {len(data)};\n'
)
PY

BYTES=$(wc -c < "$OPT_WASM" | tr -d ' ')
echo "[embed] done: $BYTES bytes embedded from $APP"
