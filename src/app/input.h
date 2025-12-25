#pragma once

#include <stdint.h>

typedef enum {
    InputKeyUp,
    InputKeyDown,
    InputKeyLeft,
    InputKeyRight,
    InputKeyOk,      // A button / Enter / Z
    InputKeyBack,    // B button / Backspace / X
} InputKey;

typedef enum {
    InputTypePress,
    InputTypeRelease,
    InputTypeShort,   // Short press (future)
    InputTypeLong,    // Long press (future)
    InputTypeRepeat,  // Key repeat (future)
} InputType;

typedef struct {
    InputKey key;
    InputType type;
} InputEvent;
