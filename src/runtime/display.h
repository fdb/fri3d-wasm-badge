#pragma once

#include <u8g2.h>
#include <cstdint>

namespace fri3d {

// Screen dimensions (matching SSD1306 128x64)
constexpr int SCREEN_WIDTH = 128;
constexpr int SCREEN_HEIGHT = 64;

/**
 * Abstract display interface.
 * Implementations provide platform-specific display initialization and rendering.
 * All implementations use u8g2 for the drawing buffer.
 */
class Display {
public:
    virtual ~Display() = default;

    /**
     * Initialize the display hardware/window.
     * @return true on success
     */
    virtual bool init() = 0;

    /**
     * Get the u8g2 instance for drawing operations.
     * Canvas uses this to perform drawing.
     */
    virtual u8g2_t* getU8g2() = 0;

    /**
     * Push the u8g2 buffer to the physical display.
     * Called after rendering each frame.
     */
    virtual void flush() = 0;

    /**
     * Check if the display/application should quit.
     * (e.g., window closed on desktop)
     */
    virtual bool shouldQuit() = 0;

    /**
     * Get screen width.
     */
    int width() const { return SCREEN_WIDTH; }

    /**
     * Get screen height.
     */
    int height() const { return SCREEN_HEIGHT; }
};

} // namespace fri3d
