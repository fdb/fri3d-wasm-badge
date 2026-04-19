#!/usr/bin/env bash
# Compile the Fri3d firmware to WebAssembly for in-browser debugging.
#
# Emits firmware/web/dist/{firmware.js, firmware.wasm} + copies index.html
# alongside. Serve the `dist/` directory over HTTP to run:
#   cd firmware/web/dist && python3 -m http.server 8080
# Then open http://localhost:8080/.
#
# Also regenerates src/embedded_apps.h from the latest release builds of
# the Rust apps, so the browser runs the exact same .wasm bytes the badge
# does.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO_ROOT"

# Regenerate the embedded app bytes.
firmware/scripts/embed_apps.sh

DIST="firmware/web/dist"
mkdir -p "$DIST"

WASM3="firmware/lib/wasm3/src"

# -std is set per-file by the driver: .c -> C default, .cpp -> modern C++.
# We deliberately don't pass -std=c++17 globally because it errors on .c files.
emcc \
  -O2 \
  -I "$WASM3" \
  -I firmware/src \
  -Wno-unused-variable \
  -Wno-unused-parameter \
  -Wno-unused-function \
  -Wno-missing-field-initializers \
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
  firmware/src/app_switcher.cpp \
  firmware/src/canvas.cpp \
  firmware/src/font.cpp \
  firmware/src/input_queue.cpp \
  firmware/src/random.cpp \
  firmware/src/screen.cpp \
  firmware/src/wasm_host.cpp \
  firmware/web/main_web.cpp \
  -s ENVIRONMENT=web \
  -s ALLOW_MEMORY_GROWTH=1 \
  -s EXPORTED_FUNCTIONS='["_main","_web_init","_web_render","_web_on_input","_web_push_input","_web_fb","_web_fb_width","_web_fb_height","_web_screen","_web_screen_width","_web_screen_height","_web_rng_get","_web_rng_seed","_malloc","_free"]' \
  -s EXPORTED_RUNTIME_METHODS='["ccall","cwrap","HEAPU8"]' \
  -o "$DIST/firmware.js"

cp firmware/web/index.html "$DIST/index.html"
cp firmware/web/test.html "$DIST/test.html"
cp firmware/web/tests.js "$DIST/tests.js"

echo "[web] built $DIST/{firmware.js,firmware.wasm,index.html,test.html,tests.js}"
echo "[web] serve with: cd $DIST && python3 -m http.server 8080"
