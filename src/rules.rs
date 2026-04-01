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
        let result = match_pattern(remaining);
        i += result.consumed();
        phonemes.extend(result);
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
//
// Compiled pattern tables: static slices eliminate per-match Vec allocations.
// Context-sensitive patterns (magic-e, c/g softening) use inline arrays.

// 4-letter pattern constants
static P_EIGH: &[Phoneme] = &[Phoneme::DiphthongEI];
static P_AUGH: &[Phoneme] = &[Phoneme::VowelOpenO];
static P_OUGH: &[Phoneme] = &[Phoneme::DiphthongOU];
static P_TION: &[Phoneme] = &[Phoneme::FricativeSh, Phoneme::VowelSchwa, Phoneme::NasalN];
static P_SION: &[Phoneme] = &[Phoneme::FricativeZh, Phoneme::VowelSchwa, Phoneme::NasalN];

// 3-letter pattern constants
static P_AIR: &[Phoneme] = &[Phoneme::VowelOpenE, Phoneme::ApproximantR];
static P_EAR: &[Phoneme] = &[Phoneme::VowelNearI, Phoneme::ApproximantR];
static P_OUR: &[Phoneme] = &[Phoneme::DiphthongAU, Phoneme::ApproximantR];
static P_IGH: &[Phoneme] = &[Phoneme::DiphthongAI];
static P_THR: &[Phoneme] = &[Phoneme::FricativeTh, Phoneme::ApproximantR];
static P_THE: &[Phoneme] = &[Phoneme::FricativeDh, Phoneme::VowelSchwa];
static P_ING: &[Phoneme] = &[Phoneme::VowelNearI, Phoneme::NasalNg];

// 2-letter pattern constants
static P_SH: &[Phoneme] = &[Phoneme::FricativeSh];
static P_CH: &[Phoneme] = &[Phoneme::AffricateCh];
static P_TH: &[Phoneme] = &[Phoneme::FricativeTh];
static P_WH: &[Phoneme] = &[Phoneme::ApproximantW];
static P_PH: &[Phoneme] = &[Phoneme::FricativeF];
static P_CK: &[Phoneme] = &[Phoneme::PlosiveK];
static P_QU: &[Phoneme] = &[Phoneme::PlosiveK, Phoneme::ApproximantW];
static P_AR: &[Phoneme] = &[Phoneme::VowelOpenA, Phoneme::ApproximantR];
static P_ER: &[Phoneme] = &[Phoneme::VowelBird, Phoneme::ApproximantR];
static P_OR: &[Phoneme] = &[Phoneme::VowelOpenO, Phoneme::ApproximantR];
static P_NG: &[Phoneme] = &[Phoneme::NasalNg];
static P_OU: &[Phoneme] = &[Phoneme::DiphthongAU];
static P_OW: &[Phoneme] = &[Phoneme::DiphthongOU];
static P_AI: &[Phoneme] = &[Phoneme::DiphthongAI];
static P_EE: &[Phoneme] = &[Phoneme::VowelE];
static P_OO: &[Phoneme] = &[Phoneme::VowelU];
static P_EA: &[Phoneme] = &[Phoneme::VowelE];
static P_XS: &[Phoneme] = &[Phoneme::PlosiveK, Phoneme::FricativeS];

/// Result from pattern matching — either a static slice or a dynamic vec.
enum PatternResult {
    Static(usize, &'static [Phoneme]),
    Dynamic(usize, Vec<Phoneme>),
}

impl PatternResult {
    fn consumed(&self) -> usize {
        match self {
            Self::Static(n, _) | Self::Dynamic(n, _) => *n,
        }
    }
}

impl IntoIterator for PatternResult {
    type Item = Phoneme;
    type IntoIter = PatternIter;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Static(_, slice) => PatternIter::Static(slice.iter()),
            Self::Dynamic(_, vec) => PatternIter::Dynamic(vec.into_iter()),
        }
    }
}

