#pragma once

#include <stdbool.h>
#include <stdint.h>
#include <stddef.h>
#include <SDL.h>

#include "display_sdl.h"
#include "input.h"

typedef struct {
    display_sdl_t* display;
    input_event_t event_queue[32];
    size_t queue_head;
    size_t queue_tail;
    bool screenshot_requested;
} input_sdl_t;

void input_sdl_init(input_sdl_t* input, display_sdl_t* display);
void input_sdl_poll(void* context);
bool input_sdl_has_event(void* context);
input_event_t input_sdl_get_event(void* context);
uint32_t input_sdl_get_time_ms(void* context);

bool input_sdl_was_screenshot_requested(input_sdl_t* input);

void input_sdl_bind_handler(input_sdl_t* input, input_handler_t* handler);
