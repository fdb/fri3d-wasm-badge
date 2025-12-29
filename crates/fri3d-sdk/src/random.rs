//! Random number generation for WASM apps

extern "C" {
    fn random_seed(seed: u32);
    fn random_get() -> u32;
    fn random_range(max: u32) -> u32;
}

/// Seed the random number generator
#[inline]
pub fn seed(s: u32) {
    unsafe { random_seed(s) }
}

/// Get a random u32
#[inline]
pub fn get() -> u32 {
    unsafe { random_get() }
}

/// Get a random number in [0, max)
#[inline]
pub fn range(max: u32) -> u32 {
    unsafe { random_range(max) }
}
