//! Text normalization: lowercasing, punctuation handling, number expansion.

use alloc::string::String;
use tracing::trace;

/// Phrase boundary marker emitted for commas (short pause).
pub const COMMA_PAUSE: &str = ",pause";
/// Phrase boundary marker emitted for periods/semicolons (longer pause).
pub const PERIOD_PAUSE: &str = ".pause";
/// Emphasis start marker (precedes an emphasized word).
pub const EMPHASIS_START: &str = "<emph>";
/// Emphasis end marker (follows an emphasized word).
pub const EMPHASIS_END: &str = "</emph>";

/// Normalizes input text for G2P processing.
///
/// - Expands numbers to words (42 → "forty two")
/// - Converts to lowercase
/// - Preserves phrase boundary markers for commas and periods
/// - Strips non-alphabetic characters (preserving spaces, apostrophes, hyphens)
/// - Collapses multiple spaces
///
/// # Examples
///
/// ```
/// use shabda::normalize::normalize;
///
/// assert_eq!(normalize("Hello World!"), "hello world");
/// assert_eq!(normalize("I have 42 cats"), "i have forty two cats");
/// ```
#[must_use]
pub fn normalize(text: &str) -> String {
    let abbr_expanded = expand_abbreviations(text);
    let acronym_expanded = expand_acronyms(&abbr_expanded);
    let expanded = expand_numbers(&acronym_expanded);

    let mut result = String::with_capacity(expanded.len());
    let mut prev_space = false;

    for ch in expanded.chars() {
        if ch.is_alphabetic() || ch == '\'' || ch == '-' {
            if prev_space && !result.is_empty() {
                result.push(' ');
            }
            result.push(ch.to_lowercase().next().unwrap_or(ch));
            prev_space = false;
        } else if ch == ',' {
            // Emit phrase boundary marker for comma (short pause)
            result.push_str(" ,pause");
            prev_space = true;
        } else if ch == '.' || ch == ';' {
            // Emit phrase boundary marker for period/semicolon (longer pause)
            result.push_str(" .pause");
            prev_space = true;
        } else if ch.is_whitespace() || ch == '!' || ch == '?' {
            prev_space = true;
        }
    }

    result
}

/// Normalizes input text with emphasis marker detection.
///
/// Like [`normalize`], but also detects emphasis patterns:
/// - ALL-CAPS words (3+ letters) → wrapped in emphasis markers
/// - `*asterisk-wrapped*` words → wrapped in emphasis markers
///
/// # Examples
///
/// ```
/// use shabda::normalize::{normalize_with_emphasis, EMPHASIS_START, EMPHASIS_END};
///
/// let result = normalize_with_emphasis("HELLO world");
/// assert!(result.contains(EMPHASIS_START));
/// assert!(result.contains("hello"));
/// assert!(result.contains(EMPHASIS_END));
/// ```
#[must_use]
pub fn normalize_with_emphasis(text: &str) -> String {
    let abbr_expanded = expand_abbreviations(text);
    let expanded = expand_numbers(&abbr_expanded);

    // First pass: detect emphasis patterns in the original casing
    let tokens: alloc::vec::Vec<&str> = expanded.split_whitespace().collect();
    let mut result = String::with_capacity(expanded.len() + tokens.len() * 8);

    for (i, token) in tokens.iter().enumerate() {
        if i > 0 && !result.is_empty() {
            result.push(' ');
        }

        // Check for *asterisk-wrapped* emphasis
        if token.len() >= 3 && token.starts_with('*') && token.ends_with('*') {
            let inner = &token[1..token.len() - 1];
            if !inner.is_empty() && inner.chars().all(|c| c.is_alphabetic()) {
                result.push_str(EMPHASIS_START);
                result.push(' ');
                for ch in inner.chars() {
                    result.push(ch.to_lowercase().next().unwrap_or(ch));
                }
                result.push(' ');
                result.push_str(EMPHASIS_END);
                continue;
            }
        }

        // Check for ALL-CAPS emphasis (3+ alphabetic chars, all uppercase)
        let alpha_chars: alloc::vec::Vec<char> =
            token.chars().filter(|c| c.is_alphabetic()).collect();
        if alpha_chars.len() >= 3 && alpha_chars.iter().all(|c| c.is_uppercase()) {
            result.push_str(EMPHASIS_START);
            result.push(' ');
            // Process token through normal char rules but lowercase
            normalize_token_into(token, &mut result);
            result.push(' ');
            result.push_str(EMPHASIS_END);
            continue;
        }

        // Normal token processing
        normalize_token_into(token, &mut result);
    }

    result
}

