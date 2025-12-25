#include <SDL.h>
#include <wasm_export.h>
#include <u8g2.h>
#include <lodepng.h>
#include <iostream>
#include <vector>
#include <fstream>
#include <random>
#include <cstring>
#include <string>

// Screen dimensions
const int SCREEN_WIDTH = 128;
const int SCREEN_HEIGHT = 64;
const int SCALE_FACTOR = 4;

// SDL
SDL_Window* window = nullptr;
SDL_Renderer* renderer = nullptr;
SDL_Texture* texture = nullptr;

// u8g2 for drawing
static u8g2_t g_u8g2;

// Random number generator
static std::mt19937 g_rng(std::random_device{}());

// WASM module instance (needed for calling app functions)
static wasm_module_inst_t g_module_inst = nullptr;
static wasm_exec_env_t g_exec_env = nullptr;
static wasm_function_inst_t g_func_render = nullptr;
static wasm_function_inst_t g_func_on_input = nullptr;
static wasm_function_inst_t g_func_set_scene = nullptr;
static wasm_function_inst_t g_func_get_scene = nullptr;
static wasm_function_inst_t g_func_get_scene_count = nullptr;

// Test mode configuration
static bool g_test_mode = false;
static int g_test_scene = -1;  // -1 means all scenes
static std::string g_screenshot_path;
static bool g_headless = false;

// Dummy u8g2 callbacks (we only use the buffer)
static uint8_t u8x8_gpio_and_delay_callback(u8x8_t* u8x8, uint8_t msg, uint8_t arg_int, void* arg_ptr) {
    (void)u8x8; (void)msg; (void)arg_int; (void)arg_ptr;
    return 1;
}

static uint8_t u8x8_byte_callback(u8x8_t* u8x8, uint8_t msg, uint8_t arg_int, void* arg_ptr) {
    (void)u8x8; (void)msg; (void)arg_int; (void)arg_ptr;
    return 1;
}

// ============================================================================
// Canvas Native Functions
// ============================================================================

