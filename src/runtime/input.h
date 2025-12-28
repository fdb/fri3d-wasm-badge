#pragma once

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

// Input key values matching the SDK
typedef enum {
    INPUT_KEY_UP = 0,
    INPUT_KEY_DOWN = 1,
    INPUT_KEY_LEFT = 2,
    INPUT_KEY_RIGHT = 3,
    INPUT_KEY_OK = 4,
    INPUT_KEY_BACK = 5,
    INPUT_KEY_COUNT = 6
} input_key_t;

// Input event types
// ShortPress and LongPress are synthesized by InputManager
typedef enum {
    INPUT_TYPE_PRESS = 0,      // Key pressed down
    INPUT_TYPE_RELEASE = 1,    // Key released
    INPUT_TYPE_SHORT_PRESS = 2, // Key was pressed and released quickly (<300ms)
    INPUT_TYPE_LONG_PRESS = 3   // Key held for >=500ms
} input_type_t;

// Input event structure
typedef struct {
    input_key_t key;
    input_type_t type;
} input_event_t;

// Timing constants (in milliseconds)
#define SHORT_PRESS_MAX_MS 300
#define LONG_PRESS_MS 500
#define RESET_COMBO_MS 500  // LEFT+BACK held for 500ms

/**
 * Abstract input handler interface.
 * Platform implementations poll for raw key presses.
 */
typedef struct input_handler input_handler_t;

struct input_handler {
    void (*poll)(input_handler_t* self);
    bool (*has_event)(const input_handler_t* self);
    input_event_t (*get_event)(input_handler_t* self);
    uint32_t (*get_time_ms)(const input_handler_t* self);
};

// Callback type for reset combo
typedef void (*reset_callback_t)(void* context);

/**
 * Key state tracking (internal)
 */
typedef struct {
    bool pressed;
    uint32_t press_time;
    bool long_press_fired;
} key_state_t;

/**
 * Input manager that handles short/long press detection and reset combo.
 * Wraps a platform-specific InputHandler.
 */
typedef struct {
    // Key state tracking
    key_state_t key_states[INPUT_KEY_COUNT];

    // Processed event queue (single event for simplicity)
    bool has_processed_event;
    input_event_t processed_event;

    // Reset combo tracking
    uint32_t combo_start_time;
    bool combo_active;
    reset_callback_t reset_callback;
    void* reset_callback_context;
} input_manager_t;

/**
 * Initialize an input manager.
 */
void input_manager_init(input_manager_t* manager);

/**
 * Set the callback to invoke when LEFT+BACK reset combo is triggered.
 */
void input_manager_set_reset_callback(input_manager_t* manager, reset_callback_t callback, void* context);

/**
 * Process input from the given handler.
 * Call this each frame.
 * @param manager Input manager
 * @param handler Platform-specific input handler
 * @param time_ms Current time in milliseconds
 */
void input_manager_update(input_manager_t* manager, input_handler_t* handler, uint32_t time_ms);

/**
 * Check if an input event is available after processing.
 */
bool input_manager_has_event(const input_manager_t* manager);

/**
 * Get the next processed event.
 */
input_event_t input_manager_get_event(input_manager_t* manager);

#ifdef __cplusplus
}
#endif
