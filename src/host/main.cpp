#include <SDL.h>
#include <wasm_export.h>
#include <u8g2.h>
#include <iostream>
#include <vector>
#include <fstream>
#include <random>

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

void native_canvas_draw_str(wasm_exec_env_t exec_env, int32_t x, int32_t y, uint32_t str_offset) {
    wasm_module_inst_t module_inst = wasm_runtime_get_module_inst(exec_env);
    if (!wasm_runtime_validate_app_str_addr(module_inst, str_offset)) {
        return;
    }
    const char* str = (const char*)wasm_runtime_addr_app_to_native(module_inst, str_offset);
    u8g2_DrawUTF8(&g_u8g2, (u8g2_uint_t)x, (u8g2_uint_t)y, str);
}

uint32_t native_canvas_string_width(wasm_exec_env_t exec_env, uint32_t str_offset) {
    wasm_module_inst_t module_inst = wasm_runtime_get_module_inst(exec_env);
    if (!wasm_runtime_validate_app_str_addr(module_inst, str_offset)) {
        return 0;
    }
    const char* str = (const char*)wasm_runtime_addr_app_to_native(module_inst, str_offset);
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
// Display Update
// ============================================================================

static void update_display() {
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
// Main
// ============================================================================

int main(int argc, char* argv[]) {
    if (argc < 2) {
        std::cerr << "Usage: " << argv[0] << " <wasm_file>" << std::endl;
        return 1;
    }

    // Initialize SDL
    if (SDL_Init(SDL_INIT_VIDEO) < 0) {
        std::cerr << "SDL could not initialize! SDL_Error: " << SDL_GetError() << std::endl;
        return 1;
    }

    window = SDL_CreateWindow("FRI3D Emulator", SDL_WINDOWPOS_UNDEFINED, SDL_WINDOWPOS_UNDEFINED,
                              SCREEN_WIDTH * SCALE_FACTOR, SCREEN_HEIGHT * SCALE_FACTOR, SDL_WINDOW_SHOWN);
    if (!window) return 1;

    renderer = SDL_CreateRenderer(window, -1, SDL_RENDERER_ACCELERATED);
    texture = SDL_CreateTexture(renderer, SDL_PIXELFORMAT_RGBA8888, SDL_TEXTUREACCESS_STREAMING, SCREEN_WIDTH, SCREEN_HEIGHT);

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
    std::ifstream file(argv[1], std::ios::binary | std::ios::ate);
    if (!file.is_open()) {
        std::cerr << "Failed to open " << argv[1] << std::endl;
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

    if (!g_func_render) {
        std::cerr << "Could not find 'render' function in WASM" << std::endl;
        return 1;
    }

    // Main loop
    bool quit = false;
    SDL_Event e;

    while (!quit) {
        // Process events
        while (SDL_PollEvent(&e) != 0) {
            if (e.type == SDL_QUIT) {
                quit = true;
            } else if (e.type == SDL_KEYDOWN || e.type == SDL_KEYUP) {
                int input_key = key_to_input_key(e.key.keysym.sym);
                if (input_key >= 0) {
                    uint32_t input_type = (e.type == SDL_KEYDOWN) ? 0 : 1; // Press = 0, Release = 1
                    call_on_input((uint32_t)input_key, input_type);
                }
            }
        }

        // Clear canvas and call render
        u8g2_ClearBuffer(&g_u8g2);

        if (g_func_render) {
            wasm_runtime_call_wasm(g_exec_env, g_func_render, 0, NULL);
            if (wasm_runtime_get_exception(g_module_inst)) {
                std::cerr << "WASM Exception in render: " << wasm_runtime_get_exception(g_module_inst) << std::endl;
                quit = true;
            }
        }

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