extern "C" {

void native_canvas_clear(wasm_exec_env_t exec_env) {
    (void)exec_env;
    u8g2_ClearBuffer(&g_u8g2);
}

uint32_t native_canvas_width(wasm_exec_env_t exec_env) {
    (void)exec_env;
    return SCREEN_WIDTH;
}

uint32_t native_canvas_height(wasm_exec_env_t exec_env) {
    (void)exec_env;
    return SCREEN_HEIGHT;
}

void native_canvas_set_color(wasm_exec_env_t exec_env, uint32_t color) {
    (void)exec_env;
    u8g2_SetDrawColor(&g_u8g2, (uint8_t)color);
}

void native_canvas_set_font(wasm_exec_env_t exec_env, uint32_t font) {
    (void)exec_env;
    switch (font) {
    case 0: // FontPrimary
        u8g2_SetFont(&g_u8g2, u8g2_font_6x10_tf);
        break;
    case 1: // FontSecondary
        u8g2_SetFont(&g_u8g2, u8g2_font_5x7_tf);
        break;
    case 2: // FontKeyboard
        u8g2_SetFont(&g_u8g2, u8g2_font_5x8_tf);
        break;
    case 3: // FontBigNumbers
        u8g2_SetFont(&g_u8g2, u8g2_font_10x20_tf);
        break;
    default:
        u8g2_SetFont(&g_u8g2, u8g2_font_6x10_tf);
        break;
    }
}

void native_canvas_draw_dot(wasm_exec_env_t exec_env, int32_t x, int32_t y) {
    (void)exec_env;
    u8g2_DrawPixel(&g_u8g2, (u8g2_uint_t)x, (u8g2_uint_t)y);
}

void native_canvas_draw_line(wasm_exec_env_t exec_env, int32_t x1, int32_t y1, int32_t x2, int32_t y2) {
    (void)exec_env;
    u8g2_DrawLine(&g_u8g2, (u8g2_uint_t)x1, (u8g2_uint_t)y1, (u8g2_uint_t)x2, (u8g2_uint_t)y2);
}

void native_canvas_draw_frame(wasm_exec_env_t exec_env, int32_t x, int32_t y, uint32_t w, uint32_t h) {
    (void)exec_env;
    u8g2_DrawFrame(&g_u8g2, (u8g2_uint_t)x, (u8g2_uint_t)y, (u8g2_uint_t)w, (u8g2_uint_t)h);
}

void native_canvas_draw_box(wasm_exec_env_t exec_env, int32_t x, int32_t y, uint32_t w, uint32_t h) {
    (void)exec_env;
    u8g2_DrawBox(&g_u8g2, (u8g2_uint_t)x, (u8g2_uint_t)y, (u8g2_uint_t)w, (u8g2_uint_t)h);
}

void native_canvas_draw_rframe(wasm_exec_env_t exec_env, int32_t x, int32_t y, uint32_t w, uint32_t h, uint32_t r) {
    (void)exec_env;
    u8g2_DrawRFrame(&g_u8g2, (u8g2_uint_t)x, (u8g2_uint_t)y, (u8g2_uint_t)w, (u8g2_uint_t)h, (u8g2_uint_t)r);
}

void native_canvas_draw_rbox(wasm_exec_env_t exec_env, int32_t x, int32_t y, uint32_t w, uint32_t h, uint32_t r) {
    (void)exec_env;
    u8g2_DrawRBox(&g_u8g2, (u8g2_uint_t)x, (u8g2_uint_t)y, (u8g2_uint_t)w, (u8g2_uint_t)h, (u8g2_uint_t)r);
}

void native_canvas_draw_circle(wasm_exec_env_t exec_env, int32_t x, int32_t y, uint32_t r) {
    (void)exec_env;
    u8g2_DrawCircle(&g_u8g2, (u8g2_uint_t)x, (u8g2_uint_t)y, (u8g2_uint_t)r, U8G2_DRAW_ALL);
}

void native_canvas_draw_disc(wasm_exec_env_t exec_env, int32_t x, int32_t y, uint32_t r) {
    (void)exec_env;
    u8g2_DrawDisc(&g_u8g2, (u8g2_uint_t)x, (u8g2_uint_t)y, (u8g2_uint_t)r, U8G2_DRAW_ALL);
}

void native_canvas_draw_str(wasm_exec_env_t exec_env, int32_t x, int32_t y, const char* str) {
    (void)exec_env;
    if (str == nullptr) {
        return;
    }
    u8g2_DrawUTF8(&g_u8g2, (u8g2_uint_t)x, (u8g2_uint_t)y, str);
}

uint32_t native_canvas_string_width(wasm_exec_env_t exec_env, const char* str) {
    (void)exec_env;
    if (str == nullptr) {
        return 0;
    }
    return u8g2_GetStrWidth(&g_u8g2, str);
}

// ============================================================================
// Random Native Functions
// ============================================================================

void native_random_seed(wasm_exec_env_t exec_env, uint32_t seed) {
    (void)exec_env;
    g_rng.seed(seed);
}

uint32_t native_random_get(wasm_exec_env_t exec_env) {
    (void)exec_env;
    return g_rng();
}

uint32_t native_random_range(wasm_exec_env_t exec_env, uint32_t max) {
    (void)exec_env;
    if (max == 0) return 0;
    return g_rng() % max;
}

} // extern "C"

// ============================================================================
// Native Symbols Table
// ============================================================================

static NativeSymbol native_symbols[] = {
    // Canvas
    { "canvas_clear", (void*)native_canvas_clear, "()", NULL },
    { "canvas_width", (void*)native_canvas_width, "()i", NULL },
    { "canvas_height", (void*)native_canvas_height, "()i", NULL },
    { "canvas_set_color", (void*)native_canvas_set_color, "(i)", NULL },
    { "canvas_set_font", (void*)native_canvas_set_font, "(i)", NULL },
    { "canvas_draw_dot", (void*)native_canvas_draw_dot, "(ii)", NULL },
    { "canvas_draw_line", (void*)native_canvas_draw_line, "(iiii)", NULL },
    { "canvas_draw_frame", (void*)native_canvas_draw_frame, "(iiii)", NULL },
    { "canvas_draw_box", (void*)native_canvas_draw_box, "(iiii)", NULL },
    { "canvas_draw_rframe", (void*)native_canvas_draw_rframe, "(iiiii)", NULL },
    { "canvas_draw_rbox", (void*)native_canvas_draw_rbox, "(iiiii)", NULL },
    { "canvas_draw_circle", (void*)native_canvas_draw_circle, "(iii)", NULL },
    { "canvas_draw_disc", (void*)native_canvas_draw_disc, "(iii)", NULL },
    { "canvas_draw_str", (void*)native_canvas_draw_str, "(ii*)", NULL },
    { "canvas_string_width", (void*)native_canvas_string_width, "(*)i", NULL },
    // Random
    { "random_seed", (void*)native_random_seed, "(i)", NULL },
    { "random_get", (void*)native_random_get, "()i", NULL },
    { "random_range", (void*)native_random_range, "(i)i", NULL },
};