enum PatternIter {
    Static(core::slice::Iter<'static, Phoneme>),
    Dynamic(alloc::vec::IntoIter<Phoneme>),
}

impl Iterator for PatternIter {
    type Item = Phoneme;
    fn next(&mut self) -> Option<Phoneme> {
        match self {
            Self::Static(it) => it.next().copied(),
            Self::Dynamic(it) => it.next(),
        }
    }
}

/// Matches the longest letter pattern at the current position.
/// Returns a `PatternResult` — static slice for fixed patterns (zero allocation),
/// dynamic Vec only for context-sensitive patterns (magic-e, c/g softening).
fn match_pattern(chars: &[char]) -> PatternResult {
    if chars.is_empty() {
        return PatternResult::Static(1, &[]);
    }

    // Try 4-letter patterns
    if chars.len() >= 4 {
        match (chars[0], chars[1], chars[2], chars[3]) {
            ('e', 'i', 'g', 'h') => return PatternResult::Static(4, P_EIGH),
            ('a', 'u', 'g', 'h') => return PatternResult::Static(4, P_AUGH),
            ('o', 'u', 'g', 'h') => return PatternResult::Static(4, P_OUGH),
            ('t', 'i', 'o', 'n') => return PatternResult::Static(4, P_TION),
            ('s', 'i', 'o', 'n') => return PatternResult::Static(4, P_SION),
            ('c', 'i', 'a', 'n') => return PatternResult::Static(4, P_TION),
            _ => {}
        }
    }

    // Try 3-letter patterns
    if chars.len() >= 3 {
        match (chars[0], chars[1], chars[2]) {
            // Magic-e: VCe at word end — dynamic (context-sensitive)
            (v, c, 'e') if chars.len() == 3 && is_vowel_char(v) && !is_vowel_char(c) => {
                let mut ph = magic_e_vowel(v);
                ph.extend_from_slice(&single_consonant(c));
                return PatternResult::Dynamic(3, ph);
            }
            ('a', 'i', 'r') => return PatternResult::Static(3, P_AIR),
            ('e', 'a', 'r') => return PatternResult::Static(3, P_EAR),
            ('o', 'u', 'r') => return PatternResult::Static(3, P_OUR),
            ('i', 'g', 'h') => return PatternResult::Static(3, P_IGH),
            ('t', 'h', 'r') => return PatternResult::Static(3, P_THR),
            ('t', 'h', 'e') => return PatternResult::Static(3, P_THE),
            ('s', 'h', _) => return PatternResult::Static(2, P_SH),
            ('c', 'h', _) => return PatternResult::Static(2, P_CH),
            ('t', 'h', _) => return PatternResult::Static(2, P_TH),
            ('n', 'g', _) if !chars[2].is_alphabetic() || chars.len() == 3 => {
                return PatternResult::Static(2, P_NG);
            }
            ('i', 'n', 'g') if chars.len() == 3 => return PatternResult::Static(3, P_ING),
            ('o', 'u', _) => return PatternResult::Static(2, P_OU),
            ('o', 'w', _) => return PatternResult::Static(2, P_OW),
            ('a', 'i', _) => return PatternResult::Static(2, P_AI),
            ('e', 'e', _) => return PatternResult::Static(2, P_EE),
            ('o', 'o', _) => return PatternResult::Static(2, P_OO),
            ('e', 'a', _) => return PatternResult::Static(2, P_EA),
            _ => {}
        }
    }

    // Try 2-letter patterns
    if chars.len() >= 2 {
        match (chars[0], chars[1]) {
            ('s', 'h') => return PatternResult::Static(2, P_SH),
            ('c', 'h') => return PatternResult::Static(2, P_CH),
            ('t', 'h') => return PatternResult::Static(2, P_TH),
            ('w', 'h') => return PatternResult::Static(2, P_WH),
            ('p', 'h') => return PatternResult::Static(2, P_PH),
            ('c', 'k') => return PatternResult::Static(2, P_CK),
            ('q', 'u') => return PatternResult::Static(2, P_QU),
            ('a', 'r') => return PatternResult::Static(2, P_AR),
            ('e', 'r') | ('i', 'r') | ('u', 'r') => return PatternResult::Static(2, P_ER),
            ('o', 'r') => return PatternResult::Static(2, P_OR),
            _ => {}
        }
    }

    // Single letter patterns — context-sensitive ones use Dynamic
    match chars[0] {
        'a' => PatternResult::Static(1, &[Phoneme::VowelAsh]),
        'b' => PatternResult::Static(1, &[Phoneme::PlosiveB]),
        'c' => {
            if chars.len() > 1 && matches!(chars[1], 'e' | 'i' | 'y') {
                PatternResult::Static(1, &[Phoneme::FricativeS])
            } else {
                PatternResult::Static(1, &[Phoneme::PlosiveK])
            }
        }
        'd' => PatternResult::Static(1, &[Phoneme::PlosiveD]),
        'e' => {
            if chars.len() == 1 {
                PatternResult::Static(1, &[])
            } else {
                PatternResult::Static(1, &[Phoneme::VowelOpenE])
            }
        }
        'f' => PatternResult::Static(1, &[Phoneme::FricativeF]),
        'g' => {
            if chars.len() > 1 && matches!(chars[1], 'e' | 'i' | 'y') {
                PatternResult::Static(1, &[Phoneme::AffricateJ])
            } else {
                PatternResult::Static(1, &[Phoneme::PlosiveG])
            }
        }
        'h' => PatternResult::Static(1, &[Phoneme::FricativeH]),
        'i' => PatternResult::Static(1, &[Phoneme::VowelNearI]),
        'j' => PatternResult::Static(1, &[Phoneme::AffricateJ]),
        'k' => PatternResult::Static(1, &[Phoneme::PlosiveK]),
        'l' => PatternResult::Static(1, &[Phoneme::LateralL]),
        'm' => PatternResult::Static(1, &[Phoneme::NasalM]),
        'n' => PatternResult::Static(1, &[Phoneme::NasalN]),
        'o' => PatternResult::Static(1, &[Phoneme::VowelO]),
        'p' => PatternResult::Static(1, &[Phoneme::PlosiveP]),
        'r' => PatternResult::Static(1, &[Phoneme::ApproximantR]),
        's' => PatternResult::Static(1, &[Phoneme::FricativeS]),
        't' => PatternResult::Static(1, &[Phoneme::PlosiveT]),
        'u' => PatternResult::Static(1, &[Phoneme::VowelCupV]),
        'v' => PatternResult::Static(1, &[Phoneme::FricativeV]),
        'w' => PatternResult::Static(1, &[Phoneme::ApproximantW]),
        'x' => PatternResult::Static(1, P_XS),
        'y' => {
            if chars.len() > 1 {
                PatternResult::Static(1, &[Phoneme::ApproximantJ])
            } else {
                PatternResult::Static(1, &[Phoneme::VowelE])
            }
        }
        'z' => PatternResult::Static(1, &[Phoneme::FricativeZ]),
        '\'' => PatternResult::Static(1, &[]),
        '-' => PatternResult::Static(1, &[Phoneme::Silence]),
        _ => PatternResult::Static(1, &[]),
    }
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
