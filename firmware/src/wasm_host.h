#pragma once

#include <stdint.h>

namespace fri3d { class Canvas; class Random; }

// Bridges the wasm3 interpreter to our Canvas + Random + Arduino runtime.
// Matches the host-import surface of fri3d-wasm-api so .wasm apps built for
// the desktop emulator run unmodified on the real badge.
namespace wasm_host {

// Initialize wasm3, load a WASM module from the given bytes, link all host
// functions, and resolve the render/on_input exports.
// The caller owns the bytes; they must outlive the runtime (wasm3 does not
// copy them). Returns nullptr on success, otherwise a static error description.
const char* init(fri3d::Canvas& canvas,
                 fri3d::Random& random,
                 const uint8_t* wasm_bytes,
                 uint32_t wasm_len);

// Tear down the current runtime (if any) and load a new module. Call this
// when the app requests start_app() or exit_to_launcher() to swap modules.
// Canvas and Random references are preserved from the previous init.
const char* reload(const uint8_t* wasm_bytes, uint32_t wasm_len);

// Call the app's render() export. No-op if init() hasn't succeeded.
void render();

// Call the app's on_input(key, kind) export, if it was exported.
void on_input(uint32_t key, uint32_t kind);

// True when exit_to_launcher() was invoked by the WASM app during the last
// on_input or render call. Consumers should poll + clear this flag.
bool exit_requested();
void clear_exit_request();

// True when start_app(id) was invoked. Paired with requested_app_id(), and
// cleared by clear_start_app_request().
bool start_app_requested();
uint32_t requested_app_id();
void clear_start_app_request();

// Input key/kind constants — duplicated here so main.cpp doesn't need to
// import wasm3 internals.
namespace key {
    constexpr uint32_t UP    = 0;
    constexpr uint32_t DOWN  = 1;
    constexpr uint32_t LEFT  = 2;
    constexpr uint32_t RIGHT = 3;
    constexpr uint32_t OK    = 4;
    constexpr uint32_t BACK  = 5;
}
namespace event {
    constexpr uint32_t PRESS       = 0;
    constexpr uint32_t RELEASE     = 1;
    constexpr uint32_t SHORT_PRESS = 2;
    constexpr uint32_t LONG_PRESS  = 3;
    constexpr uint32_t REPEAT      = 4;
}

} // namespace wasm_host
