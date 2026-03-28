//! Rule-based grapheme-to-phoneme conversion for English.
//!
//! When a word is not in the dictionary, these rules convert letter patterns
//! to phoneme sequences. The pipeline:
//!
//! 1. Strip silent letter patterns (kn→n, wr→r, etc.)
//! 2. Detect and strip morphological prefixes (un-, re-, dis-)
//! 3. Apply left-to-right pattern matching with context-sensitive vowels
//! 4. Post-process morphological suffixes (-ed, -tion, -sion)

use alloc::vec::Vec;
use svara::phoneme::Phoneme;
use tracing::trace;

/// Converts a single English word to phonemes using letter-to-sound rules.
///
/// This is the fallback when dictionary lookup fails. Handles silent letters,
/// morphological affixes, context-sensitive vowels, and common digraphs.
#[must_use]
pub fn english_rules(word: &str) -> Vec<Phoneme> {
    let chars: Vec<char> = word.to_lowercase().chars().collect();
    if chars.is_empty() {
        return Vec::new();
    }

    // Step 1: Strip silent letters
    let chars = preprocess_silent(&chars);
    trace!(word, "silent letter preprocessing complete");

    // Step 2: Detect and strip prefix
    let (prefix_phonemes, stem) = strip_prefix(&chars);
    if prefix_phonemes.is_some() {
        trace!(word, "prefix detected and stripped");
    }

    // Step 3: Detect suffix for post-processing
    let (stem, suffix) = detect_suffix(stem);

    // Step 4: Main left-to-right pattern matching on stem
    let mut phonemes = Vec::new();
    if let Some(pp) = prefix_phonemes {
        phonemes.extend_from_slice(&pp);
    }

    let mut i = 0;
    while i < stem.len() {
        let remaining = &stem[i..];
        let (consumed, ph) = match_pattern(remaining);
        phonemes.extend_from_slice(&ph);
        i += consumed;
    }

    // Step 5: Post-process suffix
    apply_suffix(&mut phonemes, suffix);

    phonemes
}

// --- Step 1: Silent letter pre-processing ---

/// Strips silent letter patterns from the character array.
fn preprocess_silent(chars: &[char]) -> Vec<char> {
    let mut result = chars.to_vec();

    // Word-initial silent patterns
    if result.len() >= 2 {
        let skip_first = matches!(
            (result[0], result[1]),
            ('k', 'n') | ('g', 'n') | ('w', 'r') | ('p', 's') | ('p', 'n') | ('m', 'n')
        );
        if skip_first {
            result.remove(0);
        }
    }

    // Word-final silent patterns (only at absolute end)
    if result.len() >= 2 {
        let len = result.len();
        let strip_last = matches!(
            (result[len - 2], result[len - 1]),
            ('m', 'b') | ('b', 't') | ('m', 'n')
        );
        if strip_last {
            result.pop();
        }
        // gn at word-final (sign, design) — strip the g
        let len = result.len();
        if len >= 2 && result[len - 2] == 'g' && result[len - 1] == 'n' {
            result.remove(len - 2);
        }
    }

    result
}

// --- Step 2: Prefix stripping ---

/// Detects and strips a morphological prefix.
/// Returns (prefix_phonemes, remaining_stem).
fn strip_prefix(chars: &[char]) -> (Option<Vec<Phoneme>>, &[char]) {
    let word: alloc::string::String = chars.iter().collect();

    // Only strip if remainder is >= 4 chars and starts with a consonant
    let prefixes: &[(&str, &[Phoneme])] = &[
        ("un", &[Phoneme::VowelCupV, Phoneme::NasalN]),
        (
            "dis",
            &[Phoneme::PlosiveD, Phoneme::VowelNearI, Phoneme::FricativeS],
        ),
        (
            "mis",
            &[Phoneme::NasalM, Phoneme::VowelNearI, Phoneme::FricativeS],
        ),
        (
            "pre",
            &[Phoneme::PlosiveP, Phoneme::ApproximantR, Phoneme::VowelE],
        ),
        ("re", &[Phoneme::ApproximantR, Phoneme::VowelE]),
    ];

    for (prefix, phonemes) in prefixes {
        if word.starts_with(prefix) {
            let remainder = &chars[prefix.len()..];
            if remainder.len() >= 4 && !is_vowel_char(remainder[0]) {
                return (Some(phonemes.to_vec()), remainder);
            }
        }
    }

    (None, chars)
}

