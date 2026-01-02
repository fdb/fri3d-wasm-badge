#pragma once

#include <stdint.h>

#define RANDOM_MT_STATE_SIZE 624

typedef struct {
    uint32_t mt[RANDOM_MT_STATE_SIZE];
    uint32_t index;
} random_t;

void random_init(random_t* random);
void random_seed(random_t* random, uint32_t seed);
uint32_t random_get(random_t* random);
uint32_t random_range(random_t* random, uint32_t max);
