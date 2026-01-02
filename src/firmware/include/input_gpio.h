#pragma once

#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// Input key values (matching SDK)
typedef enum {
    input_key_up = 0,
    input_key_down = 1,
    input_key_left = 2,
    input_key_right = 3,
    input_key_ok = 4,
    input_key_back = 5,
    input_key_count = 6,
} input_key_t;

// Input event types
typedef enum {
    input_type_press = 0,
    input_type_release = 1
} input_type_t;

// Input event
typedef struct {
    input_key_t key;
    input_type_t type;
} input_event_t;

void input_init(void);
void input_poll(void);
bool input_has_event(void);
input_event_t input_get_event(void);
bool input_is_pressed(input_key_t key);
uint32_t input_get_time_ms(void);

#ifdef __cplusplus
}
#endif
