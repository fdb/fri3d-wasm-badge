#include "display_sdl.h"
#include <lodepng.h>
#include <iostream>
#include <vector>

namespace fri3d {

// u8g2 dummy callbacks (we only use the buffer)
uint8_t DisplaySDL::u8x8GpioAndDelayCallback(u8x8_t* u8x8, uint8_t msg, uint8_t arg_int, void* arg_ptr) {
    (void)u8x8; (void)msg; (void)arg_int; (void)arg_ptr;
    return 1;
}

uint8_t DisplaySDL::u8x8ByteCallback(u8x8_t* u8x8, uint8_t msg, uint8_t arg_int, void* arg_ptr) {
    (void)u8x8; (void)msg; (void)arg_int; (void)arg_ptr;
    return 1;
}

DisplaySDL::DisplaySDL() = default;

DisplaySDL::~DisplaySDL() {
    if (m_texture) {
        SDL_DestroyTexture(m_texture);
    }
    if (m_renderer) {
        SDL_DestroyRenderer(m_renderer);
    }
    if (m_window) {
        SDL_DestroyWindow(m_window);
    }
    if (!m_headless) {
        SDL_Quit();
    }
}

bool DisplaySDL::init(bool headless) {
    m_headless = headless;

    // Initialize SDL (skip video init if headless)
    if (!headless) {
        if (SDL_Init(SDL_INIT_VIDEO) < 0) {
            std::cerr << "SDL could not initialize! SDL_Error: " << SDL_GetError() << std::endl;
            return false;
        }

        m_window = SDL_CreateWindow(
            "Fri3d Emulator",
            SDL_WINDOWPOS_UNDEFINED, SDL_WINDOWPOS_UNDEFINED,
            SCREEN_WIDTH * SCALE_FACTOR, SCREEN_HEIGHT * SCALE_FACTOR,
            SDL_WINDOW_SHOWN
        );
        if (!m_window) {
            std::cerr << "Window could not be created! SDL_Error: " << SDL_GetError() << std::endl;
            return false;
        }

        m_renderer = SDL_CreateRenderer(m_window, -1, SDL_RENDERER_ACCELERATED);
        if (!m_renderer) {
            std::cerr << "Renderer could not be created! SDL_Error: " << SDL_GetError() << std::endl;
            return false;
        }

        m_texture = SDL_CreateTexture(
            m_renderer,
            SDL_PIXELFORMAT_RGBA8888,
            SDL_TEXTUREACCESS_STREAMING,
            SCREEN_WIDTH, SCREEN_HEIGHT
        );
        if (!m_texture) {
            std::cerr << "Texture could not be created! SDL_Error: " << SDL_GetError() << std::endl;
            return false;
        }
    }

    // Initialize u8g2 (SSD1306 128x64 full buffer mode)
    u8g2_Setup_ssd1306_128x64_noname_f(
        &m_u8g2, U8G2_R0,
        u8x8ByteCallback,
        u8x8GpioAndDelayCallback
    );
    u8g2_InitDisplay(&m_u8g2);
    u8g2_SetPowerSave(&m_u8g2, 0);
    u8g2_ClearBuffer(&m_u8g2);

    return true;
}

void DisplaySDL::flush() {
    if (m_headless) return;

    uint8_t* buffer = u8g2_GetBufferPtr(&m_u8g2);

    void* pixels;
    int pitch;
    SDL_LockTexture(m_texture, nullptr, &pixels, &pitch);

    uint32_t* dst = static_cast<uint32_t*>(pixels);

    // u8g2 SSD1306 buffer layout: each byte is 8 vertical pixels
    // Byte 0: pixels (0,0)-(0,7), Byte 1: pixels (1,0)-(1,7), etc.
    for (int y = 0; y < SCREEN_HEIGHT; y++) {
        for (int x = 0; x < SCREEN_WIDTH; x++) {
            int byte_idx = x + (y / 8) * SCREEN_WIDTH;
            int bit_idx = y % 8;
            bool pixel = (buffer[byte_idx] >> bit_idx) & 1;
            // Black pixels are "on" (1 = white on screen), white pixels are "off" (0 = black on screen)
            dst[y * SCREEN_WIDTH + x] = pixel ? 0xFFFFFFFF : 0x000000FF;
        }
    }

    SDL_UnlockTexture(m_texture);
    SDL_RenderCopy(m_renderer, m_texture, nullptr, nullptr);
    SDL_RenderPresent(m_renderer);
}

bool DisplaySDL::saveScreenshot(const std::string& path) {
    uint8_t* buffer = u8g2_GetBufferPtr(&m_u8g2);

    // Create RGBA image data (lodepng expects RGBA)
    std::vector<unsigned char> image(SCREEN_WIDTH * SCREEN_HEIGHT * 4);

    // Convert u8g2 buffer to RGBA
    for (int y = 0; y < SCREEN_HEIGHT; y++) {
        for (int x = 0; x < SCREEN_WIDTH; x++) {
            int byte_idx = x + (y / 8) * SCREEN_WIDTH;
            int bit_idx = y % 8;
            bool pixel = (buffer[byte_idx] >> bit_idx) & 1;
            // Black pixels are "on" (1), white pixels are "off" (0)
            uint8_t color = pixel ? 0 : 255;
            size_t idx = 4 * (y * SCREEN_WIDTH + x);
            image[idx + 0] = color; // R
            image[idx + 1] = color; // G
            image[idx + 2] = color; // B
            image[idx + 3] = 255;   // A (fully opaque)
        }
    }

    unsigned error = lodepng::encode(path, image, SCREEN_WIDTH, SCREEN_HEIGHT);
    if (error) {
        std::cerr << "PNG encoder error: " << lodepng_error_text(error) << std::endl;
        return false;
    }

    return true;
}

} // namespace fri3d
