#include <SDL.h>
#include <wasm_export.h>
#include <iostream>
#include <vector>
#include <fstream>
#include <thread>
#include <chrono>

// Screen dimensions
const int SCREEN_WIDTH = 128;
const int SCREEN_HEIGHT = 64;
const int SCALE_FACTOR = 4;

SDL_Window* window = nullptr;
SDL_Renderer* renderer = nullptr;
SDL_Texture* texture = nullptr;

// Input state
uint32_t current_input = 0;

// Host Native Functions
void native_lcd_update(wasm_exec_env_t exec_env, uint32_t buffer_offset, uint32_t size) {
    wasm_module_inst_t module_inst = wasm_runtime_get_module_inst(exec_env);
    if (!wasm_runtime_validate_app_addr(module_inst, buffer_offset, size)) {
        printf("Invalid access to LCD buffer!\n");
        return;
    }

    uint8_t* buffer = (uint8_t*)wasm_runtime_addr_app_to_native(module_inst, buffer_offset);
    
    // Convert 1-bit buffer (u8g2 default usually) to RGBA for SDL
    // Assuming 128x64 bits = 1024 bytes.
    // However, u8g2 buffer format depends on the driver. we will assume a linear buffer for now or handle it later.
    // For simplicity, let's assume the app sends a full byte-per-pixel buffer or we convert it.
    // Actually, to support u8g2 easily, we might want to make the app render to a simple linear buffer.
    
    // For this prototype, let's just copy it to the texture assuming it's RGBA or similar?
    // No, u8g2 on embedded is usually monochrome. 
    // Let's assume the WASM app sends 1 byte per pixel for now (grayscale) or just packed bits?
    // Let's implement a simple unpacking if needed, or ask the app to send 8-bit.
    // Update: User said "raw PCB" look, probably monochrome.
    
    // Let's assume input is 1 byte per 8 pixels (standard monochrome).
    // Width 128, Height 64. 
    // Rows = 64. Bytes per row = 128/8 = 16. Total = 1024 bytes.
    
    // NOTE: u8g2 buffer layout depends on the device. proper page addressing etc.
    // Let's assume the app sends us a linear Framebuffer for now.
    
    void* pixels;
    int pitch;
    SDL_LockTexture(texture, NULL, &pixels, &pitch);
    
    // Simple 1-bit to RGB32 expansion
    // Assuming vertical paging (typical SSD1306) or linear?
    // Let's start with assuming linear byte-per-pixel for simplicity in the first pass
    // OR we can make the WASM app send R,G,B buffer.
    
    // Let's just blindly copy for now if size matches, otherwise fill with dummy.
    // Just to get it running: Assume 1 byte per pixel for now (easier debugging)
    
    if (size == SCREEN_WIDTH * SCREEN_HEIGHT) {
        uint32_t* dst = (uint32_t*)pixels;
        for (int i = 0; i < size; i++) {
            uint8_t val = buffer[i];
            dst[i] = (val > 0) ? 0xFFFFFFFF : 0x000000FF; // White or Black
        }
    } else {
        // Fallback for monochrome packed if size is small (1024 bytes)
        // Assume linear mapping 1 bit = 1 pixel
        if (size == (SCREEN_WIDTH * SCREEN_HEIGHT) / 8) {
            uint32_t* dst = (uint32_t*)pixels;
            for (int i = 0; i < SCREEN_WIDTH * SCREEN_HEIGHT; i++) {
                int byte_idx = i / 8;
                int bit_idx = i % 8;
                bool pixel = (buffer[byte_idx] >> bit_idx) & 1;
                 dst[i] = pixel ? 0xFFFFFFFF : 0x000000FF;
            }
        }
    }
    
    SDL_UnlockTexture(texture);
    SDL_RenderCopy(renderer, texture, NULL, NULL);
    SDL_RenderPresent(renderer);
}

uint32_t native_get_input(wasm_exec_env_t exec_env) {
    return current_input;
}

void native_delay(wasm_exec_env_t exec_env, uint32_t ms) {
    SDL_Delay(ms);
}

// Native symbols to export to WASM
static NativeSymbol native_symbols[] = {
    { "host_lcd_update", (void*)native_lcd_update, "(ii)", NULL },
    { "host_get_input", (void*)native_get_input, "()i", NULL },
    { "host_delay", (void*)native_delay, "(i)", NULL }
};