// ============================================================================
// Screenshot Functions
// ============================================================================

static bool save_screenshot_png(const std::string& path) {
    uint8_t* buffer = u8g2_GetBufferPtr(&g_u8g2);

    // Create RGBA image data (lodepng expects RGBA by default)
    std::vector<unsigned char> image(SCREEN_WIDTH * SCREEN_HEIGHT * 4);

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

    unsigned error = lodepng::encode(path, image, SCREEN_WIDTH, SCREEN_HEIGHT);
    if (error) {
        std::cerr << "PNG encoder error: " << lodepng_error_text(error) << std::endl;
        return false;
    }

    return true;
}

// ============================================================================
// WASM Helper Functions
// ============================================================================

static uint32_t call_wasm_get_scene_count() {
    if (!g_func_get_scene_count) return 0;

    // In WAMR, return values are passed back through the argv array
    uint32_t argv[1] = { 0 };
    if (wasm_runtime_call_wasm(g_exec_env, g_func_get_scene_count, 0, argv)) {
        return argv[0];
    }
    return 0;
}

static void call_wasm_set_scene(uint32_t scene) {
    if (!g_func_set_scene) return;

    uint32_t args[1] = { scene };
    wasm_runtime_call_wasm(g_exec_env, g_func_set_scene, 1, args);
}

static void call_wasm_render() {
    if (!g_func_render) return;

    u8g2_ClearBuffer(&g_u8g2);
    wasm_runtime_call_wasm(g_exec_env, g_func_render, 0, NULL);

    if (wasm_runtime_get_exception(g_module_inst)) {
        std::cerr << "WASM Exception in render: " << wasm_runtime_get_exception(g_module_inst) << std::endl;
        wasm_runtime_clear_exception(g_module_inst);
    }
}

// ============================================================================
// Display Update
// ============================================================================

static void update_display() {
    if (g_headless) return;

    uint8_t* buffer = u8g2_GetBufferPtr(&g_u8g2);

    void* pixels;
    int pitch;
    SDL_LockTexture(texture, NULL, &pixels, &pitch);

    uint32_t* dst = (uint32_t*)pixels;

    // u8g2 SSD1306 buffer layout: each byte is 8 vertical pixels
    // Byte 0: pixels (0,0)-(0,7), Byte 1: pixels (1,0)-(1,7), etc.
    for (int y = 0; y < SCREEN_HEIGHT; y++) {
        for (int x = 0; x < SCREEN_WIDTH; x++) {
            int byte_idx = x + (y / 8) * SCREEN_WIDTH;
            int bit_idx = y % 8;
            bool pixel = (buffer[byte_idx] >> bit_idx) & 1;
            dst[y * SCREEN_WIDTH + x] = pixel ? 0xFFFFFFFF : 0x000000FF;
        }
    }

    SDL_UnlockTexture(texture);
    SDL_RenderCopy(renderer, texture, NULL, NULL);
    SDL_RenderPresent(renderer);
}

// ============================================================================
// Input Handling
// ============================================================================

static void call_on_input(uint32_t key, uint32_t type) {
    if (!g_func_on_input) return;

    uint32_t args[2] = { key, type };
    wasm_runtime_call_wasm(g_exec_env, g_func_on_input, 2, args);

    if (wasm_runtime_get_exception(g_module_inst)) {
        std::cerr << "WASM Exception in on_input: " << wasm_runtime_get_exception(g_module_inst) << std::endl;
        wasm_runtime_clear_exception(g_module_inst);
    }
}

