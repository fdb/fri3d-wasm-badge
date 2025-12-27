// ============================================================================
// IMGUI Implementation for ESP32 128x64 Monochrome Display
// ============================================================================

#include "imgui.h"
#include "canvas.h"
#include "input.h"

// ----------------------------------------------------------------------------
// Internal Constants
// ----------------------------------------------------------------------------

#define UI_MAX_LAYOUT_DEPTH 8
#define UI_MAX_FOCUSABLE 32
#define UI_FONT_HEIGHT_PRIMARY 12
#define UI_FONT_HEIGHT_SECONDARY 10
#define UI_BUTTON_PADDING_X 4
#define UI_BUTTON_PADDING_Y 2
#define UI_MENU_ITEM_HEIGHT 12
#define UI_FOOTER_HEIGHT 12
#define UI_SCROLLBAR_WIDTH 3

// ----------------------------------------------------------------------------
// Internal Types
// ----------------------------------------------------------------------------

typedef struct {
    int16_t x;
    int16_t y;
    int16_t width;
    int16_t height;
    UiLayoutDirection direction;
    int16_t spacing;
    int16_t cursor;  // Current position in layout direction
} UiLayoutStack;

typedef struct {
    int16_t x;
    int16_t y;
    int16_t width;
    int16_t height;
} UiWidgetRect;

typedef struct {
    // Layout stack
    UiLayoutStack layout_stack[UI_MAX_LAYOUT_DEPTH];
    int8_t layout_depth;

    // Focus tracking
    int16_t focus_index;      // Currently focused widget (-1 = none)
    int16_t focus_count;      // Focusable widgets this frame
    int16_t prev_focus_count; // Focus count from previous frame

    // Input state for current frame
    UiKey last_key;
    UiInputType last_type;
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

} UiContext;

// ----------------------------------------------------------------------------
// Global State
// ----------------------------------------------------------------------------

static UiContext g_ctx = {0};

// ----------------------------------------------------------------------------
// Internal Helpers
// ----------------------------------------------------------------------------

static int16_t ui_get_font_height(UiFont font) {
    switch (font) {
        case UI_FONT_PRIMARY:
            return UI_FONT_HEIGHT_PRIMARY;
        case UI_FONT_SECONDARY:
        default:
            return UI_FONT_HEIGHT_SECONDARY;
    }
}

