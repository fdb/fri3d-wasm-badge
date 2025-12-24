
set(CMAKE_SYSTEM_NAME Generic)
set(CMAKE_SYSTEM_VERSION 1)
set(CMAKE_PROCESSOR "wasm32")

set(CMAKE_C_COMPILER_WORKS 1)
set(CMAKE_CXX_COMPILER_WORKS 1)

# Specify the compiler
set(CMAKE_C_COMPILER ${CMAKE_CURRENT_LIST_DIR}/zig-cc)
set(CMAKE_CXX_COMPILER ${CMAKE_CURRENT_LIST_DIR}/zig-c++) # If needed, create it.

# Flags for zig cc (already has target in wrapper, but good to ensure)
# -nostdlib is implied by freestanding usually, but let's be explicit if needed.
set(CMAKE_C_FLAGS "-O3")
# set(CMAKE_EXE_LINKER_FLAGS "-Wl,--no-entry -Wl,--allow-undefined")

# -nostdlib might be too aggressive if we need some utils, but for this bare metal style app it might work.
# However, u8g2 might use some stdlib stuff (memcpy, etc). 
# We should try to use built-in headers or a minimal libc if needed. 
# WAMR supports WASI, so maybe we should target wasi?
# "--target=wasm32-wasi" requires wasi-libc.
# Let's try --target=wasm32 -nostdlib and see if we can get away with it for a simple circle.
# We might need to provide memset/memcpy.

set(CMAKE_EXECUTABLE_SUFFIX ".wasm")
