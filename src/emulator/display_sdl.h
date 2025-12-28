#pragma once

#include "display.h"
#include <SDL.h>

#ifdef __cplusplus
extern "C" {
#endif

#define DISPLAY_SDL_SCALE_FACTOR 4

/**
 * SDL-based display implementation for desktop emulator.
 */
typedef struct {
    // Base display interface (must be first for vtable-style access)
    display_t base;

    // SDL resources
    SDL_Window* window;
    SDL_Renderer* renderer;
    SDL_Texture* texture;
    u8g2_t u8g2;

    bool should_quit;
    bool headless;
} display_sdl_t;

/**
 * Initialize a display_sdl instance.
 */
void display_sdl_init(display_sdl_t* display);

/**
 * Clean up a display_sdl instance.
 */
void display_sdl_cleanup(display_sdl_t* display);

/**
 * Start the SDL display.
 * @param display Display instance
 * @param headless If true, skip window creation (for testing)
 * @return true on success
 */
bool display_sdl_start(display_sdl_t* display, bool headless);

/**
 * Get the u8g2 instance for drawing operations.
 */
u8g2_t* display_sdl_get_u8g2(display_sdl_t* display);

/**
 * Push the u8g2 buffer to the SDL window.
 */
void display_sdl_flush(display_sdl_t* display);

/**
 * Check if the display/application should quit.
 */
bool display_sdl_should_quit(display_sdl_t* display);

/**
 * Set quit flag (called from input handler).
 */
void display_sdl_set_quit(display_sdl_t* display, bool quit);

/**
 * Get the SDL window (for event handling).
 */
SDL_Window* display_sdl_get_window(display_sdl_t* display);

/**
 * Check if running in headless mode.
 */
bool display_sdl_is_headless(const display_sdl_t* display);

/**
 * Save screenshot to PNG file.
 * @param display Display instance
 * @param path Output file path
 * @return true on success
 */
bool display_sdl_save_screenshot(display_sdl_t* display, const char* path);

#ifdef __cplusplus
}
#endif
