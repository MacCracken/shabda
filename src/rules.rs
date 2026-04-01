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

// =============================================================================
// Spanish G2P rules
// =============================================================================

/// Converts a single Spanish word to phonemes using letter-to-sound rules.
///
/// Spanish orthography is highly regular — nearly 1:1 grapheme-to-phoneme.
/// The main complexities are digraphs (ch, ll, rr, qu, gu) and a few
/// context-sensitive consonants (c, g, h).
#[must_use]
pub fn spanish_rules(word: &str) -> Vec<Phoneme> {
    let chars: Vec<char> = word.to_lowercase().chars().collect();
    if chars.is_empty() {
        return Vec::new();
    }

    let mut phonemes = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        let remaining = &chars[i..];
        let result = spanish_match(remaining);
        i += result.consumed();
        phonemes.extend(result);
    }

    phonemes
}

// Spanish static pattern constants
static SP_CH: &[Phoneme] = &[Phoneme::AffricateCh];
static SP_LL: &[Phoneme] = &[Phoneme::ApproximantJ];
static SP_RR: &[Phoneme] = &[Phoneme::ApproximantR];
static SP_XS_ES: &[Phoneme] = &[Phoneme::PlosiveK, Phoneme::FricativeS];

/// Pattern matching for Spanish grapheme-to-phoneme.
fn spanish_match(chars: &[char]) -> PatternResult {
    if chars.is_empty() {
        return PatternResult::Static(1, &[]);
    }

    // 2-letter digraphs first
    if chars.len() >= 2 {
        match (chars[0], chars[1]) {
            ('c', 'h') => return PatternResult::Static(2, SP_CH),
            ('l', 'l') => return PatternResult::Static(2, SP_LL),
            ('r', 'r') => return PatternResult::Static(2, SP_RR),
            ('q', 'u') => return PatternResult::Static(2, &[Phoneme::PlosiveK]),
            // gu before e/i: u is silent, g→/g/
            ('g', 'u') if chars.len() >= 3 && matches!(chars[2], 'e' | 'i') => {
                return PatternResult::Static(2, &[Phoneme::PlosiveG]);
            }
            _ => {}
        }
    }

    // Single character rules
    match chars[0] {
        'a' | 'á' => PatternResult::Static(1, &[Phoneme::VowelOpenA]),
        'e' | 'é' => PatternResult::Static(1, &[Phoneme::VowelOpenE]),
        'i' | 'í' => PatternResult::Static(1, &[Phoneme::VowelNearI]),
        'o' | 'ó' => PatternResult::Static(1, &[Phoneme::VowelO]),
        'u' | 'ú' | 'ü' => PatternResult::Static(1, &[Phoneme::VowelCupV]),
        'b' | 'v' => PatternResult::Static(1, &[Phoneme::PlosiveB]),
        'c' => {
            // c before e/i → /θ/ (Castilian)
            if chars.len() > 1 && matches!(chars[1], 'e' | 'i') {
                PatternResult::Static(1, &[Phoneme::FricativeTh])
            } else {
                PatternResult::Static(1, &[Phoneme::PlosiveK])
            }
        }
        'd' => PatternResult::Static(1, &[Phoneme::PlosiveD]),
        'f' => PatternResult::Static(1, &[Phoneme::FricativeF]),
        'g' => {
            // g before e/i → /x/ (velar fricative, mapped to FricativeH)
            if chars.len() > 1 && matches!(chars[1], 'e' | 'i') {
                PatternResult::Static(1, &[Phoneme::FricativeH])
            } else {
                PatternResult::Static(1, &[Phoneme::PlosiveG])
            }
        }
        'h' => PatternResult::Static(1, &[]), // always silent
        'j' => PatternResult::Static(1, &[Phoneme::FricativeH]), // /x/
        'k' => PatternResult::Static(1, &[Phoneme::PlosiveK]),
        'l' => PatternResult::Static(1, &[Phoneme::LateralL]),
        'm' => PatternResult::Static(1, &[Phoneme::NasalM]),
        'n' => PatternResult::Static(1, &[Phoneme::NasalN]),
        'ñ' => PatternResult::Static(1, &[Phoneme::NasalNg]), // palatal nasal
        'p' => PatternResult::Static(1, &[Phoneme::PlosiveP]),
        'r' => PatternResult::Static(1, &[Phoneme::TapFlap]),
        's' => PatternResult::Static(1, &[Phoneme::FricativeS]),
        't' => PatternResult::Static(1, &[Phoneme::PlosiveT]),
        'w' => PatternResult::Static(1, &[Phoneme::ApproximantW]),
        'x' => PatternResult::Static(1, SP_XS_ES),
        'y' => {
            if chars.len() == 1 {
                PatternResult::Static(1, &[Phoneme::VowelNearI])
            } else {
                PatternResult::Static(1, &[Phoneme::ApproximantJ])
            }
        }
        'z' => PatternResult::Static(1, &[Phoneme::FricativeTh]), // Castilian /θ/
        '\'' | '-' => PatternResult::Static(1, &[]),
        _ => PatternResult::Static(1, &[]),
    }
}

