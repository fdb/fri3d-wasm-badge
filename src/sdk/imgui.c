// ============================================================================
// IMGUI Implementation for ESP32 128x64 Monochrome Display
// ============================================================================

#include "imgui.h"
#include "canvas.h"
#include "input.h"

#include <ctype.h>
#include <stdio.h>
#include <string.h>

// ----------------------------------------------------------------------------
// Internal Constants
// ----------------------------------------------------------------------------

#define UI_MAX_LAYOUT_DEPTH 8
#define UI_MAX_FOCUSABLE 32
#define UI_MAX_DEFERRED 16
#define UI_FONT_HEIGHT_PRIMARY 12
#define UI_FONT_HEIGHT_SECONDARY 11
#define UI_BUTTON_PADDING_X 4
#define UI_BUTTON_PADDING_Y 2
#define UI_MENU_ITEM_HEIGHT 12
#define UI_FOOTER_HEIGHT 12
#define UI_SCROLLBAR_WIDTH 3
#define UI_VK_ORIGIN_X 1
#define UI_VK_ORIGIN_Y 29
#define UI_VK_ROW_COUNT 3
#define UI_VK_VALIDATOR_TIMEOUT_MS 4000

#define UI_VK_ENTER_KEY '\r'
#define UI_VK_BACKSPACE_KEY '\b'

#define UI_VK_BACKSPACE_W 16
#define UI_VK_BACKSPACE_H 9
#define UI_VK_ENTER_W 24
#define UI_VK_ENTER_H 11

// ----------------------------------------------------------------------------
// Internal Types
// ----------------------------------------------------------------------------

typedef struct {
    int16_t x;
    int16_t y;
    int16_t width;
    int16_t height;
    ui_layout_direction_t direction;
    int16_t spacing;
    int16_t cursor;  // Current position in layout direction
    bool centered;   // For hstacks: defer drawing until end_stack for centering
} ui_layout_stack_t;

// Deferred button draw operation
typedef struct {
    int16_t x;
    int16_t y;
    int16_t width;
    int16_t height;
    const char* text;
    bool focused;
} ui_deferred_button_t;

typedef struct {
    char text;
    uint8_t x;
    uint8_t y;
} ui_vk_key_t;

static const ui_vk_key_t g_vk_row_1[] = {
    {'q', 1, 8},
    {'w', 10, 8},
    {'e', 19, 8},
    {'r', 28, 8},
    {'t', 37, 8},
    {'y', 46, 8},
    {'u', 55, 8},
    {'i', 64, 8},
    {'o', 73, 8},
    {'p', 82, 8},
    {'0', 91, 8},
    {'1', 100, 8},
    {'2', 110, 8},
    {'3', 120, 8},
};

static const ui_vk_key_t g_vk_row_2[] = {
    {'a', 1, 20},
    {'s', 10, 20},
    {'d', 19, 20},
    {'f', 28, 20},
    {'g', 37, 20},
    {'h', 46, 20},
    {'j', 55, 20},
    {'k', 64, 20},
    {'l', 73, 20},
    {UI_VK_BACKSPACE_KEY, 82, 12},
    {'4', 100, 20},
    {'5', 110, 20},
    {'6', 120, 20},
};

static const ui_vk_key_t g_vk_row_3[] = {
    {'z', 1, 32},
    {'x', 10, 32},
    {'c', 19, 32},
    {'v', 28, 32},
    {'b', 37, 32},
    {'n', 46, 32},
    {'m', 55, 32},
    {'_', 64, 32},
    {UI_VK_ENTER_KEY, 74, 23},
    {'7', 100, 32},
    {'8', 110, 32},
    {'9', 120, 32},
};

typedef struct {
    // Layout stack
    ui_layout_stack_t layout_stack[UI_MAX_LAYOUT_DEPTH];
    int8_t layout_depth;

    // Focus tracking
    int16_t focus_index;      // Currently focused widget (-1 = none)
    int16_t focus_count;      // Focusable widgets this frame
    int16_t prev_focus_count; // Focus count from previous frame

    // Input state for current frame
    ui_key_t last_key;
    ui_input_type_t last_type;
    bool has_input;
    bool ok_pressed;
    bool back_pressed;

    // Menu state (during ui_menu_begin..end)
    struct {
        bool active;
        int16_t* scroll;
        int16_t visible;
        int16_t total;
        int16_t y_start;
        int16_t first_item_focus;  // Focus index of first menu item
    } menu;

    // Absolute position override
    bool use_absolute;
    int16_t abs_x;
    int16_t abs_y;

    // Deferred drawing for centered hstacks
    ui_deferred_button_t deferred_buttons[UI_MAX_DEFERRED];
    int8_t deferred_count;

} ui_context_t;

// ----------------------------------------------------------------------------
// Global State
// ----------------------------------------------------------------------------

static ui_context_t g_ctx = {0};

// ----------------------------------------------------------------------------
// Internal Helpers
// ----------------------------------------------------------------------------

static int16_t ui_get_font_height(Font font) {
    switch (font) {
        case FontPrimary:
            return UI_FONT_HEIGHT_PRIMARY;
        case FontSecondary:
        default:
            return UI_FONT_HEIGHT_SECONDARY;
    }
}

static ui_layout_stack_t* ui_current_layout(void) {
    if (g_ctx.layout_depth < 0) return NULL;
    return &g_ctx.layout_stack[g_ctx.layout_depth];
}

