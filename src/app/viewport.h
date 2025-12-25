#pragma once

#include "canvas.h"
#include "input.h"
#include <stdbool.h>

typedef struct ViewPort ViewPort;

// Callback types
typedef void (*ViewPortDrawCallback)(Canvas* canvas, void* context);
typedef void (*ViewPortInputCallback)(InputEvent* event, void* context);

// Lifecycle
ViewPort* view_port_alloc(void);
void view_port_free(ViewPort* view_port);

// Callback registration
void view_port_draw_callback_set(ViewPort* view_port, ViewPortDrawCallback callback, void* context);
void view_port_input_callback_set(ViewPort* view_port, ViewPortInputCallback callback, void* context);

// Control
void view_port_enabled_set(ViewPort* view_port, bool enabled);
void view_port_update(ViewPort* view_port);

// Internal use by OS
ViewPort* view_port_get_current(void);
bool view_port_is_enabled(ViewPort* view_port);
void view_port_render(ViewPort* view_port, Canvas* canvas);
void view_port_handle_input(ViewPort* view_port, InputEvent* event);
