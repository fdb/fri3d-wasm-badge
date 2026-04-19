// Fri3d WASM Badge — hardware firmware, phase 3.
//
// Pipeline:
//   1. fri3d::Screen owns a 296x240 RGB565 framebuffer (the full LCD).
//      New apps draw straight into it via screen_* host imports.
//   2. fri3d::Canvas keeps the legacy 128x64 mono framebuffer for apps not
//      yet ported to color. After render(), if no screen_* import was
//      called the canvas gets blitted into the centre of the screen.
//   3. wasm_host drives a wasm3 interpreter that loads embedded_apps.h and
//      calls its render() / on_input() exports via the fri3d-wasm-api ABI.
//   4. tft.pushImage() flushes the 296x240 RGB565 buffer to the LCD in a
//      single SPI burst — no per-frame conversion or upscale step.

#include <Arduino.h>
#include <TFT_eSPI.h>
#include <SPI.h>
#include <esp_heap_caps.h>

#include "Fri3dBadge_pins.h"
#include "Fri3dBadge_Button.h"
#include "app_switcher.h"
#include "canvas.h"
#include "screen.h"
#include "input_queue.h"
#include "random.h"
#include "wasm_host.h"
#include "embedded_apps.h"

// wasm3's interpreter dispatches op-by-op through nested C calls and needs
// far more stack than Arduino's default 8 KB loop task. This is the
// sanctioned override and survives sdkconfig.h wins over build -D flags.
SET_LOOP_TASK_STACK_SIZE(32 * 1024);

using fri3d::Canvas;
using fri3d::Color;
using fri3d::Random;

// ---- Display geometry ------------------------------------------------------

static constexpr int32_t  LCD_W = (int32_t)fri3d::SCREEN_W;  // 296
static constexpr int32_t  LCD_H = (int32_t)fri3d::SCREEN_H;  // 240

// Legacy mono apps get blitted in this colour scheme — the design's violet
// on near-black so old games look like they belong on the new badge OS.
static constexpr uint32_t LEGACY_FG_RGB = 0x7b6cff;
static constexpr uint32_t LEGACY_BG_RGB = 0x05050a;

// ---- Hardware objects ------------------------------------------------------

TFT_eSPI tft = TFT_eSPI();
Canvas   canvas;
Random   rng(42);

// 296x240 RGB565 framebuffer. ~142 KB — lives in PSRAM. The Screen object
// is a thin view over this buffer; it doesn't own the storage so we can
// allocate from PSRAM separately.
static uint16_t* g_screen_pixels = nullptr;
static fri3d::Screen* g_screen = nullptr;

// Button IDs matching the Fri3d example order.
enum : uint8_t {
    BID_A = 0, BID_B, BID_X, BID_Y, BID_MENU, BID_START,
    BID_UP, BID_DOWN, BID_LEFT, BID_RIGHT,
    BID_COUNT,
};

static Fri3d_Button* buttons[BID_COUNT];

// Press-time tracking lives with the polling task (which is the only
// consumer of it), not main loop state.
static constexpr uint32_t LONG_PRESS_MS = 300;

// Events produced by the polling task, drained by the main loop before
// render. Decouples input latency from render cadence.
static fri3d::InputQueue g_input;

// Button-polling task: runs at 200 Hz on core 0 independent of the render
// loop on core 1. Catches press + release transitions that fit entirely
// inside a slow render frame (Mandelbrot is ~500 ms on ESP32-S3, which
// would otherwise swallow any tap shorter than that).
static void button_task(void* /*arg*/);

// Map each button ID to the emulator's input key.
// Note: fully qualify — ESP-IDF's esp32s3/rom/ets_sys.h defines an unscoped
// `OK` enum that would otherwise collide.
static uint32_t key_for(uint8_t bid) {
    switch (bid) {
        case BID_A:     return wasm_host::key::OK;
        case BID_B:     return wasm_host::key::BACK;
        case BID_UP:    return wasm_host::key::UP;
        case BID_DOWN:  return wasm_host::key::DOWN;
        case BID_LEFT:  return wasm_host::key::LEFT;
        case BID_RIGHT: return wasm_host::key::RIGHT;
        default:        return 0xFFFFFFFFu;
    }
}

