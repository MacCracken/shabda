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
    /// Spanish (Castilian).
    Spanish,
    /// German (Standard High German).
    German,
    /// Hindi (Modern Standard Hindi, Devanagari script).
    Hindi,
    /// Arabic (Classical/Modern Standard Arabic).
    Arabic,
    /// Sanskrit (Classical, Devanagari script).
    Sanskrit,
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

    /// Phoneme-level timing profile for fine-grained duration control.
    ///
    /// `None` uses default durations from svara. Scales are multiplicative
    /// (1.0 = no change, 2.0 = double duration, 0.5 = half duration).
    #[serde(default)]
    pub timing: Option<TimingProfile>,
}

/// Phoneme-level timing control.
///
/// Allows independent scaling of vowel, consonant, and pause durations
/// for fine-grained control over speech rhythm.
///
/// # Examples
///
/// ```
/// use shabda::engine::TimingProfile;
///
/// // Crisp speech: shorter vowels, normal consonants
/// let crisp = TimingProfile::new(0.8, 1.0, 0.7);
///
/// // Deliberate speech: longer vowels, longer pauses
/// let deliberate = TimingProfile::new(1.3, 1.0, 1.5);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TimingProfile {
    /// Scale factor for vowel/diphthong durations (default 1.0).
    pub vowel_scale: f32,
    /// Scale factor for consonant durations (default 1.0).
    pub consonant_scale: f32,
    /// Scale factor for pause/silence durations (default 1.0).
    pub pause_scale: f32,
}

impl TimingProfile {
    /// Creates a new timing profile with the given scale factors.
    #[must_use]
    pub fn new(vowel_scale: f32, consonant_scale: f32, pause_scale: f32) -> Self {
        Self {
            vowel_scale,
            consonant_scale,
            pause_scale,
        }
    }
}

impl Default for TimingProfile {
    fn default() -> Self {
        Self {
            vowel_scale: 1.0,
            consonant_scale: 1.0,
            pause_scale: 1.0,
        }
    }
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

