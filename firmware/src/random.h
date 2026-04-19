#pragma once

#include <stdint.h>

// 1:1 C++ port of fri3d-runtime/src/random.rs (MT19937).
// Kept bit-exact so the real badge draws the same frames as the emulator
// for a given seed.

namespace fri3d {

class Random {
public:
    Random() { seed(0); }
    explicit Random(uint32_t s) { seed(s); }

    void seed(uint32_t s);
    uint32_t get();
    uint32_t range(uint32_t max);

private:
    void twist();

    static constexpr uint32_t N           = 624;
    static constexpr uint32_t M           = 397;
    static constexpr uint32_t MATRIX_A    = 0x9908B0DFu;
    static constexpr uint32_t UPPER_MASK  = 0x80000000u;
    static constexpr uint32_t LOWER_MASK  = 0x7FFFFFFFu;

    uint32_t m_mt[N];
    uint32_t m_index = N;
};

} // namespace fri3d