static void ui_layout_next(int16_t width, int16_t height, int16_t* out_x, int16_t* out_y, int16_t* out_w) {
    // Check for absolute position override
    if (g_ctx.use_absolute) {
        *out_x = g_ctx.abs_x;
        *out_y = g_ctx.abs_y;
        *out_w = width;
        g_ctx.use_absolute = false;
        return;
    }

    ui_layout_stack_t* layout = ui_current_layout();
    if (!layout) {
        *out_x = 0;
        *out_y = 0;
        *out_w = UI_SCREEN_WIDTH;
        return;
    }

    if (layout->direction == ui_layout_vertical) {
        *out_x = layout->x;
        *out_y = layout->y + layout->cursor;
        *out_w = layout->width;
        layout->cursor += height + layout->spacing;
    } else {
        *out_x = layout->x + layout->cursor;
        *out_y = layout->y;
        *out_w = width;
        layout->cursor += width + layout->spacing;
    }
}

static int16_t ui_register_focusable(void) {
    if (g_ctx.focus_count >= UI_MAX_FOCUSABLE) {
        return -1;
    }
    return g_ctx.focus_count++;
}

static bool ui_check_focused(int16_t widget_idx) {
    return g_ctx.focus_index == widget_idx;
}

static bool ui_check_activated(int16_t widget_idx) {
    return ui_check_focused(widget_idx) && g_ctx.ok_pressed;
}

static bool ui_in_centered_hstack(void) {
    ui_layout_stack_t* layout = ui_current_layout();
    return layout && layout->direction == ui_layout_horizontal && layout->centered;
}

static void ui_draw_button_internal(int16_t x, int16_t y, int16_t w, int16_t h, const char* text, bool focused) {
    canvas_set_font(FontSecondary);
    if (focused) {
        canvas_set_color(ColorBlack);
        canvas_draw_rbox(x, y, w, h, 2);
        canvas_set_color(ColorWhite);
        canvas_draw_str(x + UI_BUTTON_PADDING_X, y + h - UI_BUTTON_PADDING_Y, text);
    } else {
        canvas_set_color(ColorBlack);
        canvas_draw_rframe(x, y, w, h, 2);
        canvas_draw_str(x + UI_BUTTON_PADDING_X, y + h - UI_BUTTON_PADDING_Y, text);
    }
}

static uint8_t ui_vk_row_size(uint8_t row_index) {
    switch (row_index) {
        case 0:
            return (uint8_t)(sizeof(g_vk_row_1) / sizeof(g_vk_row_1[0]));
        case 1:
            return (uint8_t)(sizeof(g_vk_row_2) / sizeof(g_vk_row_2[0]));
        case 2:
            return (uint8_t)(sizeof(g_vk_row_3) / sizeof(g_vk_row_3[0]));
        default:
            return 0;
    }
}

static const ui_vk_key_t* ui_vk_row(uint8_t row_index) {
    switch (row_index) {
        case 0:
            return g_vk_row_1;
        case 1:
            return g_vk_row_2;
        case 2:
            return g_vk_row_3;
        default:
            return NULL;
    }
}

static char ui_vk_selected_char(const ui_virtual_keyboard_t* keyboard) {
    const ui_vk_key_t* row = ui_vk_row(keyboard->row);
    if (!row) {
        return 0;
    }
    return row[keyboard->col].text;
}

static bool ui_vk_is_lowercase(char letter) {
    return (letter >= 'a' && letter <= 'z');
}

static char ui_vk_to_uppercase(char letter) {
    if (letter == '_') {
        return ' ';
    }
    if (ui_vk_is_lowercase(letter)) {
        return (char)(letter - 0x20);
    }
    return letter;
}

static size_t ui_vk_text_length(const ui_virtual_keyboard_t* keyboard) {
    if (!keyboard || !keyboard->buffer) {
        return 0;
    }
    return strlen(keyboard->buffer);
}

static void ui_vk_backspace(ui_virtual_keyboard_t* keyboard) {
    if (!keyboard || !keyboard->buffer || keyboard->capacity == 0) {
        return;
    }

    if (keyboard->clear_default_text) {
        keyboard->buffer[0] = '\0';
        keyboard->clear_default_text = false;
        return;
    }

    size_t length = strlen(keyboard->buffer);
    if (length > 0) {
        keyboard->buffer[length - 1] = '\0';
    }
}

static void ui_vk_set_validator_message(ui_virtual_keyboard_t* keyboard, const char* message) {
    if (!keyboard) {
        return;
    }
    if (!message) {
        keyboard->validator_message[0] = '\0';
        return;
    }
    snprintf(keyboard->validator_message, sizeof(keyboard->validator_message), "%s", message);
}

static void ui_vk_show_validator(ui_virtual_keyboard_t* keyboard, uint32_t now_ms, const char* fallback) {
    if (!keyboard) {
        return;
    }
    keyboard->validator_visible = true;
    keyboard->validator_deadline_ms = now_ms + UI_VK_VALIDATOR_TIMEOUT_MS;
    if (keyboard->validator_message[0] == '\0') {
        ui_vk_set_validator_message(keyboard, fallback);
    }
}