    /// Sets a timing profile for phoneme-level duration control.
    #[must_use]
    pub fn with_timing(mut self, timing: TimingProfile) -> Self {
        self.timing = Some(timing);
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
            Language::Spanish
            | Language::German
            | Language::Hindi
            | Language::Arabic
            | Language::Sanskrit => PronunciationDict::new(),
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

    /// Returns the varna phoneme inventory for the active language.
    ///
    /// The inventory describes all valid phonemes for this language,
    /// including articulatory features and stress pattern.
    ///
    /// # Examples
    ///
    /// ```
    /// use shabda::prelude::*;
    ///
    /// let g2p = G2PEngine::new(Language::English);
    /// let inv = g2p.phoneme_inventory();
    /// assert!(inv.has("p"));
    /// assert!(inv.has("ʃ"));
    /// ```
    #[cfg(feature = "varna")]
    #[must_use]
    pub fn phoneme_inventory(&self) -> varna::phoneme::PhonemeInventory {
        crate::validate::inventory_for(self.language)
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

            // Collect preceding content words for heteronym context
            let preceding: Vec<&str> = words[..i]
                .iter()
                .rev()
                .filter(|w| {
                    **w != normalize::COMMA_PAUSE
                        && **w != normalize::PERIOD_PAUSE
                        && **w != normalize::EMPHASIS_START
                        && **w != normalize::EMPHASIS_END
                })
                .take(3)
                .copied()
                .collect();

            // Look up in dictionary first, fall back to rules
            let phonemes: Vec<Phoneme> = if let Some(rule) = crate::heteronym::lookup(word) {
                // Heteronym: select variant based on context
                if let Some(prons) = self.dictionary.lookup_all(word) {
                    trace!(word, variant_count = prons.len(), "heteronym lookup");
                    crate::heteronym::select_phonemes(rule, &preceding, prons).to_vec()
                } else if let Some(dict_entry) = self.dictionary.lookup(word) {
                    dict_entry.to_vec()
                } else {
                    match self.language {
                        Language::English => rules::english_rules(word),
                        Language::Spanish => rules::spanish_rules(word),
                        Language::German => rules::german_rules(word),
                        Language::Hindi => rules::hindi_rules(word),
                        Language::Arabic => rules::arabic_rules(word),
                        Language::Sanskrit => rules::sanskrit_rules(word),
                    }
                }
            } else if let Some(dict_entry) = self.dictionary.lookup(word) {
                trace!(word, phoneme_count = dict_entry.len(), "dictionary hit");
                dict_entry.to_vec()
            } else if normalize::is_foreign_word(word) {
                // Foreign word: strip diacritics and try rules
                trace!(word, "foreign word detected, stripping diacritics");
                let stripped = normalize::strip_diacritics(word);
                if let Some(dict_entry) = self.dictionary.lookup(&stripped) {
                    dict_entry.to_vec()
                } else {
                    match self.language {
                        Language::English => rules::english_rules(&stripped),
                        Language::Spanish => rules::spanish_rules(&stripped),
                        Language::German => rules::german_rules(&stripped),
                        Language::Hindi => rules::hindi_rules(&stripped),
                        Language::Arabic => rules::arabic_rules(&stripped),
                        Language::Sanskrit => rules::sanskrit_rules(&stripped),
                    }
                }
            } else {
                trace!(word, "dictionary miss, falling back to rules");
                match self.language {
                    Language::English => rules::english_rules(word),
                    Language::Spanish => rules::spanish_rules(word),
                    Language::German => rules::german_rules(word),
                    Language::Hindi => rules::hindi_rules(word),
                    Language::Arabic => rules::arabic_rules(word),
                    Language::Sanskrit => rules::sanskrit_rules(word),
                }
            };

            // Validate phoneme output against varna inventory in debug builds
            #[cfg(feature = "varna")]
            {
                let invalid = crate::validate::validate_phonemes_for(
                    &phonemes,
                    &varna_inventory,
                    self.language,
                );
                debug_assert!(
                    invalid.is_empty(),
                    "word {word:?} produced phonemes not in varna inventory: {invalid:?}"
                );
                let violations = crate::validate::validate_phonotactics(&phonemes, self.language);
                debug_assert!(
                    violations.is_empty(),
                    "word {word:?} has phonotactic violations: {violations:?}"
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

        // Apply timing profile
        if let Some(ref timing) = options.timing {
            prosody::apply_timing(&mut events, timing);
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

    /// Converts SSML-formatted text to a sequence of phoneme events.
    ///
    /// Parses the SSML markup and applies `<break>`, `<emphasis>`, and
    /// `<prosody>` elements to control the G2P pipeline.
    ///
    /// # Errors
    ///
    /// Returns `ShabdaError::InvalidInput` if the SSML is malformed or empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use shabda::prelude::*;
    ///
    /// let g2p = G2PEngine::new(Language::English);
    /// let events = g2p.convert_ssml(
    ///     "Hello <break time=\"300ms\"/> <emphasis level=\"strong\">world</emphasis>"
    /// ).unwrap();
    /// assert!(!events.is_empty());
    /// ```
    pub fn convert_ssml(&self, ssml: &str) -> Result<Vec<PhonemeEvent>> {
        if ssml.trim().is_empty() {
            return Err(ShabdaError::InvalidInput("empty SSML".to_string()));
        }

        let nodes = crate::ssml::parse(ssml)
            .map_err(|e| ShabdaError::InvalidInput(alloc::format!("SSML parse error: {e}")))?;

        let mut events = Vec::new();
        self.render_ssml_nodes(&nodes, &ConvertOptions::default(), &mut events)?;
        Ok(events)
    }

    /// Recursively renders SSML nodes into phoneme events.
    fn render_ssml_nodes(
        &self,
        nodes: &[crate::ssml::SsmlNode],
        base_options: &ConvertOptions,
        events: &mut Vec<PhonemeEvent>,
    ) -> Result<()> {
        use crate::ssml::SsmlNode;

        for node in nodes {
            match node {
                SsmlNode::Text(text) => {
                    if !text.trim().is_empty() {
                        let mut text_events = self.convert_with(text, base_options)?;
                        events.append(&mut text_events);
                    }
                }
                SsmlNode::Break { duration_ms } => {
                    let duration_secs = *duration_ms as f32 / 1000.0;
                    events.push(PhonemeEvent::new(
                        Phoneme::Silence,
                        duration_secs,
                        svara::prosody::Stress::Unstressed,
                    ));
                }
                SsmlNode::Emphasis { level, children } => {
                    let emphasis_opts = ConvertOptions {
                        emphasis: true,
                        ..base_options.clone()
                    };
                    // Convert children with emphasis, then boost based on level
                    let start_idx = events.len();
                    self.render_ssml_nodes(children, &emphasis_opts, events)?;
                    let emphasis_events = &mut events[start_idx..];
                    // Apply emphasis — convert_with already does this for emphasis=true,
                    // but we also scale by level
                    match level {
                        crate::ssml::EmphasisLevel::Strong => {
                            prosody::apply_emphasis(emphasis_events);
                        }
                        crate::ssml::EmphasisLevel::Moderate => {
                            // Moderate: lighter emphasis already handled by emphasis=true
                        }
                        crate::ssml::EmphasisLevel::Reduced => {
                            // De-stress: set all to unstressed
                            for event in emphasis_events.iter_mut() {
                                event.stress = svara::prosody::Stress::Unstressed;
                            }
                        }
                    }
                }
                SsmlNode::Prosody { rate, children } => {
                    let prosody_opts = if let Some(r) = rate {
                        ConvertOptions {
                            speaking_rate_wpm: Some(r.wpm()),
                            ..base_options.clone()
                        }
                    } else {
                        base_options.clone()
                    };
                    self.render_ssml_nodes(children, &prosody_opts, events)?;
                }
            }
        }
        Ok(())
    }

    /// Converts text word-by-word, calling a callback after each word.
    ///
    /// Useful for real-time or streaming applications where phoneme events
    /// should be processed incrementally rather than buffered.
    ///
    /// The callback receives `(word, phoneme_events)` for each content word.
    /// Pause markers are delivered as words with silence events.
    ///
    /// # Errors
    ///
    /// Returns `ShabdaError::InvalidInput` if the text is empty, or propagates
    /// any error from the callback (via `ShabdaError::RuleError`).
    ///
    /// # Examples
    ///
    /// ```
    /// use shabda::prelude::*;
    ///
    /// let g2p = G2PEngine::new(Language::English);
    /// let mut word_count = 0;
    /// g2p.convert_streaming("hello world", |_word, _events| {
    ///     word_count += 1;
    /// }).unwrap();
    /// assert!(word_count >= 2);
    /// ```
    pub fn convert_streaming<F>(&self, text: &str, mut callback: F) -> Result<()>
    where
        F: FnMut(&str, &[PhonemeEvent]),
    {
        if text.trim().is_empty() {
            return Err(ShabdaError::InvalidInput("empty text".to_string()));
        }

        let normalized = normalize::normalize(text);
        let words: Vec<&str> = normalized.split_whitespace().collect();

        for word in &words {
            if *word == normalize::COMMA_PAUSE {
                let events = [PhonemeEvent::new(
                    Phoneme::Silence,
                    0.15,
                    svara::prosody::Stress::Unstressed,
                )];
                callback(word, &events);
                continue;
            }
            if *word == normalize::PERIOD_PAUSE {
                let events = [PhonemeEvent::new(
                    Phoneme::Silence,
                    0.30,
                    svara::prosody::Stress::Unstressed,
                )];
                callback(word, &events);
                continue;
            }

            let phonemes: Vec<Phoneme> = if let Some(dict_entry) = self.dictionary.lookup(word) {
                dict_entry.to_vec()
            } else {
                match self.language {
                    Language::English => rules::english_rules(word),
                    Language::Spanish => rules::spanish_rules(word),
                    Language::German => rules::german_rules(word),
                    Language::Hindi => rules::hindi_rules(word),
                    Language::Arabic => rules::arabic_rules(word),
                    Language::Sanskrit => rules::sanskrit_rules(word),
                }
            };

            if phonemes.is_empty() {
                continue;
            }

            let is_content = prosody::is_content_word(word);
            let syllables = crate::syllable::syllabify(&phonemes);
            let word_events = if syllables.is_empty() {
                prosody::assign_stress(&phonemes, is_content)
            } else {
                prosody::assign_stress_syllabic(&syllables, is_content)
            };

            callback(word, &word_events);
        }

        Ok(())
    }
}
