//! # shabda — Grapheme-to-Phoneme Conversion
//!
//! **shabda** (Sanskrit: word / sound) provides text-to-phoneme conversion for
//! vocal synthesis. It bridges the gap between text input and svara's phoneme
//! sequences — turning "hello world" into synthesizable phoneme events.
//!
//! ## Architecture
//!
//! ```text
//! Input text
//!     |
//!     v
//! Normalizer (lowercase, expand numbers, handle punctuation)
//!     |
//!     v
//! Tokenizer (split into words, detect sentence boundaries)
//!     |
//!     v
//! G2P Engine (dictionary lookup → rule-based fallback)
//!     |
//!     v
//! Prosody Mapper (stress, intonation from punctuation/syntax)
//!     |
//!     v
//! Vec<PhonemeEvent> (ready for svara)
//! ```
//!
//! ## Quick Start
//!
//! ```rust
//! use shabda::prelude::*;
//!
//! let g2p = G2PEngine::new(Language::English);
//! let events = g2p.convert("hello world").unwrap();
//! // events is a Vec<svara::sequence::PhonemeEvent> ready for rendering
//! ```
//!
//! ## Feature Flags
//!
//! | Feature | Default | Description |
//! |---------|---------|-------------|
//! | `std` | Yes | Standard library. Disable for `no_std` + `alloc` |
//! | `logging` | No | Structured logging via tracing-subscriber |

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod dictionary;
pub mod engine;
pub mod error;
pub mod normalize;
pub mod prosody;
pub mod rules;

/// Convenience re-exports for common usage.
pub mod prelude {
    pub use crate::engine::{G2PEngine, Language};
    pub use crate::error::{Result, ShabdaError};
}

// Compile-time trait assertions.
#[cfg(test)]
mod assert_traits {
    fn _assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn public_types_are_send_sync() {
        _assert_send_sync::<crate::error::ShabdaError>();
        _assert_send_sync::<crate::engine::G2PEngine>();
        _assert_send_sync::<crate::engine::Language>();
        _assert_send_sync::<crate::dictionary::PronunciationDict>();
    }
}
