#include "app_manager.h"
#include "canvas.h"
#include "display_sdl.h"
#include "input.h"
#include "input_sdl.h"
#include "random.h"

#include <SDL.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
    const char* wasm_file;
    const char* screenshot_path;
    bool test_mode;
    bool headless;
    int test_scene;
} options_t;

static void print_usage(const char* program) {
    fprintf(stderr, "Usage: %s [options] [wasm_file]\n\n", program);
    fprintf(stderr, "Options:\n");
    fprintf(stderr, "  --test              Run in test mode (render and exit)\n");
    fprintf(stderr, "  --scene <n>         Set scene number (for test apps)\n");
    fprintf(stderr, "  --screenshot <path> Save screenshot to path (PNG format)\n");
    fprintf(stderr, "  --headless          Run without display (requires --screenshot)\n");
    fprintf(stderr, "  --help              Show this help\n\n");
    fprintf(stderr, "If no wasm_file is specified, runs the launcher.wasm app.\n");
}

static bool parse_args(int argc, char* argv[], options_t* options) {
    if (!options) {
        return false;
    }

    options->wasm_file = NULL;
    options->screenshot_path = NULL;
    options->test_mode = false;
    options->headless = false;
    options->test_scene = -1;

    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "--test") == 0) {
            options->test_mode = true;
        } else if (strcmp(argv[i], "--scene") == 0 && i + 1 < argc) {
            options->test_scene = atoi(argv[++i]);
        } else if (strcmp(argv[i], "--screenshot") == 0 && i + 1 < argc) {
            options->screenshot_path = argv[++i];
        } else if (strcmp(argv[i], "--headless") == 0) {
            options->headless = true;
        } else if (strcmp(argv[i], "--help") == 0) {
            print_usage(argv[0]);
            return false;
        } else if (argv[i][0] != '-') {
            options->wasm_file = argv[i];
        } else {
            fprintf(stderr, "Unknown option: %s\n", argv[i]);
            print_usage(argv[0]);
            return false;
        }
    }

    if (options->headless && !options->screenshot_path) {
        fprintf(stderr, "Error: --headless requires --screenshot\n");
        return false;
    }

    return true;
}

static void reset_combo_callback(void* context) {
    app_manager_t* manager = (app_manager_t*)context;
    if (!manager) {
        return;
    }
    printf("Reset combo triggered - returning to launcher\n");
    app_manager_show_launcher(manager);
}

int main(int argc, char* argv[]) {
    options_t options;
    if (!parse_args(argc, argv, &options)) {
        return (argc > 1 && strcmp(argv[1], "--help") == 0) ? 0 : 1;
    }

    display_sdl_t display;
    if (!display_sdl_init(&display, options.headless)) {
        fprintf(stderr, "Failed to initialize display\n");
        return 1;
    }

    canvas_t canvas;
    canvas_init(&canvas, display_sdl_get_u8g2(&display));

    random_t random;
    random_init(&random);

    input_sdl_t input_sdl;
    input_sdl_init(&input_sdl, &display);

    app_manager_t app_manager;
    if (!app_manager_init(&app_manager, &canvas, &random)) {
        fprintf(stderr, "Failed to initialize app manager: %s\n", app_manager_get_last_error(&app_manager));
        display_sdl_deinit(&display);
        return 1;
    }

    wasm_runner_t* runner = app_manager_get_wasm_runner(&app_manager);
    if (runner) {
        wasm_runner_set_time_provider(runner, input_sdl_get_time_ms, &input_sdl);
    }

    app_manager_set_launcher_path(&app_manager, "build/apps/launcher/launcher.wasm");
    app_manager_add_app(&app_manager, "Circles", "build/apps/circles/circles.wasm");
    app_manager_add_app(&app_manager, "Mandelbrot", "build/apps/mandelbrot/mandelbrot.wasm");
    app_manager_add_app(&app_manager, "Test Drawing", "build/apps/test_drawing/test_drawing.wasm");
    app_manager_add_app(&app_manager, "Test UI", "build/apps/test_ui/test_ui.wasm");

    if (options.wasm_file) {
        if (!app_manager_launch_app_by_path(&app_manager, options.wasm_file)) {
            fprintf(stderr, "Failed to load %s: %s\n", options.wasm_file, app_manager_get_last_error(&app_manager));
            app_manager_deinit(&app_manager);
            display_sdl_deinit(&display);
            return 1;
        }
    } else {
        app_manager_show_launcher(&app_manager);
    }

    if (options.test_mode || options.screenshot_path) {
        if (options.test_scene >= 0) {
            wasm_runner_t* runner = app_manager_get_wasm_runner(&app_manager);
            if (runner) {
                wasm_runner_set_scene(runner, (uint32_t)options.test_scene);
            }
        }

        app_manager_render(&app_manager);

        if (options.screenshot_path) {
            if (!display_sdl_save_screenshot(&display, options.screenshot_path)) {
                app_manager_deinit(&app_manager);
                display_sdl_deinit(&display);
                return 1;
            }
            printf("Screenshot saved to %s\n", options.screenshot_path);
        }

        if (!options.headless) {
            display_sdl_flush(&display);
            SDL_Delay(100);
        }

        app_manager_deinit(&app_manager);
        display_sdl_deinit(&display);
        return 0;
    }

    input_handler_t input_handler;
    input_manager_t input_manager;

    input_sdl_bind_handler(&input_sdl, &input_handler);
    input_manager_init(&input_manager);
    input_manager_set_reset_callback(&input_manager, reset_combo_callback, &app_manager);

    int screenshot_num = 0;

    while (!display_sdl_should_quit(&display)) {
        uint32_t time_ms = input_handler_get_time_ms(&input_handler);

        input_manager_update(&input_manager, &input_handler, time_ms);

        if (input_sdl_was_screenshot_requested(&input_sdl)) {
            char path[64];
            snprintf(path, sizeof(path), "screenshot_%d.png", screenshot_num++);
            if (display_sdl_save_screenshot(&display, path)) {
                printf("Screenshot saved to %s\n", path);
            }
        }

        while (input_manager_has_event(&input_manager)) {
            input_event_t event = input_manager_get_event(&input_manager);
            app_manager_handle_input(&app_manager, event.key, event.type);
        }

        app_manager_render(&app_manager);
        display_sdl_flush(&display);

        SDL_Delay(16);
    }

    app_manager_deinit(&app_manager);
    display_sdl_deinit(&display);

    return 0;
}
