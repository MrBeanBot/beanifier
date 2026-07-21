//! Phoneme building blocks for Mr-Bean-speak.
//!
//! Bean rarely uses words; he mumbles, grumbles, and interjects. We assemble
//! nonsense that *sounds* like his mutterings from these fragments.

/// Consonant onsets that begin a mumbled syllable.
pub const ONSETS: &[&str] = &[
    "b", "bw", "m", "n", "hm", "ng", "fl", "w", "g", "gr", "bl", "p", "t", "d", "hn",
];

/// Vowel nuclei — the core of each syllable.
pub const NUCLEI: &[&str] = &[
    "a", "aa", "o", "oo", "e", "ee", "u", "uh", "ah", "eh", "i", "ow",
];

/// Optional codas that can close a syllable (empty entries mean "no coda").
pub const CODAS: &[&str] = &["", "", "", "m", "n", "ng", "b", "p", "h"];

/// Iconic whole-word Bean-isms, occasionally substituted for a full word.
/// Weighted toward the signature "bean" / "teddy".
pub const SIGNATURES: &[&str] = &[
    "bean", "bean", "bean", "teddy", "teddy", "mr bean", "nyeh", "mwah", "aaargh", "hmmm", "tch",
    "nnng", "ohh", "eurgh", "wibble",
];
