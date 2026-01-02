#pragma once

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

// Input key values matching the SDK
typedef enum {
    input_key_up = 0,
    input_key_down = 1,
    input_key_left = 2,
    input_key_right = 3,
    input_key_ok = 4,
    input_key_back = 5,
} input_key_t;

#define INPUT_KEY_COUNT 6

// Input event types
// Short/long press are synthesized by the input manager
typedef enum {
    input_type_press = 0,
    input_type_release = 1,
    input_type_short_press = 2,
    input_type_long_press = 3,
} input_type_t;

// Input event structure
typedef struct {
    input_key_t key;
    input_type_t type;
} input_event_t;

// Timing constants (in milliseconds)
#define INPUT_SHORT_PRESS_MAX_MS 300
#define INPUT_LONG_PRESS_MS 500
#define INPUT_RESET_COMBO_MS 500

// Event queue size
#define INPUT_EVENT_QUEUE_SIZE 16

// Input handler interface
// Implementations should fill in the function pointers and context.
typedef struct {
    void* context;
    void (*poll)(void* context);
    bool (*has_event)(void* context);
    input_event_t (*get_event)(void* context);
    uint32_t (*get_time_ms)(void* context);
} input_handler_t;

// Reset callback type
typedef void (*input_reset_callback_t)(void* context);

// Input manager state
typedef struct {
    struct {
        bool pressed;
        uint32_t press_time;
        bool long_press_fired;
    } key_states[INPUT_KEY_COUNT];

    input_event_t event_queue[INPUT_EVENT_QUEUE_SIZE];
    size_t queue_head;
    size_t queue_tail;

    uint32_t combo_start_time;
    bool combo_active;
    input_reset_callback_t reset_callback;
    void* reset_callback_context;
} input_manager_t;

void input_manager_init(input_manager_t* manager);
void input_manager_set_reset_callback(input_manager_t* manager, input_reset_callback_t callback, void* context);
void input_manager_update(input_manager_t* manager, input_handler_t* handler, uint32_t time_ms);
bool input_manager_has_event(const input_manager_t* manager);
input_event_t input_manager_get_event(input_manager_t* manager);

// Helper wrappers for input_handler_t
static inline void input_handler_poll(input_handler_t* handler) {
    if (handler && handler->poll) {
        handler->poll(handler->context);
    }
}

static inline bool input_handler_has_event(input_handler_t* handler) {
    if (handler && handler->has_event) {
        return handler->has_event(handler->context);
    }
    return false;
}

static inline input_event_t input_handler_get_event(input_handler_t* handler) {
    input_event_t event = {0};
    if (handler && handler->get_event) {
        event = handler->get_event(handler->context);
    }
    return event;
}

static inline uint32_t input_handler_get_time_ms(input_handler_t* handler) {
    if (handler && handler->get_time_ms) {
        return handler->get_time_ms(handler->context);
    }
    return 0;
}
