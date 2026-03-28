//! Pronunciation dictionary for common/irregular words.
//!
//! English has many irregular pronunciations (e.g., "one" → /wʌn/, "colonel" → /kɜːnəl/).
//! The dictionary provides known-correct phoneme sequences for these words.
//!
//! The full English dictionary is generated at compile time from `data/cmudict-5k.txt`
//! by the build script. A minimal 28-entry variant is available via [`PronunciationDict::english_minimal`].
//!
//! ## User overlay
//!
//! Application-specific pronunciations can be added via [`PronunciationDict::insert_user`].
//! User entries take precedence over base entries during [`PronunciationDict::lookup`].

pub mod format;

use alloc::{collections::BTreeMap, string::String, vec::Vec};
use serde::{Deserialize, Serialize};
use svara::phoneme::Phoneme;

// Pull in the generated dictionary function from build.rs output.
include!(concat!(env!("OUT_DIR"), "/generated_dict.rs"));

/// A pronunciation dictionary mapping words to phoneme sequences.
///
/// Supports a two-layer lookup: user entries (overlay) take precedence over
/// base entries. This allows applications to override or extend the built-in
/// dictionary without modifying it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PronunciationDict {
    entries: BTreeMap<String, Vec<Phoneme>>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    user_entries: BTreeMap<String, Vec<Phoneme>>,
}

