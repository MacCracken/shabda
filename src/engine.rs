//! The G2P engine — ties normalization, dictionary, rules, and prosody together.

use alloc::{string::ToString, vec::Vec};
use serde::{Deserialize, Serialize};
use tracing::{trace, warn};

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

/// Detects the most likely language for the given text based on script analysis.
///
/// Uses varna's script Unicode range data to identify which writing system
/// the text uses, then maps that to a supported `Language`. Returns `None`
/// if the text uses a script not associated with any supported language.
///
/// Currently only English (Latin script) is supported. As more languages are
/// added to shabda, this function will detect them automatically.
///
/// # Examples
///
/// ```
/// use shabda::engine::detect_language;
///
/// assert_eq!(detect_language("hello world"), Some(shabda::engine::Language::English));
/// assert_eq!(detect_language(""), None);
/// ```
#[cfg(feature = "varna")]
#[must_use]
pub fn detect_language(text: &str) -> Option<Language> {
    if text.trim().is_empty() {
        return None;
    }

    // Count codepoints belonging to each known script
    let scripts = [
        ("Latn", Language::English),
        // Future: ("Deva", Language::Hindi), ("Arab", Language::Arabic), etc.
    ];

    let mut best: Option<(Language, usize)> = None;

    for (script_code, language) in &scripts {
        if let Some(script) = varna::script::by_code(script_code) {
            let count = text
                .chars()
                .filter(|c| script.contains_codepoint(u32::from(*c)))
                .count();
            if count > 0 {
                match best {
                    Some((_, best_count)) if count > best_count => {
                        best = Some((*language, count));
                    }
                    None => {
                        best = Some((*language, count));
                    }
                    _ => {}
                }
            }
        }
    }

    best.map(|(lang, _)| lang)
}

/// Options for controlling G2P conversion behavior.
///
/// Used with [`G2PEngine::convert_with`] to enable emphasis detection,
/// speaking rate control, and other prosody features. The default options
/// produce the same output as [`G2PEngine::convert`].
///
/// # Examples
///
/// ```
/// use shabda::engine::ConvertOptions;
///
/// let opts = ConvertOptions::new()
///     .with_emphasis(true)
///     .with_speaking_rate(120.0);
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ConvertOptions {
    /// Enable emphasis detection from CAPS and *asterisks*.
    ///
    /// When `true`, ALL-CAPS words (3+ chars) receive emphatic stress,
    /// and `*wrapped*` words receive focus stress.
    #[serde(default)]
    pub emphasis: bool,

    /// Target speaking rate in words per minute.
    ///
    /// `None` uses the default rate (~150 WPM). Lower values produce
    /// slower, more deliberate speech; higher values produce faster speech.
    /// Clamped to 50–300 WPM.
    #[serde(default)]
    pub speaking_rate_wpm: Option<f32>,
}

impl ConvertOptions {
    /// Creates default conversion options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Enables or disables emphasis detection.
    #[must_use]
    pub fn with_emphasis(mut self, emphasis: bool) -> Self {
        self.emphasis = emphasis;
        self
    }

    /// Sets the target speaking rate in words per minute.
    #[must_use]
    pub fn with_speaking_rate(mut self, wpm: f32) -> Self {
        self.speaking_rate_wpm = Some(wpm);
        self
    }
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

    /// Converts text to a sequence of phoneme events with default options.
    ///
    /// Equivalent to `convert_with(text, &ConvertOptions::default())`.
    ///
    /// # Errors
    ///
    /// Returns `ShabdaError::InvalidInput` if the text is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use shabda::prelude::*;
    ///
    /// let g2p = G2PEngine::new(Language::English);
    /// let events = g2p.convert("hello world").unwrap();
    /// assert!(!events.is_empty());
    /// ```
    pub fn convert(&self, text: &str) -> Result<Vec<PhonemeEvent>> {
        self.convert_with(text, &ConvertOptions::default())
    }

