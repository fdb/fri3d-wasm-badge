#pragma once

#include "input.h"
#include "display_sdl.h"
#include <SDL.h>
#include <queue>

namespace fri3d {

/**
 * SDL-based input handler for desktop emulator.
 * Maps keyboard keys to InputKey values.
 */
class InputSDL : public InputHandler {
public:
    /**
     * Create input handler.
     * @param display Reference to display (for quit handling and time)
     */
    explicit InputSDL(DisplaySDL& display);

    // InputHandler interface
    void poll() override;
    bool hasEvent() const override;
    InputEvent getEvent() override;
    uint32_t getTimeMs() const override;

    /**
     * Check if screenshot was requested (S key).
     */
    bool wasScreenshotRequested();

private:
    DisplaySDL& m_display;
    std::queue<InputEvent> m_eventQueue;
    bool m_screenshotRequested = false;

    /**
     * Convert SDL keycode to InputKey.
     * @return InputKey value, or -1 if not mapped
     */
    static int sdlKeyToInputKey(SDL_Keycode key);
};

} // namespace fri3d
