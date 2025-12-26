#pragma once

#include "display.h"
#include <SDL.h>
#include <string>

namespace fri3d {

/**
 * SDL-based display implementation for desktop emulator.
 */
class DisplaySDL : public Display {
public:
    DisplaySDL();
    ~DisplaySDL() override;

    /**
     * Initialize SDL window and u8g2 buffer.
     * @param headless If true, skip window creation (for testing)
     * @return true on success
     */
    bool init(bool headless = false);

    // Display interface
    bool init() override { return init(false); }
    u8g2_t* getU8g2() override { return &m_u8g2; }
    void flush() override;
    bool shouldQuit() override { return m_shouldQuit; }

    /**
     * Set quit flag (called from input handler).
     */
    void setQuit(bool quit) { m_shouldQuit = quit; }

    /**
     * Get the SDL window (for event handling).
     */
    SDL_Window* getWindow() { return m_window; }

    /**
     * Check if running in headless mode.
     */
    bool isHeadless() const { return m_headless; }

    /**
     * Save screenshot to PNG file.
     * @param path Output file path
     * @return true on success
     */
    bool saveScreenshot(const std::string& path);

private:
    SDL_Window* m_window = nullptr;
    SDL_Renderer* m_renderer = nullptr;
    SDL_Texture* m_texture = nullptr;
    u8g2_t m_u8g2;

    bool m_shouldQuit = false;
    bool m_headless = false;

    static constexpr int SCALE_FACTOR = 4;

    // u8g2 dummy callbacks (buffer-only mode)
    static uint8_t u8x8GpioAndDelayCallback(u8x8_t* u8x8, uint8_t msg, uint8_t arg_int, void* arg_ptr);
    static uint8_t u8x8ByteCallback(u8x8_t* u8x8, uint8_t msg, uint8_t arg_int, void* arg_ptr);
};

} // namespace fri3d
