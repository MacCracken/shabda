//! Phoneme inventory validation via varna.
//!
//! When the `varna` feature is enabled, this module provides tools to verify
//! that phoneme output from G2P rules is valid for the target language's
//! phoneme inventory as defined by varna.

use alloc::string::String;
use alloc::vec::Vec;
use svara::phoneme::Phoneme;
use varna::phoneme::PhonemeInventory;

use crate::engine::Language;

/// Maps a svara `Phoneme` to its IPA string as used in varna's English inventory.
///
/// Returns `None` for phonemes that have no single-IPA equivalent in varna's
/// current inventory system (e.g., `Silence`), or for phonemes that are valid
/// English sounds but not yet represented in varna 1.0's English inventory
/// (diphthongs `aɪ`, `aʊ`, `ɔɪ` and the NURSE vowel `ɜː`).
///
/// For non-English languages, use [`phoneme_to_ipa_for`] instead.
#[must_use]
pub fn phoneme_to_ipa(phoneme: Phoneme) -> Option<&'static str> {
    match phoneme {
        // Vowels
        Phoneme::VowelA => Some("ɑː"),
        Phoneme::VowelE => Some("iː"),
        Phoneme::VowelI => Some("ɪ"),
        Phoneme::VowelO => Some("oʊ"),
        Phoneme::VowelU => Some("uː"),
        Phoneme::VowelSchwa => Some("ə"),
        Phoneme::VowelOpenO => Some("ɔː"),
        Phoneme::VowelAsh => Some("æ"),
        Phoneme::VowelNearI => Some("ɪ"),
        Phoneme::VowelNearU => Some("ʊ"),
        Phoneme::VowelOpenA => Some("ɑː"),
        Phoneme::VowelOpenE => Some("ɛ"),
        Phoneme::VowelCupV => Some("ʌ"),
        Phoneme::VowelLongI => Some("iː"),
        // NURSE vowel (ɜː) — valid English, not yet in varna 1.0 inventory
        Phoneme::VowelBird => None,
        // Diphthongs present in varna's English inventory
        Phoneme::DiphthongEI => Some("eɪ"),
        Phoneme::DiphthongOU => Some("oʊ"),
        // Diphthongs valid in English but not yet in varna 1.0 inventory
        Phoneme::DiphthongAI => None,
        Phoneme::DiphthongAU => None,
        Phoneme::DiphthongOI => None,
        // Plosives
        Phoneme::PlosiveP => Some("p"),
        Phoneme::PlosiveB => Some("b"),
        Phoneme::PlosiveT => Some("t"),
        Phoneme::PlosiveD => Some("d"),
        Phoneme::PlosiveK => Some("k"),
        Phoneme::PlosiveG => Some("ɡ"),
        // Fricatives
        Phoneme::FricativeF => Some("f"),
        Phoneme::FricativeV => Some("v"),
        Phoneme::FricativeS => Some("s"),
        Phoneme::FricativeZ => Some("z"),
        Phoneme::FricativeSh => Some("ʃ"),
        Phoneme::FricativeZh => Some("ʒ"),
        Phoneme::FricativeTh => Some("θ"),
        Phoneme::FricativeDh => Some("ð"),
        Phoneme::FricativeH => Some("h"),
        // Nasals
        Phoneme::NasalM => Some("m"),
        Phoneme::NasalN => Some("n"),
        Phoneme::NasalNg => Some("ŋ"),
        // Affricates
        Phoneme::AffricateCh => Some("t͡ʃ"),
        Phoneme::AffricateJ => Some("d͡ʒ"),
        // Approximants & lateral
        Phoneme::ApproximantR => Some("ɹ"),
        Phoneme::ApproximantW => Some("w"),
        Phoneme::ApproximantJ => Some("j"),
        Phoneme::LateralL => Some("l"),
        // Not in English inventory
        Phoneme::GlottalStop => Some("ʔ"),
        Phoneme::TapFlap => Some("ɾ"),
        Phoneme::Silence => None,
        _ => None,
    }
}

/// Phonemes that are valid English sounds but not yet in varna 1.0's inventory.
///
/// These are skipped during validation to avoid false positives.
pub const VARNA_INVENTORY_GAPS: &[Phoneme] = &[
    Phoneme::DiphthongAI, // PRICE: aɪ
    Phoneme::DiphthongAU, // MOUTH: aʊ
    Phoneme::DiphthongOI, // CHOICE: ɔɪ
    Phoneme::VowelBird,   // NURSE: ɜː
];

