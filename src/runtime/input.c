#include "input.h"
#include <string.h>

static void input_manager_check_reset_combo(input_manager_t* manager, uint32_t time_ms);
static void input_manager_queue_event(input_manager_t* manager, input_key_t key, input_type_t type);

void input_manager_init(input_manager_t* manager) {
    if (!manager) {
        return;
    }
    memset(manager, 0, sizeof(*manager));
}

void input_manager_set_reset_callback(input_manager_t* manager, input_reset_callback_t callback, void* context) {
    if (!manager) {
        return;
    }
    manager->reset_callback = callback;
    manager->reset_callback_context = context;
}

void input_manager_update(input_manager_t* manager, input_handler_t* handler, uint32_t time_ms) {
    if (!manager || !handler) {
        return;
    }

    manager->has_processed_event = false;

    input_handler_poll(handler);

    while (input_handler_has_event(handler)) {
        input_event_t raw_event = input_handler_get_event(handler);
        size_t key_index = (size_t)raw_event.key;
        if (key_index >= INPUT_KEY_COUNT) {
            continue;
        }

        if (raw_event.type == input_type_press) {
            manager->key_states[key_index].pressed = true;
            manager->key_states[key_index].press_time = time_ms;
            manager->key_states[key_index].long_press_fired = false;

            input_manager_queue_event(manager, raw_event.key, input_type_press);
        } else if (raw_event.type == input_type_release) {
            if (manager->key_states[key_index].pressed) {
                uint32_t hold_time = time_ms - manager->key_states[key_index].press_time;
                if (!manager->key_states[key_index].long_press_fired && hold_time < INPUT_SHORT_PRESS_MAX_MS) {
                    input_manager_queue_event(manager, raw_event.key, input_type_short_press);
                }

                manager->key_states[key_index].pressed = false;
            }

            input_manager_queue_event(manager, raw_event.key, input_type_release);
        }
    }

    for (size_t i = 0; i < INPUT_KEY_COUNT; i++) {
        if (manager->key_states[i].pressed && !manager->key_states[i].long_press_fired) {
            uint32_t hold_time = time_ms - manager->key_states[i].press_time;
            if (hold_time >= INPUT_LONG_PRESS_MS) {
                manager->key_states[i].long_press_fired = true;
                input_manager_queue_event(manager, (input_key_t)i, input_type_long_press);
            }
        }
    }

    input_manager_check_reset_combo(manager, time_ms);
}

bool input_manager_has_event(const input_manager_t* manager) {
    return manager ? manager->has_processed_event : false;
}

input_event_t input_manager_get_event(input_manager_t* manager) {
    input_event_t event = {0};
    if (!manager) {
        return event;
    }
    manager->has_processed_event = false;
    event = manager->processed_event;
    return event;
}

static void input_manager_check_reset_combo(input_manager_t* manager, uint32_t time_ms) {
    bool left_held = manager->key_states[input_key_left].pressed;
    bool back_held = manager->key_states[input_key_back].pressed;

    if (left_held && back_held) {
        if (!manager->combo_active) {
            manager->combo_active = true;
            manager->combo_start_time = time_ms;
        } else if (time_ms - manager->combo_start_time >= INPUT_RESET_COMBO_MS) {
            if (manager->reset_callback) {
                manager->reset_callback(manager->reset_callback_context);
            }
            manager->combo_active = false;
            manager->combo_start_time = 0;
        }
    } else {
        manager->combo_active = false;
        manager->combo_start_time = 0;
    }
}

static void input_manager_queue_event(input_manager_t* manager, input_key_t key, input_type_t type) {
    manager->has_processed_event = true;
    manager->processed_event.key = key;
    manager->processed_event.type = type;
}
