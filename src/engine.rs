//! The G2P engine — ties normalization, dictionary, rules, and prosody together.

use alloc::{string::ToString, vec::Vec};
use serde::{Deserialize, Serialize};
use tracing::trace;

use svara::phoneme::Phoneme;
use svara::sequence::PhonemeEvent;

use crate::dictionary::PronunciationDict;
use crate::error::{Result, ShabdaError};
use crate::normalize;
use crate::prosody;
use crate::rules;

/// Supported languages for G2P conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Language {
    /// English (General American).
    English,
}

/// The grapheme-to-phoneme engine.
///
/// Converts text to svara `PhonemeEvent` sequences using dictionary lookup
/// with rule-based fallback and automatic prosody assignment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct G2PEngine {
    /// Active language.
    language: Language,
    /// Pronunciation dictionary.
    dictionary: PronunciationDict,
}

impl G2PEngine {
    /// Creates a new G2P engine for the given language.
    #[must_use]
    pub fn new(language: Language) -> Self {
        let dictionary = match language {
            Language::English => PronunciationDict::english(),
        };
        Self {
            language,
            dictionary,
        }
    }

    /// Returns the active language.
    #[must_use]
    pub fn language(&self) -> Language {
        self.language
    }

    /// Returns a reference to the pronunciation dictionary.
    #[must_use]
    pub fn dictionary(&self) -> &PronunciationDict {
        &self.dictionary
    }

    /// Returns a mutable reference to the dictionary for adding custom entries.
    pub fn dictionary_mut(&mut self) -> &mut PronunciationDict {
        &mut self.dictionary
    }

    /// Converts text to a sequence of phoneme events.
    ///
    /// The pipeline:
    /// 1. Detect sentence intonation from punctuation
    /// 2. Normalize text (lowercase, strip punctuation)
    /// 3. Split into words
    /// 4. For each word: dictionary lookup → rule-based fallback
    /// 5. Assign stress based on word class (content vs function)
    /// 6. Insert silence between words
    ///
    /// # Errors
    ///
    /// Returns `ShabdaError::InvalidInput` if the text is empty.
    pub fn convert(&self, text: &str) -> Result<Vec<PhonemeEvent>> {
        if text.trim().is_empty() {
            return Err(ShabdaError::InvalidInput("empty text".to_string()));
        }

        let intonation = normalize::detect_intonation(text);
        let normalized = normalize::normalize(text);

        trace!(
            input = text,
            normalized = normalized.as_str(),
            ?intonation,
            "converting text to phonemes"
        );

        let words: Vec<&str> = normalized.split_whitespace().collect();
        let mut events = Vec::new();

        for (i, word) in words.iter().enumerate() {
            // Look up in dictionary first, fall back to rules
            let phonemes: Vec<Phoneme> = if let Some(dict_entry) = self.dictionary.lookup(word) {
                dict_entry.to_vec()
            } else {
                match self.language {
                    Language::English => rules::english_rules(word),
                }
            };

            if phonemes.is_empty() {
                continue;
            }

            // Assign stress based on word type
            let is_content = prosody::is_content_word(word);
            let word_events = prosody::assign_stress(&phonemes, is_content);
            events.extend(word_events);

            // Insert short silence between words (not after last word)
            if i < words.len() - 1 {
                events.push(PhonemeEvent::new(
                    Phoneme::Silence,
                    0.04,
                    svara::prosody::Stress::Unstressed,
                ));
            }
        }

        Ok(events)
    }

    /// Converts text and renders directly to audio samples.
    ///
    /// Convenience method that combines G2P conversion with svara rendering.
    ///
    /// # Errors
    ///
    /// Returns errors from either G2P conversion or audio synthesis.
    pub fn speak(
        &self,
        text: &str,
        voice: &svara::voice::VoiceProfile,
        sample_rate: f32,
    ) -> Result<Vec<f32>> {
        let events = self.convert(text)?;

        let mut seq = svara::sequence::PhonemeSequence::new();
        for event in events {
            seq.push(event);
        }

        seq.render(voice, sample_rate)
            .map_err(|e| ShabdaError::RuleError(alloc::format!("{e}")))
    }
}
