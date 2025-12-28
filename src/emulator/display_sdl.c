#include "display_sdl.h"
#include "canvas.h"
#include <lodepng.h>
#include <stdio.h>
#include <string.h>
#include <stdlib.h>

// u8g2 dummy callbacks (we only use the buffer)
static uint8_t u8x8_gpio_and_delay_callback(u8x8_t* u8x8, uint8_t msg, uint8_t arg_int, void* arg_ptr) {
    (void)u8x8; (void)msg; (void)arg_int; (void)arg_ptr;
    return 1;
}

static uint8_t u8x8_byte_callback(u8x8_t* u8x8, uint8_t msg, uint8_t arg_int, void* arg_ptr) {
    (void)u8x8; (void)msg; (void)arg_int; (void)arg_ptr;
    return 1;
}

void display_sdl_init(display_sdl_t* display) {
    memset(display, 0, sizeof(display_sdl_t));
}

void display_sdl_cleanup(display_sdl_t* display) {
    if (display->texture) {
        SDL_DestroyTexture(display->texture);
        display->texture = NULL;
    }
    if (display->renderer) {
        SDL_DestroyRenderer(display->renderer);
        display->renderer = NULL;
    }
    if (display->window) {
        SDL_DestroyWindow(display->window);
        display->window = NULL;
    }
    if (!display->headless) {
        SDL_Quit();
    }
}

bool display_sdl_start(display_sdl_t* display, bool headless) {
    display->headless = headless;

    // Initialize SDL (skip video init if headless)
    if (!headless) {
        if (SDL_Init(SDL_INIT_VIDEO) < 0) {
            fprintf(stderr, "SDL could not initialize! SDL_Error: %s\n", SDL_GetError());
            return false;
        }

        display->window = SDL_CreateWindow(
            "Fri3d Emulator",
            SDL_WINDOWPOS_UNDEFINED, SDL_WINDOWPOS_UNDEFINED,
            SCREEN_WIDTH * DISPLAY_SDL_SCALE_FACTOR, SCREEN_HEIGHT * DISPLAY_SDL_SCALE_FACTOR,
            SDL_WINDOW_SHOWN
        );
        if (!display->window) {
            fprintf(stderr, "Window could not be created! SDL_Error: %s\n", SDL_GetError());
            return false;
        }

        display->renderer = SDL_CreateRenderer(display->window, -1, SDL_RENDERER_ACCELERATED);
        if (!display->renderer) {
            fprintf(stderr, "Renderer could not be created! SDL_Error: %s\n", SDL_GetError());
            return false;
        }

        display->texture = SDL_CreateTexture(
            display->renderer,
            SDL_PIXELFORMAT_RGBA8888,
            SDL_TEXTUREACCESS_STREAMING,
            SCREEN_WIDTH, SCREEN_HEIGHT
        );
        if (!display->texture) {
            fprintf(stderr, "Texture could not be created! SDL_Error: %s\n", SDL_GetError());
            return false;
        }
    }

    // Initialize u8g2 (SSD1306 128x64 full buffer mode)
    u8g2_Setup_ssd1306_128x64_noname_f(
        &display->u8g2, U8G2_R0,
        u8x8_byte_callback,
        u8x8_gpio_and_delay_callback
    );
    u8g2_InitDisplay(&display->u8g2);
    u8g2_SetPowerSave(&display->u8g2, 0);
    u8g2_ClearBuffer(&display->u8g2);

    return true;
}

u8g2_t* display_sdl_get_u8g2(display_sdl_t* display) {
    return &display->u8g2;
}

void display_sdl_flush(display_sdl_t* display) {
    if (display->headless) return;

    uint8_t* buffer = u8g2_GetBufferPtr(&display->u8g2);

    void* pixels;
    int pitch;
    SDL_LockTexture(display->texture, NULL, &pixels, &pitch);

    uint32_t* dst = (uint32_t*)pixels;

    // u8g2 SSD1306 buffer layout: each byte is 8 vertical pixels
    // Byte 0: pixels (0,0)-(0,7), Byte 1: pixels (1,0)-(1,7), etc.
    for (int y = 0; y < SCREEN_HEIGHT; y++) {
        for (int x = 0; x < SCREEN_WIDTH; x++) {
            int byte_idx = x + (y / 8) * SCREEN_WIDTH;
            int bit_idx = y % 8;
            bool pixel = (buffer[byte_idx] >> bit_idx) & 1;
            // Black pixels are "on" (1 = white on screen), white pixels are "off" (0 = black on screen)
            dst[y * SCREEN_WIDTH + x] = pixel ? 0xFFFFFFFF : 0x000000FF;
        }
    }

    SDL_UnlockTexture(display->texture);
    SDL_RenderCopy(display->renderer, display->texture, NULL, NULL);
    SDL_RenderPresent(display->renderer);
}

bool display_sdl_should_quit(display_sdl_t* display) {
    return display->should_quit;
}

void display_sdl_set_quit(display_sdl_t* display, bool quit) {
    display->should_quit = quit;
}

SDL_Window* display_sdl_get_window(display_sdl_t* display) {
    return display->window;
}

bool display_sdl_is_headless(const display_sdl_t* display) {
    return display->headless;
}

bool display_sdl_save_screenshot(display_sdl_t* display, const char* path) {
    uint8_t* buffer = u8g2_GetBufferPtr(&display->u8g2);

    // Create RGBA image data (lodepng expects RGBA)
    unsigned char* image = (unsigned char*)malloc(SCREEN_WIDTH * SCREEN_HEIGHT * 4);
    if (!image) {
        fprintf(stderr, "Failed to allocate image buffer\n");
        return false;
    }

    // Convert u8g2 buffer to RGBA
    for (int y = 0; y < SCREEN_HEIGHT; y++) {
        for (int x = 0; x < SCREEN_WIDTH; x++) {
            int byte_idx = x + (y / 8) * SCREEN_WIDTH;
            int bit_idx = y % 8;
            bool pixel = (buffer[byte_idx] >> bit_idx) & 1;
            // Black pixels are "on" (1), white pixels are "off" (0)
            uint8_t color = pixel ? 0 : 255;
            size_t idx = 4 * (y * SCREEN_WIDTH + x);
            image[idx + 0] = color; // R
            image[idx + 1] = color; // G
            image[idx + 2] = color; // B
            image[idx + 3] = 255;   // A (fully opaque)
        }
    }

    unsigned error = lodepng_encode32_file(path, image, SCREEN_WIDTH, SCREEN_HEIGHT);
    free(image);

    if (error) {
        fprintf(stderr, "PNG encoder error: %s\n", lodepng_error_text(error));
        return false;
    }

    return true;
}