static bool ui_vk_handle_ok(ui_virtual_keyboard_t* keyboard, bool shift, uint32_t now_ms) {
    if (!keyboard || !keyboard->buffer || keyboard->capacity == 0) {
        return false;
    }

    char selected = ui_vk_selected_char(keyboard);
    size_t text_length = ui_vk_text_length(keyboard);

    bool toggle_case = (text_length == 0) || keyboard->clear_default_text;
    if (shift) {
        toggle_case = !toggle_case;
    }
    if (toggle_case) {
        selected = ui_vk_to_uppercase(selected);
    }

    if (selected == UI_VK_ENTER_KEY) {
        if (keyboard->validator) {
            keyboard->validator_message[0] = '\0';
            bool ok = keyboard->validator(
                keyboard->buffer,
                keyboard->validator_message,
                sizeof(keyboard->validator_message),
                keyboard->validator_context);
            if (!ok) {
                ui_vk_show_validator(keyboard, now_ms, "Invalid input");
                return false;
            }
        }

        if (text_length >= keyboard->min_len) {
            return true;
        }
        return false;
    }

    if (selected == UI_VK_BACKSPACE_KEY) {
        ui_vk_backspace(keyboard);
        keyboard->clear_default_text = false;
        return false;
    }

    if (keyboard->clear_default_text) {
        text_length = 0;
    }

    if (text_length + 1 < keyboard->capacity) {
        keyboard->buffer[text_length] = selected;
        keyboard->buffer[text_length + 1] = '\0';
    }

    keyboard->clear_default_text = false;
    return false;
}

static void ui_vk_move_left(ui_virtual_keyboard_t* keyboard) {
    int row_len = (int)ui_vk_row_size(keyboard->row);
    if (row_len <= 0) {
        return;
    }
    if (keyboard->col > 0) {
        keyboard->col--;
    } else {
        keyboard->col = (uint8_t)(row_len - 1);
    }
}

static void ui_vk_move_right(ui_virtual_keyboard_t* keyboard) {
    int row_len = (int)ui_vk_row_size(keyboard->row);
    if (row_len <= 0) {
        return;
    }
    if (keyboard->col + 1 < (uint8_t)row_len) {
        keyboard->col++;
    } else {
        keyboard->col = 0;
    }
}

static void ui_vk_move_up(ui_virtual_keyboard_t* keyboard) {
    if (!keyboard) {
        return;
    }
    if (keyboard->row == 0) {
        return;
    }
    keyboard->row = (uint8_t)(keyboard->row - 1);

    int row_len = (int)ui_vk_row_size(keyboard->row);
    if (row_len <= 0) {
        keyboard->col = 0;
        return;
    }

    if (keyboard->col > (uint8_t)(row_len - 6)) {
        keyboard->col++;
    }
    if (keyboard->col >= (uint8_t)row_len) {
        keyboard->col = (uint8_t)(row_len - 1);
    }
}

static void ui_vk_move_down(ui_virtual_keyboard_t* keyboard) {
    if (!keyboard) {
        return;
    }
    if (keyboard->row + 1 >= UI_VK_ROW_COUNT) {
        return;
    }
    keyboard->row++;

    int row_len = (int)ui_vk_row_size(keyboard->row);
    if (row_len <= 0) {
        keyboard->col = 0;
        return;
    }

    if (keyboard->col > (uint8_t)(row_len - 4) && keyboard->col > 0) {
        keyboard->col--;
    }
    if (keyboard->col >= (uint8_t)row_len) {
        keyboard->col = (uint8_t)(row_len - 1);
    }
}

// ----------------------------------------------------------------------------
// Virtual Keyboard
// ----------------------------------------------------------------------------

void ui_virtual_keyboard_init(ui_virtual_keyboard_t* keyboard, char* buffer, size_t capacity) {
    if (!keyboard) {
        return;
    }

    memset(keyboard, 0, sizeof(*keyboard));
    keyboard->buffer = buffer;
    keyboard->capacity = capacity;
    keyboard->min_len = 1;
    keyboard->row = 0;
    keyboard->col = 0;
    keyboard->validator_message[0] = '\0';

    if (buffer && buffer[0] != '\0') {
        keyboard->row = 2;
        keyboard->col = 8;
    }
}

void ui_virtual_keyboard_set_min_length(ui_virtual_keyboard_t* keyboard, size_t min_len) {
    if (!keyboard) {
        return;
    }
    keyboard->min_len = min_len;
}

void ui_virtual_keyboard_set_validator(
    ui_virtual_keyboard_t* keyboard,
    ui_virtual_keyboard_validator_t validator,
    void* context) {
    if (!keyboard) {
        return;
    }
    keyboard->validator = validator;
    keyboard->validator_context = context;
}

