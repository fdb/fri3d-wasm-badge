#include "display_sdl.h"
#include "input_sdl.h"
#include "canvas.h"
#include "random.h"
#include "input.h"
#include "app_manager.h"

#include <iostream>
#include <cstring>
#include <string>

using namespace fri3d;

// Command line options
struct Options {
    std::string wasmFile;
    std::string screenshotPath;
    bool testMode = false;
    bool headless = false;
    int testScene = -1;
};

static void printUsage(const char* program) {
    std::cerr << "Usage: " << program << " [options] [wasm_file]\n\n";
    std::cerr << "Options:\n";
    std::cerr << "  --test              Run in test mode (render and exit)\n";
    std::cerr << "  --scene <n>         Set scene number (for test apps)\n";
    std::cerr << "  --screenshot <path> Save screenshot to path (PNG format)\n";
    std::cerr << "  --headless          Run without display (requires --screenshot)\n";
    std::cerr << "  --help              Show this help\n\n";
    std::cerr << "If no wasm_file is specified, shows the launcher with built-in apps.\n";
}

static bool parseArgs(int argc, char* argv[], Options& opts) {
    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "--test") == 0) {
            opts.testMode = true;
        } else if (strcmp(argv[i], "--scene") == 0 && i + 1 < argc) {
            opts.testScene = atoi(argv[++i]);
        } else if (strcmp(argv[i], "--screenshot") == 0 && i + 1 < argc) {
            opts.screenshotPath = argv[++i];
        } else if (strcmp(argv[i], "--headless") == 0) {
            opts.headless = true;
        } else if (strcmp(argv[i], "--help") == 0) {
            printUsage(argv[0]);
            return false;
        } else if (argv[i][0] != '-') {
            opts.wasmFile = argv[i];
        } else {
            std::cerr << "Unknown option: " << argv[i] << std::endl;
            printUsage(argv[0]);
            return false;
        }
    }

    if (opts.headless && opts.screenshotPath.empty()) {
        std::cerr << "Error: --headless requires --screenshot" << std::endl;
        return false;
    }

    return true;
}

int main(int argc, char* argv[]) {
    Options opts;
    if (!parseArgs(argc, argv, opts)) {
        return (argc > 1 && strcmp(argv[1], "--help") == 0) ? 0 : 1;
    }

    // Initialize display
    DisplaySDL display;
    if (!display.init(opts.headless)) {
        std::cerr << "Failed to initialize display" << std::endl;
        return 1;
    }

    // Initialize canvas and random
    Canvas canvas;
    canvas.init(display.getU8g2());

    Random random;

    // Initialize app manager
    AppManager appManager;
    if (!appManager.init(&canvas, &random)) {
        std::cerr << "Failed to initialize app manager: " << appManager.getLastError() << std::endl;
        return 1;
    }

    // Add built-in apps (relative to working directory)
    appManager.addApp("Circles", "build/apps/circles/circles.wasm");
    appManager.addApp("Mandelbrot", "build/apps/mandelbrot/mandelbrot.wasm");
    appManager.addApp("Test Drawing", "build/apps/test_drawing/test_drawing.wasm");

    // If a WASM file was specified, launch it directly
    if (!opts.wasmFile.empty()) {
        if (!appManager.launchAppByPath(opts.wasmFile)) {
            std::cerr << "Failed to load " << opts.wasmFile << ": " << appManager.getLastError() << std::endl;
            return 1;
        }
    }

    // Test mode: render one frame and optionally save screenshot
    if (opts.testMode || !opts.screenshotPath.empty()) {
        // Set scene if specified
        if (opts.testScene >= 0) {
            appManager.getWasmRunner().setScene(static_cast<uint32_t>(opts.testScene));
        }

        // Render one frame
        appManager.render();

        // Save screenshot if requested
        if (!opts.screenshotPath.empty()) {
            if (!display.saveScreenshot(opts.screenshotPath)) {
                return 1;
            }
            std::cout << "Screenshot saved to " << opts.screenshotPath << std::endl;
        }

        if (!opts.headless) {
            display.flush();
            SDL_Delay(100);  // Brief display before exit
        }

        return 0;
    }

    // Create input handler
    InputSDL inputHandler(display);
    InputManager inputManager;

    // Set reset combo callback
    inputManager.setResetCallback([&appManager]() {
        std::cout << "Reset combo triggered - returning to launcher" << std::endl;
        appManager.showLauncher();
    });

    // Main loop
    int screenshotNum = 0;

    while (!display.shouldQuit()) {
        uint32_t timeMs = inputHandler.getTimeMs();

        // Process input
        inputManager.update(inputHandler, timeMs);

        // Handle screenshot request
        if (inputHandler.wasScreenshotRequested()) {
            std::string path = "screenshot_" + std::to_string(screenshotNum++) + ".png";
            if (display.saveScreenshot(path)) {
                std::cout << "Screenshot saved to " << path << std::endl;
            }
        }

        // Handle processed input events
        while (inputManager.hasEvent()) {
            InputEvent event = inputManager.getEvent();
            appManager.handleInput(event.key, event.type);
        }

        // Render
        appManager.render();

        // Update display
        display.flush();

        // ~60 FPS
        SDL_Delay(16);
    }

    return 0;
}