/// Normalizes a single token into the result buffer (lowercase, handle punctuation).
fn normalize_token_into(token: &str, result: &mut String) {
    let mut prev_space = false;
    for ch in token.chars() {
        if ch.is_alphabetic() || ch == '\'' || ch == '-' {
            if prev_space && !result.is_empty() && !result.ends_with(' ') {
                result.push(' ');
            }
            result.push(ch.to_lowercase().next().unwrap_or(ch));
            prev_space = false;
        } else if ch == ',' {
            result.push_str(" ,pause");
            prev_space = true;
        } else if ch == '.' || ch == ';' {
            result.push_str(" .pause");
            prev_space = true;
        } else if ch.is_whitespace() || ch == '!' || ch == '?' {
            prev_space = true;
        }
    }
}

/// Expands common abbreviations to their full spoken forms.
///
/// Runs before number expansion and punctuation handling so that
/// abbreviations like "Dr." don't get their periods eaten by
/// the pause marker logic. Only matches abbreviations at word boundaries.
#[must_use]
pub fn expand_abbreviations(text: &str) -> String {
    let mut result = String::with_capacity(text.len());

    // Split into words, expand abbreviations at word boundaries
    let words: alloc::vec::Vec<&str> = text.split_whitespace().collect();
    for (i, word) in words.iter().enumerate() {
        if i > 0 {
            result.push(' ');
        }
        if let Some((expansion, consumed)) = match_abbreviation(word) {
            result.push_str(expansion);
            // Append any remaining characters after the abbreviation
            result.push_str(&word[consumed..]);
        } else {
            result.push_str(word);
        }
    }

    result
}

/// Tries to match an abbreviation at the start of a word.
/// Returns (expansion, bytes_consumed) if matched.
fn match_abbreviation(text: &str) -> Option<(&'static str, usize)> {
    static ABBREVIATIONS: &[(&str, &str)] = &[
        ("approx.", "approximately"),
        ("blvd.", "boulevard"),
        ("capt.", "captain"),
        ("col.", "colonel"),
        ("corp.", "corporation"),
        ("dept.", "department"),
        ("govt.", "government"),
        ("prof.", "professor"),
        ("mrs.", "missus"),
        ("mr.", "mister"),
        ("dr.", "doctor"),
        ("sr.", "senior"),
        ("jr.", "junior"),
        ("st.", "saint"),
        ("ave.", "avenue"),
        ("etc.", "et cetera"),
        ("vs.", "versus"),
        ("inc.", "incorporated"),
        ("ltd.", "limited"),
        ("gen.", "general"),
        ("sgt.", "sergeant"),
        ("lt.", "lieutenant"),
        ("pt.", "point"),
        ("ft.", "feet"),
        ("mt.", "mount"),
    ];

    let lower = text.to_lowercase();

    for &(abbr, expansion) in ABBREVIATIONS {
        if lower.starts_with(abbr) {
            // Must be followed by whitespace, end of string, or non-alphabetic
            // to avoid matching "st." inside "test."
            let after = &text[abbr.len()..];
            if after.is_empty()
                || after.starts_with(|c: char| c.is_whitespace() || !c.is_alphabetic())
            {
                return Some((expansion, abbr.len()));
            }
        }
    }

    None
}