/// Maps a svara `Phoneme` to its IPA string for the given language's inventory.
///
/// Different languages use different IPA symbols for the same svara phoneme.
/// For example, `VowelO` maps to "oʊ" (diphthong) in English but "o" in Spanish.
#[must_use]
pub fn phoneme_to_ipa_for(phoneme: Phoneme, language: Language) -> Option<&'static str> {
    match language {
        Language::English => phoneme_to_ipa(phoneme),
        Language::Spanish => spanish_phoneme_to_ipa(phoneme),
    }
}

/// Maps a svara `Phoneme` to its IPA string in varna's Spanish inventory.
fn spanish_phoneme_to_ipa(phoneme: Phoneme) -> Option<&'static str> {
    match phoneme {
        // Spanish vowels (5 pure vowels, no diphthongs in inventory)
        Phoneme::VowelOpenA => Some("a"),
        Phoneme::VowelOpenE => Some("e"),
        Phoneme::VowelNearI => Some("i"),
        Phoneme::VowelO => Some("o"),
        Phoneme::VowelCupV => Some("u"),
        // Plosives
        Phoneme::PlosiveP => Some("p"),
        Phoneme::PlosiveB => Some("b"),
        Phoneme::PlosiveT => Some("t"),
        Phoneme::PlosiveD => Some("d"),
        Phoneme::PlosiveK => Some("k"),
        Phoneme::PlosiveG => Some("ɡ"),
        // Fricatives
        Phoneme::FricativeF => Some("f"),
        Phoneme::FricativeTh => Some("θ"),
        Phoneme::FricativeS => Some("s"),
        Phoneme::FricativeH => Some("x"), // Spanish /x/ (jota)
        // Nasals
        Phoneme::NasalM => Some("m"),
        Phoneme::NasalN => Some("n"),
        Phoneme::NasalNg => Some("ɲ"), // ñ → palatal nasal
        // Liquids & approximants
        Phoneme::LateralL => Some("l"),
        Phoneme::TapFlap => Some("r"),       // Spanish tap
        Phoneme::ApproximantR => Some("rr"), // Spanish trill
        Phoneme::ApproximantJ => Some("j"),
        Phoneme::ApproximantW => Some("w"),
        // Affricate
        Phoneme::AffricateCh => Some("t͡ʃ"),
        // No IPA for these in Spanish
        Phoneme::Silence => None,
        _ => None,
    }
}

/// Returns the varna `PhonemeInventory` for the given language.
#[must_use]
pub fn inventory_for(language: Language) -> PhonemeInventory {
    match language {
        Language::English => varna::phoneme::english(),
        Language::Spanish => varna::phoneme::inventories::spanish(),
    }
}

/// Returns the varna phonotactic constraints for the given language, if available.
///
/// Returns `None` for languages where varna doesn't yet define phonotactics.
#[must_use]
pub fn phonotactics_for(language: Language) -> Option<varna::phoneme::syllable::Phonotactics> {
    match language {
        Language::English => Some(varna::phoneme::syllable::english_phonotactics()),
        Language::Spanish => None, // not yet defined in varna
    }
}

/// Validates a phoneme sequence against phonotactic constraints.
///
/// Checks consecutive consonant pairs in onset and coda positions against
/// the language's phonotactic rules. Returns descriptions of violations found.
/// An empty return means no violations detected.
#[must_use]
pub fn validate_phonotactics(phonemes: &[Phoneme], language: Language) -> Vec<String> {
    let Some(constraints) = phonotactics_for(language) else {
        return Vec::new(); // no constraints available
    };

    let mut violations = Vec::new();

    // Check consecutive consonant sequences against onset constraints
    let mut consonant_run = Vec::new();
    for &ph in phonemes {
        if let Some(ipa) = phoneme_to_ipa(ph) {
            let is_vowel = matches!(
                ph.class(),
                svara::phoneme::PhonemeClass::Vowel | svara::phoneme::PhonemeClass::Diphthong
            ) || ph == Phoneme::Silence;

            if is_vowel {
                // Check if consonant run exceeds max onset
                if consonant_run.len() > constraints.syllable.max_onset as usize {
                    violations.push(alloc::format!(
                        "consonant cluster too long for onset: {}",
                        consonant_run.join("")
                    ));
                }
                consonant_run.clear();
            } else {
                consonant_run.push(String::from(ipa));
            }
        }
    }

    // Check final consonant run against max coda
    if consonant_run.len() > constraints.syllable.max_coda as usize {
        violations.push(alloc::format!(
            "consonant cluster too long for coda: {}",
            consonant_run.join("")
        ));
    }

    violations
}

