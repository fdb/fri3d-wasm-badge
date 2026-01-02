#include "input.h"
#include <string.h>

#ifdef FRD_TRACE
#include "trace.h"
#endif

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
            manager->key_states[key_index].last_repeat_time = time_ms;

            input_manager_queue_event(manager, raw_event.key, input_type_press);
        } else if (raw_event.type == input_type_release) {
            if (manager->key_states[key_index].pressed) {
                uint32_t hold_time = time_ms - manager->key_states[key_index].press_time;
                if (!manager->key_states[key_index].long_press_fired) {
                    if (hold_time >= INPUT_LONG_PRESS_MS) {
                        input_manager_queue_event(manager, raw_event.key, input_type_long_press);
                    } else {
                        input_manager_queue_event(manager, raw_event.key, input_type_short_press);
                    }
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
                manager->key_states[i].last_repeat_time = time_ms;
                input_manager_queue_event(manager, (input_key_t)i, input_type_long_press);
            }
        }

        if (manager->key_states[i].pressed && manager->key_states[i].long_press_fired) {
            uint32_t since_repeat = time_ms - manager->key_states[i].last_repeat_time;
            if (since_repeat >= INPUT_REPEAT_INTERVAL_MS) {
                manager->key_states[i].last_repeat_time = time_ms;
                input_manager_queue_event(manager, (input_key_t)i, input_type_repeat);
            }
        }
    }

    input_manager_check_reset_combo(manager, time_ms);
}

bool input_manager_has_event(const input_manager_t* manager) {
    return manager ? manager->queue_head != manager->queue_tail : false;
}

input_event_t input_manager_get_event(input_manager_t* manager) {
    input_event_t event = {0};
    if (!manager) {
        return event;
    }
    if (manager->queue_head == manager->queue_tail) {
        return event;
    }
    event = manager->event_queue[manager->queue_tail];
    manager->queue_tail = (manager->queue_tail + 1) % INPUT_EVENT_QUEUE_SIZE;
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
    if (!manager) {
        return;
    }
    size_t next_head = (manager->queue_head + 1) % INPUT_EVENT_QUEUE_SIZE;
    if (next_head == manager->queue_tail) {
        return;
    }
#ifdef FRD_TRACE
    trace_arg_t args[] = { TRACE_ARG_INT((int32_t)key), TRACE_ARG_INT((int32_t)type) };
    trace_call("input_event", args, 2);
#endif
    manager->event_queue[manager->queue_head].key = key;
    manager->event_queue[manager->queue_head].type = type;
    manager->queue_head = next_head;
}