/// Expands acronyms in text to spelled-out or pronounceable forms.
///
/// ALL-CAPS sequences of 2–5 letters are treated as acronyms:
/// - Pronounceable (contains vowel, valid structure): lowercased as a word
/// - Not pronounceable: spelled out with spaces ("FBI" → "f b i")
#[must_use]
pub fn expand_acronyms(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let words: alloc::vec::Vec<&str> = text.split_whitespace().collect();

    for (i, word) in words.iter().enumerate() {
        if i > 0 {
            result.push(' ');
        }

        // Strip trailing punctuation for detection
        let (core, trailing) = split_trailing_punct(word);

        if is_acronym(core) {
            if is_pronounceable_acronym(core) {
                // Treat as a word: lowercase it
                for ch in core.chars() {
                    result.push(ch.to_lowercase().next().unwrap_or(ch));
                }
            } else {
                // Spell out: "FBI" → "f b i"
                for (j, ch) in core.chars().enumerate() {
                    if j > 0 {
                        result.push(' ');
                    }
                    result.push(ch.to_lowercase().next().unwrap_or(ch));
                }
            }
            result.push_str(trailing);
        } else {
            result.push_str(word);
        }
    }

    result
}

/// Splits trailing punctuation from a word.
fn split_trailing_punct(word: &str) -> (&str, &str) {
    let end = word
        .char_indices()
        .rev()
        .take_while(|(_, c)| !c.is_alphanumeric())
        .last()
        .map(|(i, _)| i)
        .unwrap_or(word.len());
    (&word[..end], &word[end..])
}

/// Returns true if the word is an acronym (3–5 uppercase letters).
fn is_acronym(word: &str) -> bool {
    let len = word.len();
    (3..=5).contains(&len) && word.chars().all(|c| c.is_ascii_uppercase())
}

/// Returns true if an acronym is pronounceable as a word.
///
/// Heuristic: the word must begin with a valid English onset (single consonant
/// or valid cluster) followed by a vowel, or start with a vowel. Words like
/// "NASA" (C+V) are pronounceable; "FBI" (C+C with invalid cluster) are not.
fn is_pronounceable_acronym(word: &str) -> bool {
    let lower: String = word.to_lowercase();
    let chars: alloc::vec::Vec<char> = lower.chars().collect();

    fn is_vowel(c: char) -> bool {
        matches!(c, 'a' | 'e' | 'i' | 'o' | 'u')
    }

    // Must have at least one vowel
    if !chars.iter().any(|c| is_vowel(*c)) {
        return false;
    }

    // Starts with vowel → pronounceable (e.g., "AIDS", "AWOL")
    if is_vowel(chars[0]) {
        return true;
    }

    // Starts with single consonant + vowel → pronounceable (e.g., "NASA", "NATO")
    if chars.len() >= 2 && !is_vowel(chars[0]) && is_vowel(chars[1]) {
        return true;
    }

    // Starts with valid English onset cluster + vowel
    if chars.len() >= 3 && !is_vowel(chars[0]) && !is_vowel(chars[1]) && is_vowel(chars[2]) {
        let cluster = (chars[0], chars[1]);
        let valid_onsets = [
            ('b', 'l'),
            ('b', 'r'),
            ('c', 'l'),
            ('c', 'r'),
            ('d', 'r'),
            ('f', 'l'),
            ('f', 'r'),
            ('g', 'l'),
            ('g', 'r'),
            ('p', 'l'),
            ('p', 'r'),
            ('s', 'c'),
            ('s', 'k'),
            ('s', 'l'),
            ('s', 'm'),
            ('s', 'n'),
            ('s', 'p'),
            ('s', 't'),
            ('s', 'w'),
            ('t', 'r'),
            ('t', 'w'),
            ('t', 'h'),
            ('s', 'h'),
            ('c', 'h'),
            ('w', 'h'),
        ];
        if valid_onsets.contains(&cluster) {
            return true;
        }
    }

    false
}

