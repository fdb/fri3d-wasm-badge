// Fri3d WASM Badge — hardware firmware, phase 2.
//
// Pipeline:
//   1. fri3d::Canvas maintains the emulator's 128x64 mono framebuffer.
//   2. wasm_host drives a wasm3 interpreter that loads embedded_app.wasm and
//      calls its render() / on_input() exports via the fri3d-wasm-api ABI.
//      Host imports on the "env" module map to Canvas + Random + millis().
//   3. blit_to_screen() upscales 2x into a 256x128 uint16_t buffer in PSRAM.
//   4. tft.pushImage() flushes that buffer to the GC9307 LCD in one SPI burst.

#include <Arduino.h>
#include <TFT_eSPI.h>
#include <SPI.h>
#include <esp_heap_caps.h>

#include "Fri3dBadge_pins.h"
#include "Fri3dBadge_Button.h"
#include "canvas.h"
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

static constexpr int32_t  SCALE     = 2;
static constexpr int32_t  FB_W      = (int32_t)fri3d::SCREEN_WIDTH;
static constexpr int32_t  FB_H      = (int32_t)fri3d::SCREEN_HEIGHT;
static constexpr int32_t  UP_W      = FB_W * SCALE;   // 256
static constexpr int32_t  UP_H      = FB_H * SCALE;   // 128
static constexpr int32_t  LCD_W     = 296;
static constexpr int32_t  LCD_H     = 240;
static constexpr int32_t  CANVAS_X  = (LCD_W - UP_W) / 2; // 20
static constexpr int32_t  CANVAS_Y  = (LCD_H - UP_H) / 2; // 56

static constexpr uint16_t COLOR_BG     = TFT_BLACK;
static constexpr uint16_t COLOR_FG     = TFT_GREEN;
static constexpr uint16_t COLOR_CHROME = 0x18C3;

// ---- Hardware objects ------------------------------------------------------

TFT_eSPI tft = TFT_eSPI();
Canvas   canvas;
Random   rng(42);

static uint16_t* upscaled = nullptr;

// Button IDs matching the Fri3d example order.
enum : uint8_t {
    BID_A = 0, BID_B, BID_X, BID_Y, BID_MENU, BID_START,
    BID_UP, BID_DOWN, BID_LEFT, BID_RIGHT,
    BID_COUNT,
};

static Fri3d_Button* buttons[BID_COUNT];

// Press-time tracking so we can emit SHORT_PRESS / LONG_PRESS on release.
static constexpr uint32_t LONG_PRESS_MS = 300;
static uint32_t press_start_ms[BID_COUNT] = {0};

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

// ---- Chrome --------------------------------------------------------------

static void draw_chrome() {
    tft.fillScreen(COLOR_CHROME);

    tft.setTextColor(TFT_WHITE, COLOR_CHROME);
    tft.setTextSize(2);
    tft.setCursor(40, 20);
    tft.print("Fri3d WASM Badge");

    tft.setTextSize(1);
    tft.setCursor(40, 200);
    tft.print("A: OK   B: Back   Joystick: D-pad");

    tft.drawRect(CANVAS_X - 2, CANVAS_Y - 2, UP_W + 4, UP_H + 4, TFT_DARKGREY);
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

static void blit_to_screen() {
    if (!upscaled) return;
    const uint8_t* src = canvas.buffer();

    for (int32_t y = 0; y < FB_H; ++y) {
        const uint8_t* row = &src[y * FB_W];
        uint16_t* dst0 = &upscaled[(y * SCALE + 0) * UP_W];
        uint16_t* dst1 = &upscaled[(y * SCALE + 1) * UP_W];
        for (int32_t x = 0; x < FB_W; ++x) {
            uint16_t c = row[x] ? COLOR_FG : COLOR_BG;
            dst0[x * 2 + 0] = c;
            dst0[x * 2 + 1] = c;
            dst1[x * 2 + 0] = c;
            dst1[x * 2 + 1] = c;
        }
    }

    tft.pushImage(CANVAS_X, CANVAS_Y, UP_W, UP_H, upscaled);
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

    upscaled = (uint16_t*) ps_malloc((size_t)UP_W * UP_H * sizeof(uint16_t));
    if (!upscaled) {
        Serial.println("[fri3d] PSRAM alloc failed, falling back to heap");
        upscaled = (uint16_t*) malloc((size_t)UP_W * UP_H * sizeof(uint16_t));
    }
    log_heap("post-upscale-alloc");

    tft.init();
    tft.writecommand(TFT_MADCTL);
    tft.writedata(TFT_MAD_BGR | TFT_MAD_MV);
    tft.setTextFont(1);
    draw_chrome();
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
    const char* err = wasm_host::init(canvas, rng, launcher.wasm, launcher.wasm_len);
    if (err) {
        Serial.printf("[fri3d] wasm init failed: %s\n", err);
        show_fatal(err);
        while (true) { delay(1000); }
    }
    log_heap("post-wasm-init");
    Serial.println("[fri3d] wasm ready");
}

void loop() {
    for (uint8_t i = 0; i < BID_COUNT; ++i) buttons[i]->read();

    // Emit PRESS immediately, and RELEASE + SHORT/LONG_PRESS on lift.
    for (uint8_t i = 0; i < BID_COUNT; ++i) {
        uint32_t k = key_for(i);
        if (k == 0xFFFFFFFFu) continue;

        if (buttons[i]->wasPressed()) {
            press_start_ms[i] = millis();
            wasm_host::on_input(k, wasm_host::event::PRESS);
        }
        if (buttons[i]->wasReleased()) {
            wasm_host::on_input(k, wasm_host::event::RELEASE);
            uint32_t held = millis() - press_start_ms[i];
            wasm_host::on_input(k,
                held >= LONG_PRESS_MS ? wasm_host::event::LONG_PRESS
                                      : wasm_host::event::SHORT_PRESS);
        }
    }

    if (wasm_host::exit_requested()) {
        wasm_host::clear_exit_request();
        const EmbeddedApp& launcher = EMBEDDED_APPS[0];
        Serial.printf("[fri3d] exit_to_launcher -> %s\n", launcher.name);
        const char* err = wasm_host::reload(launcher.wasm, launcher.wasm_len);
        if (err) Serial.printf("[fri3d] launcher reload failed: %s\n", err);
    }

    if (wasm_host::start_app_requested()) {
        uint32_t req = wasm_host::requested_app_id();
        wasm_host::clear_start_app_request();
        const EmbeddedApp* target = nullptr;
        for (uint32_t i = 0; i < EMBEDDED_APPS_COUNT; ++i) {
            if (EMBEDDED_APPS[i].app_id == req) { target = &EMBEDDED_APPS[i]; break; }
        }
        if (!target) {
            Serial.printf("[fri3d] start_app(%u): no such app\n", req);
        } else {
            Serial.printf("[fri3d] start_app(%u) -> %s\n", req, target->name);
            const char* err = wasm_host::reload(target->wasm, target->wasm_len);
            if (err) Serial.printf("[fri3d] %s reload failed: %s\n", target->name, err);
        }
    }

    wasm_host::render();
    blit_to_screen();

    delay(33); // ~30 fps
}
