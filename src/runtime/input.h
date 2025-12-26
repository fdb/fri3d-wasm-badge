#pragma once

#include <cstddef>
#include <cstdint>
#include <functional>

namespace fri3d {

// Input key values matching the SDK
enum class InputKey : uint8_t {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
    Ok = 4,
    Back = 5,
    Count = 6  // Number of keys
};

// Input event types
// ShortPress and LongPress are synthesized by InputManager
enum class InputType : uint8_t {
    Press = 0,      // Key pressed down
    Release = 1,    // Key released
    ShortPress = 2, // Key was pressed and released quickly (<300ms)
    LongPress = 3   // Key held for >=500ms
};

// Input event structure
struct InputEvent {
    InputKey key;
    InputType type;
};

// Timing constants (in milliseconds)
constexpr uint32_t SHORT_PRESS_MAX_MS = 300;
constexpr uint32_t LONG_PRESS_MS = 500;
constexpr uint32_t RESET_COMBO_MS = 500;  // LEFT+BACK held for 500ms

/**
 * Abstract input handler interface.
 * Platform implementations poll for raw key presses.
 */
class InputHandler {
public:
    virtual ~InputHandler() = default;

    /**
     * Poll for input events.
     * Should be called each frame.
     */
    virtual void poll() = 0;

    /**
     * Check if an event is available.
     */
    virtual bool hasEvent() const = 0;

    /**
     * Get the next event. Call hasEvent() first.
     */
    virtual InputEvent getEvent() = 0;

    /**
     * Get current timestamp in milliseconds.
     * Used for timing calculations.
     */
    virtual uint32_t getTimeMs() const = 0;
};

/**
 * Input manager that handles short/long press detection and reset combo.
 * Wraps a platform-specific InputHandler.
 */
class InputManager {
public:
    using ResetCallback = std::function<void()>;

    /**
     * Set the callback to invoke when LEFT+BACK reset combo is triggered.
     */
    void setResetCallback(ResetCallback callback) { m_resetCallback = callback; }

    /**
     * Process input from the given handler.
     * Call this each frame.
     * @param handler Platform-specific input handler
     * @param timeMs Current time in milliseconds
     */
    void update(InputHandler& handler, uint32_t timeMs);

    /**
     * Check if an input event is available after processing.
     */
    bool hasEvent() const { return m_hasProcessedEvent; }

    /**
     * Get the next processed event.
     */
    InputEvent getEvent();

private:
    // Key state tracking
    struct KeyState {
        bool pressed = false;
        uint32_t pressTime = 0;
        bool longPressFired = false;
    };

    KeyState m_keyStates[static_cast<size_t>(InputKey::Count)];

    // Processed event queue (single event for simplicity)
    bool m_hasProcessedEvent = false;
    InputEvent m_processedEvent;

    // Reset combo tracking
    uint32_t m_comboStartTime = 0;
    bool m_comboActive = false;
    ResetCallback m_resetCallback;

    void checkResetCombo(uint32_t timeMs);
    void queueEvent(InputKey key, InputType type);
};

} // namespace fri3d
