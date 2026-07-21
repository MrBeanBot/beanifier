//! # beanifier-core
//!
//! The engine that turns ordinary text into Mr-Bean-speak — a stream of
//! deterministic mumbles, grumbles, and the occasional "bean".
//!
//! The transformation is **structure preserving**: only alphanumeric runs are
//! rewritten, so whitespace, punctuation and line breaks survive untouched.
//! It is also **deterministic**: a given word under a given [`BeanConfig`]
//! always maps to the same mumble, which keeps output stable across runs.
//!
//! ```
//! use beanifier_core::{Beanifier, BeanConfig};
//!
//! let bean = Beanifier::new(BeanConfig::default());
//! let out = bean.beanify_text("Hello, world!");
//! assert!(out.contains(','));
//! assert!(out.contains('!'));
//! // Same input, same output — every time.
//! assert_eq!(out, bean.beanify_text("Hello, world!"));
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod beanify;
mod config;
pub mod rng;
pub mod syllables;

pub use beanify::Beanifier;
pub use config::BeanConfig;

/// Convenience: beanify text with the default configuration.
pub fn beanify(input: &str) -> String {
    Beanifier::default().beanify_text(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convenience_matches_default_engine() {
        assert_eq!(
            beanify("hello world"),
            Beanifier::default().beanify_text("hello world")
        );
    }
}
