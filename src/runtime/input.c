#include "input.h"
#include <string.h>

static void input_manager_check_reset_combo(input_manager_t* manager, uint32_t time_ms);
static void input_manager_queue_event(input_manager_t* manager, input_key_t key, input_type_t type);

void input_manager_init(input_manager_t* manager) {
    memset(manager, 0, sizeof(input_manager_t));
}

void input_manager_set_reset_callback(input_manager_t* manager, reset_callback_t callback, void* context) {
    manager->reset_callback = callback;
    manager->reset_callback_context = context;
}

void input_manager_update(input_manager_t* manager, input_handler_t* handler, uint32_t time_ms) {
    // Clear previous processed event
    manager->has_processed_event = false;

    // Poll for new raw events
    handler->poll(handler);

    // Process raw events
    while (handler->has_event(handler)) {
        input_event_t raw_event = handler->get_event(handler);
        size_t key_index = (size_t)raw_event.key;

        if (raw_event.type == INPUT_TYPE_PRESS) {
            // Key pressed down
            manager->key_states[key_index].pressed = true;
            manager->key_states[key_index].press_time = time_ms;
            manager->key_states[key_index].long_press_fired = false;

            // Forward the press event
            input_manager_queue_event(manager, raw_event.key, INPUT_TYPE_PRESS);

        } else if (raw_event.type == INPUT_TYPE_RELEASE) {
            // Key released
            key_state_t* state = &manager->key_states[key_index];

            if (state->pressed) {
                uint32_t hold_time = time_ms - state->press_time;

                // If long press wasn't fired yet, check if it was a short press
                if (!state->long_press_fired && hold_time < SHORT_PRESS_MAX_MS) {
                    input_manager_queue_event(manager, raw_event.key, INPUT_TYPE_SHORT_PRESS);
                }

                state->pressed = false;
            }

            // Forward the release event
            input_manager_queue_event(manager, raw_event.key, INPUT_TYPE_RELEASE);
        }
    }

    // Check for long press on held keys
    for (size_t i = 0; i < INPUT_KEY_COUNT; i++) {
        key_state_t* state = &manager->key_states[i];
        if (state->pressed && !state->long_press_fired) {
            uint32_t hold_time = time_ms - state->press_time;
            if (hold_time >= LONG_PRESS_MS) {
                state->long_press_fired = true;
                input_manager_queue_event(manager, (input_key_t)i, INPUT_TYPE_LONG_PRESS);
            }
        }
    }

    // Check reset combo (LEFT + BACK held for 500ms)
    input_manager_check_reset_combo(manager, time_ms);
}

static void input_manager_check_reset_combo(input_manager_t* manager, uint32_t time_ms) {
    bool left_held = manager->key_states[INPUT_KEY_LEFT].pressed;
    bool back_held = manager->key_states[INPUT_KEY_BACK].pressed;

    if (left_held && back_held) {
        if (!manager->combo_active) {
            // Combo just started
            manager->combo_active = true;
            manager->combo_start_time = time_ms;
        } else {
            // Check if held long enough
            if (time_ms - manager->combo_start_time >= RESET_COMBO_MS) {
                if (manager->reset_callback) {
                    manager->reset_callback(manager->reset_callback_context);
                }
                // Reset combo state to prevent repeated triggers
                manager->combo_active = false;
                manager->combo_start_time = 0;
            }
        }
    } else {
        // One or both keys released, reset combo
        manager->combo_active = false;
        manager->combo_start_time = 0;
    }
}

bool input_manager_has_event(const input_manager_t* manager) {
    return manager->has_processed_event;
}

input_event_t input_manager_get_event(input_manager_t* manager) {
    manager->has_processed_event = false;
    return manager->processed_event;
}

static void input_manager_queue_event(input_manager_t* manager, input_key_t key, input_type_t type) {
    // Simple single-event queue - last event wins
    // For a more robust system, use a proper queue
    manager->has_processed_event = true;
    manager->processed_event.key = key;
    manager->processed_event.type = type;
}
