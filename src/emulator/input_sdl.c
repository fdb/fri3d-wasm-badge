#include "input_sdl.h"

#include <string.h>

static bool input_sdl_queue_is_empty(const input_sdl_t* input) {
    return input->queue_head == input->queue_tail;
}

static bool input_sdl_queue_is_full(const input_sdl_t* input) {
    return ((input->queue_head + 1) % (sizeof(input->event_queue) / sizeof(input->event_queue[0]))) == input->queue_tail;
}

static void input_sdl_queue_push(input_sdl_t* input, input_event_t event) {
    if (input_sdl_queue_is_full(input)) {
        return;
    }
    input->event_queue[input->queue_head] = event;
    input->queue_head = (input->queue_head + 1) % (sizeof(input->event_queue) / sizeof(input->event_queue[0]));
}

static input_event_t input_sdl_queue_pop(input_sdl_t* input) {
    input_event_t event = {0};
    if (input_sdl_queue_is_empty(input)) {
        return event;
    }
    event = input->event_queue[input->queue_tail];
    input->queue_tail = (input->queue_tail + 1) % (sizeof(input->event_queue) / sizeof(input->event_queue[0]));
    return event;
}

static bool input_sdl_key_to_input_key(SDL_Keycode key, input_key_t* out_key) {
    switch (key) {
        case SDLK_UP:
            *out_key = input_key_up;
            return true;
        case SDLK_DOWN:
            *out_key = input_key_down;
            return true;
        case SDLK_LEFT:
            *out_key = input_key_left;
            return true;
        case SDLK_RIGHT:
            *out_key = input_key_right;
            return true;
        case SDLK_RETURN:
        case SDLK_z:
            *out_key = input_key_ok;
            return true;
        case SDLK_BACKSPACE:
        case SDLK_x:
            *out_key = input_key_back;
            return true;
        default:
            return false;
    }
}

void input_sdl_init(input_sdl_t* input, display_sdl_t* display) {
    if (!input) {
        return;
    }
    memset(input, 0, sizeof(*input));
    input->display = display;
}

void input_sdl_poll(void* context) {
    input_sdl_t* input = (input_sdl_t*)context;
    if (!input) {
        return;
    }

    SDL_Event event;
    while (SDL_PollEvent(&event) != 0) {
        if (event.type == SDL_QUIT) {
            if (input->display) {
                display_sdl_set_quit(input->display, true);
            }
        } else if (event.type == SDL_KEYDOWN || event.type == SDL_KEYUP) {
            if (event.type == SDL_KEYDOWN && event.key.repeat != 0) {
                continue;
            }
            if (event.type == SDL_KEYDOWN && event.key.keysym.sym == SDLK_s) {
                input->screenshot_requested = true;
                continue;
            }

            input_key_t key;
            if (input_sdl_key_to_input_key(event.key.keysym.sym, &key)) {
                input_event_t input_event;
                input_event.key = key;
                input_event.type = (event.type == SDL_KEYDOWN) ? input_type_press : input_type_release;
                input_sdl_queue_push(input, input_event);
            }
        }
    }
}

bool input_sdl_has_event(void* context) {
    input_sdl_t* input = (input_sdl_t*)context;
    if (!input) {
        return false;
    }
    return !input_sdl_queue_is_empty(input);
}

input_event_t input_sdl_get_event(void* context) {
    input_sdl_t* input = (input_sdl_t*)context;
    if (!input) {
        input_event_t empty = {0};
        return empty;
    }
    return input_sdl_queue_pop(input);
}

uint32_t input_sdl_get_time_ms(void* context) {
    (void)context;
    return SDL_GetTicks();
}

bool input_sdl_was_screenshot_requested(input_sdl_t* input) {
    if (!input) {
        return false;
    }
    bool requested = input->screenshot_requested;
    input->screenshot_requested = false;
    return requested;
}

void input_sdl_bind_handler(input_sdl_t* input, input_handler_t* handler) {
    if (!input || !handler) {
        return;
    }

    handler->context = input;
    handler->poll = input_sdl_poll;
    handler->has_event = input_sdl_has_event;
    handler->get_event = input_sdl_get_event;
    handler->get_time_ms = input_sdl_get_time_ms;
}
