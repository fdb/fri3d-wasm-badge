#include <app.h>
#include <canvas.h>
#include <frd.h>
#include <imgui.h>
#include <input.h>

typedef struct {
    const char* name;
    uint32_t id;
} launcher_entry_t;

// App IDs must match the host app registry order (1..N). ID 0 is the launcher.
static const launcher_entry_t g_apps[] = {
    { "Circles", 1 },
    { "Mandelbrot", 2 },
    { "Test Drawing", 3 },
    { "Test UI", 4 },
    { "Snake", 5 },
};

static int16_t g_menu_scroll = 0;

void render(void) {
    ui_begin();

    ui_label("Fri3d Apps", FontPrimary, AlignCenter);
    ui_separator();

    ui_menu_begin(&g_menu_scroll, 4, (int16_t)(sizeof(g_apps) / sizeof(g_apps[0])));
    for (int16_t i = 0; i < (int16_t)(sizeof(g_apps) / sizeof(g_apps[0])); i++) {
        if (ui_menu_item(g_apps[i].name, i)) {
            start_app(g_apps[i].id);
        }
    }
    ui_menu_end();

    ui_end();
}

void on_input(input_key_t key, input_type_t type) {
    ui_input((ui_key_t)key, (ui_input_type_t)type);
}

int main(void) {
    return 0;
}
