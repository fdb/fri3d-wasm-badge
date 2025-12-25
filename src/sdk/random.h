#pragma once

#include <stdint.h>
#include "frd.h"

// Seed the random number generator
WASM_IMPORT("random_seed") extern void random_seed(uint32_t seed);

// Get a random 32-bit value
WASM_IMPORT("random_get") extern uint32_t random_get(void);

// Get a random value in range [0, max)
WASM_IMPORT("random_range") extern uint32_t random_range(uint32_t max);
