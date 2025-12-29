//! Mersenne Twister PRNG

const N: usize = 624;
const M: usize = 397;
const MATRIX_A: u32 = 0x9908b0df;
const UPPER_MASK: u32 = 0x80000000;
const LOWER_MASK: u32 = 0x7fffffff;

/// Mersenne Twister random number generator
pub struct Random {
    state: [u32; N],
    index: usize,
}

impl Default for Random {
    fn default() -> Self {
        Self::new(5489)
    }
}

impl Random {
    /// Create a new RNG with the given seed
    pub fn new(seed: u32) -> Self {
        let mut rng = Self {
            state: [0; N],
            index: N + 1,
        };
        rng.seed(seed);
        rng
    }

    /// Seed the RNG
    pub fn seed(&mut self, seed: u32) {
        self.state[0] = seed;
        for i in 1..N {
            self.state[i] = 1812433253u32
                .wrapping_mul(self.state[i - 1] ^ (self.state[i - 1] >> 30))
                .wrapping_add(i as u32);
        }
        self.index = N;
    }

    /// Generate a random u32
    pub fn next(&mut self) -> u32 {
        if self.index >= N {
            self.twist();
        }

        let mut y = self.state[self.index];
        self.index += 1;

        // Tempering
        y ^= y >> 11;
        y ^= (y << 7) & 0x9d2c5680;
        y ^= (y << 15) & 0xefc60000;
        y ^= y >> 18;

        y
    }

    /// Generate a random number in [0, max)
    pub fn range(&mut self, max: u32) -> u32 {
        if max == 0 {
            return 0;
        }
        self.next() % max
    }

    /// Twist the state array
    fn twist(&mut self) {
        for i in 0..N {
            let x = (self.state[i] & UPPER_MASK) | (self.state[(i + 1) % N] & LOWER_MASK);
            let mut xa = x >> 1;
            if x & 1 != 0 {
                xa ^= MATRIX_A;
            }
            self.state[i] = self.state[(i + M) % N] ^ xa;
        }
        self.index = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic() {
        let mut rng1 = Random::new(12345);
        let mut rng2 = Random::new(12345);

        for _ in 0..100 {
            assert_eq!(rng1.next(), rng2.next());
        }
    }

    #[test]
    fn test_range() {
        let mut rng = Random::new(42);
        for _ in 0..1000 {
            let val = rng.range(10);
            assert!(val < 10);
        }
    }
}
