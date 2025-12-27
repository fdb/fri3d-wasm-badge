// ============================================================================
// Test UI App - Tests all IMGUI components
// ============================================================================

#include <frd.h>
#include <canvas.h>
#include <input.h>
#include <imgui.h>
#include <stdio.h>

// Scene definitions
typedef enum {
    SCENE_COUNTER = 0,
    SCENE_MENU,
    SCENE_LAYOUT,
    SCENE_PROGRESS,
    SCENE_CHECKBOX,
    SCENE_FOOTER,
    SCENE_COUNT
} Scene;

static Scene g_current_scene = SCENE_COUNTER;

// Scene state
static int g_counter = 0;
static int16_t g_menu_scroll = 0;
static int g_brightness = 5;
static bool g_sound = true;
static bool g_vibration = true;
static bool g_wifi = false;
static float g_progress = 0.0f;
static bool g_check1 = false;
static bool g_check2 = true;
static bool g_check3 = false;

// Export scene functions for testing
uint32_t get_scene(void) {
    return (uint32_t)g_current_scene;
}

void set_scene(uint32_t scene) {
    if (scene < SCENE_COUNT) {
        g_current_scene = (Scene)scene;
        // Reset scene-specific state
        g_menu_scroll = 0;
    }
}

uint32_t get_scene_count(void) {
    return SCENE_COUNT;
}

// ----------------------------------------------------------------------------
// Scene: Counter (simple IMGUI demonstration)
// ----------------------------------------------------------------------------

static void render_counter(void) {
    ui_begin();

    ui_label("Counter Demo", UI_FONT_PRIMARY, UI_ALIGN_CENTER);
    ui_spacer(8);

    // Display counter value
    char buf[32];
    snprintf(buf, sizeof(buf), "Count: %d", g_counter);
    ui_label(buf, UI_FONT_SECONDARY, UI_ALIGN_CENTER);
    ui_spacer(8);

    // Buttons in centered horizontal layout
    ui_hstack_centered(4);
    if (ui_button("+")) {
        g_counter++;
    }
    if (ui_button("-")) {
        g_counter--;
    }
    if (ui_button("Reset")) {
        g_counter = 0;
    }
    ui_end_stack();

    ui_end();
}

// ----------------------------------------------------------------------------
// Scene: Menu (scrollable menu with values)
// ----------------------------------------------------------------------------

static void render_menu(void) {
    ui_begin();

    ui_label("Settings Menu", UI_FONT_PRIMARY, UI_ALIGN_CENTER);
    ui_spacer(4);

    ui_menu_begin(&g_menu_scroll, 4, 6);

    char bright_str[8];
    snprintf(bright_str, sizeof(bright_str), "%d", g_brightness);
    if (ui_menu_item_value("Brightness", bright_str, 0)) {
        // Could navigate to brightness screen
    }

    if (ui_menu_item_value("Sound", g_sound ? "On" : "Off", 1)) {
        g_sound = !g_sound;
    }

    if (ui_menu_item_value("Vibration", g_vibration ? "On" : "Off", 2)) {
        g_vibration = !g_vibration;
    }

    if (ui_menu_item_value("WiFi", g_wifi ? "On" : "Off", 3)) {
        g_wifi = !g_wifi;
    }

    if (ui_menu_item("About", 4)) {
        // Could navigate to about screen
    }

    if (ui_menu_item("Reset All", 5)) {
        g_brightness = 5;
        g_sound = true;
        g_vibration = true;
        g_wifi = false;
    }

    ui_menu_end();

    ui_end();
}

// ----------------------------------------------------------------------------
// Scene: Layout (testing stacks)
// ----------------------------------------------------------------------------

static void render_layout(void) {
    ui_begin();

    ui_label("Layout Demo", UI_FONT_PRIMARY, UI_ALIGN_CENTER);
    ui_spacer(4);

    // Vertical stack with custom spacing
    ui_vstack(2);
    ui_label("VStack items:", UI_FONT_SECONDARY, UI_ALIGN_LEFT);
    ui_label("  Item 1", UI_FONT_SECONDARY, UI_ALIGN_LEFT);
    ui_label("  Item 2", UI_FONT_SECONDARY, UI_ALIGN_LEFT);
    ui_label("  Item 3", UI_FONT_SECONDARY, UI_ALIGN_LEFT);
    ui_end_stack();

    ui_spacer(4);
    ui_separator();
    ui_spacer(4);

    // Labels with different alignments
    ui_label("Left", UI_FONT_SECONDARY, UI_ALIGN_LEFT);
    ui_label("Center", UI_FONT_SECONDARY, UI_ALIGN_CENTER);
    ui_label("Right", UI_FONT_SECONDARY, UI_ALIGN_RIGHT);

    ui_end();
}

// ----------------------------------------------------------------------------
// Scene: Progress bars
// ----------------------------------------------------------------------------