    /// Converts text to a sequence of phoneme events with the given options.
    ///
    /// The pipeline:
    /// 1. Expand numbers to words and normalize text
    /// 2. Detect sentence intonation from punctuation
    /// 3. For each word: dictionary lookup → rule-based fallback
    /// 4. Syllabify and assign stress based on syllable weight
    /// 5. Apply emphasis markers (if enabled)
    /// 6. Apply speaking rate scaling (if set)
    /// 7. Insert phrase pauses at commas (150ms) and periods (300ms)
    /// 8. Insert word-boundary silence (40ms) between words
    ///
    /// # Errors
    ///
    /// Returns `ShabdaError::InvalidInput` if the text is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use shabda::prelude::*;
    ///
    /// let g2p = G2PEngine::new(Language::English);
    /// let opts = ConvertOptions::new()
    ///     .with_emphasis(true)
    ///     .with_speaking_rate(120.0);
    /// let events = g2p.convert_with("HELLO world", &opts).unwrap();
    /// assert!(!events.is_empty());
    /// ```
    pub fn convert_with(&self, text: &str, options: &ConvertOptions) -> Result<Vec<PhonemeEvent>> {
        if text.trim().is_empty() {
            return Err(ShabdaError::InvalidInput("empty text".to_string()));
        }

        #[cfg(feature = "varna")]
        let varna_inventory = crate::validate::inventory_for(self.language);

        let intonation = normalize::detect_intonation(text);
        let normalized = if options.emphasis {
            normalize::normalize_with_emphasis(text)
        } else {
            normalize::normalize(text)
        };

        trace!(
            input = text,
            normalized = normalized.as_str(),
            ?intonation,
            emphasis = options.emphasis,
            rate = ?options.speaking_rate_wpm,
            "converting text to phonemes"
        );

        let words: Vec<&str> = normalized.split_whitespace().collect();
        let mut events = Vec::new();
        let mut emphasis_active = false;

        for (i, word) in words.iter().enumerate() {
            // Handle emphasis markers
            if *word == normalize::EMPHASIS_START {
                emphasis_active = true;
                continue;
            }
            if *word == normalize::EMPHASIS_END {
                emphasis_active = false;
                continue;
            }

            // Handle phrase boundary markers
            if *word == normalize::COMMA_PAUSE {
                events.push(PhonemeEvent::new(
                    Phoneme::Silence,
                    0.15,
                    svara::prosody::Stress::Unstressed,
                ));
                continue;
            }
            if *word == normalize::PERIOD_PAUSE {
                events.push(PhonemeEvent::new(
                    Phoneme::Silence,
                    0.30,
                    svara::prosody::Stress::Unstressed,
                ));
                continue;
            }

            // Look up in dictionary first, fall back to rules
            let phonemes: Vec<Phoneme> = if let Some(dict_entry) = self.dictionary.lookup(word) {
                trace!(word, phoneme_count = dict_entry.len(), "dictionary hit");
                dict_entry.to_vec()
            } else {
                trace!(word, "dictionary miss, falling back to rules");
                match self.language {
                    Language::English => rules::english_rules(word),
                }
            };

            // Validate phoneme output against varna inventory in debug builds
            #[cfg(feature = "varna")]
            {
                let invalid = crate::validate::validate_phonemes(&phonemes, &varna_inventory);
                debug_assert!(
                    invalid.is_empty(),
                    "word {word:?} produced phonemes not in varna inventory: {invalid:?}"
                );
            }

            if phonemes.is_empty() {
                warn!(word, "no phonemes produced, skipping word");
                continue;
            }

            // Syllabify and assign stress based on syllable weight
            let is_content = prosody::is_content_word(word);
            let syllables = crate::syllable::syllabify(&phonemes);
            let mut word_events = if syllables.is_empty() {
                trace!(word, "no syllables (consonant-only), using simple stress");
                prosody::assign_stress(&phonemes, is_content)
            } else {
                trace!(
                    word,
                    syllable_count = syllables.len(),
                    is_content,
                    "syllabified"
                );
                prosody::assign_stress_syllabic(&syllables, is_content)
            };

            // Apply emphasis if active
            if emphasis_active {
                prosody::apply_emphasis(&mut word_events);
            }

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

        // Apply speaking rate scaling
        if let Some(wpm) = options.speaking_rate_wpm {
            prosody::apply_rate(&mut events, wpm);
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
    ///
    /// # Examples
    ///
    /// ```
    /// use shabda::prelude::*;
    ///
    /// let g2p = G2PEngine::new(Language::English);
    /// let voice = svara::voice::VoiceProfile::new_male();
    /// let samples = g2p.speak("hello", &voice, 44100.0).unwrap();
    /// assert!(!samples.is_empty());
    /// ```
    pub fn speak(
        &self,
        text: &str,
        voice: &svara::voice::VoiceProfile,
        sample_rate: f32,
    ) -> Result<Vec<f32>> {
        self.speak_with(text, voice, sample_rate, &ConvertOptions::default())
    }

    /// Converts text and renders to audio samples with the given options.
    ///
    /// # Errors
    ///
    /// Returns errors from either G2P conversion or audio synthesis.
    pub fn speak_with(
        &self,
        text: &str,
        voice: &svara::voice::VoiceProfile,
        sample_rate: f32,
        options: &ConvertOptions,
    ) -> Result<Vec<f32>> {
        let events = self.convert_with(text, options)?;

        let mut seq = svara::sequence::PhonemeSequence::new();
        for event in events {
            seq.push(event);
        }

        seq.render(voice, sample_rate)
            .map_err(|e| ShabdaError::RuleError(alloc::format!("audio synthesis failed: {e}")))
    }
}
