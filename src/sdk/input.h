#pragma once

#include <stdint.h>

typedef enum {
    input_key_up = 0,
    input_key_down = 1,
    input_key_left = 2,
    input_key_right = 3,
    input_key_ok = 4,
    input_key_back = 5,
} input_key_t;

typedef enum {
    input_type_press = 0,
    input_type_release = 1,
    input_type_short_press = 2,
    input_type_long_press = 3,
} input_type_t;
