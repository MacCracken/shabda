//! Prosody assignment from text features.
//!
//! Maps sentence structure and punctuation to stress patterns and
//! intonation, producing svara-compatible prosody parameters.

use alloc::vec::Vec;
use svara::phoneme::Phoneme;
use svara::prosody::Stress;
use svara::sequence::PhonemeEvent;

use crate::normalize::SentenceType;
use crate::syllable::Syllable;

/// Converts a sequence of phonemes for a word into PhonemeEvents with stress.
///
/// Applies simple stress rules:
/// - First vowel in a content word gets primary stress
/// - Other vowels get secondary or unstressed
/// - Consonants are unstressed
#[must_use]
pub fn assign_stress(phonemes: &[Phoneme], is_content_word: bool) -> Vec<PhonemeEvent> {
    let mut events = Vec::with_capacity(phonemes.len());
    let mut found_primary = false;

    for &ph in phonemes {
        let dur = svara::phoneme::phoneme_duration(&ph);
        let stress = if !is_content_word {
            Stress::Unstressed
        } else if is_vowel_like(&ph) && !found_primary {
            found_primary = true;
            Stress::Primary
        } else if is_vowel_like(&ph) {
            Stress::Secondary
        } else {
            Stress::Unstressed
        };
        events.push(PhonemeEvent::new(ph, dur, stress));
    }

    events
}

/// Assigns stress using syllable weight analysis.
///
/// Uses a simplified English stress rule:
/// - Monosyllables: primary stress
/// - Two syllables: stress first
/// - Heavy penult: stress penult
/// - Otherwise: stress antepenult
/// - Function words: all unstressed
#[must_use]
pub fn assign_stress_syllabic(syllables: &[Syllable], is_content_word: bool) -> Vec<PhonemeEvent> {
    if syllables.is_empty() {
        return Vec::new();
    }

    // Determine which syllable index gets primary stress
    let primary_idx = if !is_content_word {
        usize::MAX // no primary stress for function words
    } else if syllables.len() <= 2 {
        0 // Monosyllables and 2-syllable words: stress first
    } else {
        // 3+ syllables: check penult weight
        let penult = syllables.len() - 2;
        if syllables[penult].is_heavy() {
            penult
        } else {
            // Stress antepenult (third from end)
            syllables.len().saturating_sub(3)
        }
    };

    let mut events = Vec::new();
    for (syl_idx, syllable) in syllables.iter().enumerate() {
        let syl_stress = if syl_idx == primary_idx {
            Stress::Primary
        } else {
            Stress::Unstressed
        };

        // Onset: unstressed
        for &ph in &syllable.onset {
            let dur = svara::phoneme::phoneme_duration(&ph);
            events.push(PhonemeEvent::new(ph, dur, Stress::Unstressed));
        }
        // Nucleus: carries the syllable's stress
        let dur = svara::phoneme::phoneme_duration(&syllable.nucleus);
        events.push(PhonemeEvent::new(syllable.nucleus, dur, syl_stress));
        // Coda: unstressed
        for &ph in &syllable.coda {
            let dur = svara::phoneme::phoneme_duration(&ph);
            events.push(PhonemeEvent::new(ph, dur, Stress::Unstressed));
        }
    }

    events
}

/// Returns true if the phoneme is a vowel or diphthong (stress-bearing).
#[must_use]
fn is_vowel_like(ph: &Phoneme) -> bool {
    use svara::phoneme::PhonemeClass;
    matches!(ph.class(), PhonemeClass::Vowel | PhonemeClass::Diphthong)
}

/// Returns true if the word is likely a content word (noun, verb, adjective, adverb).
///
/// Function words (the, a, is, etc.) get reduced stress.
#[must_use]
pub fn is_content_word(word: &str) -> bool {
    // Common English function words that get reduced stress
    !matches!(
        word.to_lowercase().as_str(),
        "a" | "an"
            | "the"
            | "is"
            | "am"
            | "are"
            | "was"
            | "were"
            | "be"
            | "been"
            | "to"
            | "of"
            | "in"
            | "on"
            | "at"
            | "by"
            | "for"
            | "and"
            | "or"
            | "but"
            | "if"
            | "it"
            | "he"
            | "she"
            | "we"
            | "they"
            | "that"
            | "this"
            | "with"
            | "not"
            | "do"
            | "did"
            | "has"
            | "had"
            | "have"
    )
}

/// Maps sentence type to svara intonation pattern.
#[must_use]
pub fn sentence_intonation(sentence_type: SentenceType) -> svara::prosody::IntonationPattern {
    match sentence_type {
        SentenceType::Statement => svara::prosody::IntonationPattern::Declarative,
        SentenceType::Question => svara::prosody::IntonationPattern::Interrogative,
        SentenceType::Exclamation => svara::prosody::IntonationPattern::Exclamatory,
    }
}