// --- Step 3: Suffix detection ---

#[derive(Debug, Clone, Copy)]
enum Suffix {
    None,
    Ed,
}

/// Detects a morphological suffix and returns the stem without it.
fn detect_suffix(chars: &[char]) -> (&[char], Suffix) {
    // -ed: only when preceded by a consonant or vowel+consonant (not just "ed")
    if chars.len() >= 3 && chars[chars.len() - 2] == 'e' && chars[chars.len() - 1] == 'd' {
        // Don't strip -ed from short words like "bed", "red", "fed"
        if chars.len() >= 4 {
            return (&chars[..chars.len() - 2], Suffix::Ed);
        }
    }
    (chars, Suffix::None)
}

/// Applies suffix post-processing based on the stem's final phoneme.
fn apply_suffix(phonemes: &mut Vec<Phoneme>, suffix: Suffix) {
    match suffix {
        Suffix::None => {}
        Suffix::Ed => {
            let last = phonemes.last().copied();
            match last {
                // After /t/ or /d/: -ed = /ɪd/
                Some(Phoneme::PlosiveT | Phoneme::PlosiveD) => {
                    phonemes.push(Phoneme::VowelNearI);
                    phonemes.push(Phoneme::PlosiveD);
                }
                // After voiceless consonants: -ed = /t/
                Some(
                    Phoneme::PlosiveP
                    | Phoneme::PlosiveK
                    | Phoneme::FricativeF
                    | Phoneme::FricativeS
                    | Phoneme::FricativeSh
                    | Phoneme::AffricateCh
                    | Phoneme::FricativeTh,
                ) => {
                    phonemes.push(Phoneme::PlosiveT);
                }
                // After voiced consonants and vowels: -ed = /d/
                _ => {
                    phonemes.push(Phoneme::PlosiveD);
                }
            }
        }
    }
}

// --- Step 4: Pattern matching ---

