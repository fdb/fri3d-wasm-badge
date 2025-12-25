#pragma once

#include <stdint.h>

// Seed the random number generator
void random_seed(uint32_t seed);

// Get a random 32-bit value
uint32_t random_get(void);

// Get a random value in range [0, max) - convenience function
uint32_t random_range(uint32_t max);