bool ui_virtual_keyboard(ui_virtual_keyboard_t* keyboard, const char* header, uint32_t now_ms) {
    if (!keyboard) {
        return false;
    }

    if (keyboard->row >= UI_VK_ROW_COUNT) {
        keyboard->row = 0;
    }
    uint8_t row_size = ui_vk_row_size(keyboard->row);
    if (row_size == 0) {
        keyboard->col = 0;
    } else if (keyboard->col >= row_size) {
        keyboard->col = row_size - 1;
    }

    if (keyboard->validator_visible && now_ms >= keyboard->validator_deadline_ms) {
        keyboard->validator_visible = false;
    }

    bool submitted = false;

    if (g_ctx.has_input) {
        ui_key_t key = g_ctx.last_key;
        ui_input_type_t type = g_ctx.last_type;

        if (keyboard->validator_visible &&
            (type == ui_input_short || type == ui_input_long || type == ui_input_repeat)) {
            keyboard->validator_visible = false;
        } else if (type == ui_input_short) {
            switch (key) {
                case ui_key_up:
                    ui_vk_move_up(keyboard);
                    break;
                case ui_key_down:
                    ui_vk_move_down(keyboard);
                    break;
                case ui_key_left:
                    ui_vk_move_left(keyboard);
                    break;
                case ui_key_right:
                    ui_vk_move_right(keyboard);
                    break;
                case ui_key_ok:
                    submitted = ui_vk_handle_ok(keyboard, false, now_ms);
                    break;
                default:
                    break;
            }
        } else if (type == ui_input_long) {
            switch (key) {
                case ui_key_ok:
                    submitted = ui_vk_handle_ok(keyboard, true, now_ms);
                    break;
                case ui_key_back:
                    ui_vk_backspace(keyboard);
                    break;
                default:
                    break;
            }
        } else if (type == ui_input_repeat) {
            switch (key) {
                case ui_key_up:
                    ui_vk_move_up(keyboard);
                    break;
                case ui_key_down:
                    ui_vk_move_down(keyboard);
                    break;
                case ui_key_left:
                    ui_vk_move_left(keyboard);
                    break;
                case ui_key_right:
                    ui_vk_move_right(keyboard);
                    break;
                case ui_key_back:
                    ui_vk_backspace(keyboard);
                    break;
                default:
                    break;
            }
        }
    }

    const char* header_text = header ? header : "";
    const char* text = keyboard->buffer ? keyboard->buffer : "";
    size_t text_length = ui_vk_text_length(keyboard);

    canvas_set_color(ColorBlack);

    canvas_set_font(FontPrimary);
    canvas_draw_str(2, 8, header_text);

    canvas_draw_rframe(1, 12, 126, 15, 2);

    canvas_set_font(FontSecondary);

    uint32_t needed_width = UI_SCREEN_WIDTH - 8;
    int16_t start_pos = 4;

    if (canvas_string_width(text) > needed_width) {
        canvas_draw_str(start_pos, 22, "...");
        start_pos += 6;
        needed_width -= 8;
    }

    const char* visible_text = text;
    while (visible_text && canvas_string_width(visible_text) > needed_width) {
        visible_text++;
    }

    uint32_t visible_width = canvas_string_width(visible_text);

    if (keyboard->clear_default_text) {
        canvas_draw_rbox(start_pos - 1, 14, visible_width + 2, 10, 2);
        canvas_set_color(ColorWhite);
    } else {
        canvas_draw_str(start_pos + (int16_t)visible_width + 1, 22, "|");
    }

    canvas_draw_str(start_pos, 22, visible_text);

    canvas_set_font(FontKeyboard);

    for (uint8_t row = 0; row < UI_VK_ROW_COUNT; row++) {
        const ui_vk_key_t* keys = ui_vk_row(row);
        uint8_t column_count = ui_vk_row_size(row);

        for (uint8_t column = 0; column < column_count; column++) {
            const ui_vk_key_t* key = &keys[column];
            bool selected = (keyboard->row == row && keyboard->col == column);

            int16_t key_x = UI_VK_ORIGIN_X + key->x;
            int16_t key_y = UI_VK_ORIGIN_Y + key->y;

            if (key->text == UI_VK_ENTER_KEY) {
                canvas_set_color(ColorBlack);
                if (selected) {
                    canvas_draw_rbox(key_x, key_y, UI_VK_ENTER_W, UI_VK_ENTER_H, 2);
                    canvas_set_color(ColorWhite);
                } else {
                    canvas_draw_rframe(key_x, key_y, UI_VK_ENTER_W, UI_VK_ENTER_H, 2);
                }
                canvas_set_font(FontSecondary);
                const char* label = "OK";
                uint32_t label_width = canvas_string_width(label);
                int16_t label_x = key_x + ((int16_t)UI_VK_ENTER_W - (int16_t)label_width) / 2;
                canvas_draw_str(label_x, key_y + UI_VK_ENTER_H - 2, label);
                canvas_set_font(FontKeyboard);
                continue;
            }

            if (key->text == UI_VK_BACKSPACE_KEY) {
                canvas_set_color(ColorBlack);
                if (selected) {
                    canvas_draw_rbox(key_x, key_y, UI_VK_BACKSPACE_W, UI_VK_BACKSPACE_H, 2);
                    canvas_set_color(ColorWhite);
                } else {
                    canvas_draw_rframe(key_x, key_y, UI_VK_BACKSPACE_W, UI_VK_BACKSPACE_H, 2);
                }
                int16_t mid_y = key_y + (UI_VK_BACKSPACE_H / 2);
                int16_t left_x = key_x + 3;
                int16_t right_x = key_x + UI_VK_BACKSPACE_W - 4;
                canvas_draw_line(left_x, mid_y, right_x, mid_y);
                canvas_draw_line(left_x, mid_y, left_x + 3, mid_y - 3);
                canvas_draw_line(left_x, mid_y, left_x + 3, mid_y + 3);
                continue;
            }

            if (selected) {
                canvas_set_color(ColorBlack);
                canvas_draw_box(key_x - 1, key_y - 8, 7, 10);
                canvas_set_color(ColorWhite);
            } else {
                canvas_set_color(ColorBlack);
            }

            char glyph = key->text;
            if (keyboard->clear_default_text ||
                (text_length == 0 && ui_vk_is_lowercase(key->text))) {
                glyph = ui_vk_to_uppercase(glyph);
            }

            char str[2] = { glyph, '\0' };
            canvas_draw_str(key_x, key_y, str);
        }
    }

    if (keyboard->validator_visible) {
        canvas_set_font(FontSecondary);
        canvas_set_color(ColorWhite);
        canvas_draw_box(8, 10, 112, 44);
        canvas_set_color(ColorBlack);
        canvas_draw_rframe(8, 8, 112, 48, 3);
        canvas_draw_rframe(9, 9, 110, 46, 2);

        const char* message = keyboard->validator_message;
        uint32_t msg_width = canvas_string_width(message);
        int16_t msg_x = (UI_SCREEN_WIDTH - (int16_t)msg_width) / 2;
        canvas_draw_str(msg_x, 34, message);
    }

    return submitted;
}

