//! The beanification engine: turn ordinary text into Mr-Bean-speak.

use crate::config::BeanConfig;
use crate::rng::Rng;
use crate::syllables::{CODAS, NUCLEI, ONSETS, SIGNATURES};

/// Casing pattern detected on a source word, reapplied to its mumble.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Case {
    /// All cased letters were upper case (and there was more than one).
    Upper,
    /// The first letter was upper case, the rest lower.
    Title,
    /// Anything else (all lower, mixed, or no cased letters).
    Lower,
}

/// The beanifier: transforms text deterministically given its [`BeanConfig`].
#[derive(Debug, Clone)]
pub struct Beanifier {
    config: BeanConfig,
}

impl Default for Beanifier {
    fn default() -> Self {
        Beanifier::new(BeanConfig::default())
    }
}

impl Beanifier {
    /// Create a beanifier with the given configuration.
    pub fn new(config: BeanConfig) -> Self {
        Beanifier { config }
    }

    /// Borrow the configuration in use.
    pub fn config(&self) -> &BeanConfig {
        &self.config
    }

    /// Beanify an entire block of text.
    ///
    /// Alphanumeric runs (words) become mumbles; everything else — whitespace,
    /// punctuation, newlines — is preserved verbatim so the shape of the input
    /// (and thus of a source file) survives.
    pub fn beanify_text(&self, input: &str) -> String {
        let mut out = String::with_capacity(input.len() + input.len() / 2);
        let mut word = String::new();

        for ch in input.chars() {
            if ch.is_alphanumeric() {
                word.push(ch);
            } else {
                if !word.is_empty() {
                    out.push_str(&self.beanify_word(&word));
                    word.clear();
                }
                out.push(ch);
            }
        }
        if !word.is_empty() {
            out.push_str(&self.beanify_word(&word));
        }
        out
    }

    /// Beanify a single word (an alphanumeric run).
    ///
    /// The word seeds a per-word PRNG, so the mapping is stable: the same word
    /// always beanifies to the same mumble under a given config.
    pub fn beanify_word(&self, word: &str) -> String {
        if word.is_empty() {
            return String::new();
        }

        let lower = word.to_lowercase();
        let mut rng = Rng::from_bytes(lower.as_bytes(), self.config.seed);

        let raw = if rng.chance(self.config.signature_frequency) {
            rng.choose(SIGNATURES).to_string()
        } else {
            self.mumble(word, &mut rng)
        };

        if self.config.preserve_case {
            apply_case(&raw, detect_case(word))
        } else {
            raw
        }
    }

    /// Assemble a nonsense mumble whose length scales with the source word.
    fn mumble(&self, word: &str, rng: &mut Rng) -> String {
        let len = word.chars().count();
        let max = self.config.max_syllables.max(1);
        // Roughly one syllable per two characters, always at least one.
        let syllables = len.div_ceil(2).clamp(1, max);

        let mut out = String::new();
        for _ in 0..syllables {
            out.push_str(rng.choose(ONSETS));
            out.push_str(rng.choose(NUCLEI));
            out.push_str(rng.choose(CODAS));
        }
        out
    }
}

/// Detect the casing pattern of a word.
fn detect_case(word: &str) -> Case {
    let mut has_upper = false;
    let mut has_lower = false;
    let mut cased_count = 0usize;
    let mut first_is_upper = false;
    let mut first_seen = false;

    for ch in word.chars() {
        if ch.is_uppercase() {
            has_upper = true;
            cased_count += 1;
            if !first_seen {
                first_is_upper = true;
                first_seen = true;
            }
        } else if ch.is_lowercase() {
            has_lower = true;
            cased_count += 1;
            if !first_seen {
                first_seen = true;
            }
        }
    }

    if has_upper && !has_lower && cased_count > 1 {
        Case::Upper
    } else if first_is_upper {
        Case::Title
    } else {
        Case::Lower
    }
}

