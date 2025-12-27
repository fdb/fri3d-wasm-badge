# IMGUI View System

An Immediate Mode GUI library for ESP32 devices with 128x64 monochrome displays. Inspired by Flipper Zero's view system, redesigned for simplicity using the IMGUI paradigm.

## Table of Contents

- [Design Philosophy](#design-philosophy)
- [Quick Start](#quick-start)
- [Frame Lifecycle](#frame-lifecycle)
- [Input Handling](#input-handling)
- [Layout System](#layout-system)
- [Widgets](#widgets)
- [Focus Navigation](#focus-navigation)
- [Menu System](#menu-system)
- [Footer Buttons](#footer-buttons)
- [Complete Example](#complete-example)
- [API Reference](#api-reference)
- [Implementation Notes](#implementation-notes)

---

## Design Philosophy

### Retained Mode vs Immediate Mode

**Flipper Zero uses Retained Mode:**
```c
// Allocate persistent widget
Submenu* menu = submenu_alloc();

// Configure with callbacks
submenu_add_item(menu, "Option A", 0, callback, context);
submenu_add_item(menu, "Option B", 1, callback, context);

// Register with view dispatcher
view_dispatcher_add_view(dispatcher, ViewMenu, submenu_get_view(menu));

// Later: free everything
submenu_free(menu);
```

**This library uses Immediate Mode:**
```c
void render(void) {
    ui_begin();

    if (ui_menu_item("Option A", 0)) {
        handle_option_a();
    }
    if (ui_menu_item("Option B", 1)) {
        handle_option_b();
    }

    ui_end();
}
```

### Key Principles

1. **No Widget Allocation**: Widgets exist only during the render call
2. **Immediate Returns**: Interactive widgets return `true` when activated
3. **State Lives in App**: The app owns all state, UI just reflects it
4. **Minimal Persistence**: Only focus position persists between frames
5. **WASM-Friendly**: No callbacks, no function pointers crossing the boundary

---

## Quick Start

A minimal app using IMGUI:

```c
#include <frd.h>
#include <imgui.h>

static int counter = 0;

void render(void) {
    ui_begin();

    char buf[16];
    snprintf(buf, sizeof(buf), "Count: %d", counter);
    ui_label(buf, UI_FONT_PRIMARY, UI_ALIGN_CENTER);

    ui_spacer(8);

    if (ui_button("Increment")) {
        counter++;
    }

    if (ui_button("Reset")) {
        counter = 0;
    }

    ui_end();
}

void on_input(InputKey key, InputType type) {
    ui_input(key, type);
}

int main(void) {
    return 0;
}
```

---

## Frame Lifecycle

Every frame follows this pattern:

```c
void render(void) {
    // 1. Begin frame - clears canvas, resets widget state
    ui_begin();

    // 2. Call widget functions - they draw and return interaction state
    ui_label("Title", UI_FONT_PRIMARY, UI_ALIGN_CENTER);

    if (ui_button("OK")) {
        // Button was pressed this frame
    }

    // 3. End frame - commits canvas to display
    ui_end();
}
```

### What Happens Each Frame

| Phase | Action |
|-------|--------|
| `ui_begin()` | Clear canvas, reset focus counter, clear input flags |
| Widget calls | Draw to canvas, register for focus, check activation |
| `ui_end()` | Commit canvas buffer to display hardware |

---

## Input Handling

Input events are fed to the UI system, which handles focus navigation automatically:

```c
void on_input(InputKey key, InputType type) {
    // Feed input to UI system
    ui_input(key, type);

    // Check for back button (not consumed by widgets)
    if (ui_back_pressed()) {
        exit_app();
    }

    // Custom input handling
    if (key == InputKeyLeft && type == InputTypeShort) {
        // Handle left press
    }
}
```

### Input Types

| Type | Description |
|------|-------------|
| `UI_INPUT_PRESS` | Physical button press |
| `UI_INPUT_RELEASE` | Physical button release |
| `UI_INPUT_SHORT` | Short press completed (< 500ms) |
| `UI_INPUT_LONG` | Long press threshold reached |
| `UI_INPUT_REPEAT` | Repeat while held |

### Input Keys

| Key | Description |
|-----|-------------|
| `UI_KEY_UP` | D-pad up |
| `UI_KEY_DOWN` | D-pad down |
| `UI_KEY_LEFT` | D-pad left |
| `UI_KEY_RIGHT` | D-pad right |
| `UI_KEY_OK` | Center/confirm button |
| `UI_KEY_BACK` | Back/cancel button |

### Built-in Navigation

The UI system automatically handles:
- **Up/Down**: Move focus between widgets
- **OK (short)**: Activate focused widget
- **Back**: Sets `ui_back_pressed()` flag

---

## Layout System

Widgets are positioned using a stack-based layout system. Without explicit layout, widgets stack vertically from top-left.

### Vertical Stack

```c
ui_begin();

// Default: vertical stack from (0, 0)
ui_label("First", UI_FONT_PRIMARY, UI_ALIGN_LEFT);   // y = 0
ui_label("Second", UI_FONT_SECONDARY, UI_ALIGN_LEFT); // y = 12
ui_label("Third", UI_FONT_SECONDARY, UI_ALIGN_LEFT);  // y = 24

ui_end();
```

### Explicit Stacks

```c
ui_begin();

// Create a vertical stack with custom spacing
ui_vstack(4);  // 4px between items
    ui_label("Item 1", UI_FONT_SECONDARY, UI_ALIGN_LEFT);
    ui_label("Item 2", UI_FONT_SECONDARY, UI_ALIGN_LEFT);
    ui_label("Item 3", UI_FONT_SECONDARY, UI_ALIGN_LEFT);
ui_end_stack();

ui_end();
```

### Horizontal Stack

```c
ui_begin();

ui_hstack(8);  // 8px between items
    ui_button("A");
    ui_button("B");
    ui_button("C");
ui_end_stack();

ui_end();
```

### Nested Layouts

```c
ui_begin();

ui_vstack(4);
    ui_label("Header", UI_FONT_PRIMARY, UI_ALIGN_CENTER);

    ui_hstack(8);
        ui_button("Left");
        ui_button("Right");
    ui_end_stack();

    ui_label("Footer", UI_FONT_SECONDARY, UI_ALIGN_CENTER);
ui_end_stack();

ui_end();
```

### Layout Functions

| Function | Description |
|----------|-------------|
| `ui_vstack(spacing)` | Start vertical stack with pixel spacing |
| `ui_hstack(spacing)` | Start horizontal stack with pixel spacing |
| `ui_end_stack()` | End current stack, return to parent |
| `ui_spacer(pixels)` | Add empty space in current direction |

---

## Widgets

### Labels

Non-interactive text display:

```c
// Simple label
ui_label("Hello World", UI_FONT_PRIMARY, UI_ALIGN_LEFT);

// Centered header
ui_label("Settings", UI_FONT_PRIMARY, UI_ALIGN_CENTER);

// Right-aligned value
ui_label("100%", UI_FONT_SECONDARY, UI_ALIGN_RIGHT);
```

**Fonts:**
| Font | Height | Use Case |
|------|--------|----------|
| `UI_FONT_PRIMARY` | 12px | Headers, titles |
| `UI_FONT_SECONDARY` | 10px | Body text, values |

**Alignment:**
| Alignment | Description |
|-----------|-------------|
| `UI_ALIGN_LEFT` | Left edge of layout area |
| `UI_ALIGN_CENTER` | Center of layout area |
| `UI_ALIGN_RIGHT` | Right edge of layout area |

### Buttons

Interactive, focusable buttons:

```c
// Returns true the frame the button is pressed
if (ui_button("Save")) {
    save_data();
}

if (ui_button("Cancel")) {
    go_back();
}
```

Buttons automatically:
- Register for focus navigation
- Show inverted colors when focused
- Return `true` on OK press when focused

### Absolute Positioning

For precise control:

```c
// Button at specific position
if (ui_button_at(10, 40, "OK")) {
    confirm();
}
```

### Progress Bars

Display progress (0.0 to 1.0):

```c
// Simple progress bar (full width)
ui_progress(0.75, 0);

// Progress bar with specific width
ui_progress(download_progress, 100);

// Progress with label
ui_progress_text(0.5, "Loading...", 80);
```

### Checkboxes

Toggle boolean state:

```c
static bool sound_enabled = true;

// Returns true when value changes
if (ui_checkbox("Sound", &sound_enabled)) {
    apply_sound_setting(sound_enabled);
}
```

### Icons

Display bitmap icons:

```c
// XBM format icon data
static const uint8_t icon_wifi[] = { ... };

ui_icon(icon_wifi, 16, 16);
```

### Separator

Horizontal line:

```c
ui_label("Section 1", UI_FONT_PRIMARY, UI_ALIGN_LEFT);
ui_separator();
ui_label("Section 2", UI_FONT_PRIMARY, UI_ALIGN_LEFT);
```

---

## Focus Navigation

Focus moves between interactive widgets using Up/Down keys.

### How Focus Works

1. Each focusable widget (button, menu item, checkbox) registers itself during render
2. Widgets are assigned focus indices in render order
3. Up/Down keys decrement/increment the focus index
4. Focus wraps around at boundaries

### Focus State

```c
// Get current focus index
int16_t focused = ui_get_focus();

// Set focus programmatically
ui_set_focus(2);  // Focus third widget

// Check if specific index is focused
if (ui_is_focused(0)) {
    // First widget is focused
}
```

### Visual Feedback

Focused widgets are drawn with inverted colors:
- **Normal**: Black outline, white fill
- **Focused**: White outline, black fill (inverted)

---

## Menu System

Scrollable menus for selection:

```c
static int16_t scroll = 0;  // Persists scroll position

void render(void) {
    ui_begin();

    ui_label("Select Option", UI_FONT_PRIMARY, UI_ALIGN_CENTER);
    ui_spacer(4);

    // Begin menu: scroll state, visible items, total items
    ui_menu_begin(&scroll, 4, 6);

    if (ui_menu_item("Option 1", 0)) handle_option(0);
    if (ui_menu_item("Option 2", 1)) handle_option(1);
    if (ui_menu_item("Option 3", 2)) handle_option(2);
    if (ui_menu_item("Option 4", 3)) handle_option(3);
    if (ui_menu_item("Option 5", 4)) handle_option(4);
    if (ui_menu_item("Option 6", 5)) handle_option(5);

    ui_menu_end();  // Draws scrollbar

    ui_end();
}
```

### Menu with Values

Settings-style menu with current values:

```c
static int volume = 5;
static bool bluetooth = true;

ui_menu_begin(&scroll, 4, 2);

char vol_str[8];
snprintf(vol_str, sizeof(vol_str), "%d", volume);
if (ui_menu_item_value("Volume", vol_str, 0)) {
    // Open volume adjustment
}

if (ui_menu_item_value("Bluetooth", bluetooth ? "On" : "Off", 1)) {
    bluetooth = !bluetooth;
}

ui_menu_end();
```

### Adjusting Values with Left/Right

```c
void on_input(InputKey key, InputType type) {
    ui_input(key, type);

    // Adjust focused menu item with left/right
    if (type == UI_INPUT_SHORT) {
        if (ui_get_focus() == 0) {  // Volume item
            if (key == UI_KEY_LEFT && volume > 0) volume--;
            if (key == UI_KEY_RIGHT && volume < 10) volume++;
        }
    }
}
```

---

## Footer Buttons

Fixed-position buttons at screen bottom, matching Flipper Zero's pattern:

```c
void render(void) {
    ui_begin();

    // Main content...
    ui_label("Confirm Action?", UI_FONT_PRIMARY, UI_ALIGN_CENTER);

    // Footer buttons (drawn at bottom)
    if (ui_footer_left("Cancel")) {
        go_back();
    }
    if (ui_footer_center("Info")) {
        show_info();
    }
    if (ui_footer_right("OK")) {
        confirm();
    }

    ui_end();
}
```

Footer buttons:
- Are positioned at screen bottom automatically
- Show direction arrow + text
- Respond to their respective keys (Left, OK, Right)
- Don't participate in Up/Down focus navigation

---

## Complete Example

A settings app demonstrating all major features:

```c
#include <frd.h>
#include <imgui.h>
#include <stdio.h>

// App state
typedef enum {
    SCREEN_MAIN,
    SCREEN_BRIGHTNESS,
    SCREEN_ABOUT,
} Screen;

static Screen current_screen = SCREEN_MAIN;
static int16_t menu_scroll = 0;
static int brightness = 5;
static bool sound_enabled = true;
static bool vibration_enabled = true;

// Forward declarations
static void render_main(void);
static void render_brightness(void);
static void render_about(void);

void render(void) {
    switch (current_screen) {
        case SCREEN_MAIN:
            render_main();
            break;
        case SCREEN_BRIGHTNESS:
            render_brightness();
            break;
        case SCREEN_ABOUT:
            render_about();
            break;
    }
}

static void render_main(void) {
    ui_begin();

    ui_label("Settings", UI_FONT_PRIMARY, UI_ALIGN_CENTER);
    ui_spacer(4);

    ui_menu_begin(&menu_scroll, 4, 4);

    // Brightness with value
    char bright_str[8];
    snprintf(bright_str, sizeof(bright_str), "%d", brightness);
    if (ui_menu_item_value("Brightness", bright_str, 0)) {
        current_screen = SCREEN_BRIGHTNESS;
    }

    // Sound toggle
    if (ui_menu_item_value("Sound", sound_enabled ? "On" : "Off", 1)) {
        sound_enabled = !sound_enabled;
    }

    // Vibration toggle
    if (ui_menu_item_value("Vibration", vibration_enabled ? "On" : "Off", 2)) {
        vibration_enabled = !vibration_enabled;
    }

    // About
    if (ui_menu_item("About", 3)) {
        current_screen = SCREEN_ABOUT;
    }

    ui_menu_end();

    if (ui_footer_left("Back")) {
        // Exit app
    }

    ui_end();
}

static void render_brightness(void) {
    ui_begin();

    ui_label("Brightness", UI_FONT_PRIMARY, UI_ALIGN_CENTER);
    ui_spacer(16);

    // Visual brightness bar
    ui_progress((float)brightness / 10.0f, 100);
    ui_spacer(8);

    // Numeric value
    char buf[16];
    snprintf(buf, sizeof(buf), "%d / 10", brightness);
    ui_label(buf, UI_FONT_SECONDARY, UI_ALIGN_CENTER);

    if (ui_footer_left("Back")) {
        current_screen = SCREEN_MAIN;
    }

    ui_end();
}

static void render_about(void) {
    ui_begin();

    ui_label("About", UI_FONT_PRIMARY, UI_ALIGN_CENTER);
    ui_spacer(8);

    ui_label("IMGUI Demo App", UI_FONT_SECONDARY, UI_ALIGN_CENTER);
    ui_label("Version 1.0.0", UI_FONT_SECONDARY, UI_ALIGN_CENTER);
    ui_spacer(4);
    ui_label("Built with IMGUI", UI_FONT_SECONDARY, UI_ALIGN_CENTER);

    if (ui_footer_left("Back")) {
        current_screen = SCREEN_MAIN;
    }

    ui_end();
}

void on_input(InputKey key, InputType type) {
    ui_input(key, type);

    if (ui_back_pressed()) {
        if (current_screen != SCREEN_MAIN) {
            current_screen = SCREEN_MAIN;
        }
        // else: exit app handled by system
    }

    // Brightness adjustment with left/right
    if (current_screen == SCREEN_BRIGHTNESS && type == UI_INPUT_SHORT) {
        if (key == UI_KEY_LEFT && brightness > 0) brightness--;
        if (key == UI_KEY_RIGHT && brightness < 10) brightness++;
    }

    // Quick adjust in menu
    if (current_screen == SCREEN_MAIN && ui_get_focus() == 0) {
        if (type == UI_INPUT_SHORT || type == UI_INPUT_REPEAT) {
            if (key == UI_KEY_LEFT && brightness > 0) brightness--;
            if (key == UI_KEY_RIGHT && brightness < 10) brightness++;
        }
    }
}

int main(void) {
    return 0;
}
```

---

## API Reference

### Context Management

```c
void ui_begin(void);
```
Begin a new frame. Clears the canvas and resets widget state.

```c
void ui_end(void);
```
End the frame. Commits the canvas buffer to display.

```c
void ui_input(UiKey key, UiInputType type);
```
Feed an input event to the UI system. Call from `on_input()`.

```c
bool ui_back_pressed(void);
```
Returns `true` if Back was pressed this frame.

### Layout

```c
void ui_vstack(int16_t spacing);
```
Begin a vertical stack with specified pixel spacing between items.

```c
void ui_hstack(int16_t spacing);
```
Begin a horizontal stack with specified pixel spacing between items.

```c
void ui_end_stack(void);
```
End the current stack and return to parent layout.

```c
void ui_spacer(int16_t pixels);
```
Add empty space in the current stack direction.

### Widgets

```c
void ui_label(const char* text, UiFont font, UiAlign align);
```
Draw a text label. Not focusable.

```c
bool ui_button(const char* text);
```
Draw a focusable button. Returns `true` when activated.

```c
bool ui_button_at(int16_t x, int16_t y, const char* text);
```
Draw a button at absolute position.

```c
void ui_progress(float value, int16_t width);
```
Draw a progress bar. `value` is 0.0 to 1.0, `width` 0 means full width.

```c
void ui_progress_text(float value, const char* text, int16_t width);
```
Draw a progress bar with text label.

```c
bool ui_checkbox(const char* text, bool* checked);
```
Draw a checkbox. Returns `true` when value changes.

```c
void ui_icon(const uint8_t* data, uint8_t width, uint8_t height);
```
Draw an XBM-format icon.

```c
void ui_separator(void);
```
Draw a horizontal separator line.

### Menu

```c
void ui_menu_begin(int16_t* scroll, int16_t visible, int16_t total);
```
Begin a scrollable menu. `scroll` pointer persists scroll position.

```c
bool ui_menu_item(const char* text, int16_t index);
```
Add a menu item. Returns `true` when selected.

```c
bool ui_menu_item_value(const char* label, const char* value, int16_t index);
```
Add a menu item with right-aligned value text.

```c
void ui_menu_end(void);
```
End menu and draw scrollbar.

### Footer

```c
bool ui_footer_left(const char* text);
```
Draw left footer button. Returns `true` on Left+Short press.

```c
bool ui_footer_center(const char* text);
```
Draw center footer button. Returns `true` on OK+Short press.

```c
bool ui_footer_right(const char* text);
```
Draw right footer button. Returns `true` on Right+Short press.

### Focus

```c
int16_t ui_get_focus(void);
```
Get current focus index (-1 if no focusable widgets).

```c
void ui_set_focus(int16_t index);
```
Set focus to specific widget index.

```c
bool ui_is_focused(int16_t index);
```
Check if specific widget index is focused.

---

## Implementation Notes

### Internal State Structure

```c
typedef struct {
    // Layout stack (max 8 levels deep)
    UiLayoutStack layout_stack[8];
    int8_t layout_depth;

    // Focus tracking
    int16_t focus_index;      // Current focus
    int16_t focus_count;      // Focusable widgets this frame

    // Input state
    UiKey last_key;
    UiInputType last_type;
    bool ok_pressed;
    bool back_pressed;

    // Active menu state
    struct {
        bool active;
        int16_t* scroll;
        int16_t visible;
        int16_t total;
    } menu;
} UiContext;
```

### Widget Registration

Focusable widgets call an internal registration function:

```c
static int16_t ui_register_focusable(int16_t x, int16_t y, int16_t w, int16_t h) {
    int16_t index = g_ctx.focus_count++;
    g_ctx.widgets[index] = (UiWidgetRect){ x, y, w, h };
    return index;
}
```

### Focus Navigation

Handled automatically in `ui_input()`:

```c
if (type == UI_INPUT_SHORT || type == UI_INPUT_REPEAT) {
    if (key == UI_KEY_UP) {
        g_ctx.focus_index--;
        if (g_ctx.focus_index < 0) {
            g_ctx.focus_index = g_ctx.focus_count - 1;
        }
    }
    if (key == UI_KEY_DOWN) {
        g_ctx.focus_index++;
        if (g_ctx.focus_index >= g_ctx.focus_count) {
            g_ctx.focus_index = 0;
        }
    }
}
```

### WASM Considerations

- No callbacks cross the WASM boundary
- State is simple integers/booleans (no pointers)
- Canvas operations are host-provided imports
- String handling uses null-terminated C strings

### Performance

The 128x64 display is only 1KB of framebuffer data, so full redraws every frame are acceptable. No dirty rectangle tracking is needed.