// ----------------------------------------------------------------------------
// Context Management
// ----------------------------------------------------------------------------

void ui_begin(void) {
    // Clear canvas
    canvas_clear();

    // Save previous focus count for navigation bounds
    g_ctx.prev_focus_count = g_ctx.focus_count;

    // Reset widget counter
    g_ctx.focus_count = 0;

    // Reset layout to default full-screen vertical
    g_ctx.layout_depth = 0;
    g_ctx.layout_stack[0] = (ui_layout_stack_t){
        .x = 0,
        .y = 0,
        .width = UI_SCREEN_WIDTH,
        .height = UI_SCREEN_HEIGHT,
        .direction = ui_layout_vertical,
        .spacing = 0,
        .cursor = 0,
    };

    // Reset menu state
    g_ctx.menu.active = false;
    g_ctx.menu.scroll = NULL;

    // Reset position override
    g_ctx.use_absolute = false;

    // Reset deferred drawing
    g_ctx.deferred_count = 0;

    // Consume the ok_pressed flag (checked by widgets during this frame)
    // It was set by ui_input and will be cleared at end of frame
}

void ui_end(void) {
    // Clamp focus to valid range
    if (g_ctx.focus_count > 0) {
        if (g_ctx.focus_index < 0) {
            g_ctx.focus_index = 0;
        } else if (g_ctx.focus_index >= g_ctx.focus_count) {
            g_ctx.focus_index = g_ctx.focus_count - 1;
        }
    } else {
        g_ctx.focus_index = -1;
    }

    // Clear input state for next frame
    g_ctx.has_input = false;
    g_ctx.ok_pressed = false;
    g_ctx.back_pressed = false;
}

void ui_input(ui_key_t key, ui_input_type_t type) {
    if (type != ui_input_release) {
        g_ctx.last_key = key;
        g_ctx.last_type = type;
        g_ctx.has_input = true;
    }

    // Handle navigation on short press or repeat
    if (type == ui_input_short || type == ui_input_repeat) {
        switch (key) {
            case ui_key_up:
                if (g_ctx.prev_focus_count > 0) {
                    g_ctx.focus_index--;
                    if (g_ctx.focus_index < 0) {
                        g_ctx.focus_index = g_ctx.prev_focus_count - 1;
                    }
                }
                break;

            case ui_key_down:
                if (g_ctx.prev_focus_count > 0) {
                    g_ctx.focus_index++;
                    if (g_ctx.focus_index >= g_ctx.prev_focus_count) {
                        g_ctx.focus_index = 0;
                    }
                }
                break;

            case ui_key_ok:
                g_ctx.ok_pressed = true;
                break;

            case ui_key_back:
                g_ctx.back_pressed = true;
                break;

            default:
                break;
        }
    }
}

bool ui_back_pressed(void) {
    return g_ctx.back_pressed;
}

// ----------------------------------------------------------------------------
// Layout System
// ----------------------------------------------------------------------------

void ui_vstack(int16_t spacing) {
    if (g_ctx.layout_depth >= UI_MAX_LAYOUT_DEPTH - 1) return;

    ui_layout_stack_t* parent = ui_current_layout();
    int16_t new_x = parent ? parent->x : 0;
    int16_t new_y = parent ? (parent->y + parent->cursor) : 0;
    int16_t new_width = parent ? parent->width : UI_SCREEN_WIDTH;

    g_ctx.layout_depth++;
    g_ctx.layout_stack[g_ctx.layout_depth] = (ui_layout_stack_t){
        .x = new_x,
        .y = new_y,
        .width = new_width,
        .height = UI_SCREEN_HEIGHT - new_y,
        .direction = ui_layout_vertical,
        .spacing = spacing,
        .cursor = 0,
    };
}

void ui_hstack(int16_t spacing) {
    if (g_ctx.layout_depth >= UI_MAX_LAYOUT_DEPTH - 1) return;

    ui_layout_stack_t* parent = ui_current_layout();
    int16_t new_x = parent ? parent->x : 0;
    int16_t new_y = parent ? (parent->y + parent->cursor) : 0;
    int16_t new_width = parent ? parent->width : UI_SCREEN_WIDTH;

    g_ctx.layout_depth++;
    g_ctx.layout_stack[g_ctx.layout_depth] = (ui_layout_stack_t){
        .x = new_x,
        .y = new_y,
        .width = new_width,
        .height = UI_SCREEN_HEIGHT - new_y,
        .direction = ui_layout_horizontal,
        .spacing = spacing,
        .cursor = 0,
        .centered = false,
    };
}

void ui_hstack_centered(int16_t spacing) {
    if (g_ctx.layout_depth >= UI_MAX_LAYOUT_DEPTH - 1) return;

    ui_layout_stack_t* parent = ui_current_layout();
    int16_t new_x = parent ? parent->x : 0;
    int16_t new_y = parent ? (parent->y + parent->cursor) : 0;
    int16_t new_width = parent ? parent->width : UI_SCREEN_WIDTH;

    // Reset deferred buffer for this centered stack
    g_ctx.deferred_count = 0;

    g_ctx.layout_depth++;
    g_ctx.layout_stack[g_ctx.layout_depth] = (ui_layout_stack_t){
        .x = new_x,
        .y = new_y,
        .width = new_width,
        .height = UI_SCREEN_HEIGHT - new_y,
        .direction = ui_layout_horizontal,
        .spacing = spacing,
        .cursor = 0,
        .centered = true,
    };
}

