#include "random.h"

#include <time.h>

#define MT19937_N 624
#define MT19937_M 397
#define MT19937_MATRIX_A 0x9908B0DFu
#define MT19937_UPPER_MASK 0x80000000u
#define MT19937_LOWER_MASK 0x7FFFFFFFu

static void random_twist(random_t* random) {
    for (uint32_t i = 0; i < MT19937_N; i++) {
        uint32_t y = (random->mt[i] & MT19937_UPPER_MASK) |
                     (random->mt[(i + 1) % MT19937_N] & MT19937_LOWER_MASK);
        uint32_t next = random->mt[(i + MT19937_M) % MT19937_N] ^ (y >> 1);
        if (y & 1u) {
            next ^= MT19937_MATRIX_A;
        }
        random->mt[i] = next;
    }
    random->index = 0;
}

void random_seed(random_t* random, uint32_t seed) {
    if (!random) {
        return;
    }

    random->mt[0] = seed;
    for (uint32_t i = 1; i < MT19937_N; i++) {
        uint32_t prev = random->mt[i - 1];
        random->mt[i] = 1812433253u * (prev ^ (prev >> 30)) + i;
    }
    random->index = MT19937_N;
}

void random_init(random_t* random) {
    if (!random) {
        return;
    }
    uint32_t seed = (uint32_t)time(NULL);
    random_seed(random, seed);
}

uint32_t random_get(random_t* random) {
    if (!random) {
        return 0;
    }

    if (random->index >= MT19937_N) {
        random_twist(random);
    }

    uint32_t y = random->mt[random->index++];

    y ^= (y >> 11);
    y ^= (y << 7) & 0x9D2C5680u;
    y ^= (y << 15) & 0xEFC60000u;
    y ^= (y >> 18);

    return y;
}

uint32_t random_range(random_t* random, uint32_t max) {
    if (!random || max == 0) {
        return 0;
    }
    return random_get(random) % max;
}