static int key_to_input_key(SDL_Keycode key) {
    switch (key) {
        case SDLK_UP: return 0;      // InputKeyUp
        case SDLK_DOWN: return 1;    // InputKeyDown
        case SDLK_LEFT: return 2;    // InputKeyLeft
        case SDLK_RIGHT: return 3;   // InputKeyRight
        case SDLK_RETURN:
        case SDLK_z: return 4;       // InputKeyOk
        case SDLK_BACKSPACE:
        case SDLK_x: return 5;       // InputKeyBack
        default: return -1;
    }
}

// ============================================================================
// Usage
// ============================================================================

static void print_usage(const char* program) {
    std::cerr << "Usage: " << program << " [options] <wasm_file>\n\n";
    std::cerr << "Options:\n";
    std::cerr << "  --test              Run in test mode (render and exit)\n";
    std::cerr << "  --scene <n>         Set scene number (for test_drawing app)\n";
    std::cerr << "  --screenshot <path> Save screenshot to path (PNG format)\n";
    std::cerr << "  --headless          Run without display (requires --screenshot)\n";
    std::cerr << "  --help              Show this help\n";
}

// ============================================================================
// Main
// ============================================================================

int main(int argc, char* argv[]) {
    std::string wasm_file;

    // Parse command line arguments
    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "--test") == 0) {
            g_test_mode = true;
        } else if (strcmp(argv[i], "--scene") == 0 && i + 1 < argc) {
            g_test_scene = atoi(argv[++i]);
        } else if (strcmp(argv[i], "--screenshot") == 0 && i + 1 < argc) {
            g_screenshot_path = argv[++i];
        } else if (strcmp(argv[i], "--headless") == 0) {
            g_headless = true;
        } else if (strcmp(argv[i], "--help") == 0) {
            print_usage(argv[0]);
            return 0;
        } else if (argv[i][0] != '-') {
            wasm_file = argv[i];
        } else {
            std::cerr << "Unknown option: " << argv[i] << std::endl;
            print_usage(argv[0]);
            return 1;
        }
    }

    if (wasm_file.empty()) {
        std::cerr << "Error: No WASM file specified" << std::endl;
        print_usage(argv[0]);
        return 1;
    }

    if (g_headless && g_screenshot_path.empty()) {
        std::cerr << "Error: --headless requires --screenshot" << std::endl;
        return 1;
    }

    // Initialize SDL (skip video init if headless)
    if (!g_headless) {
        if (SDL_Init(SDL_INIT_VIDEO) < 0) {
            std::cerr << "SDL could not initialize! SDL_Error: " << SDL_GetError() << std::endl;
            return 1;
        }

        window = SDL_CreateWindow("FRI3D Emulator", SDL_WINDOWPOS_UNDEFINED, SDL_WINDOWPOS_UNDEFINED,
                                  SCREEN_WIDTH * SCALE_FACTOR, SCREEN_HEIGHT * SCALE_FACTOR, SDL_WINDOW_SHOWN);
        if (!window) return 1;

        renderer = SDL_CreateRenderer(window, -1, SDL_RENDERER_ACCELERATED);
        texture = SDL_CreateTexture(renderer, SDL_PIXELFORMAT_RGBA8888, SDL_TEXTUREACCESS_STREAMING, SCREEN_WIDTH, SCREEN_HEIGHT);
    }

    // Initialize u8g2
    u8g2_Setup_ssd1306_128x64_noname_f(&g_u8g2, U8G2_R0, u8x8_byte_callback, u8x8_gpio_and_delay_callback);
    u8g2_InitDisplay(&g_u8g2);
    u8g2_SetPowerSave(&g_u8g2, 0);
    u8g2_ClearBuffer(&g_u8g2);

    // Initialize WAMR
    RuntimeInitArgs init_args;
    memset(&init_args, 0, sizeof(RuntimeInitArgs));
    init_args.mem_alloc_type = Alloc_With_Pool;
    init_args.mem_alloc_option.pool.heap_buf = malloc(10 * 1024 * 1024);
    init_args.mem_alloc_option.pool.heap_size = 10 * 1024 * 1024;

    if (!wasm_runtime_full_init(&init_args)) {
        std::cerr << "Init runtime environment failed." << std::endl;
        return 1;
    }

    // Register native functions
    wasm_runtime_register_natives("env", native_symbols, sizeof(native_symbols)/sizeof(NativeSymbol));

    // Load WASM file
    std::ifstream file(wasm_file, std::ios::binary | std::ios::ate);
    if (!file.is_open()) {
        std::cerr << "Failed to open " << wasm_file << std::endl;
        return 1;
    }
    std::streamsize size = file.tellg();
    file.seekg(0, std::ios::beg);
    std::vector<char> buffer(size);
    if (!file.read(buffer.data(), size)) return 1;

    char error_buf[128];
    wasm_module_t module = wasm_runtime_load((uint8_t*)buffer.data(), size, error_buf, sizeof(error_buf));
    if (!module) {
        std::cerr << "Load wasm module failed. error: " << error_buf << std::endl;
        return 1;
    }

    g_module_inst = wasm_runtime_instantiate(module, 16 * 1024, 16 * 1024, error_buf, sizeof(error_buf));
    if (!g_module_inst) {
        std::cerr << "Instantiate wasm module failed. error: " << error_buf << std::endl;
        return 1;
    }

    g_exec_env = wasm_runtime_create_exec_env(g_module_inst, 16 * 1024);

    // Look up app functions
    g_func_render = wasm_runtime_lookup_function(g_module_inst, "render");
    g_func_on_input = wasm_runtime_lookup_function(g_module_inst, "on_input");
    g_func_set_scene = wasm_runtime_lookup_function(g_module_inst, "set_scene");
    g_func_get_scene = wasm_runtime_lookup_function(g_module_inst, "get_scene");
    g_func_get_scene_count = wasm_runtime_lookup_function(g_module_inst, "get_scene_count");

    if (!g_func_render) {
        std::cerr << "Could not find 'render' function in WASM" << std::endl;
        return 1;
    }

    // Test mode: render one frame and optionally save screenshot
    if (g_test_mode || !g_screenshot_path.empty()) {
        if (g_test_scene >= 0 && g_func_set_scene) {
            call_wasm_set_scene((uint32_t)g_test_scene);
        }

        call_wasm_render();

        if (!g_screenshot_path.empty()) {
            if (!save_screenshot_png(g_screenshot_path)) {
                return 1;
            }
            std::cout << "Screenshot saved to " << g_screenshot_path << std::endl;
        }

        if (!g_headless) {
            update_display();
            SDL_Delay(100);  // Brief display before exit
        }

        // Cleanup and exit
        wasm_runtime_destroy_exec_env(g_exec_env);
        wasm_runtime_deinstantiate(g_module_inst);
        wasm_runtime_unload(module);
        wasm_runtime_destroy();
        if (!g_headless) {
            SDL_DestroyTexture(texture);
            SDL_DestroyRenderer(renderer);
            SDL_DestroyWindow(window);
            SDL_Quit();
        }
        return 0;
    }

    // Main loop (interactive mode)
    bool quit = false;
    SDL_Event e;

    while (!quit) {
        // Process events
        while (SDL_PollEvent(&e) != 0) {
            if (e.type == SDL_QUIT) {
                quit = true;
            } else if (e.type == SDL_KEYDOWN || e.type == SDL_KEYUP) {
                // S key for screenshot
                if (e.type == SDL_KEYDOWN && e.key.keysym.sym == SDLK_s) {
                    static int screenshot_num = 0;
                    std::string path = "screenshot_" + std::to_string(screenshot_num++) + ".png";
                    if (save_screenshot_png(path)) {
                        std::cout << "Screenshot saved to " << path << std::endl;
                    }
                    continue;
                }

                int input_key = key_to_input_key(e.key.keysym.sym);
                if (input_key >= 0) {
                    uint32_t input_type = (e.type == SDL_KEYDOWN) ? 0 : 1; // Press = 0, Release = 1
                    call_on_input((uint32_t)input_key, input_type);
                }
            }
        }

        // Clear canvas and call render
        call_wasm_render();

        // Update display
        update_display();

        // ~60 FPS
        SDL_Delay(16);
    }

    // Cleanup
    wasm_runtime_destroy_exec_env(g_exec_env);
    wasm_runtime_deinstantiate(g_module_inst);
    wasm_runtime_unload(module);
    wasm_runtime_destroy();
    SDL_DestroyTexture(texture);
    SDL_DestroyRenderer(renderer);
    SDL_DestroyWindow(window);
    SDL_Quit();

    return 0;
}
