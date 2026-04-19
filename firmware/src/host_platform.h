// Platform abstraction so canvas.cpp / random.cpp / wasm_host.cpp compile
// for both the ESP32-S3 badge (Arduino) and the browser (Emscripten).
//
// Two hooks that differ between the two targets:
//   host_log(fmt, ...)  -- printf-style diagnostic output.
//   host_millis()       -- monotonic millisecond clock since startup.

#pragma once

#include <stdint.h>

#ifdef __EMSCRIPTEN__

#include <cstdio>
#include <emscripten/emscripten.h>

#define host_log(...) std::printf(__VA_ARGS__)

static inline uint32_t host_millis() {
    return (uint32_t)emscripten_get_now();
}

#elif defined(ARDUINO)

#include <Arduino.h>

#define host_log(...) Serial.printf(__VA_ARGS__)

static inline uint32_t host_millis() {
    return millis();
}

#else
// Native (desktop) target: used by firmware/tools/app-runner and other
// offline tooling. Uses printf for logging and a monotonic clock for time.

#include <cstdio>
#include <chrono>

#define host_log(...) std::printf(__VA_ARGS__)

static inline uint32_t host_millis() {
    using namespace std::chrono;
    static auto start = steady_clock::now();
    return (uint32_t)duration_cast<milliseconds>(steady_clock::now() - start).count();
}

#endif