impl PronunciationDict {
    /// Creates a new empty dictionary.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
            user_entries: BTreeMap::new(),
        }
    }

    /// Creates the built-in English pronunciation dictionary (5,000+ entries).
    ///
    /// Generated at compile time from `data/cmudict-5k.txt`.
    #[must_use]
    pub fn english() -> Self {
        Self {
            entries: generated_english_entries(),
            user_entries: BTreeMap::new(),
        }
    }

    /// Creates a minimal English dictionary with ~28 common function words.
    ///
    /// Useful for testing or memory-constrained environments.
    #[must_use]
    pub fn english_minimal() -> Self {
        let mut dict = Self::new();

        dict.insert("the", &[Phoneme::FricativeDh, Phoneme::VowelSchwa]);
        dict.insert("a", &[Phoneme::VowelSchwa]);
        dict.insert("an", &[Phoneme::VowelSchwa, Phoneme::NasalN]);
        dict.insert("i", &[Phoneme::DiphthongAI]);
        dict.insert("is", &[Phoneme::VowelNearI, Phoneme::FricativeZ]);
        dict.insert(
            "was",
            &[
                Phoneme::ApproximantW,
                Phoneme::VowelOpenO,
                Phoneme::FricativeZ,
            ],
        );
        dict.insert("are", &[Phoneme::VowelOpenA, Phoneme::ApproximantR]);
        dict.insert("to", &[Phoneme::PlosiveT, Phoneme::VowelU]);
        dict.insert("of", &[Phoneme::VowelOpenO, Phoneme::FricativeV]);
        dict.insert("in", &[Phoneme::VowelNearI, Phoneme::NasalN]);
        dict.insert("it", &[Phoneme::VowelNearI, Phoneme::PlosiveT]);
        dict.insert(
            "and",
            &[Phoneme::VowelAsh, Phoneme::NasalN, Phoneme::PlosiveD],
        );
        dict.insert(
            "that",
            &[Phoneme::FricativeDh, Phoneme::VowelAsh, Phoneme::PlosiveT],
        );
        dict.insert(
            "for",
            &[
                Phoneme::FricativeF,
                Phoneme::VowelOpenO,
                Phoneme::ApproximantR,
            ],
        );
        dict.insert("you", &[Phoneme::ApproximantJ, Phoneme::VowelU]);
        dict.insert("he", &[Phoneme::FricativeH, Phoneme::VowelE]);
        dict.insert("she", &[Phoneme::FricativeSh, Phoneme::VowelE]);
        dict.insert("we", &[Phoneme::ApproximantW, Phoneme::VowelE]);
        dict.insert("they", &[Phoneme::FricativeDh, Phoneme::DiphthongEI]);
        dict.insert(
            "this",
            &[
                Phoneme::FricativeDh,
                Phoneme::VowelNearI,
                Phoneme::FricativeS,
            ],
        );
        dict.insert(
            "with",
            &[
                Phoneme::ApproximantW,
                Phoneme::VowelNearI,
                Phoneme::FricativeTh,
            ],
        );
        dict.insert(
            "not",
            &[Phoneme::NasalN, Phoneme::VowelOpenO, Phoneme::PlosiveT],
        );
        dict.insert(
            "but",
            &[Phoneme::PlosiveB, Phoneme::VowelCupV, Phoneme::PlosiveT],
        );
        dict.insert(
            "have",
            &[Phoneme::FricativeH, Phoneme::VowelAsh, Phoneme::FricativeV],
        );
        dict.insert(
            "one",
            &[Phoneme::ApproximantW, Phoneme::VowelCupV, Phoneme::NasalN],
        );
        dict.insert(
            "hello",
            &[
                Phoneme::FricativeH,
                Phoneme::VowelOpenE,
                Phoneme::LateralL,
                Phoneme::DiphthongOU,
            ],
        );
        dict.insert(
            "world",
            &[
                Phoneme::ApproximantW,
                Phoneme::VowelBird,
                Phoneme::LateralL,
                Phoneme::PlosiveD,
            ],
        );
        dict.insert(
            "yes",
            &[
                Phoneme::ApproximantJ,
                Phoneme::VowelOpenE,
                Phoneme::FricativeS,
            ],
        );
        dict.insert("no", &[Phoneme::NasalN, Phoneme::DiphthongOU]);

        dict
    }

    /// Creates a dictionary from a pre-built entries map.
    #[must_use]
    pub fn from_entries(entries: BTreeMap<String, Vec<Phoneme>>) -> Self {
        Self {
            entries,
            user_entries: BTreeMap::new(),
        }
    }

    /// Inserts a word into the base dictionary.
    pub fn insert(&mut self, word: &str, phonemes: &[Phoneme]) {
        self.entries.insert(
            alloc::string::ToString::to_string(&word.to_lowercase()),
            phonemes.to_vec(),
        );
    }

    /// Inserts a word into the user overlay.
    ///
    /// User entries take precedence over base entries during lookup.
    pub fn insert_user(&mut self, word: &str, phonemes: &[Phoneme]) {
        self.user_entries.insert(
            alloc::string::ToString::to_string(&word.to_lowercase()),
            phonemes.to_vec(),
        );
    }

    /// Removes a word from the user overlay.
    ///
    /// Returns `true` if the word was present in the user overlay.
    pub fn remove_user(&mut self, word: &str) -> bool {
        self.user_entries
            .remove(&alloc::string::ToString::to_string(&word.to_lowercase()))
            .is_some()
    }

    /// Returns a reference to the user overlay entries.
    #[must_use]
    pub fn user_entries(&self) -> &BTreeMap<String, Vec<Phoneme>> {
        &self.user_entries
    }

    /// Returns the number of user overlay entries.
    #[must_use]
    pub fn user_len(&self) -> usize {
        self.user_entries.len()
    }

    /// Looks up a word's pronunciation.
    ///
    /// Checks the user overlay first, then the base dictionary.
    #[must_use]
    pub fn lookup(&self, word: &str) -> Option<&[Phoneme]> {
        let key = alloc::string::ToString::to_string(&word.to_lowercase());
        self.user_entries
            .get(&key)
            .or_else(|| self.entries.get(&key))
            .map(|v| v.as_slice())
    }

    /// Returns the number of base dictionary entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns whether the base dictionary is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns a reference to the base entries.
    #[must_use]
    pub fn entries(&self) -> &BTreeMap<String, Vec<Phoneme>> {
        &self.entries
    }
}

impl Default for PronunciationDict {
    fn default() -> Self {
        Self::new()
    }
}
