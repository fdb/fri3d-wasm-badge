#pragma once

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#include "canvas.h"

// ============================================================================
// IMGUI View System for ESP32 128x64 Monochrome Display
// Immediate Mode GUI - no persistent widgets, render and query in same call
// ============================================================================

// ----------------------------------------------------------------------------
// Input Types (mirroring input_key_t/input_type_t for UI layer)
// ----------------------------------------------------------------------------

typedef enum {
    ui_key_up = 0,
    ui_key_down = 1,
    ui_key_left = 2,
    ui_key_right = 3,
    ui_key_ok = 4,
    ui_key_back = 5,
} ui_key_t;

typedef enum {
    ui_input_press = 0,
    ui_input_release = 1,
    ui_input_short = 2,
    ui_input_long = 3,
    ui_input_repeat = 4,
} ui_input_type_t;

// ----------------------------------------------------------------------------
// Layout Direction
// ----------------------------------------------------------------------------

typedef enum {
    ui_layout_vertical = 0,
    ui_layout_horizontal = 1,
} ui_layout_direction_t;

// ----------------------------------------------------------------------------
// Virtual Keyboard
// ----------------------------------------------------------------------------

typedef bool (*ui_virtual_keyboard_validator_t)(
    const char* text,
    char* message,
    size_t message_size,
    void* context);

typedef struct {
    char* buffer;
    size_t capacity;
    size_t min_len;

    uint8_t row;
    uint8_t col;

    bool clear_default_text;

    bool validator_visible;
    uint32_t validator_deadline_ms;
    ui_virtual_keyboard_validator_t validator;
    void* validator_context;
    char validator_message[64];
} ui_virtual_keyboard_t;

void ui_virtual_keyboard_init(ui_virtual_keyboard_t* keyboard, char* buffer, size_t capacity);
void ui_virtual_keyboard_set_min_length(ui_virtual_keyboard_t* keyboard, size_t min_len);
void ui_virtual_keyboard_set_validator(
    ui_virtual_keyboard_t* keyboard,
    ui_virtual_keyboard_validator_t validator,
    void* context);
bool ui_virtual_keyboard(ui_virtual_keyboard_t* keyboard, const char* header, uint32_t now_ms);

// ----------------------------------------------------------------------------
// Context Management
// ----------------------------------------------------------------------------

void ui_begin(void);
void ui_end(void);
void ui_input(ui_key_t key, ui_input_type_t type);
bool ui_back_pressed(void);

// ----------------------------------------------------------------------------
// Layout System
// ----------------------------------------------------------------------------

void ui_vstack(int16_t spacing);
void ui_hstack(int16_t spacing);
void ui_hstack_centered(int16_t spacing);
void ui_end_stack(void);
void ui_spacer(int16_t pixels);
void ui_set_position(int16_t x, int16_t y);

// ----------------------------------------------------------------------------
// Basic Widgets
// ----------------------------------------------------------------------------

void ui_label(const char* text, Font font, Align align);
void ui_separator(void);
bool ui_button(const char* text);
bool ui_button_at(int16_t x, int16_t y, const char* text);
void ui_progress(float value, int16_t width);
void ui_icon(const uint8_t* data, uint8_t width, uint8_t height);
bool ui_checkbox(const char* text, bool* checked);

// ----------------------------------------------------------------------------
// Menu System
// ----------------------------------------------------------------------------

void ui_menu_begin(int16_t* scroll, int16_t visible, int16_t total);
bool ui_menu_item(const char* text, int16_t index);
bool ui_menu_item_value(const char* label, const char* value, int16_t index);
void ui_menu_end(void);

// ----------------------------------------------------------------------------
// Footer Buttons (Fixed Position at Screen Bottom)
// ----------------------------------------------------------------------------

bool ui_footer_left(const char* text);
bool ui_footer_center(const char* text);
bool ui_footer_right(const char* text);

// ----------------------------------------------------------------------------
// Focus Management
// ----------------------------------------------------------------------------

int16_t ui_get_focus(void);
void ui_set_focus(int16_t index);
bool ui_is_focused(int16_t index);

// ----------------------------------------------------------------------------
// Screen Constants
// ----------------------------------------------------------------------------

#define UI_SCREEN_WIDTH  128
#define UI_SCREEN_HEIGHT 64