/// Matches the longest letter pattern at the current position.
/// Returns (chars_consumed, phonemes_produced).
fn match_pattern(chars: &[char]) -> (usize, Vec<Phoneme>) {
    if chars.is_empty() {
        return (1, Vec::new());
    }

    // Try 4-letter patterns
    if chars.len() >= 4 {
        match (chars[0], chars[1], chars[2], chars[3]) {
            ('e', 'i', 'g', 'h') => return (4, alloc::vec![Phoneme::DiphthongEI]),
            ('a', 'u', 'g', 'h') => return (4, alloc::vec![Phoneme::VowelOpenO]),
            ('o', 'u', 'g', 'h') => return (4, alloc::vec![Phoneme::DiphthongOU]),
            ('t', 'i', 'o', 'n') => {
                return (
                    4,
                    alloc::vec![Phoneme::FricativeSh, Phoneme::VowelSchwa, Phoneme::NasalN],
                );
            }
            ('s', 'i', 'o', 'n') => {
                return (
                    4,
                    alloc::vec![Phoneme::FricativeZh, Phoneme::VowelSchwa, Phoneme::NasalN],
                );
            }
            ('c', 'i', 'a', 'n') => {
                return (
                    4,
                    alloc::vec![Phoneme::FricativeSh, Phoneme::VowelSchwa, Phoneme::NasalN],
                );
            }
            _ => {}
        }
    }

    // Try 3-letter patterns
    if chars.len() >= 3 {
        match (chars[0], chars[1], chars[2]) {
            // Magic-e: VCe at word end (remaining == exactly 3 chars)
            (v, c, 'e') if chars.len() == 3 && is_vowel_char(v) && !is_vowel_char(c) => {
                let vowel = magic_e_vowel(v);
                let cons = single_consonant(c);
                let mut ph = vowel;
                ph.extend_from_slice(&cons);
                return (3, ph);
            }
            // R-colored 3-letter vowels
            ('a', 'i', 'r') => return (3, alloc::vec![Phoneme::VowelOpenE, Phoneme::ApproximantR]),
            ('e', 'a', 'r') => return (3, alloc::vec![Phoneme::VowelNearI, Phoneme::ApproximantR]),
            ('o', 'u', 'r') => {
                return (3, alloc::vec![Phoneme::DiphthongAU, Phoneme::ApproximantR]);
            }
            ('i', 'g', 'h') => return (3, alloc::vec![Phoneme::DiphthongAI]),
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
            // R-colored vowels
            ('a', 'r') => return (2, alloc::vec![Phoneme::VowelOpenA, Phoneme::ApproximantR]),
            ('e', 'r') => return (2, alloc::vec![Phoneme::VowelBird, Phoneme::ApproximantR]),
            ('i', 'r') => return (2, alloc::vec![Phoneme::VowelBird, Phoneme::ApproximantR]),
            ('o', 'r') => return (2, alloc::vec![Phoneme::VowelOpenO, Phoneme::ApproximantR]),
            ('u', 'r') => return (2, alloc::vec![Phoneme::VowelBird, Phoneme::ApproximantR]),
            _ => {}
        }
    }

    // Single letter patterns
    let ph = match chars[0] {
        'a' => alloc::vec![Phoneme::VowelAsh],
        'b' => alloc::vec![Phoneme::PlosiveB],
        'c' => {
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
            if chars.len() > 1 {
                alloc::vec![Phoneme::ApproximantJ]
            } else {
                alloc::vec![Phoneme::VowelE]
            }
        }
        'z' => alloc::vec![Phoneme::FricativeZ],
        '\'' => alloc::vec![],
        '-' => alloc::vec![Phoneme::Silence],
        _ => alloc::vec![],
    };

    (1, ph)
}

/// Returns the "long" vowel phoneme for magic-e patterns.
fn magic_e_vowel(v: char) -> Vec<Phoneme> {
    match v {
        'a' => alloc::vec![Phoneme::DiphthongEI],
        'i' => alloc::vec![Phoneme::DiphthongAI],
        'o' => alloc::vec![Phoneme::DiphthongOU],
        'u' => alloc::vec![Phoneme::VowelU],
        'e' => alloc::vec![Phoneme::VowelE],
        _ => alloc::vec![Phoneme::VowelSchwa],
    }
}

/// Returns the phoneme for a single consonant letter (used in magic-e).
fn single_consonant(c: char) -> Vec<Phoneme> {
    match c {
        'b' => alloc::vec![Phoneme::PlosiveB],
        'c' => alloc::vec![Phoneme::PlosiveK],
        'd' => alloc::vec![Phoneme::PlosiveD],
        'f' => alloc::vec![Phoneme::FricativeF],
        'g' => alloc::vec![Phoneme::PlosiveG],
        'k' => alloc::vec![Phoneme::PlosiveK],
        'l' => alloc::vec![Phoneme::LateralL],
        'm' => alloc::vec![Phoneme::NasalM],
        'n' => alloc::vec![Phoneme::NasalN],
        'p' => alloc::vec![Phoneme::PlosiveP],
        'r' => alloc::vec![Phoneme::ApproximantR],
        's' => alloc::vec![Phoneme::FricativeS],
        't' => alloc::vec![Phoneme::PlosiveT],
        'v' => alloc::vec![Phoneme::FricativeV],
        'z' => alloc::vec![Phoneme::FricativeZ],
        _ => alloc::vec![],
    }
}

