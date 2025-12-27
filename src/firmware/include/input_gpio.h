#pragma once

#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// Input key values (matching SDK)
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
typedef enum {
    INPUT_TYPE_PRESS = 0,
    INPUT_TYPE_RELEASE = 1
} input_type_t;

// Input event
typedef struct {
    input_key_t key;
    input_type_t type;
} input_event_t;

/**
 * Initialize GPIO input handling.
 */
void input_init(void);

/**
 * Poll for input changes.
 * Call this each frame.
 */
void input_poll(void);

/**
 * Check if an input event is available.
 */
bool input_has_event(void);

/**
 * Get the next input event.
 */
input_event_t input_get_event(void);

/**
 * Check if a key is currently pressed.
 */
bool input_is_pressed(input_key_t key);

/**
 * Get current time in milliseconds.
 */
uint32_t input_get_time_ms(void);

#ifdef __cplusplus
}
#endif
