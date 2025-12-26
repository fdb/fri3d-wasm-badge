#pragma once

#include <cstdint>
#include <random>

namespace fri3d {

/**
 * Random number generator for WASM apps.
 * Uses Mersenne Twister for consistent behavior across platforms.
 */
class Random {
public:
    /**
     * Initialize with a random seed.
     */
    Random();

    /**
     * Set the seed for deterministic output.
     */
    void seed(uint32_t seed);

    /**
     * Get a random 32-bit number.
     */
    uint32_t get();

    /**
     * Get a random number in range [0, max).
     * Returns 0 if max is 0.
     */
    uint32_t range(uint32_t max);

private:
    std::mt19937 m_rng;
};

} // namespace fri3d
