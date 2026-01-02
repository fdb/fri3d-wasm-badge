#include "display_sdl.h"

#define STB_IMAGE_WRITE_IMPLEMENTATION
#include "stb_image_write.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define DISPLAY_SDL_SCALE_FACTOR 4

static uint8_t display_sdl_u8x8_gpio_and_delay(u8x8_t* u8x8, uint8_t msg, uint8_t arg_int, void* arg_ptr) {
    (void)u8x8;
    (void)msg;
    (void)arg_int;
    (void)arg_ptr;
    return 1;
}

static uint8_t display_sdl_u8x8_byte(u8x8_t* u8x8, uint8_t msg, uint8_t arg_int, void* arg_ptr) {
    (void)u8x8;
    (void)msg;
    (void)arg_int;
    (void)arg_ptr;
    return 1;
}

bool display_sdl_init(display_sdl_t* display, bool headless) {
    if (!display) {
        return false;
    }

    memset(display, 0, sizeof(*display));
    display->headless = headless;

    if (!headless) {
        if (SDL_Init(SDL_INIT_VIDEO) < 0) {
            fprintf(stderr, "SDL could not initialize! SDL_Error: %s\n", SDL_GetError());
            return false;
        }

        display->window = SDL_CreateWindow(
            "Fri3d Emulator",
            SDL_WINDOWPOS_UNDEFINED, SDL_WINDOWPOS_UNDEFINED,
            FRI3D_SCREEN_WIDTH * DISPLAY_SDL_SCALE_FACTOR,
            FRI3D_SCREEN_HEIGHT * DISPLAY_SDL_SCALE_FACTOR,
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
            FRI3D_SCREEN_WIDTH,
            FRI3D_SCREEN_HEIGHT
        );
        if (!display->texture) {
            fprintf(stderr, "Texture could not be created! SDL_Error: %s\n", SDL_GetError());
            return false;
        }
    }

    u8g2_Setup_ssd1306_128x64_noname_f(
        &display->u8g2, U8G2_R0,
        display_sdl_u8x8_byte,
        display_sdl_u8x8_gpio_and_delay
    );
    u8g2_InitDisplay(&display->u8g2);
    u8g2_SetPowerSave(&display->u8g2, 0);
    u8g2_ClearBuffer(&display->u8g2);

    return true;
}

void display_sdl_deinit(display_sdl_t* display) {
    if (!display) {
        return;
    }

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

u8g2_t* display_sdl_get_u8g2(display_sdl_t* display) {
    return display ? &display->u8g2 : NULL;
}

void display_sdl_flush(display_sdl_t* display) {
    if (!display || display->headless) {
        return;
    }

    uint8_t* buffer = u8g2_GetBufferPtr(&display->u8g2);

    void* pixels = NULL;
    int pitch = 0;
    SDL_LockTexture(display->texture, NULL, &pixels, &pitch);

    uint32_t* dst = (uint32_t*)pixels;

    for (int y = 0; y < FRI3D_SCREEN_HEIGHT; y++) {
        for (int x = 0; x < FRI3D_SCREEN_WIDTH; x++) {
            int byte_idx = x + (y / 8) * FRI3D_SCREEN_WIDTH;
            int bit_idx = y % 8;
            bool pixel = (buffer[byte_idx] >> bit_idx) & 1;
            dst[y * FRI3D_SCREEN_WIDTH + x] = pixel ? 0xFFFFFFFFu : 0x000000FFu;
        }
    }

    SDL_UnlockTexture(display->texture);
    SDL_RenderCopy(display->renderer, display->texture, NULL, NULL);
    SDL_RenderPresent(display->renderer);
}

bool display_sdl_should_quit(const display_sdl_t* display) {
    return display ? display->should_quit : true;
}

void display_sdl_set_quit(display_sdl_t* display, bool should_quit) {
    if (!display) {
        return;
    }
    display->should_quit = should_quit;
}

bool display_sdl_is_headless(const display_sdl_t* display) {
    return display ? display->headless : false;
}

bool display_sdl_save_screenshot(display_sdl_t* display, const char* path) {
    if (!display || !path) {
        return false;
    }

    uint8_t* buffer = u8g2_GetBufferPtr(&display->u8g2);
    size_t image_size = (size_t)FRI3D_SCREEN_WIDTH * (size_t)FRI3D_SCREEN_HEIGHT * 4;
    unsigned char* image = (unsigned char*)malloc(image_size);
    if (!image) {
        return false;
    }

    for (int y = 0; y < FRI3D_SCREEN_HEIGHT; y++) {
        for (int x = 0; x < FRI3D_SCREEN_WIDTH; x++) {
            int byte_idx = x + (y / 8) * FRI3D_SCREEN_WIDTH;
            int bit_idx = y % 8;
            bool pixel = (buffer[byte_idx] >> bit_idx) & 1;
            uint8_t color = pixel ? 0 : 255;
            size_t idx = 4u * ((size_t)y * (size_t)FRI3D_SCREEN_WIDTH + (size_t)x);
            image[idx + 0] = color;
            image[idx + 1] = color;
            image[idx + 2] = color;
            image[idx + 3] = 255;
        }
    }

    int result = stbi_write_png(path,
                                FRI3D_SCREEN_WIDTH,
                                FRI3D_SCREEN_HEIGHT,
                                4,
                                image,
                                FRI3D_SCREEN_WIDTH * 4);
    free(image);

    if (result == 0) {
        fprintf(stderr, "PNG encoder error: failed to write %s\n", path);
        return false;
    }

    return true;
}
