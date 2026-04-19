#pragma once

// Full-screen color framebuffer (296x240 RGB565). Lives alongside the
// legacy 128x64 mono Canvas — apps written against the new design system
// (idle scene, app grid, color games) draw straight into this buffer via
// the screen_* host imports. Colors are passed in as packed 24-bit RGB
// (0x00RRGGBB) for ergonomics; the host packs them to RGB565 once per
// draw call.
//
// Backed by PSRAM on hardware (296*240*2 = ~142 KB, won't fit in internal
// SRAM alongside the wasm runtime). The browser build allocates it on the
// heap.

#include <stdint.h>
#include <stddef.h>

#include "font.h"

namespace fri3d {

constexpr uint32_t SCREEN_W = 296;
constexpr uint32_t SCREEN_H = 240;
constexpr size_t   SCREEN_PIXELS = SCREEN_W * SCREEN_H;
constexpr size_t   SCREEN_BYTES  = SCREEN_PIXELS * sizeof(uint16_t);

// Pack a 24-bit 0x00RRGGBB color into RGB565 (matches TFT_eSPI's color16()).
inline uint16_t rgb_to_565(uint32_t rgb) {
    const uint8_t r = (uint8_t)((rgb >> 16) & 0xFF);
    const uint8_t g = (uint8_t)((rgb >>  8) & 0xFF);
    const uint8_t b = (uint8_t)( rgb        & 0xFF);
    return (uint16_t)(((r & 0xF8) << 8) | ((g & 0xFC) << 3) | (b >> 3));
}

class Screen {
public:
    // `pixels` must point to at least SCREEN_PIXELS uint16_t cells. Caller
    // owns the storage (lets us put it in PSRAM on hardware, on the heap in
    // the browser, etc).
    explicit Screen(uint16_t* pixels);

    uint32_t width()  const { return SCREEN_W; }
    uint32_t height() const { return SCREEN_H; }
    const uint16_t* pixels() const { return m_pixels; }
    uint16_t*       pixels()       { return m_pixels; }

    // Per-frame "did the app draw to the color screen?" flag. main.cpp
    // resets it before render() and inspects it after — if false, the
    // legacy 128x64 mono canvas gets blitted into the centre of the
    // screen so old apps still work without modification.
    bool used() const { return m_used; }
    void mark_used() { m_used = true; }
    void reset_used() { m_used = false; }

    void clear(uint32_t rgb);
    void fill_rect  (int32_t x, int32_t y, int32_t w, int32_t h, uint32_t rgb);
    void stroke_rect(int32_t x, int32_t y, int32_t w, int32_t h, uint32_t rgb);
    void pixel(int32_t x, int32_t y, uint32_t rgb);
    void hline(int32_t x, int32_t y, int32_t length, uint32_t rgb);
    void vline(int32_t x, int32_t y, int32_t length, uint32_t rgb);
    void line (int32_t x1, int32_t y1, int32_t x2, int32_t y2, uint32_t rgb);
    void circle(int32_t x, int32_t y, int32_t radius, uint32_t rgb);
    void disc  (int32_t x, int32_t y, int32_t radius, uint32_t rgb);

    // Render text with one of the legacy fonts in `rgb`, baseline at (x, y).
    void text(int32_t x, int32_t y, const char* str, uint32_t rgb, FontId font);

    // Blit the legacy mono canvas (128x64) centred and 2x-upscaled into the
    // screen, painting set pixels in `fg_rgb` and unset in `bg_rgb`.
    void blit_legacy_canvas(const uint8_t* mono_buf,
                            uint32_t mono_w, uint32_t mono_h,
                            uint32_t fg_rgb, uint32_t bg_rgb);

private:
    uint16_t* m_pixels;
    bool      m_used = false;

    // Internal hline that doesn't trigger m_used (used by blit_legacy_canvas).
    void raw_hline(int32_t x, int32_t y, int32_t length, uint16_t color565);
};

} // namespace fri3d