static void render_progress(void) {
    ui_begin();

    ui_label("Progress Demo", UI_FONT_PRIMARY, UI_ALIGN_CENTER);
    ui_spacer(8);

    // Animated progress (increments each frame when focused)
    ui_label("Loading:", UI_FONT_SECONDARY, UI_ALIGN_LEFT);
    ui_progress(g_progress, 0);
    ui_spacer(4);

    // Fixed progress bars
    ui_label("25%:", UI_FONT_SECONDARY, UI_ALIGN_LEFT);
    ui_progress(0.25f, 100);
    ui_spacer(2);

    ui_label("50%:", UI_FONT_SECONDARY, UI_ALIGN_LEFT);
    ui_progress(0.50f, 100);
    ui_spacer(2);

    ui_label("75%:", UI_FONT_SECONDARY, UI_ALIGN_LEFT);
    ui_progress(0.75f, 100);
    ui_spacer(2);

    ui_label("100%:", UI_FONT_SECONDARY, UI_ALIGN_LEFT);
    ui_progress(1.0f, 100);

    // Animate the loading bar
    g_progress += 0.02f;
    if (g_progress > 1.0f) {
        g_progress = 0.0f;
    }

    ui_end();
}

// ----------------------------------------------------------------------------
// Scene: Checkboxes
// ----------------------------------------------------------------------------

static void render_checkbox(void) {
    ui_begin();

    ui_label("Checkbox Demo", UI_FONT_PRIMARY, UI_ALIGN_CENTER);
    ui_spacer(8);

    if (ui_checkbox("Option 1", &g_check1)) {
        // Value changed
    }

    if (ui_checkbox("Option 2 (default on)", &g_check2)) {
        // Value changed
    }

    if (ui_checkbox("Option 3", &g_check3)) {
        // Value changed
    }

    ui_spacer(8);

    // Show current state
    char buf[64];
    snprintf(buf, sizeof(buf), "State: %d %d %d",
             g_check1 ? 1 : 0,
             g_check2 ? 1 : 0,
             g_check3 ? 1 : 0);
    ui_label(buf, UI_FONT_SECONDARY, UI_ALIGN_CENTER);

    ui_end();
}

// ----------------------------------------------------------------------------
// Scene: Footer buttons
// ----------------------------------------------------------------------------

static void render_footer(void) {
    ui_begin();

    ui_label("Footer Demo", UI_FONT_PRIMARY, UI_ALIGN_CENTER);
    ui_spacer(8);

    static int left_count = 0;
    static int right_count = 0;

    // Show both counters on one line
    char buf[32];
    snprintf(buf, sizeof(buf), "Left: %d   Right: %d", left_count, right_count);
    ui_label(buf, UI_FONT_SECONDARY, UI_ALIGN_CENTER);

    ui_spacer(4);
    ui_label("Press </> to change", UI_FONT_SECONDARY, UI_ALIGN_CENTER);

    // Footer buttons
    if (ui_footer_left("Dec")) {
        left_count--;
    }

    ui_footer_center("OK");

    if (ui_footer_right("Inc")) {
        right_count++;
    }

    ui_end();
}

// ----------------------------------------------------------------------------
// Main render function
// ----------------------------------------------------------------------------

void render(void) {
    switch (g_current_scene) {
        case SCENE_COUNTER:
            render_counter();
            break;
        case SCENE_MENU:
            render_menu();
            break;
        case SCENE_LAYOUT:
            render_layout();
            break;
        case SCENE_PROGRESS:
            render_progress();
            break;
        case SCENE_CHECKBOX:
            render_checkbox();
            break;
        case SCENE_FOOTER:
            render_footer();
            break;
        default:
            break;
    }
}

// ----------------------------------------------------------------------------
// Input handling
// ----------------------------------------------------------------------------

void on_input(InputKey key, InputType type) {
    // Convert InputKey/InputType to UiKey/UiInputType
    UiKey ui_key = (UiKey)key;
    UiInputType ui_type = (type == InputTypePress) ? UI_INPUT_SHORT : UI_INPUT_RELEASE;

    // Feed to IMGUI system
    ui_input(ui_key, ui_type);

    // Handle back to exit or change scene for demo
    if (ui_back_pressed()) {
        // In a real app, this would exit
        // For demo, cycle scenes backwards
        if (g_current_scene == 0) {
            g_current_scene = SCENE_COUNT - 1;
        } else {
            g_current_scene--;
        }
        g_menu_scroll = 0;
    }

    // Additional scene-specific input handling
    if (type == InputTypePress) {
        // Brightness adjustment in menu scene
        if (g_current_scene == SCENE_MENU && ui_get_focus() == 0) {
            if (key == InputKeyLeft && g_brightness > 0) {
                g_brightness--;
            }
            if (key == InputKeyRight && g_brightness < 10) {
                g_brightness++;
            }
        }
    }
}

// Required by WASI libc
int main(void) {
    return 0;
}
