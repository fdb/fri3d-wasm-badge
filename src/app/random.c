#include "random.h"

// Host function imports
#define WASM_IMPORT(name) __attribute__((import_module("env"), import_name(name)))

WASM_IMPORT("host_random_seed") extern void host_random_seed(uint32_t seed);
WASM_IMPORT("host_random_get") extern uint32_t host_random_get(void);

void random_seed(uint32_t seed) {
    host_random_seed(seed);
}

uint32_t random_get(void) {
    return host_random_get();
}

uint32_t random_range(uint32_t max) {
    if (max == 0) return 0;
    return random_get() % max;
}
