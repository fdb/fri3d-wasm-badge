/**
 * Fri3d Web Emulator
 * Emscripten entry point - runs the emulator in a browser
 */

#include "display_sdl.h"
#include "input_sdl.h"
#include "canvas.h"
#include "random.h"
#include "input.h"
#include "app_manager.h"

#include <iostream>
#include <string>

#ifdef __EMSCRIPTEN__
#include <emscripten.h>
#include <emscripten/html5.h>
#endif

using namespace fri3d;

// Global state for main loop
static DisplaySDL* g_display = nullptr;
static Canvas* g_canvas = nullptr;
static Random* g_random = nullptr;
static AppManager* g_appManager = nullptr;
static InputSDL* g_inputHandler = nullptr;
static InputManager* g_inputManager = nullptr;

static void mainLoop() {
    if (!g_display || !g_appManager || !g_inputHandler || !g_inputManager) {
        return;
    }

    if (g_display->shouldQuit()) {
#ifdef __EMSCRIPTEN__
        emscripten_cancel_main_loop();
#endif
        return;
    }

    uint32_t timeMs = g_inputHandler->getTimeMs();

    // Process input
    g_inputManager->update(*g_inputHandler, timeMs);

    // Handle processed input events
    while (g_inputManager->hasEvent()) {
        InputEvent event = g_inputManager->getEvent();
        g_appManager->handleInput(event.key, event.type);
    }

    // Render
    g_appManager->render();

    // Update display
    g_display->flush();
}

int main(int argc, char* argv[]) {
    (void)argc;
    (void)argv;

    std::cout << "Fri3d Web Emulator starting..." << std::endl;

    // Initialize display
    g_display = new DisplaySDL();
    if (!g_display->init(false)) {
        std::cerr << "Failed to initialize display" << std::endl;
        return 1;
    }

    // Initialize canvas and random
    g_canvas = new Canvas();
    g_canvas->init(g_display->getU8g2());

    g_random = new Random();

    // Initialize app manager
    g_appManager = new AppManager();
    if (!g_appManager->init(g_canvas, g_random)) {
        std::cerr << "Failed to initialize app manager: " << g_appManager->getLastError() << std::endl;
        return 1;
    }

    // Add built-in apps (paths relative to Emscripten virtual filesystem)
    g_appManager->addApp("Circles", "/apps/circles/circles.wasm");
    g_appManager->addApp("Mandelbrot", "/apps/mandelbrot/mandelbrot.wasm");
    g_appManager->addApp("Test Drawing", "/apps/test_drawing/test_drawing.wasm");

    // Create input handler
    g_inputHandler = new InputSDL(*g_display);
    g_inputManager = new InputManager();

    // Set reset combo callback
    g_inputManager->setResetCallback([]() {
        std::cout << "Reset combo triggered - returning to launcher" << std::endl;
        g_appManager->showLauncher();
    });

    std::cout << "Fri3d Web Emulator ready!" << std::endl;

#ifdef __EMSCRIPTEN__
    // Use Emscripten's main loop (60 FPS, simulate infinite loop)
    emscripten_set_main_loop(mainLoop, 60, 1);
#else
    // Fallback for non-Emscripten builds (shouldn't happen)
    while (!g_display->shouldQuit()) {
        mainLoop();
        SDL_Delay(16);
    }
#endif

    // Cleanup (may not be reached in Emscripten)
    delete g_inputManager;
    delete g_inputHandler;
    delete g_appManager;
    delete g_random;
    delete g_canvas;
    delete g_display;

    return 0;
}