void ui_end_stack(void) {
    if (g_ctx.layout_depth <= 0) return;

    ui_layout_stack_t* ending = &g_ctx.layout_stack[g_ctx.layout_depth];

    // Handle centered hstack: draw deferred buttons with centering offset
    if (ending->centered && ending->direction == ui_layout_horizontal) {
        int16_t content_width = ending->cursor;
        if (ending->spacing > 0 && content_width > 0) {
            content_width -= ending->spacing;  // Remove trailing spacing
        }

        // Calculate centering offset
        int16_t offset = (ending->width - content_width) / 2;

        // Draw all deferred buttons with offset
        for (int8_t i = 0; i < g_ctx.deferred_count; i++) {
            ui_deferred_button_t* btn = &g_ctx.deferred_buttons[i];
            ui_draw_button_internal(
                btn->x + offset,
                btn->y,
                btn->width,
                btn->height,
                btn->text,
                btn->focused
            );
        }

        // Clear deferred buffer
        g_ctx.deferred_count = 0;
    }

    // Get the size used by this stack
    int16_t used_size = ending->cursor;
    if (ending->spacing > 0 && used_size > 0) {
        used_size -= ending->spacing;  // Remove trailing spacing
    }

    // For hstacks, track the max height (for now, assume button height)
    int16_t used_height = (ending->direction == ui_layout_horizontal)
        ? (UI_FONT_HEIGHT_SECONDARY + UI_BUTTON_PADDING_Y * 2)
        : used_size;

    g_ctx.layout_depth--;

    // Advance parent cursor by the space this stack used
    ui_layout_stack_t* parent = ui_current_layout();
    if (parent && parent->direction == ui_layout_vertical) {
        parent->cursor += used_height + parent->spacing;
    }
}

void ui_spacer(int16_t pixels) {
    ui_layout_stack_t* layout = ui_current_layout();
    if (layout) {
        layout->cursor += pixels;
    }
}

void ui_set_position(int16_t x, int16_t y) {
    g_ctx.use_absolute = true;
    g_ctx.abs_x = x;
    g_ctx.abs_y = y;
}

// ----------------------------------------------------------------------------
// Basic Widgets
// ----------------------------------------------------------------------------

void ui_label(const char* text, Font font, Align align) {
    int16_t font_height = ui_get_font_height(font);
    int16_t x, y, w;
    ui_layout_next(0, font_height, &x, &y, &w);

    // Set font
    canvas_set_font(font);
    canvas_set_color(ColorBlack);

    // Calculate text position based on alignment
    int16_t text_x;
    uint32_t text_width = canvas_string_width(text);

    switch (align) {
        case AlignCenter:
            text_x = x + (w - (int16_t)text_width) / 2;
            break;
        case AlignRight:
            text_x = x + w - (int16_t)text_width;
            break;
        case AlignLeft:
        default:
            text_x = x;
            break;
    }

    // Draw text (y is baseline, so add font height)
    canvas_draw_str(text_x, y + font_height, text);
}

void ui_separator(void) {
    int16_t x, y, w;
    ui_layout_next(0, 5, &x, &y, &w);

    canvas_set_color(ColorBlack);
    canvas_draw_line(x, y + 2, x + w - 1, y + 2);
}

bool ui_button(const char* text) {
    canvas_set_font(FontSecondary);
    uint32_t text_width = canvas_string_width(text);

    int16_t btn_width = (int16_t)text_width + UI_BUTTON_PADDING_X * 2;
    int16_t btn_height = UI_FONT_HEIGHT_SECONDARY + UI_BUTTON_PADDING_Y * 2;

    int16_t x, y, w;
    ui_layout_next(btn_width, btn_height, &x, &y, &w);

    // Register for focus
    int16_t idx = ui_register_focusable();
    bool focused = ui_check_focused(idx);
    bool activated = ui_check_activated(idx);

    // Check if we're in a centered hstack - defer drawing
    if (ui_in_centered_hstack()) {
        if (g_ctx.deferred_count < UI_MAX_DEFERRED) {
            g_ctx.deferred_buttons[g_ctx.deferred_count++] = (ui_deferred_button_t){
                .x = x,
                .y = y,
                .width = btn_width,
                .height = btn_height,
                .text = text,
                .focused = focused,
            };
        }
    } else {
        // Center button in layout width for vertical stacks
        int16_t btn_x = x + (w - btn_width) / 2;
        ui_draw_button_internal(btn_x, y, btn_width, btn_height, text, focused);
    }

    return activated;
}

bool ui_button_at(int16_t x, int16_t y, const char* text) {
    ui_set_position(x, y);
    return ui_button(text);
}

void ui_progress(float value, int16_t width) {
    if (value < 0.0f) value = 0.0f;
    if (value > 1.0f) value = 1.0f;

    int16_t bar_height = 8;
    int16_t x, y, w;
    ui_layout_next(width, bar_height, &x, &y, &w);

    // Use layout width minus margins if width=0
    int16_t bar_width = (width > 0) ? width : (w - 8);  // 4px margin each side
    int16_t bar_x = x + (w - bar_width) / 2;

    // Draw outline
    canvas_set_color(ColorBlack);
    canvas_draw_frame(bar_x, y, bar_width, bar_height);

    // Draw filled portion
    int16_t fill_width = (int16_t)(value * (float)(bar_width - 2));
    if (fill_width > 0) {
        canvas_draw_box(bar_x + 1, y + 1, fill_width, bar_height - 2);
    }
}

