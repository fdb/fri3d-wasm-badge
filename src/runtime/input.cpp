#include "input.h"
#include <cstddef>

namespace fri3d {

void InputManager::update(InputHandler& handler, uint32_t timeMs) {
    // Clear previous processed event
    m_hasProcessedEvent = false;

    // Poll for new raw events
    handler.poll();

    // Process raw events
    while (handler.hasEvent()) {
        InputEvent rawEvent = handler.getEvent();
        size_t keyIndex = static_cast<size_t>(rawEvent.key);

        if (rawEvent.type == InputType::Press) {
            // Key pressed down
            m_keyStates[keyIndex].pressed = true;
            m_keyStates[keyIndex].pressTime = timeMs;
            m_keyStates[keyIndex].longPressFired = false;

            // Forward the press event
            queueEvent(rawEvent.key, InputType::Press);

        } else if (rawEvent.type == InputType::Release) {
            // Key released
            KeyState& state = m_keyStates[keyIndex];

            if (state.pressed) {
                uint32_t holdTime = timeMs - state.pressTime;

                // If long press wasn't fired yet, check if it was a short press
                if (!state.longPressFired && holdTime < SHORT_PRESS_MAX_MS) {
                    queueEvent(rawEvent.key, InputType::ShortPress);
                }

                state.pressed = false;
            }

            // Forward the release event
            queueEvent(rawEvent.key, InputType::Release);
        }
    }

    // Check for long press on held keys
    for (size_t i = 0; i < static_cast<size_t>(InputKey::Count); i++) {
        KeyState& state = m_keyStates[i];
        if (state.pressed && !state.longPressFired) {
            uint32_t holdTime = timeMs - state.pressTime;
            if (holdTime >= LONG_PRESS_MS) {
                state.longPressFired = true;
                queueEvent(static_cast<InputKey>(i), InputType::LongPress);
            }
        }
    }

    // Check reset combo (LEFT + BACK held for 500ms)
    checkResetCombo(timeMs);
}

void InputManager::checkResetCombo(uint32_t timeMs) {
    bool leftHeld = m_keyStates[static_cast<size_t>(InputKey::Left)].pressed;
    bool backHeld = m_keyStates[static_cast<size_t>(InputKey::Back)].pressed;

    if (leftHeld && backHeld) {
        if (!m_comboActive) {
            // Combo just started
            m_comboActive = true;
            m_comboStartTime = timeMs;
        } else {
            // Check if held long enough
            if (timeMs - m_comboStartTime >= RESET_COMBO_MS) {
                if (m_resetCallback) {
                    m_resetCallback();
                }
                // Reset combo state to prevent repeated triggers
                m_comboActive = false;
                m_comboStartTime = 0;
            }
        }
    } else {
        // One or both keys released, reset combo
        m_comboActive = false;
        m_comboStartTime = 0;
    }
}

InputEvent InputManager::getEvent() {
    m_hasProcessedEvent = false;
    return m_processedEvent;
}

void InputManager::queueEvent(InputKey key, InputType type) {
    // Simple single-event queue - last event wins
    // For a more robust system, use a proper queue
    m_hasProcessedEvent = true;
    m_processedEvent.key = key;
    m_processedEvent.type = type;
}

} // namespace fri3d
