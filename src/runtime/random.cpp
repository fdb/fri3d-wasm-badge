#include "random.h"

namespace fri3d {

Random::Random() : m_rng(std::random_device{}()) {
}

void Random::seed(uint32_t seed) {
    m_rng.seed(seed);
}

uint32_t Random::get() {
    return m_rng();
}

uint32_t Random::range(uint32_t max) {
    if (max == 0) return 0;
    return m_rng() % max;
}

} // namespace fri3d