void ui_icon(const uint8_t* data, uint8_t width, uint8_t height) {
    int16_t x, y, w;
    ui_layout_next(width, height, &x, &y, &w);

    // Center icon in layout width
    int16_t icon_x = x + (w - width) / 2;

    // Draw XBM icon pixel by pixel
    canvas_set_color(ColorBlack);
    for (uint8_t iy = 0; iy < height; iy++) {
        for (uint8_t ix = 0; ix < width; ix++) {
            uint16_t byte_idx = (iy * ((width + 7) / 8)) + (ix / 8);
            uint8_t bit_idx = ix % 8;
            if (data[byte_idx] & (1 << bit_idx)) {
                canvas_draw_dot(icon_x + ix, y + iy);
            }
        }
    }
}

bool ui_checkbox(const char* text, bool* checked) {
    canvas_set_font(FontSecondary);

    int16_t box_size = 10;
    int16_t item_height = UI_FONT_HEIGHT_SECONDARY + 2;
    if (item_height < box_size) item_height = box_size;

    int16_t x, y, w;
    ui_layout_next(0, item_height, &x, &y, &w);

    // Register for focus
    int16_t idx = ui_register_focusable();
    bool focused = ui_check_focused(idx);
    bool activated = ui_check_activated(idx);

    // Toggle on activation
    if (activated && checked) {
        *checked = !*checked;
    }

    // Draw checkbox
    int16_t box_x = x + 2;
    int16_t box_y = y + (item_height - box_size) / 2;

    canvas_set_color(ColorBlack);

    if (focused) {
        // Focused: filled background
        canvas_draw_box(x, y, w, item_height);
        canvas_set_color(ColorWhite);
    }

    // Draw checkbox outline
    canvas_draw_frame(box_x, box_y, box_size, box_size);

    // Draw checkmark if checked
    if (checked && *checked) {
        canvas_draw_line(box_x + 2, box_y + 5, box_x + 4, box_y + 7);
        canvas_draw_line(box_x + 4, box_y + 7, box_x + 7, box_y + 2);
    }

    // Draw label
    canvas_draw_str(box_x + box_size + 4, y + item_height - 2, text);

    return activated;
}

// ----------------------------------------------------------------------------
// Menu System
// ----------------------------------------------------------------------------

void ui_menu_begin(int16_t* scroll, int16_t visible, int16_t total) {
    g_ctx.menu.active = true;
    g_ctx.menu.scroll = scroll;
    g_ctx.menu.visible = visible;
    g_ctx.menu.total = total;
    g_ctx.menu.first_item_focus = g_ctx.focus_count;

    // Store y position where menu starts
    ui_layout_stack_t* layout = ui_current_layout();
    g_ctx.menu.y_start = layout ? (layout->y + layout->cursor) : 0;
}

bool ui_menu_item(const char* text, int16_t index) {
    if (!g_ctx.menu.active || !g_ctx.menu.scroll) return false;

    // Register for focus first (all items, visible or not)
    int16_t idx = ui_register_focusable();
    bool focused = ui_check_focused(idx);
    bool activated = ui_check_activated(idx);

    // Update scroll to keep focused item visible BEFORE visibility check
    if (focused) {
        int16_t scroll = *g_ctx.menu.scroll;
        if (index < scroll) {
            *g_ctx.menu.scroll = index;
        } else if (index >= scroll + g_ctx.menu.visible) {
            *g_ctx.menu.scroll = index - g_ctx.menu.visible + 1;
        }
    }

    // Now read scroll (may have been updated)
    int16_t scroll = *g_ctx.menu.scroll;

    // Check if item is visible
    if (index < scroll || index >= scroll + g_ctx.menu.visible) {
        return false;
    }

    canvas_set_font(FontSecondary);

    // Calculate position within visible area
    int16_t visible_index = index - scroll;
    int16_t y = g_ctx.menu.y_start + visible_index * UI_MENU_ITEM_HEIGHT;

    // Calculate width (leave room for scrollbar)
    int16_t item_width = UI_SCREEN_WIDTH - UI_SCROLLBAR_WIDTH - 2;

    // Draw item
    canvas_set_color(ColorBlack);

    if (focused) {
        // Focused: filled background with inverted text
        canvas_draw_box(0, y, item_width, UI_MENU_ITEM_HEIGHT);
        canvas_set_color(ColorWhite);
    }

    canvas_draw_str(2, y + UI_MENU_ITEM_HEIGHT - 2, text);

    return activated;
}

bool ui_menu_item_value(const char* label, const char* value, int16_t index) {
    if (!g_ctx.menu.active || !g_ctx.menu.scroll) return false;

    // Register for focus first (all items, visible or not)
    int16_t idx = ui_register_focusable();
    bool focused = ui_check_focused(idx);
    bool activated = ui_check_activated(idx);

    // Update scroll to keep focused item visible BEFORE visibility check
    if (focused) {
        int16_t scroll = *g_ctx.menu.scroll;
        if (index < scroll) {
            *g_ctx.menu.scroll = index;
        } else if (index >= scroll + g_ctx.menu.visible) {
            *g_ctx.menu.scroll = index - g_ctx.menu.visible + 1;
        }
    }

    // Now read scroll (may have been updated)
    int16_t scroll = *g_ctx.menu.scroll;

    // Check if item is visible
    if (index < scroll || index >= scroll + g_ctx.menu.visible) {
        return false;
    }

    canvas_set_font(FontSecondary);

    // Calculate position within visible area
    int16_t visible_index = index - scroll;
    int16_t y = g_ctx.menu.y_start + visible_index * UI_MENU_ITEM_HEIGHT;

    // Calculate width (leave room for scrollbar)
    int16_t item_width = UI_SCREEN_WIDTH - UI_SCROLLBAR_WIDTH - 2;

    // Draw item
    canvas_set_color(ColorBlack);

    if (focused) {
        // Focused: filled background with inverted text
        canvas_draw_box(0, y, item_width, UI_MENU_ITEM_HEIGHT);
        canvas_set_color(ColorWhite);
    }

    // Draw label on left
    canvas_draw_str(2, y + UI_MENU_ITEM_HEIGHT - 2, label);

    // Draw value on right
    uint32_t value_width = canvas_string_width(value);
    canvas_draw_str(item_width - (int16_t)value_width - 2, y + UI_MENU_ITEM_HEIGHT - 2, value);

    return activated;
}

