#pragma once

#include "input.h"
#include "display_sdl.h"
#include <SDL.h>

#ifdef __cplusplus
extern "C" {
#endif

// Event queue size for input events
#define INPUT_SDL_QUEUE_SIZE 32

/**
 * SDL-based input handler for desktop emulator.
 * Maps keyboard keys to InputKey values.
 */
typedef struct {
    // Base handler interface (for compatibility)
    input_handler_t base;

    display_sdl_t* display;

    // Ring buffer for events
    input_event_t event_queue[INPUT_SDL_QUEUE_SIZE];
    size_t queue_head;
    size_t queue_tail;

    bool screenshot_requested;
} input_sdl_t;

/**
 * Initialize an input_sdl instance.
 * @param input Input handler instance
 * @param display Display for quit handling
 */
void input_sdl_init(input_sdl_t* input, display_sdl_t* display);

/**
 * Poll for SDL input events.
 */
void input_sdl_poll(input_sdl_t* input);

/**
 * Check if an event is available.
 */
bool input_sdl_has_event(const input_sdl_t* input);

/**
 * Get the next event.
 */
input_event_t input_sdl_get_event(input_sdl_t* input);

/**
 * Get current time in milliseconds.
 */
uint32_t input_sdl_get_time_ms(const input_sdl_t* input);

/**
 * Check if screenshot was requested (S key).
 */
bool input_sdl_was_screenshot_requested(input_sdl_t* input);

/**
 * Get the input handler interface for use with input_manager.
 */
input_handler_t* input_sdl_get_handler(input_sdl_t* input);

#ifdef __cplusplus
}
#endif
