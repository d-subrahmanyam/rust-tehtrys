//! Internal PRNG (no external crate; deterministic). SplitMix64. (NFR-1, §3.3)
//!
//! SplitMix64 has a 2^64 period; excellent statistical quality for shuffling a
//! 7-piece bag. Seedable; reproducible.

/// RNG abstraction for testing / replay.
pub trait Rng {
    fn next_u64(&mut self) -> u64;

    /// Uniform in `0..n`. (`n > 0`)
    fn next_below(&mut self, n: u32) -> u32 {
        debug_assert!(n > 0);
        // Lemire-style reduction (multiply-shift); adequate for n<=7 bags.
        let x = self.next_u64() as u128;
        ((x * n as u128) >> 64) as u32
    }

    /// Fisher–Yates shuffle. (§3.3 shuffle)
    fn shuffle<T>(&mut self, slice: &mut [T]) {
        let len = slice.len();
        if len <= 1 {
            return;
        }
        for i in (1..len).rev() {
            let j = self.next_below((i + 1) as u32) as usize;
            slice.swap(i, j);
        }
    }
}

/// SplitMix64. (`https://prng.di.unimi.it/splitmix64.c`)
#[derive(Clone, Debug)]
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Split a seed (e.g. from entropy at app layer); core takes explicit seed.
    pub fn from_seed(seed: u64) -> Self {
        Self::new(seed)
    }

    pub fn seed(&self) -> u64 {
        self.state
    }
}

impl Rng for SplitMix64 {
    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_same_seed() {
        let mut a = SplitMix64::new(42);
        let mut b = SplitMix64::new(42);
        for _ in 0..1000 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn different_seeds_differ() {
        let mut a = SplitMix64::new(1);
        let mut b = SplitMix64::new(2);
        assert_ne!(a.next_u64(), b.next_u64());
    }

    #[test]
    fn next_below_range() {
        let mut r = SplitMix64::new(123);
        for _ in 0..10_000 {
            assert!(r.next_below(7) < 7);
        }
    }

    #[test]
    fn shuffle_is_permutation() {
        let mut r = SplitMix64::new(7);
        let mut v: Vec<u32> = (0..100).collect();
        let orig = v.clone();
        r.shuffle(&mut v);
        let mut sorted = v.clone();
        sorted.sort();
        assert_eq!(sorted, orig); // same multiset
        assert_ne!(v, orig); // almost surely changed order
    }
}
