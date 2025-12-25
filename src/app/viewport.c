#include "viewport.h"
#include <stdlib.h>

struct ViewPort {
    ViewPortDrawCallback draw_callback;
    void* draw_context;
    ViewPortInputCallback input_callback;
    void* input_context;
    bool enabled;
    bool needs_update;
};

// Global viewport (single viewport for now, OS accesses this)
ViewPort* g_viewport = NULL;

ViewPort* view_port_alloc(void) {
    ViewPort* vp = malloc(sizeof(ViewPort));
    vp->draw_callback = NULL;
    vp->draw_context = NULL;
    vp->input_callback = NULL;
    vp->input_context = NULL;
    vp->enabled = true;
    vp->needs_update = true;
    g_viewport = vp;
    return vp;
}

void view_port_free(ViewPort* view_port) {
    if (g_viewport == view_port) {
        g_viewport = NULL;
    }
    free(view_port);
}

void view_port_draw_callback_set(ViewPort* view_port, ViewPortDrawCallback callback, void* context) {
    view_port->draw_callback = callback;
    view_port->draw_context = context;
}

void view_port_input_callback_set(ViewPort* view_port, ViewPortInputCallback callback, void* context) {
    view_port->input_callback = callback;
    view_port->input_context = context;
}

void view_port_enabled_set(ViewPort* view_port, bool enabled) {
    view_port->enabled = enabled;
}

void view_port_update(ViewPort* view_port) {
    view_port->needs_update = true;
}

ViewPort* view_port_get_current(void) {
    return g_viewport;
}

bool view_port_is_enabled(ViewPort* view_port) {
    return view_port && view_port->enabled;
}

void view_port_render(ViewPort* view_port, Canvas* canvas) {
    if (view_port && view_port->draw_callback) {
        view_port->draw_callback(canvas, view_port->draw_context);
    }
}

void view_port_handle_input(ViewPort* view_port, InputEvent* event) {
    if (view_port && view_port->input_callback) {
        view_port->input_callback(event, view_port->input_context);
    }
}
