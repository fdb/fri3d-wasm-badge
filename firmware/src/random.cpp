#include "random.h"

namespace fri3d {

void Random::seed(uint32_t s) {
    m_mt[0] = s;
    for (uint32_t i = 1; i < N; ++i) {
        uint32_t prev = m_mt[i - 1];
        m_mt[i] = 1812433253u * (prev ^ (prev >> 30)) + i;
    }
    m_index = N;
}

void Random::twist() {
    for (uint32_t i = 0; i < N; ++i) {
        uint32_t y = (m_mt[i] & UPPER_MASK) | (m_mt[(i + 1) % N] & LOWER_MASK);
        uint32_t next = m_mt[(i + M) % N] ^ (y >> 1);
        if (y & 1) next ^= MATRIX_A;
        m_mt[i] = next;
    }
    m_index = 0;
}

uint32_t Random::get() {
    if (m_index >= N) twist();

    uint32_t y = m_mt[m_index++];
    y ^= y >> 11;
    y ^= (y << 7)  & 0x9D2C5680u;
    y ^= (y << 15) & 0xEFC60000u;
    y ^= y >> 18;
    return y;
}

uint32_t Random::range(uint32_t max) {
    if (max == 0) return 0;
    return get() % max;
}

} // namespace fri3d