/// Reapply a casing pattern to a freshly generated mumble.
fn apply_case(text: &str, case: Case) -> String {
    match case {
        Case::Upper => text.to_uppercase(),
        Case::Lower => text.to_string(),
        Case::Title => {
            let mut chars = text.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_stays_empty() {
        assert_eq!(Beanifier::default().beanify_text(""), "");
        assert_eq!(Beanifier::default().beanify_word(""), "");
    }

    #[test]
    fn is_deterministic() {
        let b = Beanifier::default();
        assert_eq!(b.beanify_text("hello world"), b.beanify_text("hello world"));
        assert_eq!(b.beanify_word("teddy"), b.beanify_word("teddy"));
    }

    #[test]
    fn preserves_non_word_structure() {
        let b = Beanifier::default();
        let out = b.beanify_text("one, two.\nthree!");
        // Punctuation, spaces, and the newline must survive verbatim.
        assert!(out.contains(','));
        assert!(out.contains('.'));
        assert!(out.contains('\n'));
        assert!(out.contains('!'));
        assert_eq!(out.matches(' ').count(), 1);
    }

    #[test]
    fn whitespace_only_is_untouched() {
        let b = Beanifier::default();
        assert_eq!(b.beanify_text("  \t\n "), "  \t\n ");
    }

    #[test]
    fn preserves_uppercase() {
        let b = Beanifier::default();
        let out = b.beanify_word("HELLO");
        assert_eq!(out, out.to_uppercase());
        assert!(!out.is_empty());
    }

    #[test]
    fn preserves_titlecase() {
        let b = Beanifier::default();
        let out = b.beanify_word("Hello");
        let mut chars = out.chars();
        assert!(chars.next().unwrap().is_uppercase());
        assert!(chars.all(|c| !c.is_uppercase()));
    }

    #[test]
    fn lowercase_stays_lowercase() {
        let b = Beanifier::default();
        let out = b.beanify_word("hello");
        assert_eq!(out, out.to_lowercase());
    }

    #[test]
    fn different_seeds_differ() {
        let a = Beanifier::new(BeanConfig::default().with_seed(1));
        let b = Beanifier::new(BeanConfig::default().with_seed(2));
        // Overwhelmingly likely to differ across a sentence.
        assert_ne!(
            a.beanify_text("the quick brown fox"),
            b.beanify_text("the quick brown fox")
        );
    }

    #[test]
    fn case_insensitive_word_maps_consistently_when_case_ignored() {
        let cfg = BeanConfig {
            preserve_case: false,
            ..BeanConfig::default()
        };
        let b = Beanifier::new(cfg);
        assert_eq!(b.beanify_word("Hello"), b.beanify_word("hello"));
        assert_eq!(b.beanify_word("HELLO"), b.beanify_word("hello"));
    }

    #[test]
    fn syllable_count_is_bounded() {
        let cfg = BeanConfig::default()
            .with_signature_frequency(0.0)
            .with_max_syllables(2);
        let b = Beanifier::new(cfg);
        // A long word must not exceed the syllable cap; each syllable is at
        // most 5 chars (onset<=2, nucleus<=2, coda<=2 -> <=6 to be safe).
        let out = b.beanify_word("supercalifragilisticexpialidocious");
        assert!(out.len() <= 2 * 6);
    }

    #[test]
    fn zero_signature_frequency_never_uses_signatures() {
        let cfg = BeanConfig::default().with_signature_frequency(0.0);
        let b = Beanifier::new(cfg);
        // Signatures contain spaces ("mr bean") or letters not in our phoneme
        // coda/onset sets; the strongest tell is that no space appears.
        for w in ["hello", "world", "teddy", "rowan", "atkinson"] {
            assert!(!b.beanify_word(w).contains(' '));
        }
    }

    #[test]
    fn full_signature_frequency_always_uses_signatures() {
        let cfg = BeanConfig::default().with_signature_frequency(1.0);
        let b = Beanifier::new(cfg);
        for w in ["hello", "world", "programming"] {
            assert!(SIGNATURES.contains(&b.beanify_word(w).to_lowercase().as_str()));
        }
    }

    #[test]
    fn digits_are_treated_as_words() {
        let b = Beanifier::default();
        let out = b.beanify_text("abc 123 def");
        // Two spaces preserved, three tokens transformed.
        assert_eq!(out.matches(' ').count(), 2);
    }
}
