//! Pronunciation dictionary for common/irregular words.
//!
//! English has many irregular pronunciations (e.g., "one" → /wʌn/, "colonel" → /kɜːnəl/).
//! The dictionary provides known-correct phoneme sequences for these words.

use alloc::{collections::BTreeMap, string::String, vec::Vec};
use serde::{Deserialize, Serialize};
use svara::phoneme::Phoneme;

/// A pronunciation dictionary mapping words to phoneme sequences.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PronunciationDict {
    entries: BTreeMap<String, Vec<Phoneme>>,
}

impl PronunciationDict {
    /// Creates a new empty dictionary.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    /// Creates the built-in English pronunciation dictionary.
    ///
    /// Contains common irregular words and function words.
    #[must_use]
    pub fn english() -> Self {
        let mut dict = Self::new();

        // Common function words and irregulars
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

    /// Inserts a word with its phoneme sequence.
    pub fn insert(&mut self, word: &str, phonemes: &[Phoneme]) {
        self.entries.insert(
            alloc::string::ToString::to_string(&word.to_lowercase()),
            phonemes.to_vec(),
        );
    }

    /// Looks up a word's pronunciation.
    #[must_use]
    pub fn lookup(&self, word: &str) -> Option<&[Phoneme]> {
        self.entries
            .get(&alloc::string::ToString::to_string(&word.to_lowercase()))
            .map(|v| v.as_slice())
    }

    /// Returns the number of entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns whether the dictionary is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for PronunciationDict {
    fn default() -> Self {
        Self::new()
    }
}
