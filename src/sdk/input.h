#pragma once

#include <stdint.h>

typedef enum {
    InputKeyUp = 0,
    InputKeyDown = 1,
    InputKeyLeft = 2,
    InputKeyRight = 3,
    InputKeyOk = 4,      // A button / Enter / Z
    InputKeyBack = 5,    // B button / Backspace / X
} InputKey;

typedef enum {
    InputTypePress = 0,
    InputTypeRelease = 1,
} InputType;