static void show_fatal(const char* msg) {
    tft.fillScreen(TFT_RED);
    tft.setTextColor(TFT_WHITE, TFT_RED);
    tft.setTextSize(2);
    tft.setCursor(20, 20);
    tft.print("WASM init failed");
    tft.setTextSize(1);
    tft.setCursor(20, 50);
    tft.print(msg ? msg : "unknown");
}

// ---- Framebuffer blit ----------------------------------------------------

// Push the entire 296x240 RGB565 buffer to the LCD in one SPI burst. With
// the buffer already in the LCD's native format there's no per-pixel
// conversion; pushImage just DMA-streams the bytes.
static void blit_to_screen() {
    if (!g_screen_pixels) return;
    tft.pushImage(0, 0, LCD_W, LCD_H, g_screen_pixels);
}

// ---- Setup / loop --------------------------------------------------------

// IMPORTANT: never call Serial.flush() in setup() or any path that runs
// before a USB-CDC host attaches. On ESP32-S3 native USB-CDC, flush() spins
// until the host drains the TX ring buffer — if the badge booted standalone
// (no laptop plugged in), it blocks forever and the board looks dead.
static void log_heap(const char* tag) {
    Serial.printf("[fri3d] %s: heap internal=%u psram=%u\n",
        tag,
        (unsigned)heap_caps_get_free_size(MALLOC_CAP_INTERNAL),
        (unsigned)heap_caps_get_free_size(MALLOC_CAP_SPIRAM));
}

void setup() {
    Serial.begin(115200);
    delay(200);
    Serial.println("[fri3d] boot");

    log_heap("pre-alloc");

    g_screen_pixels = (uint16_t*) ps_malloc(fri3d::SCREEN_BYTES);
    if (!g_screen_pixels) {
        Serial.println("[fri3d] PSRAM alloc failed, falling back to heap");
        g_screen_pixels = (uint16_t*) malloc(fri3d::SCREEN_BYTES);
    }
    if (!g_screen_pixels) {
        Serial.println("[fri3d] FATAL: screen allocation failed");
        while (true) { delay(1000); }
    }
    g_screen = new fri3d::Screen(g_screen_pixels);
    log_heap("post-screen-alloc");

    tft.init();
    tft.writecommand(TFT_MADCTL);
    tft.writedata(TFT_MAD_BGR | TFT_MAD_MV);
    tft.setTextFont(1);
    tft.fillScreen(TFT_BLACK);
    Serial.println("[fri3d] display ready");

    buttons[BID_A]     = new Fri3d_Button(FRI3D_BUTTON_TYPE_DIGITAL, PIN_A,     25, INPUT_PULLUP, true);
    buttons[BID_B]     = new Fri3d_Button(FRI3D_BUTTON_TYPE_DIGITAL, PIN_B,     25, INPUT_PULLUP, true);
    buttons[BID_X]     = new Fri3d_Button(FRI3D_BUTTON_TYPE_DIGITAL, PIN_X,     25, INPUT_PULLUP, true);
    buttons[BID_Y]     = new Fri3d_Button(FRI3D_BUTTON_TYPE_DIGITAL, PIN_Y,     25, INPUT_PULLUP, true);
    buttons[BID_MENU]  = new Fri3d_Button(FRI3D_BUTTON_TYPE_DIGITAL, PIN_MENU,  25, INPUT_PULLUP, true);
    buttons[BID_START] = new Fri3d_Button(FRI3D_BUTTON_TYPE_DIGITAL, PIN_START, 25, INPUT,        true);
    buttons[BID_UP]    = new Fri3d_Button(FRI3D_BUTTON_TYPE_ANALOGUE_HIGH, PIN_JOY_Y, 0, INPUT, false);
    buttons[BID_DOWN]  = new Fri3d_Button(FRI3D_BUTTON_TYPE_ANALOGUE_LOW,  PIN_JOY_Y, 0, INPUT, false);
    buttons[BID_LEFT]  = new Fri3d_Button(FRI3D_BUTTON_TYPE_ANALOGUE_LOW,  PIN_JOY_X, 0, INPUT, false);
    buttons[BID_RIGHT] = new Fri3d_Button(FRI3D_BUTTON_TYPE_ANALOGUE_HIGH, PIN_JOY_X, 0, INPUT, false);

    for (uint8_t i = 0; i < BID_COUNT; ++i) buttons[i]->begin();
    log_heap("post-buttons");

    // Boot into the launcher (app_id = 0). The launcher's start_app() call
    // will trigger a module reload, dispatched in loop().
    const EmbeddedApp& launcher = EMBEDDED_APPS[0];
    Serial.printf("[fri3d] initializing wasm3 (%s, %u bytes)\n", launcher.name, launcher.wasm_len);
    const char* err = wasm_host::init(canvas, *g_screen, rng, launcher.wasm, launcher.wasm_len);
    if (err) {
        Serial.printf("[fri3d] wasm init failed: %s\n", err);
        show_fatal(err);
        while (true) { delay(1000); }
    }
    log_heap("post-wasm-init");
    Serial.println("[fri3d] wasm ready");

    // Spawn the button-polling task on the other core (Arduino's loop runs
    // on core 1 by default). Core 0 is mostly idle outside Wi-Fi workloads,
    // so giving it a 200 Hz polling task doesn't contend with anything.
    xTaskCreatePinnedToCore(button_task, "btn_poll", 4096, nullptr, 2, nullptr, 0);
}

