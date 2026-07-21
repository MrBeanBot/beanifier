//! A tiny deterministic PRNG so that beanification is fully reproducible.
//!
//! Reproducibility matters here: the same word with the same seed must always
//! produce the same mumble, otherwise tests are flaky and re-running the tool
//! over a tree would churn every file on every run.

/// FNV-1a 64-bit hash of a byte slice.
pub fn fnv1a(bytes: &[u8]) -> u64 {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    let mut hash = OFFSET;
    for &b in bytes {
        hash ^= b as u64;
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

/// A `splitmix64` generator — small, fast, and stable across platforms.
#[derive(Debug, Clone)]
pub struct Rng {
    state: u64,
}

impl Rng {
    /// Seed the generator directly.
    pub fn seeded(seed: u64) -> Self {
        Rng { state: seed }
    }

    /// Seed the generator from arbitrary bytes.
    pub fn from_bytes(bytes: &[u8], salt: u64) -> Self {
        Rng::seeded(fnv1a(bytes) ^ salt)
    }

    /// Draw the next 64-bit value.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Draw a value in `0..n`. Returns 0 when `n == 0`.
    pub fn below(&mut self, n: usize) -> usize {
        if n == 0 {
            return 0;
        }
        (self.next_u64() % n as u64) as usize
    }

    /// Pick a reference to one element of a non-empty slice.
    ///
    /// # Panics
    /// Panics if `items` is empty.
    pub fn choose<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        assert!(!items.is_empty(), "cannot choose from an empty slice");
        &items[self.below(items.len())]
    }

    /// Return `true` with probability `prob` (clamped to `0.0..=1.0`).
    pub fn chance(&mut self, prob: f64) -> bool {
        let p = prob.clamp(0.0, 1.0);
        // Compare against a uniform value in [0, 1).
        let u = (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64;
        u < p
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fnv1a_matches_known_vector() {
        // Well-known FNV-1a 64-bit test vector for the empty string.
        assert_eq!(fnv1a(b""), 0xcbf2_9ce4_8422_2325);
    }

    #[test]
    fn same_seed_same_sequence() {
        let mut a = Rng::seeded(42);
        let mut b = Rng::seeded(42);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn different_seeds_diverge() {
        let mut a = Rng::seeded(1);
        let mut b = Rng::seeded(2);
        assert_ne!(a.next_u64(), b.next_u64());
    }

    #[test]
    fn below_is_in_range() {
        let mut r = Rng::seeded(7);
        for _ in 0..1000 {
            assert!(r.below(10) < 10);
        }
        assert_eq!(r.below(0), 0);
        assert_eq!(r.below(1), 0);
    }

    #[test]
    fn chance_bounds_are_absolute() {
        let mut r = Rng::seeded(9);
        for _ in 0..1000 {
            assert!(!r.chance(0.0));
            assert!(r.chance(1.0));
        }
    }
}