/// Returns true if a word contains diacritics or non-ASCII Latin characters,
/// suggesting it may be a foreign loan word.
#[must_use]
pub fn is_foreign_word(word: &str) -> bool {
    word.chars()
        .any(|c| c.is_alphabetic() && !c.is_ascii_alphabetic() && is_latin_extended(c))
}

/// Returns true if the character is in a Latin extended Unicode block (accented letters).
fn is_latin_extended(c: char) -> bool {
    let cp = c as u32;
    // Latin-1 Supplement (accented), Latin Extended-A, Latin Extended-B
    (0x00C0..=0x024F).contains(&cp)
}

/// Strips diacritics from a word by mapping accented characters to ASCII equivalents.
#[must_use]
pub fn strip_diacritics(word: &str) -> String {
    word.chars()
        .map(|c| match c {
            'á' | 'à' | 'â' | 'ä' | 'ã' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' => 'i',
            'ó' | 'ò' | 'ô' | 'ö' | 'õ' => 'o',
            'ú' | 'ù' | 'û' | 'ü' => 'u',
            'ñ' => 'n',
            'ç' => 's',
            _ => c,
        })
        .collect()
}

/// Expands digit sequences in text to English words.
///
/// Handles integers (0–999,999), decimals ("3.14" → "three point one four"),
/// and negative numbers ("-5" → "negative five").
#[must_use]
pub fn expand_numbers(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let chars: alloc::vec::Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Check for negative number
        if chars[i] == '-' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit() {
            i += 1; // skip the minus
            let (num_str, consumed) = collect_number(&chars[i..]);
            let expanded = expand_number_token(&num_str);
            trace!(
                num_str = num_str.as_str(),
                expanded = expanded.as_str(),
                "expanded negative number"
            );
            if !result.is_empty() && !result.ends_with(' ') {
                result.push(' ');
            }
            result.push_str("negative ");
            result.push_str(&expanded);
            i += consumed;
            continue;
        }

        if chars[i].is_ascii_digit() {
            let (num_str, consumed) = collect_number(&chars[i..]);
            let expanded = expand_number_token(&num_str);
            trace!(
                num_str = num_str.as_str(),
                expanded = expanded.as_str(),
                "expanded number"
            );
            if !result.is_empty() && !result.ends_with(' ') {
                result.push(' ');
            }
            result.push_str(&expanded);
            i += consumed;
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    result
}

/// Collects a contiguous number token (digits, commas, one decimal point).
/// Returns (collected_string, chars_consumed).
fn collect_number(chars: &[char]) -> (String, usize) {
    let mut s = String::new();
    let mut i = 0;
    let mut has_dot = false;

    while i < chars.len() {
        if chars[i].is_ascii_digit() {
            s.push(chars[i]);
            i += 1;
        } else if chars[i] == ',' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit() {
            // Skip commas in numbers (1,000)
            i += 1;
        } else if chars[i] == '.'
            && !has_dot
            && i + 1 < chars.len()
            && chars[i + 1].is_ascii_digit()
        {
            s.push('.');
            has_dot = true;
            i += 1;
        } else {
            break;
        }
    }

    (s, i)
}

/// Expands a number token string to words.
fn expand_number_token(token: &str) -> String {
    if let Some(dot_pos) = token.find('.') {
        // Decimal: "3.14" -> "three point one four"
        let integer_part = &token[..dot_pos];
        let decimal_part = &token[dot_pos + 1..];

        let mut result = if integer_part.is_empty() {
            String::from("zero")
        } else if let Ok(n) = integer_part.parse::<u64>() {
            number_to_words(n)
        } else {
            digits_to_words(integer_part)
        };

        result.push_str(" point");
        for ch in decimal_part.chars() {
            if let Some(d) = ch.to_digit(10) {
                result.push(' ');
                result.push_str(ONES[d as usize]);
            }
        }
        result
    } else if let Ok(n) = token.parse::<u64>() {
        number_to_words(n)
    } else {
        digits_to_words(token)
    }
}

/// Reads digits one by one as a fallback for very large numbers.
fn digits_to_words(s: &str) -> String {
    let mut result = String::new();
    for ch in s.chars() {
        if let Some(d) = ch.to_digit(10) {
            if !result.is_empty() {
                result.push(' ');
            }
            result.push_str(ONES[d as usize]);
        }
    }
    result
}

/// Converts an integer (0–999,999,999) to English words.
fn number_to_words(n: u64) -> String {
    if n == 0 {
        return String::from("zero");
    }

    let mut parts = alloc::vec::Vec::new();

    if n >= 1_000_000_000 {
        let billions = n / 1_000_000_000;
        parts.push(alloc::format!("{} billion", number_to_words(billions)));
    }

    let remainder = n % 1_000_000_000;
    if remainder >= 1_000_000 {
        let millions = remainder / 1_000_000;
        parts.push(alloc::format!("{} million", number_to_words(millions)));
    }

    let remainder = remainder % 1_000_000;
    if remainder >= 1000 {
        let thousands = remainder / 1000;
        parts.push(alloc::format!("{} thousand", number_to_words(thousands)));
    }

    let remainder = remainder % 1000;
    if remainder > 0 {
        parts.push(hundreds_to_words(remainder));
    }

    parts.join(" ")
}

/// Converts 1–999 to words.
fn hundreds_to_words(n: u64) -> String {
    let mut parts = alloc::vec::Vec::new();

    if n >= 100 {
        parts.push(alloc::format!("{} hundred", ONES[(n / 100) as usize]));
    }

    let remainder = n % 100;
    if remainder > 0 {
        parts.push(tens_to_words(remainder));
    }

    parts.join(" ")
}

/// Converts 1–99 to words.
fn tens_to_words(n: u64) -> String {
    if n < 20 {
        return String::from(ONES[n as usize]);
    }
    let ten = TENS[(n / 10) as usize];
    let one = n % 10;
    if one > 0 {
        alloc::format!("{ten} {}", ONES[one as usize])
    } else {
        String::from(ten)
    }
}

static ONES: &[&str] = &[
    "zero",
    "one",
    "two",
    "three",
    "four",
    "five",
    "six",
    "seven",
    "eight",
    "nine",
    "ten",
    "eleven",
    "twelve",
    "thirteen",
    "fourteen",
    "fifteen",
    "sixteen",
    "seventeen",
    "eighteen",
    "nineteen",
];

static TENS: &[&str] = &[
    "", "", "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety",
];

/// Detects sentence-ending punctuation and returns the intonation type.
#[must_use]
pub fn detect_intonation(text: &str) -> SentenceType {
    let trimmed = text.trim_end();
    if trimmed.ends_with('?') {
        SentenceType::Question
    } else if trimmed.ends_with('!') {
        SentenceType::Exclamation
    } else {
        SentenceType::Statement
    }
}

/// Sentence type for prosody mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SentenceType {
    /// Declarative statement — falling intonation.
    Statement,
    /// Question — rising intonation.
    Question,
    /// Exclamation — high-start falling intonation.
    Exclamation,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_basic() {
        assert_eq!(normalize("Hello World!"), "hello world");
    }

    #[test]
    fn test_normalize_punctuation() {
        // Period at end produces a .pause marker
        assert_eq!(normalize("it's a test."), "it's a test .pause");
    }

    #[test]
    fn test_normalize_multiple_spaces() {
        assert_eq!(normalize("too   many   spaces"), "too many spaces");
    }

    #[test]
    fn test_detect_intonation() {
        assert_eq!(detect_intonation("hello?"), SentenceType::Question);
        assert_eq!(detect_intonation("wow!"), SentenceType::Exclamation);
        assert_eq!(detect_intonation("ok."), SentenceType::Statement);
    }

    // --- Number expansion tests ---

    #[test]
    fn test_expand_zero() {
        assert_eq!(expand_numbers("0"), "zero");
    }

    #[test]
    fn test_expand_small() {
        assert_eq!(expand_numbers("42"), "forty two");
    }

    #[test]
    fn test_expand_hundred() {
        assert_eq!(expand_numbers("100"), "one hundred");
    }

    #[test]
    fn test_expand_thousand() {
        assert_eq!(expand_numbers("1000"), "one thousand");
    }

    #[test]
    fn test_expand_complex() {
        assert_eq!(
            expand_numbers("1234"),
            "one thousand two hundred thirty four"
        );
    }

    #[test]
    fn test_expand_with_comma() {
        assert_eq!(expand_numbers("1,000"), "one thousand");
    }

    #[test]
    fn test_expand_decimal() {
        assert_eq!(expand_numbers("3.14"), "three point one four");
    }

    #[test]
    fn test_expand_negative() {
        assert_eq!(expand_numbers("-5"), "negative five");
    }

    #[test]
    fn test_expand_in_sentence() {
        assert_eq!(expand_numbers("I have 42 cats"), "I have forty two cats");
    }

    #[test]
    fn test_expand_no_numbers() {
        assert_eq!(expand_numbers("no numbers here"), "no numbers here");
    }

    #[test]
    fn test_normalize_with_numbers() {
        assert_eq!(normalize("I have 42 cats!"), "i have forty two cats");
    }

    #[test]
    fn test_normalize_comma_pause() {
        let result = normalize("hello, world");
        assert!(result.contains(COMMA_PAUSE));
    }

    #[test]
    fn test_normalize_period_pause() {
        let result = normalize("first. second");
        assert!(result.contains(PERIOD_PAUSE));
    }

    // --- Abbreviation tests ---

    #[test]
    fn test_expand_abbreviation_dr() {
        assert_eq!(expand_abbreviations("Dr. Smith"), "doctor Smith");
    }

    #[test]
    fn test_expand_abbreviation_mr() {
        assert_eq!(expand_abbreviations("Mr. Jones"), "mister Jones");
    }

    #[test]
    fn test_abbreviation_not_mid_word() {
        // "test." should NOT match "st."
        assert_eq!(expand_abbreviations("test."), "test.");
    }

    #[test]
    fn test_abbreviation_in_normalize() {
        let result = normalize("Dr. Smith is here");
        assert!(result.contains("doctor"), "Dr. should expand to doctor");
    }

    // --- Acronym tests ---

    #[test]
    fn test_acronym_spell_out() {
        // FBI has no vowel → spell out
        assert_eq!(expand_acronyms("FBI"), "f b i");
    }

    #[test]
    fn test_acronym_pronounceable() {
        // NASA has vowels → keep as word
        assert_eq!(expand_acronyms("NASA"), "nasa");
    }

    #[test]
    fn test_acronym_short_not_matched() {
        // 2-letter words not treated as acronyms
        assert_eq!(expand_acronyms("I am OK"), "I am OK");
    }

    #[test]
    fn test_acronym_in_sentence() {
        assert_eq!(expand_acronyms("the FBI and NASA"), "the f b i and nasa");
    }

    // --- Foreign word tests ---

    #[test]
    fn test_foreign_word_detection() {
        assert!(is_foreign_word("café"));
        assert!(is_foreign_word("naïve"));
        assert!(!is_foreign_word("hello"));
    }

    #[test]
    fn test_strip_diacritics() {
        assert_eq!(strip_diacritics("café"), "cafe");
        assert_eq!(strip_diacritics("naïve"), "naive");
        assert_eq!(strip_diacritics("résumé"), "resume");
    }
}
