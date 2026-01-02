#pragma once

#include <stdbool.h>
#include <SDL.h>
#include <u8g2.h>

#include "display.h"

typedef struct {
    SDL_Window* window;
    SDL_Renderer* renderer;
    SDL_Texture* texture;
    u8g2_t u8g2;
    bool should_quit;
    bool headless;
} display_sdl_t;

bool display_sdl_init(display_sdl_t* display, bool headless);
void display_sdl_deinit(display_sdl_t* display);

u8g2_t* display_sdl_get_u8g2(display_sdl_t* display);
void display_sdl_flush(display_sdl_t* display);
bool display_sdl_should_quit(const display_sdl_t* display);
void display_sdl_set_quit(display_sdl_t* display, bool should_quit);
bool display_sdl_is_headless(const display_sdl_t* display);

bool display_sdl_save_screenshot(display_sdl_t* display, const char* path);