void ui_menu_end(void) {
    if (!g_ctx.menu.active) return;

    // Draw scrollbar if needed (Flipper Zero style: dotted track + solid thumb)
    if (g_ctx.menu.total > g_ctx.menu.visible) {
        int16_t scroll = g_ctx.menu.scroll ? *g_ctx.menu.scroll : 0;
        int16_t scrollbar_height = g_ctx.menu.visible * UI_MENU_ITEM_HEIGHT;
        int16_t scrollbar_x = UI_SCREEN_WIDTH - 2;  // Single pixel column near edge

        // Calculate thumb position and size
        int16_t thumb_height = (scrollbar_height * g_ctx.menu.visible) / g_ctx.menu.total;
        if (thumb_height < 4) thumb_height = 4;

        int16_t thumb_y = g_ctx.menu.y_start +
            ((scrollbar_height - thumb_height) * scroll) / (g_ctx.menu.total - g_ctx.menu.visible);

        canvas_set_color(ColorBlack);

        // Draw dotted track line
        for (int16_t y = g_ctx.menu.y_start; y < g_ctx.menu.y_start + scrollbar_height; y += 2) {
            canvas_draw_dot(scrollbar_x, y);
        }

        // Draw solid thumb (thicker, 3px wide)
        canvas_draw_box(scrollbar_x - 1, thumb_y, 3, thumb_height);
    }

    // Advance layout cursor
    ui_layout_stack_t* layout = ui_current_layout();
    if (layout) {
        int16_t visible_count = g_ctx.menu.visible;
        if (g_ctx.menu.total < visible_count) {
            visible_count = g_ctx.menu.total;
        }
        layout->cursor += visible_count * UI_MENU_ITEM_HEIGHT + layout->spacing;
    }

    g_ctx.menu.active = false;
}

// ----------------------------------------------------------------------------
// Footer Buttons
// ----------------------------------------------------------------------------

bool ui_footer_left(const char* text) {
    canvas_set_font(FontSecondary);

    int16_t y = UI_SCREEN_HEIGHT - UI_FOOTER_HEIGHT;

    canvas_set_color(ColorBlack);

    // Draw left arrow
    canvas_draw_line(2, y + 5, 6, y + 2);
    canvas_draw_line(2, y + 5, 6, y + 8);

    // Draw text
    canvas_draw_str(9, y + UI_FOOTER_HEIGHT - 2, text);

    // Check for left key press
    return g_ctx.has_input &&
           g_ctx.last_key == ui_key_left &&
           (g_ctx.last_type == ui_input_short || g_ctx.last_type == ui_input_press);
}

bool ui_footer_center(const char* text) {
    canvas_set_font(FontSecondary);

    int16_t y = UI_SCREEN_HEIGHT - UI_FOOTER_HEIGHT;

    // Calculate center position
    uint32_t text_width = canvas_string_width(text);
    int16_t total_width = (int16_t)text_width + 12;  // text + OK symbol + spacing
    int16_t x = (UI_SCREEN_WIDTH - total_width) / 2;

    canvas_set_color(ColorBlack);

    // Draw OK symbol (small filled circle)
    canvas_draw_disc(x + 4, y + 5, 3);

    // Draw text
    canvas_draw_str(x + 12, y + UI_FOOTER_HEIGHT - 2, text);

    // Check for OK key press (only if not consumed by focused widget)
    // Footer center typically doesn't consume OK if there are focusable widgets
    return false;  // Usually handled by focused widget instead
}

bool ui_footer_right(const char* text) {
    canvas_set_font(FontSecondary);

    int16_t y = UI_SCREEN_HEIGHT - UI_FOOTER_HEIGHT;

    // Calculate position from right
    uint32_t text_width = canvas_string_width(text);
    int16_t x = UI_SCREEN_WIDTH - (int16_t)text_width - 10;

    canvas_set_color(ColorBlack);

    // Draw text
    canvas_draw_str(x, y + UI_FOOTER_HEIGHT - 2, text);

    // Draw right arrow
    int16_t arrow_x = UI_SCREEN_WIDTH - 7;
    canvas_draw_line(arrow_x + 4, y + 5, arrow_x, y + 2);
    canvas_draw_line(arrow_x + 4, y + 5, arrow_x, y + 8);

    // Check for right key press
    return g_ctx.has_input &&
           g_ctx.last_key == ui_key_right &&
           (g_ctx.last_type == ui_input_short || g_ctx.last_type == ui_input_press);
}

// ----------------------------------------------------------------------------
// Focus Management
// ----------------------------------------------------------------------------

int16_t ui_get_focus(void) {
    return g_ctx.focus_index;
}

void ui_set_focus(int16_t index) {
    g_ctx.focus_index = index;
}

bool ui_is_focused(int16_t index) {
    return g_ctx.focus_index == index;
}
