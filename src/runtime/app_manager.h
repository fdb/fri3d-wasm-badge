#pragma once

#include "canvas.h"
#include "input.h"
#include "wasm_runner.h"
#include <string>
#include <vector>

namespace fri3d {

/**
 * Application entry for the launcher.
 */
struct AppEntry {
    std::string name;      // Display name
    std::string path;      // Path to .wasm file
};

/**
 * App manager with built-in launcher.
 * Handles app discovery, launching, and switching.
 */
class AppManager {
public:
    AppManager();
    ~AppManager();

    /**
     * Initialize the app manager.
     * @param canvas Canvas for drawing
     * @param random Random number generator
     * @return true on success
     */
    bool init(Canvas* canvas, Random* random);

    /**
     * Add an app to the list.
     */
    void addApp(const std::string& name, const std::string& path);

    /**
     * Clear all apps from the list.
     */
    void clearApps();

    /**
     * Get the number of apps.
     */
    size_t appCount() const { return m_apps.size(); }

    /**
     * Show the launcher UI.
     * Returns to launcher from any running app.
     */
    void showLauncher();

    /**
     * Launch an app by index.
     * @param index App index in the list
     * @return true if launched successfully
     */
    bool launchApp(size_t index);

    /**
     * Launch an app by path.
     * @param path Path to .wasm file
     * @return true if launched successfully
     */
    bool launchAppByPath(const std::string& path);

    /**
     * Check if currently in launcher mode.
     */
    bool isInLauncher() const { return m_inLauncher; }

    /**
     * Check if an app is currently running.
     */
    bool isAppRunning() const { return !m_inLauncher && m_wasmRunner.isModuleLoaded(); }

    /**
     * Render the current view (launcher or app).
     */
    void render();

    /**
     * Handle input event.
     * @param key Input key
     * @param type Input type (use raw Press/Release for apps)
     */
    void handleInput(InputKey key, InputType type);

    /**
     * Get the last error message.
     */
    const std::string& getLastError() const { return m_lastError; }

    /**
     * Get the WASM runner (for test mode access).
     */
    WasmRunner& getWasmRunner() { return m_wasmRunner; }

private:
    Canvas* m_canvas = nullptr;
    Random* m_random = nullptr;
    WasmRunner m_wasmRunner;

    std::vector<AppEntry> m_apps;
    size_t m_selectedIndex = 0;
    size_t m_scrollOffset = 0;
    bool m_inLauncher = true;

    std::string m_lastError;

    // Launcher UI constants
    static constexpr int VISIBLE_ITEMS = 4;
    static constexpr int ITEM_HEIGHT = 14;
    static constexpr int START_Y = 12;
    static constexpr int TEXT_X = 16;
    static constexpr int CIRCLE_X = 6;
    static constexpr int CIRCLE_RADIUS = 3;

    void renderLauncher();
    void launcherInput(InputKey key, InputType type);
};

} // namespace fri3d