// =============================================================================
// German G2P rules
// =============================================================================

/// Converts a single German word to phonemes using letter-to-sound rules.
///
/// German orthography is fairly regular. Key features: sch→/ʃ/, ch→/ç/ or /x/,
/// ei/ie/eu/äu digraphs, umlauts (ä/ö/ü), final devoicing (b→p, d→t, g→k).
#[must_use]
pub fn german_rules(word: &str) -> Vec<Phoneme> {
    let chars: Vec<char> = word.to_lowercase().chars().collect();
    if chars.is_empty() {
        return Vec::new();
    }

    let mut phonemes = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        let remaining = &chars[i..];
        let result = german_match(remaining);
        i += result.consumed();
        phonemes.extend(result);
    }

    // Final devoicing: voiced → voiceless at word end
    if let Some(last) = phonemes.last_mut() {
        *last = match *last {
            Phoneme::PlosiveB => Phoneme::PlosiveP,
            Phoneme::PlosiveD => Phoneme::PlosiveT,
            Phoneme::PlosiveG => Phoneme::PlosiveK,
            Phoneme::FricativeV => Phoneme::FricativeF,
            Phoneme::FricativeZ => Phoneme::FricativeS,
            other => other,
        };
    }

    phonemes
}

// German static pattern constants
static DE_SCH: &[Phoneme] = &[Phoneme::FricativeSh];
static DE_EI: &[Phoneme] = &[Phoneme::DiphthongAI];
static DE_IE: &[Phoneme] = &[Phoneme::VowelE]; // long /iː/
static DE_EU: &[Phoneme] = &[Phoneme::DiphthongOI];
static DE_CH: &[Phoneme] = &[Phoneme::FricativeSh]; // ich/ach-Laut — closest in svara
static DE_CK: &[Phoneme] = &[Phoneme::PlosiveK];
static DE_PF: &[Phoneme] = &[Phoneme::PlosiveP, Phoneme::FricativeF];
static DE_TS: &[Phoneme] = &[Phoneme::PlosiveT, Phoneme::FricativeS];