static void button_task(void* /*arg*/) {
    // Per-button press-start timestamps; only this task touches them.
    uint32_t press_start_ms[BID_COUNT] = {0};

    for (;;) {
        for (uint8_t i = 0; i < BID_COUNT; ++i) buttons[i]->read();

        for (uint8_t i = 0; i < BID_COUNT; ++i) {
            const uint32_t k = key_for(i);
            if (k == 0xFFFFFFFFu) continue;

            if (buttons[i]->wasPressed()) {
                press_start_ms[i] = millis();
                g_input.push({k, wasm_host::event::PRESS});
            }
            if (buttons[i]->wasReleased()) {
                const uint32_t held = millis() - press_start_ms[i];
                g_input.push({k, wasm_host::event::RELEASE});
                g_input.push({k,
                    held >= LONG_PRESS_MS ? wasm_host::event::LONG_PRESS
                                          : wasm_host::event::SHORT_PRESS});
            }
        }

        // 5ms = 200 Hz. Human presses are ≥50ms so every transition is
        // seen at least twice, and the queue has plenty of capacity even
        // if the main loop drains it rarely.
        vTaskDelay(pdMS_TO_TICKS(5));
    }
}

void loop() {
    // Drain everything the polling task produced since the last iteration.
    fri3d::InputEvent ev;
    while (g_input.pop(ev)) {
        wasm_host::on_input(ev.key, ev.type);
    }

    app_switcher::dispatch();

    const uint32_t t_render0 = millis();
    wasm_host::render();
    const uint32_t dt_render = millis() - t_render0;
    // Only log renders that took real work — avoids spamming on idle
    // screens like the launcher. Compute-heavy apps (Mandelbrot, snake AI)
    // will print per-frame.
    if (dt_render > 30) {
        Serial.printf("[perf] render %u ms\n", dt_render);
    }

    // Legacy fallback: if the app didn't touch the new color screen this
    // frame, blit the 128x64 mono canvas centred + 2x-upscaled into the
    // screen using the design's violet-on-black palette. Lets old apps
    // (mandelbrot, snake, etc.) keep running unmodified.
    if (g_screen && !g_screen->used()) {
        g_screen->blit_legacy_canvas(canvas.buffer(),
                                     fri3d::SCREEN_WIDTH, fri3d::SCREEN_HEIGHT,
                                     LEGACY_FG_RGB, LEGACY_BG_RGB);
    }

    blit_to_screen();
    delay(33); // ~30 fps
}
