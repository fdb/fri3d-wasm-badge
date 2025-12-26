#include "input_sdl.h"

namespace fri3d {

InputSDL::InputSDL(DisplaySDL& display) : m_display(display) {
}

void InputSDL::poll() {
    SDL_Event e;

    while (SDL_PollEvent(&e) != 0) {
        if (e.type == SDL_QUIT) {
            m_display.setQuit(true);
        } else if (e.type == SDL_KEYDOWN || e.type == SDL_KEYUP) {
            // Handle screenshot key (S)
            if (e.type == SDL_KEYDOWN && e.key.keysym.sym == SDLK_s) {
                m_screenshotRequested = true;
                continue;
            }

            int inputKey = sdlKeyToInputKey(e.key.keysym.sym);
            if (inputKey >= 0) {
                InputEvent event;
                event.key = static_cast<InputKey>(inputKey);
                event.type = (e.type == SDL_KEYDOWN) ? InputType::Press : InputType::Release;
                m_eventQueue.push(event);
            }
        }
    }
}

bool InputSDL::hasEvent() const {
    return !m_eventQueue.empty();
}

InputEvent InputSDL::getEvent() {
    InputEvent event = m_eventQueue.front();
    m_eventQueue.pop();
    return event;
}

uint32_t InputSDL::getTimeMs() const {
    return SDL_GetTicks();
}

bool InputSDL::wasScreenshotRequested() {
    bool requested = m_screenshotRequested;
    m_screenshotRequested = false;
    return requested;
}

int InputSDL::sdlKeyToInputKey(SDL_Keycode key) {
    switch (key) {
        case SDLK_UP:
            return static_cast<int>(InputKey::Up);
        case SDLK_DOWN:
            return static_cast<int>(InputKey::Down);
        case SDLK_LEFT:
            return static_cast<int>(InputKey::Left);
        case SDLK_RIGHT:
            return static_cast<int>(InputKey::Right);
        case SDLK_RETURN:
        case SDLK_z:
            return static_cast<int>(InputKey::Ok);
        case SDLK_BACKSPACE:
        case SDLK_x:
            return static_cast<int>(InputKey::Back);
        default:
            return -1;
    }
}

} // namespace fri3d