fn german_match(chars: &[char]) -> PatternResult {
    if chars.is_empty() {
        return PatternResult::Static(1, &[]);
    }

    // 3-letter patterns
    if chars.len() >= 3 && chars[0] == 's' && chars[1] == 'c' && chars[2] == 'h' {
        return PatternResult::Static(3, DE_SCH);
    }

    // 2-letter patterns
    if chars.len() >= 2 {
        match (chars[0], chars[1]) {
            ('c', 'h') => {
                // ich-Laut after front vowels/consonants, ach-Laut after back vowels
                // Simplified: use ich-Laut as default
                return PatternResult::Static(2, DE_CH);
            }
            ('c', 'k') => return PatternResult::Static(2, DE_CK),
            ('e', 'i') => return PatternResult::Static(2, DE_EI),
            ('i', 'e') => return PatternResult::Static(2, DE_IE),
            ('e', 'u') => return PatternResult::Static(2, DE_EU),
            ('ä', 'u') => return PatternResult::Static(2, DE_EU),
            ('p', 'f') => return PatternResult::Static(2, DE_PF),
            ('t', 'z') | ('z', _) if chars[0] == 'z' => {}
            ('s', 'p') | ('s', 't') => {
                // sp/st at word start → /ʃp/, /ʃt/
                return PatternResult::Dynamic(1, alloc::vec![Phoneme::FricativeSh]);
            }
            ('t', 'h') => return PatternResult::Static(2, &[Phoneme::PlosiveT]),
            ('p', 'h') => return PatternResult::Static(2, &[Phoneme::FricativeF]),
            ('q', 'u') => {
                return PatternResult::Static(2, &[Phoneme::PlosiveK, Phoneme::FricativeV]);
            }
            // Double consonants → single phoneme
            (a, b) if a == b && !is_vowel_char(a) => {
                return PatternResult::Static(
                    2,
                    match a {
                        'b' => &[Phoneme::PlosiveB],
                        'c' => &[Phoneme::PlosiveK],
                        'd' => &[Phoneme::PlosiveD],
                        'f' => &[Phoneme::FricativeF],
                        'g' => &[Phoneme::PlosiveG],
                        'l' => &[Phoneme::LateralL],
                        'm' => &[Phoneme::NasalM],
                        'n' => &[Phoneme::NasalN],
                        'p' => &[Phoneme::PlosiveP],
                        'r' => &[Phoneme::ApproximantR],
                        's' => &[Phoneme::FricativeS],
                        't' => &[Phoneme::PlosiveT],
                        _ => &[],
                    },
                );
            }
            _ => {}
        }
    }

    // Single characters
    match chars[0] {
        'a' | 'á' => PatternResult::Static(1, &[Phoneme::VowelOpenA]),
        'ä' => PatternResult::Static(1, &[Phoneme::VowelOpenE]),
        'e' => PatternResult::Static(1, &[Phoneme::VowelOpenE]),
        'i' => PatternResult::Static(1, &[Phoneme::VowelNearI]),
        'o' => PatternResult::Static(1, &[Phoneme::VowelO]),
        'ö' => PatternResult::Static(1, &[Phoneme::VowelOpenE]), // /œ/ — closest
        'u' => PatternResult::Static(1, &[Phoneme::VowelU]),
        'ü' => PatternResult::Static(1, &[Phoneme::VowelE]), // /yː/ — closest
        'b' => PatternResult::Static(1, &[Phoneme::PlosiveB]),
        'c' => PatternResult::Static(1, &[Phoneme::PlosiveK]),
        'd' => PatternResult::Static(1, &[Phoneme::PlosiveD]),
        'f' => PatternResult::Static(1, &[Phoneme::FricativeF]),
        'g' => PatternResult::Static(1, &[Phoneme::PlosiveG]),
        'h' => PatternResult::Static(1, &[Phoneme::FricativeH]),
        'j' => PatternResult::Static(1, &[Phoneme::ApproximantJ]),
        'k' => PatternResult::Static(1, &[Phoneme::PlosiveK]),
        'l' => PatternResult::Static(1, &[Phoneme::LateralL]),
        'm' => PatternResult::Static(1, &[Phoneme::NasalM]),
        'n' => PatternResult::Static(1, &[Phoneme::NasalN]),
        'p' => PatternResult::Static(1, &[Phoneme::PlosiveP]),
        'r' => PatternResult::Static(1, &[Phoneme::ApproximantR]),
        's' => {
            // s before vowel → voiced /z/
            if chars.len() > 1 && is_vowel_char(chars[1]) {
                PatternResult::Static(1, &[Phoneme::FricativeZ])
            } else {
                PatternResult::Static(1, &[Phoneme::FricativeS])
            }
        }
        'ß' => PatternResult::Static(1, &[Phoneme::FricativeS]),
        't' => PatternResult::Static(1, &[Phoneme::PlosiveT]),
        'v' => PatternResult::Static(1, &[Phoneme::FricativeF]), // German v = /f/ typically
        'w' => PatternResult::Static(1, &[Phoneme::FricativeV]), // German w = /v/
        'x' => PatternResult::Static(1, &[Phoneme::PlosiveK]),
        'y' => PatternResult::Static(1, &[Phoneme::VowelE]), // /yː/
        'z' => PatternResult::Static(1, DE_TS),
        '\'' | '-' => PatternResult::Static(1, &[]),
        _ => PatternResult::Static(1, &[]),
    }
}

// =============================================================================
// Hindi G2P rules (Devanagari)
// =============================================================================

