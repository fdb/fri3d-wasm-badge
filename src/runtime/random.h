#pragma once

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Random number generator state.
 * Uses xorshift32 algorithm for fast, portable random numbers.
 */
typedef struct {
    uint32_t state;
} random_t;

/**
 * Initialize with a random seed based on system time.
 */
void random_init(random_t* rng);

/**
 * Set the seed for deterministic output.
 */
void random_seed(random_t* rng, uint32_t seed);

/**
 * Get a random 32-bit number.
 */
uint32_t random_get(random_t* rng);

/**
 * Get a random number in range [0, max).
 * Returns 0 if max is 0.
 */
uint32_t random_range(random_t* rng, uint32_t max);

#ifdef __cplusplus
}
#endif
