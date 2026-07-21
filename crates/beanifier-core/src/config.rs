//! Configuration for the beanification engine.

/// Tunable knobs controlling how text is turned into Mr-Bean-speak.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct BeanConfig {
    /// Global seed mixed into every word's PRNG. Change it to get a different —
    /// but still fully deterministic — dialect of Bean-speak.
    pub seed: u64,

    /// Probability in `0.0..=1.0` that a word is replaced wholesale by a
    /// signature Bean-ism (e.g. "bean", "teddy") instead of a generated mumble.
    pub signature_frequency: f64,

    /// Upper bound on the number of syllables a single generated mumble may have.
    pub max_syllables: usize,

    /// When `true`, the casing pattern of the source word (UPPER, Title, lower)
    /// is reapplied to the mumble so shouting stays shouty.
    pub preserve_case: bool,
}

impl Default for BeanConfig {
    fn default() -> Self {
        BeanConfig {
            seed: 0xB3A4,
            signature_frequency: 0.18,
            max_syllables: 4,
            preserve_case: true,
        }
    }
}

impl BeanConfig {
    /// Builder-style override of the seed.
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// Builder-style override of the signature frequency (clamped on use).
    pub fn with_signature_frequency(mut self, freq: f64) -> Self {
        self.signature_frequency = freq;
        self
    }

    /// Builder-style override of the maximum syllable count (min 1 on use).
    pub fn with_max_syllables(mut self, max: usize) -> Self {
        self.max_syllables = max;
        self
    }
}