/// Converts a single Hindi word (Devanagari script) to phonemes.
///
/// Hindi/Devanagari has a near-1:1 grapheme-to-phoneme mapping. Each
/// consonant has an inherent schwa that is suppressed by virama (्) or
/// at word-final position (schwa deletion rule).
#[must_use]
pub fn hindi_rules(word: &str) -> Vec<Phoneme> {
    let chars: Vec<char> = word.chars().collect();
    if chars.is_empty() {
        return Vec::new();
    }

    let mut phonemes = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];
        let next = chars.get(i + 1).copied();

        match ch {
            // Independent vowels
            'अ' => phonemes.push(Phoneme::VowelSchwa),
            'आ' | 'ा' => phonemes.push(Phoneme::VowelOpenA),
            'इ' | 'ि' => phonemes.push(Phoneme::VowelNearI),
            'ई' | 'ी' => phonemes.push(Phoneme::VowelE),
            'उ' | 'ु' => phonemes.push(Phoneme::VowelCupV),
            'ऊ' | 'ू' => phonemes.push(Phoneme::VowelU),
            'ए' | 'े' => phonemes.push(Phoneme::VowelOpenE),
            'ऐ' | 'ै' => phonemes.push(Phoneme::VowelOpenA),
            'ओ' | 'ो' => phonemes.push(Phoneme::VowelO),
            'औ' | 'ौ' => phonemes.push(Phoneme::VowelOpenO),
            'ऋ' | 'ृ' => {
                phonemes.push(Phoneme::TapFlap);
                phonemes.push(Phoneme::VowelNearI);
            }

            // Velar consonants
            'क' => push_consonant(&mut phonemes, Phoneme::PlosiveK, next),
            'ख' => push_consonant(&mut phonemes, Phoneme::PlosiveK, next), // aspirated
            'ग' => push_consonant(&mut phonemes, Phoneme::PlosiveG, next),
            'घ' => push_consonant(&mut phonemes, Phoneme::PlosiveG, next), // aspirated
            'ङ' => push_consonant(&mut phonemes, Phoneme::NasalNg, next),

            // Palatal consonants
            'च' => push_consonant(&mut phonemes, Phoneme::AffricateCh, next),
            'छ' => push_consonant(&mut phonemes, Phoneme::AffricateCh, next),
            'ज' => push_consonant(&mut phonemes, Phoneme::AffricateJ, next),
            'झ' => push_consonant(&mut phonemes, Phoneme::AffricateJ, next),
            'ञ' => push_consonant(&mut phonemes, Phoneme::NasalN, next),

            // Retroflex consonants (mapped to closest alveolar equivalents)
            'ट' => push_consonant(&mut phonemes, Phoneme::PlosiveT, next),
            'ठ' => push_consonant(&mut phonemes, Phoneme::PlosiveT, next),
            'ड' => push_consonant(&mut phonemes, Phoneme::PlosiveD, next),
            'ढ' => push_consonant(&mut phonemes, Phoneme::PlosiveD, next),
            'ण' => push_consonant(&mut phonemes, Phoneme::NasalN, next),

            // Dental consonants
            'त' => push_consonant(&mut phonemes, Phoneme::PlosiveT, next),
            'थ' => push_consonant(&mut phonemes, Phoneme::PlosiveT, next),
            'द' => push_consonant(&mut phonemes, Phoneme::PlosiveD, next),
            'ध' => push_consonant(&mut phonemes, Phoneme::PlosiveD, next),
            'न' => push_consonant(&mut phonemes, Phoneme::NasalN, next),

            // Labial consonants
            'प' => push_consonant(&mut phonemes, Phoneme::PlosiveP, next),
            'फ' => push_consonant(&mut phonemes, Phoneme::PlosiveP, next),
            'ब' => push_consonant(&mut phonemes, Phoneme::PlosiveB, next),
            'भ' => push_consonant(&mut phonemes, Phoneme::PlosiveB, next),
            'म' => push_consonant(&mut phonemes, Phoneme::NasalM, next),

            // Semi-vowels and liquids
            'य' => push_consonant(&mut phonemes, Phoneme::ApproximantJ, next),
            'र' => push_consonant(&mut phonemes, Phoneme::TapFlap, next),
            'ल' => push_consonant(&mut phonemes, Phoneme::LateralL, next),
            'व' => push_consonant(&mut phonemes, Phoneme::FricativeV, next),

            // Sibilants
            'श' => push_consonant(&mut phonemes, Phoneme::FricativeSh, next),
            'ष' => push_consonant(&mut phonemes, Phoneme::FricativeSh, next),
            'स' => push_consonant(&mut phonemes, Phoneme::FricativeS, next),
            'ह' => push_consonant(&mut phonemes, Phoneme::FricativeH, next),

            // Nukta (़ U+093C) — combines with preceding consonant for borrowed sounds.
            // The base consonant was already handled above; nukta modifies it.
            // We skip the nukta mark itself here; the base consonant match handles the sound.
            '\u{093C}' => {}

            // Virama (halant) — suppresses inherent schwa, handled by push_consonant
            '्' => {}

            // Anusvara (nasal) and visarga
            'ं' => phonemes.push(Phoneme::NasalN),
            'ः' => phonemes.push(Phoneme::FricativeH),

            // Chandrabindu (nasalization marker) — approximate with nasal
            'ँ' => phonemes.push(Phoneme::NasalN),

            // Latin fallback for romanized Hindi
            c if c.is_ascii_alphabetic() => {
                let lower = c.to_lowercase().next().unwrap_or(c);
                match lower {
                    'a' => phonemes.push(Phoneme::VowelSchwa),
                    'e' => phonemes.push(Phoneme::VowelOpenE),
                    'i' => phonemes.push(Phoneme::VowelNearI),
                    'o' => phonemes.push(Phoneme::VowelO),
                    'u' => phonemes.push(Phoneme::VowelCupV),
                    'k' => phonemes.push(Phoneme::PlosiveK),
                    'g' => phonemes.push(Phoneme::PlosiveG),
                    'c' => phonemes.push(Phoneme::AffricateCh),
                    'j' => phonemes.push(Phoneme::AffricateJ),
                    't' => phonemes.push(Phoneme::PlosiveT),
                    'd' => phonemes.push(Phoneme::PlosiveD),
                    'n' => phonemes.push(Phoneme::NasalN),
                    'p' => phonemes.push(Phoneme::PlosiveP),
                    'b' => phonemes.push(Phoneme::PlosiveB),
                    'm' => phonemes.push(Phoneme::NasalM),
                    'y' => phonemes.push(Phoneme::ApproximantJ),
                    'r' => phonemes.push(Phoneme::TapFlap),
                    'l' => phonemes.push(Phoneme::LateralL),
                    'v' | 'w' => phonemes.push(Phoneme::FricativeV),
                    's' => phonemes.push(Phoneme::FricativeS),
                    'h' => phonemes.push(Phoneme::FricativeH),
                    'f' => phonemes.push(Phoneme::FricativeF),
                    'z' => phonemes.push(Phoneme::FricativeZ),
                    _ => {}
                }
            }

            _ => {} // skip unknown characters
        }

        i += 1;
    }

    // Hindi schwa deletion: remove word-final schwa
    if phonemes.last() == Some(&Phoneme::VowelSchwa) && phonemes.len() > 1 {
        phonemes.pop();
    }

    phonemes
}

