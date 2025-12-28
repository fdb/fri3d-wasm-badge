#include "input_sdl.h"
#include <string.h>

// Forward declarations for handler interface functions
static void input_sdl_poll_handler(input_handler_t* self);
static bool input_sdl_has_event_handler(const input_handler_t* self);
static input_event_t input_sdl_get_event_handler(input_handler_t* self);
static uint32_t input_sdl_get_time_ms_handler(const input_handler_t* self);

static int sdl_key_to_input_key(SDL_Keycode key);
static void queue_event(input_sdl_t* input, input_key_t key, input_type_t type);

void input_sdl_init(input_sdl_t* input, display_sdl_t* display) {
    memset(input, 0, sizeof(input_sdl_t));
    input->display = display;

    // Set up handler interface
    input->base.poll = input_sdl_poll_handler;
    input->base.has_event = input_sdl_has_event_handler;
    input->base.get_event = input_sdl_get_event_handler;
    input->base.get_time_ms = input_sdl_get_time_ms_handler;
}

void input_sdl_poll(input_sdl_t* input) {
    SDL_Event e;

    while (SDL_PollEvent(&e) != 0) {
        if (e.type == SDL_QUIT) {
            display_sdl_set_quit(input->display, true);
        } else if (e.type == SDL_KEYDOWN || e.type == SDL_KEYUP) {
            // Handle screenshot key (S)
            if (e.type == SDL_KEYDOWN && e.key.keysym.sym == SDLK_s) {
                input->screenshot_requested = true;
                continue;
            }

            int input_key = sdl_key_to_input_key(e.key.keysym.sym);
            if (input_key >= 0) {
                queue_event(input,
                           (input_key_t)input_key,
                           (e.type == SDL_KEYDOWN) ? INPUT_TYPE_PRESS : INPUT_TYPE_RELEASE);
            }
        }
    }
}

bool input_sdl_has_event(const input_sdl_t* input) {
    return input->queue_head != input->queue_tail;
}

input_event_t input_sdl_get_event(input_sdl_t* input) {
    input_event_t event = input->event_queue[input->queue_tail];
    input->queue_tail = (input->queue_tail + 1) % INPUT_SDL_QUEUE_SIZE;
    return event;
}

uint32_t input_sdl_get_time_ms(const input_sdl_t* input) {
    (void)input;
    return SDL_GetTicks();
}

bool input_sdl_was_screenshot_requested(input_sdl_t* input) {
    bool requested = input->screenshot_requested;
    input->screenshot_requested = false;
    return requested;
}

input_handler_t* input_sdl_get_handler(input_sdl_t* input) {
    return &input->base;
}

// Handler interface implementation (wrappers that cast self to input_sdl_t)
static void input_sdl_poll_handler(input_handler_t* self) {
    input_sdl_poll((input_sdl_t*)self);
}

static bool input_sdl_has_event_handler(const input_handler_t* self) {
    return input_sdl_has_event((const input_sdl_t*)self);
}

static input_event_t input_sdl_get_event_handler(input_handler_t* self) {
    return input_sdl_get_event((input_sdl_t*)self);
}

static uint32_t input_sdl_get_time_ms_handler(const input_handler_t* self) {
    return input_sdl_get_time_ms((const input_sdl_t*)self);
}

static int sdl_key_to_input_key(SDL_Keycode key) {
    switch (key) {
        case SDLK_UP:
            return (int)INPUT_KEY_UP;
        case SDLK_DOWN:
            return (int)INPUT_KEY_DOWN;
        case SDLK_LEFT:
            return (int)INPUT_KEY_LEFT;
        case SDLK_RIGHT:
            return (int)INPUT_KEY_RIGHT;
        case SDLK_RETURN:
        case SDLK_z:
            return (int)INPUT_KEY_OK;
        case SDLK_BACKSPACE:
        case SDLK_x:
            return (int)INPUT_KEY_BACK;
        default:
            return -1;
    }
}

static void queue_event(input_sdl_t* input, input_key_t key, input_type_t type) {
    size_t next_head = (input->queue_head + 1) % INPUT_SDL_QUEUE_SIZE;
    if (next_head != input->queue_tail) {  // Not full
        input->event_queue[input->queue_head].key = key;
        input->event_queue[input->queue_head].type = type;
        input->queue_head = next_head;
    }
}
