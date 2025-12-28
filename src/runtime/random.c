#include "random.h"
#include <time.h>

/**
 * xorshift32 algorithm - fast and portable PRNG.
 * Good statistical properties for games/demos.
 */
static uint32_t xorshift32(uint32_t* state) {
    uint32_t x = *state;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    *state = x;
    return x;
}

void random_init(random_t* rng) {
    // Use time as initial seed
    // Cast through uintptr_t to handle different time_t sizes
    uint32_t seed = (uint32_t)((uintptr_t)time(NULL) & 0xFFFFFFFF);
    if (seed == 0) seed = 1;  // xorshift requires non-zero seed
    rng->state = seed;
}

void random_seed(random_t* rng, uint32_t seed) {
    if (seed == 0) seed = 1;  // xorshift requires non-zero seed
    rng->state = seed;
}

uint32_t random_get(random_t* rng) {
    return xorshift32(&rng->state);
}

uint32_t random_range(random_t* rng, uint32_t max) {
    if (max == 0) return 0;
    return xorshift32(&rng->state) % max;
}
