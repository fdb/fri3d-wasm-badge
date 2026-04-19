// Fri3d WASM Badge — hardware first-light firmware.
//
// Pipeline:
//   1. fri3d::Canvas maintains the emulator's 128x64 mono framebuffer.
//   2. Per frame, we run a native C++ re-implementation of fri3d-app-circles.
//   3. blit_to_screen() upscales 2x into a 256x128 uint16_t buffer in PSRAM.
//   4. tft.pushImage() flushes that buffer to the GC9307 LCD in one SPI burst.
//
// Next steps (not yet implemented): drop a WASM interpreter (wasm3) in step 2
// so the existing .wasm apps run unchanged.

#include <Arduino.h>
#include <TFT_eSPI.h>
#include <SPI.h>

#include "Fri3dBadge_pins.h"
#include "Fri3dBadge_Button.h"
#include "canvas.h"

using fri3d::Canvas;
using fri3d::Color;

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

// Emulator palette -> LCD colour mapping.
// Canvas buffer byte 0 == white (background), 1 == black (foreground).
// We map to black/green for an oscilloscope-like monochrome look on the
// colour LCD; flip these two constants to switch to white-on-black etc.
static constexpr uint16_t COLOR_BG = TFT_BLACK;
static constexpr uint16_t COLOR_FG = TFT_GREEN;
// Around the centered canvas — distinguishes the "badge screen" from the
// rest of the physical panel.
static constexpr uint16_t COLOR_CHROME = 0x18C3; // very dark blue/grey

// ---- Hardware objects ------------------------------------------------------

TFT_eSPI tft = TFT_eSPI();
Canvas   canvas;

// 2x-upscaled, pre-swapped 16-bit framebuffer for one-shot pushImage().
// 256 * 128 * 2 bytes = 65536 bytes. Allocated in PSRAM at setup.
static uint16_t* upscaled = nullptr;

// Button layout matches the Fri3d buttons example.
enum : uint8_t {
    BID_A = 0, BID_B, BID_X, BID_Y, BID_MENU, BID_START,
    BID_UP, BID_DOWN, BID_LEFT, BID_RIGHT,
    BID_COUNT,
};

static Fri3d_Button* buttons[BID_COUNT];

// ---- PRNG (matches fri3d-runtime/src/random.rs xorshift32 behaviour) -------

static uint32_t prng_state = 42;

static uint32_t prng_next() {
    uint32_t s = prng_state;
    s ^= s << 13;
    s ^= s >> 17;
    s ^= s << 5;
    prng_state = s;
    return s;
}

static uint32_t prng_range(uint32_t bound) {
    if (bound == 0) return 0;
    return prng_next() % bound;
}

static void prng_seed(uint32_t seed) {
    prng_state = seed ? seed : 1;
}

// ---- Native port of the fri3d-app-circles render -------------------------
// See fri3d-app-circles/src/lib.rs. We re-seed each frame in the "animated"
// mode so motion is visible; pressing A freezes a new random layout by
// rolling the seed once.

static uint32_t circles_seed    = 42;
static bool     circles_animate = true;

static void circles_render() {
    canvas.clear();
    prng_seed(circles_animate ? (circles_seed ^ (uint32_t)millis()) : circles_seed);
    canvas.set_color(Color::Black);
    for (int i = 0; i < 10; ++i) {
        int32_t x      = (int32_t)prng_range(128);
        int32_t y      = (int32_t)prng_range(64);
        uint32_t radius = prng_range(15) + 3;
        canvas.draw_circle(x, y, radius);
    }
}

// ---- Blit + chrome ---------------------------------------------------------

static void draw_chrome() {
    tft.fillScreen(COLOR_CHROME);

    tft.setTextColor(TFT_WHITE, COLOR_CHROME);
    tft.setTextSize(2);
    tft.setCursor(40, 20);
    tft.print("Fri3d WASM Badge");

    tft.setTextSize(1);
    tft.setCursor(40, 200);
    tft.print("A: reshuffle   B: freeze/animate");

    // Subtle border around the 256x128 canvas area.
    tft.drawRect(CANVAS_X - 2, CANVAS_Y - 2, UP_W + 4, UP_H + 4, TFT_DARKGREY);
}

static void blit_to_screen() {
    if (!upscaled) return;
    const uint8_t* src = canvas.buffer();

    // Build the 2x upscaled 16-bit buffer.
    // Row-major: for each FB pixel, fan out to a 2x2 block.
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

// ---- Setup / loop ----------------------------------------------------------

void setup() {
    Serial.begin(115200);
    delay(200);
    Serial.println("[fri3d] boot");

    // 64 KB in PSRAM. ps_malloc gives us the external RAM; heap_caps_malloc
    // with MALLOC_CAP_SPIRAM would also work.
    upscaled = (uint16_t*) ps_malloc((size_t)UP_W * UP_H * sizeof(uint16_t));
    if (!upscaled) {
        Serial.println("[fri3d] PSRAM alloc failed, falling back to heap");
        upscaled = (uint16_t*) malloc((size_t)UP_W * UP_H * sizeof(uint16_t));
    }

    tft.init();
    tft.writecommand(TFT_MADCTL);
    tft.writedata(TFT_MAD_BGR | TFT_MAD_MV); // landscape + BGR swap, per Fri3d buttons example
    tft.setTextFont(1);
    draw_chrome();

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

    for (uint8_t i = 0; i < BID_COUNT; ++i) {
        buttons[i]->begin();
    }

    Serial.println("[fri3d] ready");
}

void loop() {
    for (uint8_t i = 0; i < BID_COUNT; ++i) buttons[i]->read();

    if (buttons[BID_A]->wasPressed()) {
        // Reshuffle: match fri3d-app-circles' KEY_OK behaviour.
        circles_seed = prng_next();
        Serial.printf("[fri3d] reseed -> %u\n", circles_seed);
    }
    if (buttons[BID_B]->wasPressed()) {
        circles_animate = !circles_animate;
        Serial.printf("[fri3d] animate=%d\n", circles_animate ? 1 : 0);
    }

    circles_render();
    blit_to_screen();

    // ~30 fps cap. The circles render itself is cheap; SPI blit dominates.
    delay(33);
}