/// Validates that every phoneme in the slice exists in the given inventory.
///
/// Uses language-aware IPA mapping. Returns a list of phonemes (as IPA strings)
/// that are NOT in the inventory. An empty return means all phonemes are valid.
#[must_use]
pub fn validate_phonemes_for(
    phonemes: &[Phoneme],
    inventory: &PhonemeInventory,
    language: Language,
) -> Vec<String> {
    let mut invalid = Vec::new();
    for &ph in phonemes {
        if let Some(ipa) = phoneme_to_ipa_for(ph, language)
            && !inventory.has(ipa)
        {
            invalid.push(String::from(ipa));
        }
    }
    invalid
}

/// Validates phonemes against the English inventory (backwards-compatible alias).
#[must_use]
pub fn validate_phonemes(phonemes: &[Phoneme], inventory: &PhonemeInventory) -> Vec<String> {
    validate_phonemes_for(phonemes, inventory, Language::English)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_rules_produce_valid_phonemes() {
        let inventory = inventory_for(Language::English);
        let test_words = [
            "cat", "dog", "hello", "world", "knight", "write", "nation", "she", "the", "church",
            "think", "sing", "make", "time", "home", "bird", "car", "unhappy", "walked", "wanted",
        ];
        for word in &test_words {
            let phonemes = crate::rules::english_rules(word);
            let invalid = validate_phonemes(&phonemes, &inventory);
            assert!(
                invalid.is_empty(),
                "word {word:?} produced invalid phonemes: {invalid:?}"
            );
        }
    }

    #[test]
    fn mapped_phonemes_exist_in_inventory() {
        // Verify that every phoneme with an IPA mapping exists in varna's English inventory.
        let inventory = inventory_for(Language::English);
        let rule_phonemes = [
            Phoneme::VowelAsh,
            Phoneme::VowelOpenE,
            Phoneme::VowelNearI,
            Phoneme::VowelO,
            Phoneme::VowelCupV,
            Phoneme::VowelSchwa,
            Phoneme::VowelE,
            Phoneme::VowelU,
            Phoneme::VowelOpenA,
            Phoneme::VowelOpenO,
            Phoneme::DiphthongEI,
            Phoneme::DiphthongOU,
            Phoneme::PlosiveP,
            Phoneme::PlosiveB,
            Phoneme::PlosiveT,
            Phoneme::PlosiveD,
            Phoneme::PlosiveK,
            Phoneme::PlosiveG,
            Phoneme::FricativeF,
            Phoneme::FricativeV,
            Phoneme::FricativeS,
            Phoneme::FricativeZ,
            Phoneme::FricativeSh,
            Phoneme::FricativeZh,
            Phoneme::FricativeTh,
            Phoneme::FricativeDh,
            Phoneme::FricativeH,
            Phoneme::NasalM,
            Phoneme::NasalN,
            Phoneme::NasalNg,
            Phoneme::AffricateCh,
            Phoneme::AffricateJ,
            Phoneme::ApproximantR,
            Phoneme::ApproximantW,
            Phoneme::ApproximantJ,
            Phoneme::LateralL,
        ];
        for ph in &rule_phonemes {
            let ipa = phoneme_to_ipa(*ph).expect("phoneme should have IPA mapping");
            assert!(
                inventory.has(ipa),
                "phoneme {ph:?} (IPA: {ipa:?}) not in varna English inventory"
            );
        }
    }

    #[test]
    fn inventory_gaps_are_documented() {
        // These phonemes are valid English but not in varna 1.0 — verify they return None.
        for ph in VARNA_INVENTORY_GAPS {
            assert!(
                phoneme_to_ipa(*ph).is_none(),
                "phoneme {ph:?} is listed as a gap but has an IPA mapping"
            );
        }
    }

    #[test]
    fn silence_is_skipped_in_validation() {
        let inventory = inventory_for(Language::English);
        let phonemes = [Phoneme::Silence];
        let invalid = validate_phonemes(&phonemes, &inventory);
        assert!(invalid.is_empty());
    }

    #[test]
    fn serde_roundtrip_language() {
        // Language already has serde roundtrip tests elsewhere, but validate module
        // uses it — verify the inventory_for path works with deserialized values.
        let lang: Language = serde_json::from_str("\"English\"").unwrap();
        let inv = inventory_for(lang);
        assert!(inv.has("p"));
    }
}
