//! Rule-based grapheme-to-phoneme conversion for English.
//!
//! When a word is not in the dictionary, these rules convert letter patterns
//! to phoneme sequences. English G2P rules are ~90% regular — the dictionary
//! handles the remaining irregular words.

use alloc::vec::Vec;
use svara::phoneme::Phoneme;

/// Converts a single English word to phonemes using letter-to-sound rules.
///
/// This is the fallback when dictionary lookup fails. Rules are applied
/// left-to-right, matching the longest pattern first.
#[must_use]
pub fn english_rules(word: &str) -> Vec<Phoneme> {
    let chars: Vec<char> = word.to_lowercase().chars().collect();
    let mut phonemes = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        let remaining = &chars[i..];
        let (consumed, ph) = match_pattern(remaining);
        phonemes.extend_from_slice(&ph);
        i += consumed;
    }

    phonemes
}

/// Matches the longest letter pattern at the current position.
/// Returns (chars_consumed, phonemes_produced).
fn match_pattern(chars: &[char]) -> (usize, Vec<Phoneme>) {
    if chars.is_empty() {
        return (1, Vec::new());
    }

    // Try 3-letter patterns first
    if chars.len() >= 3 {
        match (chars[0], chars[1], chars[2]) {
            ('t', 'h', 'r') => {
                return (3, alloc::vec![Phoneme::FricativeTh, Phoneme::ApproximantR]);
            }
            ('t', 'h', 'e') => return (3, alloc::vec![Phoneme::FricativeDh, Phoneme::VowelSchwa]),
            ('s', 'h', _) => return (2, alloc::vec![Phoneme::FricativeSh]),
            ('c', 'h', _) => return (2, alloc::vec![Phoneme::AffricateCh]),
            ('t', 'h', _) => return (2, alloc::vec![Phoneme::FricativeTh]),
            ('n', 'g', _) if !chars[2].is_alphabetic() || chars.len() == 3 => {
                return (2, alloc::vec![Phoneme::NasalNg]);
            }
            ('i', 'n', 'g') if chars.len() == 3 => {
                return (3, alloc::vec![Phoneme::VowelNearI, Phoneme::NasalNg]);
            }
            ('o', 'u', _) => return (2, alloc::vec![Phoneme::DiphthongAU]),
            ('o', 'w', _) => return (2, alloc::vec![Phoneme::DiphthongOU]),
            ('a', 'i', _) => return (2, alloc::vec![Phoneme::DiphthongAI]),
            ('e', 'e', _) => return (2, alloc::vec![Phoneme::VowelE]),
            ('o', 'o', _) => return (2, alloc::vec![Phoneme::VowelU]),
            ('e', 'a', _) => return (2, alloc::vec![Phoneme::VowelE]),
            _ => {}
        }
    }

    // Try 2-letter patterns
    if chars.len() >= 2 {
        match (chars[0], chars[1]) {
            ('s', 'h') => return (2, alloc::vec![Phoneme::FricativeSh]),
            ('c', 'h') => return (2, alloc::vec![Phoneme::AffricateCh]),
            ('t', 'h') => return (2, alloc::vec![Phoneme::FricativeTh]),
            ('w', 'h') => return (2, alloc::vec![Phoneme::ApproximantW]),
            ('p', 'h') => return (2, alloc::vec![Phoneme::FricativeF]),
            ('c', 'k') => return (2, alloc::vec![Phoneme::PlosiveK]),
            ('q', 'u') => return (2, alloc::vec![Phoneme::PlosiveK, Phoneme::ApproximantW]),
            _ => {}
        }
    }

    // Single letter patterns
    let ph = match chars[0] {
        'a' => alloc::vec![Phoneme::VowelAsh],
        'b' => alloc::vec![Phoneme::PlosiveB],
        'c' => {
            // 'c' before e/i/y is /s/, otherwise /k/
            if chars.len() > 1 && matches!(chars[1], 'e' | 'i' | 'y') {
                alloc::vec![Phoneme::FricativeS]
            } else {
                alloc::vec![Phoneme::PlosiveK]
            }
        }
        'd' => alloc::vec![Phoneme::PlosiveD],
        'e' => {
            // Silent 'e' at end of word
            if chars.len() == 1 {
                alloc::vec![]
            } else {
                alloc::vec![Phoneme::VowelOpenE]
            }
        }
        'f' => alloc::vec![Phoneme::FricativeF],
        'g' => {
            if chars.len() > 1 && matches!(chars[1], 'e' | 'i' | 'y') {
                alloc::vec![Phoneme::AffricateJ]
            } else {
                alloc::vec![Phoneme::PlosiveG]
            }
        }
        'h' => alloc::vec![Phoneme::FricativeH],
        'i' => alloc::vec![Phoneme::VowelNearI],
        'j' => alloc::vec![Phoneme::AffricateJ],
        'k' => alloc::vec![Phoneme::PlosiveK],
        'l' => alloc::vec![Phoneme::LateralL],
        'm' => alloc::vec![Phoneme::NasalM],
        'n' => alloc::vec![Phoneme::NasalN],
        'o' => alloc::vec![Phoneme::VowelO],
        'p' => alloc::vec![Phoneme::PlosiveP],
        'r' => alloc::vec![Phoneme::ApproximantR],
        's' => alloc::vec![Phoneme::FricativeS],
        't' => alloc::vec![Phoneme::PlosiveT],
        'u' => alloc::vec![Phoneme::VowelCupV],
        'v' => alloc::vec![Phoneme::FricativeV],
        'w' => alloc::vec![Phoneme::ApproximantW],
        'x' => alloc::vec![Phoneme::PlosiveK, Phoneme::FricativeS],
        'y' => {
            // 'y' at start = consonant, elsewhere = vowel
            if chars.len() > 1 {
                alloc::vec![Phoneme::ApproximantJ]
            } else {
                alloc::vec![Phoneme::VowelE]
            }
        }
        'z' => alloc::vec![Phoneme::FricativeZ],
        '\'' => alloc::vec![],                // Apostrophe: skip
        '-' => alloc::vec![Phoneme::Silence], // Hyphen: brief pause
        _ => alloc::vec![],                   // Unknown: skip
    };

    (1, ph)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_word() {
        let phonemes = english_rules("cat");
        assert!(!phonemes.is_empty());
        assert_eq!(phonemes[0], Phoneme::PlosiveK); // 'c' before 'a' = /k/
    }

    #[test]
    fn test_digraph_sh() {
        let phonemes = english_rules("she");
        assert_eq!(phonemes[0], Phoneme::FricativeSh);
    }

    #[test]
    fn test_digraph_th() {
        let phonemes = english_rules("the");
        assert_eq!(phonemes[0], Phoneme::FricativeDh);
    }

    #[test]
    fn test_digraph_ch() {
        let phonemes = english_rules("chat");
        assert_eq!(phonemes[0], Phoneme::AffricateCh);
    }

    #[test]
    fn test_empty() {
        let phonemes = english_rules("");
        assert!(phonemes.is_empty());
    }
}
