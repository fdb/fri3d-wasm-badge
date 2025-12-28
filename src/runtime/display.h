#pragma once

#include <u8g2.h>
#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

// Note: SCREEN_WIDTH and SCREEN_HEIGHT are defined in canvas.h
// They are included here for convenience when only display.h is needed
#ifndef SCREEN_WIDTH
#define SCREEN_WIDTH 128
#endif
#ifndef SCREEN_HEIGHT
#define SCREEN_HEIGHT 64
#endif

/**
 * Abstract display interface.
 * Implementations provide platform-specific display initialization and rendering.
 * All implementations use u8g2 for the drawing buffer.
 *
 * Usage: Implementations should embed this struct and provide function pointers.
 */
typedef struct display display_t;

struct display {
    /**
     * Initialize the display hardware/window.
     * @return true on success
     */
    bool (*init)(display_t* self);

    /**
     * Get the u8g2 instance for drawing operations.
     * Canvas uses this to perform drawing.
     */
    u8g2_t* (*get_u8g2)(display_t* self);

    /**
     * Push the u8g2 buffer to the physical display.
     * Called after rendering each frame.
     */
    void (*flush)(display_t* self);

    /**
     * Check if the display/application should quit.
     * (e.g., window closed on desktop)
     */
    bool (*should_quit)(display_t* self);
};

/**
 * Helper function to get screen width.
 */
static inline int display_width(void) {
    return SCREEN_WIDTH;
}

/**
 * Helper function to get screen height.
 */
static inline int display_height(void) {
    return SCREEN_HEIGHT;
}

#ifdef __cplusplus
}
#endif
