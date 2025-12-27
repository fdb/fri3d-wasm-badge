#pragma once

#include <wasm_export.h>
#include <cstdint>
#include <string>
#include <vector>

namespace fri3d {

// Forward declarations
class Canvas;
class Random;

/**
 * WASM module runner using WAMR (WebAssembly Micro Runtime).
 * Handles module loading, native function registration, and execution.
 */
class WasmRunner {
public:
    WasmRunner();
    ~WasmRunner();

    /**
     * Initialize the WAMR runtime.
     * Must be called once before loading any modules.
     * @param heapSize Size of the WAMR heap in bytes
     * @return true on success
     */
    bool init(size_t heapSize = 10 * 1024 * 1024);

    /**
     * Set the canvas and random instances for native functions.
     * Must be called before loading a module.
     */
    void setCanvas(Canvas* canvas);
    void setRandom(Random* random);

    /**
     * Load and instantiate a WASM module from file.
     * Unloads any previously loaded module.
     * @param path Path to the .wasm file
     * @return true on success
     */
    bool loadModule(const std::string& path);

    /**
     * Load and instantiate a WASM module from memory.
     * Unloads any previously loaded module.
     * @param data WASM binary data
     * @param size Size of the data
     * @return true on success
     */
    bool loadModuleFromMemory(const uint8_t* data, size_t size);

    /**
     * Unload the current module.
     */
    void unloadModule();

    /**
     * Check if a module is currently loaded.
     */
    bool isModuleLoaded() const { return m_moduleInst != nullptr; }

    /**
     * Call the module's render() function.
     */
    void callRender();

    /**
     * Call the module's on_input(key, type) function.
     */
    void callOnInput(uint32_t key, uint32_t type);

    /**
     * Get the number of scenes (for test apps).
     */
    uint32_t getSceneCount();

    /**
     * Set the current scene.
     */
    void setScene(uint32_t scene);

    /**
     * Get the current scene.
     */
    uint32_t getScene();

    /**
     * Check if render function exists.
     */
    bool hasRenderFunction() const { return m_funcRender != nullptr; }

    /**
     * Check if on_input function exists.
     */
    bool hasOnInputFunction() const { return m_funcOnInput != nullptr; }

    /**
     * Get the last error message.
     */
    const std::string& getLastError() const { return m_lastError; }

private:
    // WAMR state
    uint8_t* m_heapBuffer = nullptr;
    wasm_module_t m_module = nullptr;
    wasm_module_inst_t m_moduleInst = nullptr;
    wasm_exec_env_t m_execEnv = nullptr;

    // Function handles
    wasm_function_inst_t m_funcRender = nullptr;
    wasm_function_inst_t m_funcOnInput = nullptr;
    wasm_function_inst_t m_funcSetScene = nullptr;
    wasm_function_inst_t m_funcGetScene = nullptr;
    wasm_function_inst_t m_funcGetSceneCount = nullptr;

    // Canvas and Random for native functions (not owned)
    Canvas* m_canvas = nullptr;
    Random* m_random = nullptr;

    // Error handling
    std::string m_lastError;
    char m_errorBuffer[256];

    bool m_initialized = false;

    void registerNativeFunctions();
    void lookupFunctions();
};

} // namespace fri3d
