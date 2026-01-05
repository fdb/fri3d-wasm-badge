const MT19937_N: usize = 624;
const MT19937_M: usize = 397;
const MT19937_MATRIX_A: u32 = 0x9908B0DF;
const MT19937_UPPER_MASK: u32 = 0x80000000;
const MT19937_LOWER_MASK: u32 = 0x7FFFFFFF;

pub struct Random {
    mt: [u32; MT19937_N],
    index: usize,
}

impl Random {
    pub fn new(seed: u32) -> Self {
        let mut rng = Self {
            mt: [0; MT19937_N],
            index: MT19937_N,
        };
        rng.seed(seed);
        rng
    }

    pub fn seed(&mut self, seed: u32) {
        self.mt[0] = seed;
        for i in 1..MT19937_N {
            let prev = self.mt[i - 1];
            self.mt[i] = 1812433253u32.wrapping_mul(prev ^ (prev >> 30)).wrapping_add(i as u32);
        }
        self.index = MT19937_N;
    }

    pub fn get(&mut self) -> u32 {
        if self.index >= MT19937_N {
            self.twist();
        }

        let mut y = self.mt[self.index];
        self.index += 1;

        y ^= y >> 11;
        y ^= (y << 7) & 0x9D2C5680;
        y ^= (y << 15) & 0xEFC60000;
        y ^= y >> 18;

        y
    }

    pub fn range(&mut self, max: u32) -> u32 {
        if max == 0 {
            return 0;
        }
        self.get() % max
    }

    fn twist(&mut self) {
        for i in 0..MT19937_N {
            let y = (self.mt[i] & MT19937_UPPER_MASK)
                | (self.mt[(i + 1) % MT19937_N] & MT19937_LOWER_MASK);
            let mut next = self.mt[(i + MT19937_M) % MT19937_N] ^ (y >> 1);
            if y & 1 != 0 {
                next ^= MT19937_MATRIX_A;
            }
            self.mt[i] = next;
        }
        self.index = 0;
    }
}
