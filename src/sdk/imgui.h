#pragma once

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

// ============================================================================
// IMGUI View System for ESP32 128x64 Monochrome Display
// Immediate Mode GUI - no persistent widgets, render and query in same call
// ============================================================================

// ----------------------------------------------------------------------------
// Input Types (mirroring InputKey/InputType for UI layer)
// ----------------------------------------------------------------------------

typedef enum {
    UI_KEY_UP = 0,
    UI_KEY_DOWN = 1,
    UI_KEY_LEFT = 2,
    UI_KEY_RIGHT = 3,
    UI_KEY_OK = 4,
    UI_KEY_BACK = 5,
} UiKey;

typedef enum {
    UI_INPUT_PRESS = 0,
    UI_INPUT_RELEASE = 1,
    UI_INPUT_SHORT = 2,    // Short press (< 500ms)
    UI_INPUT_LONG = 3,     // Long press (>= 500ms)
    UI_INPUT_REPEAT = 4,   // Repeat while held
} UiInputType;

// ----------------------------------------------------------------------------
// Font and Alignment (matching canvas.h enums)
// ----------------------------------------------------------------------------

typedef enum {
    UI_FONT_PRIMARY = 0,    // 12px - headers, titles
    UI_FONT_SECONDARY = 1,  // 10px - body text, values
    UI_FONT_KEYBOARD = 2,   // Keyboard font
    UI_FONT_BIG_NUMBERS = 3,// Large numbers
} UiFont;

typedef enum {
    UI_ALIGN_LEFT = 0,
    UI_ALIGN_RIGHT = 1,
    UI_ALIGN_CENTER = 2,
} UiAlign;

// ----------------------------------------------------------------------------
// Layout Direction
// ----------------------------------------------------------------------------

typedef enum {
    UI_LAYOUT_VERTICAL = 0,
    UI_LAYOUT_HORIZONTAL = 1,
} UiLayoutDirection;

// ----------------------------------------------------------------------------
// Context Management
// ----------------------------------------------------------------------------

// Begin a new frame - clears canvas, resets widget state
void ui_begin(void);

// End the frame - commits canvas buffer to display
void ui_end(void);

// Feed an input event to the UI system (call from on_input)
void ui_input(UiKey key, UiInputType type);

// Check if Back button was pressed this frame
bool ui_back_pressed(void);

// ----------------------------------------------------------------------------
// Layout System
// ----------------------------------------------------------------------------

// Begin a vertical stack with specified pixel spacing between items
void ui_vstack(int16_t spacing);

// Begin a horizontal stack with specified pixel spacing between items
void ui_hstack(int16_t spacing);

// End the current stack and return to parent layout
void ui_end_stack(void);

// Add empty space in the current stack direction
void ui_spacer(int16_t pixels);

// Set absolute position for next widget (bypasses layout)
void ui_set_position(int16_t x, int16_t y);

// ----------------------------------------------------------------------------
// Basic Widgets
// ----------------------------------------------------------------------------

// Draw a text label (not focusable)
void ui_label(const char* text, UiFont font, UiAlign align);

// Draw a horizontal separator line
void ui_separator(void);

// Draw a focusable button - returns true when activated
bool ui_button(const char* text);

// Draw a button at absolute position - returns true when activated
bool ui_button_at(int16_t x, int16_t y, const char* text);

// Draw a progress bar (value 0.0 to 1.0, width 0 = full layout width)
void ui_progress(float value, int16_t width);

// Draw an XBM-format icon
void ui_icon(const uint8_t* data, uint8_t width, uint8_t height);

// Draw a focusable checkbox - returns true when value changes
bool ui_checkbox(const char* text, bool* checked);

// ----------------------------------------------------------------------------
// Menu System
// ----------------------------------------------------------------------------

// Begin a scrollable menu
// scroll: pointer to scroll position (persists across frames)
// visible: number of visible items
// total: total number of items
void ui_menu_begin(int16_t* scroll, int16_t visible, int16_t total);

// Add a menu item - returns true when selected (OK pressed)
bool ui_menu_item(const char* text, int16_t index);

// Add a menu item with right-aligned value - returns true when selected
bool ui_menu_item_value(const char* label, const char* value, int16_t index);

// End menu and draw scrollbar
void ui_menu_end(void);

// ----------------------------------------------------------------------------
// Footer Buttons (Fixed Position at Screen Bottom)
// ----------------------------------------------------------------------------

// Left footer button (Left arrow + text) - returns true on Left+Short
bool ui_footer_left(const char* text);

// Center footer button (OK + text) - returns true on OK+Short (if not focused elsewhere)
bool ui_footer_center(const char* text);

// Right footer button (Right arrow + text) - returns true on Right+Short
bool ui_footer_right(const char* text);

// ----------------------------------------------------------------------------
// Focus Management
// ----------------------------------------------------------------------------

// Get current focus index (-1 if no focusable widgets)
int16_t ui_get_focus(void);

// Set focus to specific widget index
void ui_set_focus(int16_t index);

// Check if specific widget index is focused
bool ui_is_focused(int16_t index);

// ----------------------------------------------------------------------------
// Screen Constants
// ----------------------------------------------------------------------------

#define UI_SCREEN_WIDTH  128
#define UI_SCREEN_HEIGHT 64