int main(int argc, char* argv[]) {
    if (argc < 2) {
        std::cerr << "Usage: " << argv[0] << " <wasm_file>" << std::endl;
        return 1;
    }

    if (SDL_Init(SDL_INIT_VIDEO) < 0) {
        std::cerr << "SDL could not initialize! SDL_Error: " << SDL_GetError() << std::endl;
        return 1;
    }

    window = SDL_CreateWindow("FRI3D Emulator", SDL_WINDOWPOS_UNDEFINED, SDL_WINDOWPOS_UNDEFINED, 
                              SCREEN_WIDTH * SCALE_FACTOR, SCREEN_HEIGHT * SCALE_FACTOR, SDL_WINDOW_SHOWN);
    if (!window) return 1;

    renderer = SDL_CreateRenderer(window, -1, SDL_RENDERER_ACCELERATED);
    texture = SDL_CreateTexture(renderer, SDL_PIXELFORMAT_RGBA8888, SDL_TEXTUREACCESS_STREAMING, SCREEN_WIDTH, SCREEN_HEIGHT);

    // WAMR Initialization
    RuntimeInitArgs init_args;
    memset(&init_args, 0, sizeof(RuntimeInitArgs));
    init_args.mem_alloc_type = Alloc_With_Pool;
    init_args.mem_alloc_option.pool.heap_buf = malloc(10 * 1024 * 1024);
    init_args.mem_alloc_option.pool.heap_size = 10 * 1024 * 1024;

    if (!wasm_runtime_full_init(&init_args)) {
        std::cerr << "Init runtime environment failed." << std::endl;
        return 1;
    }
    
    // Register natives
    wasm_runtime_register_natives("env", native_symbols, sizeof(native_symbols)/sizeof(NativeSymbol));

    // Load WASM
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

    wasm_module_inst_t module_inst = wasm_runtime_instantiate(module, 16 * 1024, 16 * 1024, error_buf, sizeof(error_buf));
    if (!module_inst) {
        std::cerr << "Instantiate wasm module failed. error: " << error_buf << std::endl;
        return 1;
    }

    // Main loop
    bool quit = false;
    SDL_Event e;
    
    // Look for on_tick function
    wasm_function_inst_t func_on_tick = wasm_runtime_lookup_function(module_inst, "on_tick"); // signature not needed anymore
    if (!func_on_tick) {
        std::cerr << "Could not find on_tick function in WASM" << std::endl;
        // Try looking for main?
    }

    wasm_exec_env_t exec_env = wasm_runtime_create_exec_env(module_inst, 16 * 1024);

    while (!quit) {
        while (SDL_PollEvent(&e) != 0) {
            if (e.type == SDL_QUIT) quit = true;
            else if (e.type == SDL_KEYDOWN || e.type == SDL_KEYUP) {
                // Map keys to bitmask
                // Bit 0: Up, 1: Down, 2: Left, 3: Right, 4: A (Enter/Z), 5: B (Backspace/X)
                uint32_t mask = 0;
                switch (e.key.keysym.sym) {
                    case SDLK_UP: mask = 1 << 0; break;
                    case SDLK_DOWN: mask = 1 << 1; break;
                    case SDLK_LEFT: mask = 1 << 2; break;
                    case SDLK_RIGHT: mask = 1 << 3; break;
                    case SDLK_RETURN: case SDLK_z: mask = 1 << 4; break;
                    case SDLK_BACKSPACE: case SDLK_x: mask = 1 << 5; break;
                }
                
                if (e.type == SDL_KEYDOWN) current_input |= mask;
                else current_input &= ~mask;
            }
        }

        if (func_on_tick) {
            wasm_runtime_call_wasm(exec_env, func_on_tick, 0, NULL);
            // Check for exception
             if (wasm_runtime_get_exception(module_inst)) {
                std::cerr << "WASM Exception: " << wasm_runtime_get_exception(module_inst) << std::endl;
                quit = true;
            }
        } else {
             SDL_Delay(16); // Idle if no app
        }
    }

    wasm_runtime_destroy_exec_env(exec_env);
    wasm_runtime_deinstantiate(module_inst);
    wasm_runtime_unload(module);
    wasm_runtime_destroy();
    SDL_DestroyTexture(texture);
    SDL_DestroyRenderer(renderer);
    SDL_DestroyWindow(window);
    SDL_Quit();

    return 0;
}
