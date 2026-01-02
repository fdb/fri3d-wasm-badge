#pragma once

#include <u8g2.h>
#include <stdbool.h>

// Screen dimensions (matching SSD1306 128x64)
#define FRI3D_SCREEN_WIDTH 128
#define FRI3D_SCREEN_HEIGHT 64

// Display interface typedef for platform implementations
typedef struct display_api {
    bool (*init)(void* display, bool headless);
    u8g2_t* (*get_u8g2)(void* display);
    void (*flush)(void* display);
    bool (*should_quit)(void* display);
} display_api_t;
