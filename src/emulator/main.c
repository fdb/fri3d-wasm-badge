#include "display_sdl.h"
#include "input_sdl.h"
#include "canvas.h"
#include "random.h"
#include "input.h"
#include "app_manager.h"

#include <stdio.h>
#include <string.h>
#include <stdlib.h>

// Command line options
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
    fprintf(stderr, "If no wasm_file is specified, shows the launcher with built-in apps.\n");
}

static bool parse_args(int argc, char* argv[], options_t* opts) {
    memset(opts, 0, sizeof(options_t));
    opts->test_scene = -1;

    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "--test") == 0) {
            opts->test_mode = true;
        } else if (strcmp(argv[i], "--scene") == 0 && i + 1 < argc) {
            opts->test_scene = atoi(argv[++i]);
        } else if (strcmp(argv[i], "--screenshot") == 0 && i + 1 < argc) {
            opts->screenshot_path = argv[++i];
        } else if (strcmp(argv[i], "--headless") == 0) {
            opts->headless = true;
        } else if (strcmp(argv[i], "--help") == 0) {
            print_usage(argv[0]);
            return false;
        } else if (argv[i][0] != '-') {
            opts->wasm_file = argv[i];
        } else {
            fprintf(stderr, "Unknown option: %s\n", argv[i]);
            print_usage(argv[0]);
            return false;
        }
    }

    if (opts->headless && !opts->screenshot_path) {
        fprintf(stderr, "Error: --headless requires --screenshot\n");
        return false;
    }

    return true;
}

// Reset callback context
typedef struct {
    app_manager_t* app_manager;
} reset_context_t;

static void reset_callback(void* context) {
    reset_context_t* ctx = (reset_context_t*)context;
    printf("Reset combo triggered - returning to launcher\n");
    app_manager_show_launcher(ctx->app_manager);
}

int main(int argc, char* argv[]) {
    options_t opts;
    if (!parse_args(argc, argv, &opts)) {
        return (argc > 1 && strcmp(argv[1], "--help") == 0) ? 0 : 1;
    }

    // Initialize display
    display_sdl_t display;
    display_sdl_init(&display);
    if (!display_sdl_start(&display, opts.headless)) {
        fprintf(stderr, "Failed to initialize display\n");
        return 1;
    }

    // Initialize canvas and random
    canvas_t canvas;
    canvas_init(&canvas, display_sdl_get_u8g2(&display));

    random_t random;
    random_init(&random);

    // Initialize app manager
    app_manager_t app_manager;
    app_manager_init(&app_manager);
    if (!app_manager_start(&app_manager, &canvas, &random)) {
        fprintf(stderr, "Failed to initialize app manager: %s\n", app_manager_get_last_error(&app_manager));
        display_sdl_cleanup(&display);
        return 1;
    }

    // Add built-in apps (relative to working directory)
    app_manager_add_app(&app_manager, "Circles", "build/apps/circles/circles.wasm");
    app_manager_add_app(&app_manager, "Mandelbrot", "build/apps/mandelbrot/mandelbrot.wasm");
    app_manager_add_app(&app_manager, "Test Drawing", "build/apps/test_drawing/test_drawing.wasm");
    app_manager_add_app(&app_manager, "Test UI", "build/apps/test_ui/test_ui.wasm");

    // If a WASM file was specified, launch it directly
    if (opts.wasm_file) {
        if (!app_manager_launch_app_by_path(&app_manager, opts.wasm_file)) {
            fprintf(stderr, "Failed to load %s: %s\n", opts.wasm_file, app_manager_get_last_error(&app_manager));
            app_manager_cleanup(&app_manager);
            display_sdl_cleanup(&display);
            return 1;
        }
    }

    // Test mode: render one frame and optionally save screenshot
    if (opts.test_mode || opts.screenshot_path) {
        // Set scene if specified
        if (opts.test_scene >= 0) {
            wasm_runner_set_scene(app_manager_get_wasm_runner(&app_manager), (uint32_t)opts.test_scene);
        }

        // Render one frame
        app_manager_render(&app_manager);

        // Save screenshot if requested
        if (opts.screenshot_path) {
            if (!display_sdl_save_screenshot(&display, opts.screenshot_path)) {
                app_manager_cleanup(&app_manager);
                display_sdl_cleanup(&display);
                return 1;
            }
            printf("Screenshot saved to %s\n", opts.screenshot_path);
        }

        if (!opts.headless) {
            display_sdl_flush(&display);
            SDL_Delay(100);  // Brief display before exit
        }

        app_manager_cleanup(&app_manager);
        display_sdl_cleanup(&display);
        return 0;
    }

    // Create input handler
    input_sdl_t input_handler;
    input_sdl_init(&input_handler, &display);

    input_manager_t input_manager;
    input_manager_init(&input_manager);

    // Set reset combo callback
    reset_context_t reset_ctx = { &app_manager };
    input_manager_set_reset_callback(&input_manager, reset_callback, &reset_ctx);

    // Main loop
    int screenshot_num = 0;

    while (!display_sdl_should_quit(&display)) {
        uint32_t time_ms = input_sdl_get_time_ms(&input_handler);

        // Process input
        input_manager_update(&input_manager, input_sdl_get_handler(&input_handler), time_ms);

        // Handle screenshot request
        if (input_sdl_was_screenshot_requested(&input_handler)) {
            char path[64];
            snprintf(path, sizeof(path), "screenshot_%d.png", screenshot_num++);
            if (display_sdl_save_screenshot(&display, path)) {
                printf("Screenshot saved to %s\n", path);
            }
        }

        // Handle processed input events
        while (input_manager_has_event(&input_manager)) {
            input_event_t event = input_manager_get_event(&input_manager);
            app_manager_handle_input(&app_manager, event.key, event.type);
        }

        // Render
        app_manager_render(&app_manager);

        // Update display
        display_sdl_flush(&display);

        // ~60 FPS
        SDL_Delay(16);
    }

    app_manager_cleanup(&app_manager);
    display_sdl_cleanup(&display);
    return 0;
}
