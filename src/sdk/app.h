#pragma once

#include <stdint.h>

#include "frd.h"

// App lifecycle helpers (handled by runtime). App IDs follow the host app registry.
WASM_IMPORT("exit_to_launcher") extern void exit_to_launcher(void);
WASM_IMPORT("start_app") extern void start_app(uint32_t app_id);