/// Pushes a consonant phoneme, adding inherent schwa unless followed by
/// a vowel matra or virama.
fn push_consonant(phonemes: &mut Vec<Phoneme>, consonant: Phoneme, next: Option<char>) {
    phonemes.push(consonant);
    // Add inherent schwa unless next char is a vowel matra or virama
    let suppress = matches!(
        next,
        Some(
            'ा' | 'ि'
                | 'ी'
                | 'ु'
                | 'ू'
                | 'े'
                | 'ै'
                | 'ो'
                | 'ौ'
                | 'ृ'
                | '्'
                | 'ं'
                | 'ँ'
        )
    );
    if !suppress {
        phonemes.push(Phoneme::VowelSchwa);
    }
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

    // --- Spanish G2P tests ---

    #[test]
    fn test_spanish_hola() {
        let phonemes = spanish_rules("hola");
        // h is silent, o→VowelO, l→LateralL, a→VowelOpenA
        assert_eq!(phonemes[0], Phoneme::VowelO);
        assert_eq!(phonemes[1], Phoneme::LateralL);
        assert_eq!(phonemes[2], Phoneme::VowelOpenA);
    }

    #[test]
    fn test_spanish_ch() {
        let phonemes = spanish_rules("chico");
        assert_eq!(phonemes[0], Phoneme::AffricateCh);
    }

    #[test]
    fn test_spanish_ll() {
        let phonemes = spanish_rules("llamar");
        assert_eq!(phonemes[0], Phoneme::ApproximantJ);
    }

    #[test]
    fn test_spanish_rr() {
        let phonemes = spanish_rules("perro");
        // p→PlosiveP, e→VowelOpenE, rr→ApproximantR (trill), o→VowelO
        assert!(phonemes.contains(&Phoneme::ApproximantR));
    }

    #[test]
    fn test_spanish_que() {
        let phonemes = spanish_rules("que");
        // qu→/k/, e→VowelOpenE
        assert_eq!(phonemes[0], Phoneme::PlosiveK);
        assert_eq!(phonemes[1], Phoneme::VowelOpenE);
    }

    #[test]
    fn test_spanish_gui() {
        let phonemes = spanish_rules("guitarra");
        // gu before i → /g/ (u silent)
        assert_eq!(phonemes[0], Phoneme::PlosiveG);
    }

    #[test]
    fn test_spanish_empty() {
        let phonemes = spanish_rules("");
        assert!(phonemes.is_empty());
    }

    #[test]
    fn test_spanish_nino() {
        let phonemes = spanish_rules("niño");
        assert!(phonemes.contains(&Phoneme::NasalNg));
    }

    // --- German G2P tests ---

    #[test]
    fn test_german_hallo() {
        let phonemes = german_rules("hallo");
        assert_eq!(phonemes[0], Phoneme::FricativeH);
        assert!(!phonemes.is_empty());
    }

    #[test]
    fn test_german_sch() {
        let phonemes = german_rules("schule");
        assert_eq!(phonemes[0], Phoneme::FricativeSh);
    }

    #[test]
    fn test_german_ch() {
        let phonemes = german_rules("ich");
        assert!(phonemes.contains(&Phoneme::FricativeSh)); // ich-Laut
    }

    #[test]
    fn test_german_ei() {
        let phonemes = german_rules("ein");
        assert!(phonemes.contains(&Phoneme::DiphthongAI));
    }

    #[test]
    fn test_german_ie() {
        let phonemes = german_rules("die");
        assert!(phonemes.contains(&Phoneme::VowelE)); // long /iː/
    }

    #[test]
    fn test_german_final_devoicing() {
        let phonemes = german_rules("hund");
        // Final d → t
        assert_eq!(*phonemes.last().unwrap(), Phoneme::PlosiveT);
    }

    #[test]
    fn test_german_umlaut() {
        let phonemes = german_rules("über");
        assert_eq!(phonemes[0], Phoneme::VowelE); // ü → /yː/ mapped to VowelE
    }

    #[test]
    fn test_german_z() {
        let phonemes = german_rules("zeit");
        // z → /ts/
        assert_eq!(phonemes[0], Phoneme::PlosiveT);
        assert_eq!(phonemes[1], Phoneme::FricativeS);
    }

    #[test]
    fn test_german_empty() {
        assert!(german_rules("").is_empty());
    }

    // --- Hindi G2P tests ---

    #[test]
    fn test_hindi_namaste() {
        let phonemes = hindi_rules("नमस्ते");
        // न→n+schwa, म→m+schwa, स→s (virama suppresses schwa), ते→t+e
        assert!(!phonemes.is_empty());
        assert!(phonemes.contains(&Phoneme::NasalN));
        assert!(phonemes.contains(&Phoneme::NasalM));
    }

    #[test]
    fn test_hindi_simple_ka() {
        let phonemes = hindi_rules("क");
        // Single consonant → k + inherent schwa (but schwa deletion removes final)
        assert_eq!(phonemes[0], Phoneme::PlosiveK);
    }

    #[test]
    fn test_hindi_vowels() {
        let phonemes = hindi_rules("आ");
        assert_eq!(phonemes[0], Phoneme::VowelOpenA);
    }

    #[test]
    fn test_hindi_romanized() {
        let phonemes = hindi_rules("namaste");
        assert!(!phonemes.is_empty());
        assert!(phonemes.contains(&Phoneme::NasalN));
    }

    #[test]
    fn test_hindi_empty() {
        assert!(hindi_rules("").is_empty());
    }
}