static UiLayoutStack* ui_current_layout(void) {
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

    UiLayoutStack* layout = ui_current_layout();
    if (!layout) {
        *out_x = 0;
        *out_y = 0;
        *out_w = UI_SCREEN_WIDTH;
        return;
    }

    if (layout->direction == UI_LAYOUT_VERTICAL) {
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
    g_ctx.layout_stack[0] = (UiLayoutStack){
        .x = 0,
        .y = 0,
        .width = UI_SCREEN_WIDTH,
        .height = UI_SCREEN_HEIGHT,
        .direction = UI_LAYOUT_VERTICAL,
        .spacing = 0,
        .cursor = 0,
    };

    // Reset menu state
    g_ctx.menu.active = false;
    g_ctx.menu.scroll = NULL;

    // Reset position override
    g_ctx.use_absolute = false;

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

void ui_input(UiKey key, UiInputType type) {
    g_ctx.last_key = key;
    g_ctx.last_type = type;
    g_ctx.has_input = true;

    // Handle navigation on short press or repeat
    if (type == UI_INPUT_SHORT || type == UI_INPUT_REPEAT || type == UI_INPUT_PRESS) {
        switch (key) {
            case UI_KEY_UP:
                if (g_ctx.prev_focus_count > 0) {
                    g_ctx.focus_index--;
                    if (g_ctx.focus_index < 0) {
                        g_ctx.focus_index = g_ctx.prev_focus_count - 1;
                    }
                }
                break;

            case UI_KEY_DOWN:
                if (g_ctx.prev_focus_count > 0) {
                    g_ctx.focus_index++;
                    if (g_ctx.focus_index >= g_ctx.prev_focus_count) {
                        g_ctx.focus_index = 0;
                    }
                }
                break;

            case UI_KEY_OK:
                g_ctx.ok_pressed = true;
                break;

            case UI_KEY_BACK:
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

    UiLayoutStack* parent = ui_current_layout();
    int16_t new_x = parent ? parent->x : 0;
    int16_t new_y = parent ? (parent->y + parent->cursor) : 0;
    int16_t new_width = parent ? parent->width : UI_SCREEN_WIDTH;

    g_ctx.layout_depth++;
    g_ctx.layout_stack[g_ctx.layout_depth] = (UiLayoutStack){
        .x = new_x,
        .y = new_y,
        .width = new_width,
        .height = UI_SCREEN_HEIGHT - new_y,
        .direction = UI_LAYOUT_VERTICAL,
        .spacing = spacing,
        .cursor = 0,
    };
}

void ui_hstack(int16_t spacing) {
    if (g_ctx.layout_depth >= UI_MAX_LAYOUT_DEPTH - 1) return;

    UiLayoutStack* parent = ui_current_layout();
    int16_t new_x = parent ? parent->x : 0;
    int16_t new_y = parent ? (parent->y + parent->cursor) : 0;
    int16_t new_width = parent ? parent->width : UI_SCREEN_WIDTH;

    g_ctx.layout_depth++;
    g_ctx.layout_stack[g_ctx.layout_depth] = (UiLayoutStack){
        .x = new_x,
        .y = new_y,
        .width = new_width,
        .height = UI_SCREEN_HEIGHT - new_y,
        .direction = UI_LAYOUT_HORIZONTAL,
        .spacing = spacing,
        .cursor = 0,
    };
}

void ui_end_stack(void) {
    if (g_ctx.layout_depth <= 0) return;

    // Get the height used by this stack
    UiLayoutStack* ending = &g_ctx.layout_stack[g_ctx.layout_depth];
    int16_t used_height = ending->cursor;
    if (ending->spacing > 0 && used_height > 0) {
        used_height -= ending->spacing;  // Remove trailing spacing
    }

    g_ctx.layout_depth--;

    // Advance parent cursor by the height used
    UiLayoutStack* parent = ui_current_layout();
    if (parent) {
        if (parent->direction == UI_LAYOUT_VERTICAL) {
            parent->cursor += used_height + parent->spacing;
        }
        // For horizontal parent, we'd need width tracking
    }
}

void ui_spacer(int16_t pixels) {
    UiLayoutStack* layout = ui_current_layout();
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

void ui_label(const char* text, UiFont font, UiAlign align) {
    int16_t font_height = ui_get_font_height(font);
    int16_t x, y, w;
    ui_layout_next(0, font_height, &x, &y, &w);

    // Set font
    canvas_set_font((Font)font);
    canvas_set_color(ColorBlack);

    // Calculate text position based on alignment
    int16_t text_x;
    uint32_t text_width = canvas_string_width(text);

    switch (align) {
        case UI_ALIGN_CENTER:
            text_x = x + (w - (int16_t)text_width) / 2;
            break;
        case UI_ALIGN_RIGHT:
            text_x = x + w - (int16_t)text_width;
            break;
        case UI_ALIGN_LEFT:
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

    // Center button in layout width
    int16_t btn_x = x + (w - btn_width) / 2;

    // Register for focus
    int16_t idx = ui_register_focusable();
    bool focused = ui_check_focused(idx);
    bool activated = ui_check_activated(idx);

    // Draw button
    if (focused) {
        // Focused: filled box with inverted text
        canvas_set_color(ColorBlack);
        canvas_draw_rbox(btn_x, y, btn_width, btn_height, 2);
        canvas_set_color(ColorWhite);
        canvas_draw_str(btn_x + UI_BUTTON_PADDING_X, y + btn_height - UI_BUTTON_PADDING_Y, text);
    } else {
        // Normal: outline with black text
        canvas_set_color(ColorBlack);
        canvas_draw_rframe(btn_x, y, btn_width, btn_height, 2);
        canvas_draw_str(btn_x + UI_BUTTON_PADDING_X, y + btn_height - UI_BUTTON_PADDING_Y, text);
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

    int16_t bar_width = (width > 0) ? width : w;
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
    UiLayoutStack* layout = ui_current_layout();
    g_ctx.menu.y_start = layout ? (layout->y + layout->cursor) : 0;
}

bool ui_menu_item(const char* text, int16_t index) {
    if (!g_ctx.menu.active || !g_ctx.menu.scroll) return false;

    int16_t scroll = *g_ctx.menu.scroll;

    // Check if item is visible
    if (index < scroll || index >= scroll + g_ctx.menu.visible) {
        // Still register for focus even if not visible
        ui_register_focusable();
        return false;
    }

    canvas_set_font(FontSecondary);

    // Calculate position within visible area
    int16_t visible_index = index - scroll;
    int16_t y = g_ctx.menu.y_start + visible_index * UI_MENU_ITEM_HEIGHT;

    // Register for focus
    int16_t idx = ui_register_focusable();
    bool focused = ui_check_focused(idx);
    bool activated = ui_check_activated(idx);

    // Update scroll to keep focused item visible
    if (focused) {
        if (index < scroll) {
            *g_ctx.menu.scroll = index;
        } else if (index >= scroll + g_ctx.menu.visible) {
            *g_ctx.menu.scroll = index - g_ctx.menu.visible + 1;
        }
    }

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

    int16_t scroll = *g_ctx.menu.scroll;

    // Check if item is visible
    if (index < scroll || index >= scroll + g_ctx.menu.visible) {
        // Still register for focus even if not visible
        ui_register_focusable();
        return false;
    }

    canvas_set_font(FontSecondary);

    // Calculate position within visible area
    int16_t visible_index = index - scroll;
    int16_t y = g_ctx.menu.y_start + visible_index * UI_MENU_ITEM_HEIGHT;

    // Register for focus
    int16_t idx = ui_register_focusable();
    bool focused = ui_check_focused(idx);
    bool activated = ui_check_activated(idx);

    // Update scroll to keep focused item visible
    if (focused) {
        if (index < scroll) {
            *g_ctx.menu.scroll = index;
        } else if (index >= scroll + g_ctx.menu.visible) {
            *g_ctx.menu.scroll = index - g_ctx.menu.visible + 1;
        }
    }

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

    // Draw scrollbar if needed
    if (g_ctx.menu.total > g_ctx.menu.visible) {
        int16_t scroll = g_ctx.menu.scroll ? *g_ctx.menu.scroll : 0;
        int16_t scrollbar_height = g_ctx.menu.visible * UI_MENU_ITEM_HEIGHT;
        int16_t scrollbar_x = UI_SCREEN_WIDTH - UI_SCROLLBAR_WIDTH;

        // Calculate thumb position and size
        int16_t thumb_height = (scrollbar_height * g_ctx.menu.visible) / g_ctx.menu.total;
        if (thumb_height < 4) thumb_height = 4;

        int16_t thumb_y = g_ctx.menu.y_start +
            ((scrollbar_height - thumb_height) * scroll) / (g_ctx.menu.total - g_ctx.menu.visible);

        // Draw scrollbar track
        canvas_set_color(ColorBlack);
        canvas_draw_frame(scrollbar_x, g_ctx.menu.y_start, UI_SCROLLBAR_WIDTH, scrollbar_height);

        // Draw scrollbar thumb
        canvas_draw_box(scrollbar_x, thumb_y, UI_SCROLLBAR_WIDTH, thumb_height);
    }

    // Advance layout cursor
    UiLayoutStack* layout = ui_current_layout();
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
           g_ctx.last_key == UI_KEY_LEFT &&
           (g_ctx.last_type == UI_INPUT_SHORT || g_ctx.last_type == UI_INPUT_PRESS);
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
           g_ctx.last_key == UI_KEY_RIGHT &&
           (g_ctx.last_type == UI_INPUT_SHORT || g_ctx.last_type == UI_INPUT_PRESS);
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