fn is_vowel_char(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u')
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Existing tests ---

    #[test]
    fn test_simple_word() {
        let phonemes = english_rules("cat");
        assert!(!phonemes.is_empty());
        assert_eq!(phonemes[0], Phoneme::PlosiveK);
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

    // --- Silent letter tests ---

    #[test]
    fn test_silent_kn() {
        let phonemes = english_rules("knight");
        assert_ne!(phonemes[0], Phoneme::PlosiveK);
        assert_eq!(phonemes[0], Phoneme::NasalN);
    }

    #[test]
    fn test_silent_gn_initial() {
        let phonemes = english_rules("gnome");
        assert_eq!(phonemes[0], Phoneme::NasalN);
    }

    #[test]
    fn test_silent_wr() {
        let phonemes = english_rules("write");
        assert_eq!(phonemes[0], Phoneme::ApproximantR);
    }

    #[test]
    fn test_silent_ps() {
        let phonemes = english_rules("psalm");
        assert_eq!(phonemes[0], Phoneme::FricativeS);
    }

    #[test]
    fn test_silent_mb_final() {
        let phonemes = english_rules("lamb");
        assert!(!phonemes.contains(&Phoneme::PlosiveB));
    }

    #[test]
    fn test_ghost_not_silent() {
        // gh at word-initial is NOT silent (handled by normal g rule)
        let phonemes = english_rules("go");
        assert_eq!(phonemes[0], Phoneme::PlosiveG);
    }

    #[test]
    fn test_igh_pattern() {
        let phonemes = english_rules("high");
        assert!(phonemes.contains(&Phoneme::DiphthongAI));
    }

    // --- Context-sensitive vowel tests ---

    #[test]
    fn test_magic_e_make() {
        let phonemes = english_rules("make");
        assert!(phonemes.contains(&Phoneme::DiphthongEI));
    }

    #[test]
    fn test_magic_e_time() {
        let phonemes = english_rules("time");
        assert!(phonemes.contains(&Phoneme::DiphthongAI));
    }

    #[test]
    fn test_magic_e_home() {
        let phonemes = english_rules("home");
        assert!(phonemes.contains(&Phoneme::DiphthongOU));
    }

    #[test]
    fn test_r_colored_car() {
        let phonemes = english_rules("car");
        assert!(phonemes.contains(&Phoneme::VowelOpenA));
        assert!(phonemes.contains(&Phoneme::ApproximantR));
    }

    #[test]
    fn test_r_colored_bird() {
        let phonemes = english_rules("bird");
        assert!(phonemes.contains(&Phoneme::VowelBird));
    }

    // --- Morphological tests ---

    #[test]
    fn test_tion_suffix() {
        let phonemes = english_rules("nation");
        assert!(phonemes.contains(&Phoneme::FricativeSh));
    }

    #[test]
    fn test_ed_after_voiceless() {
        // "walked" -> stem "walk" ends in /k/ (voiceless) -> -ed = /t/
        let phonemes = english_rules("walked");
        let last = phonemes.last().copied();
        assert_eq!(last, Some(Phoneme::PlosiveT));
    }

    #[test]
    fn test_ed_after_t() {
        // "wanted" -> stem "want" ends in /t/ -> -ed = /ɪd/
        let phonemes = english_rules("wanted");
        let len = phonemes.len();
        assert!(len >= 2);
        assert_eq!(phonemes[len - 2], Phoneme::VowelNearI);
        assert_eq!(phonemes[len - 1], Phoneme::PlosiveD);
    }

    #[test]
    fn test_ed_after_voiced() {
        // "played" -> stem "play" ends in vowel (DiphthongEI) -> -ed = /d/
        let phonemes = english_rules("played");
        let last = phonemes.last().copied();
        assert_eq!(last, Some(Phoneme::PlosiveD));
    }

    #[test]
    fn test_prefix_un() {
        let phonemes = english_rules("unhappy");
        // Should start with /ʌn/
        assert_eq!(phonemes[0], Phoneme::VowelCupV);
        assert_eq!(phonemes[1], Phoneme::NasalN);
    }
}
