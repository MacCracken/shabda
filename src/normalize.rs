//! Text normalization: lowercasing, punctuation handling, number expansion.

use alloc::string::String;

/// Normalizes input text for G2P processing.
///
/// - Expands numbers to words (42 → "forty two")
/// - Converts to lowercase
/// - Strips non-alphabetic characters (preserving spaces, apostrophes, hyphens)
/// - Collapses multiple spaces
#[must_use]
pub fn normalize(text: &str) -> String {
    let expanded = expand_numbers(text);

    let mut result = String::with_capacity(expanded.len());
    let mut prev_space = false;

    for ch in expanded.chars() {
        if ch.is_alphabetic() || ch == '\'' || ch == '-' {
            if prev_space && !result.is_empty() {
                result.push(' ');
            }
            result.push(ch.to_lowercase().next().unwrap_or(ch));
            prev_space = false;
        } else if ch.is_whitespace() || ch == ',' || ch == '.' || ch == '!' || ch == '?' {
            prev_space = true;
        }
    }

    result
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
            if !result.is_empty() && !result.ends_with(' ') {
                result.push(' ');
            }
            result.push_str("negative ");
            result.push_str(&expand_number_token(&num_str));
            i += consumed;
            continue;
        }

        if chars[i].is_ascii_digit() {
            let (num_str, consumed) = collect_number(&chars[i..]);
            if !result.is_empty() && !result.ends_with(' ') {
                result.push(' ');
            }
            result.push_str(&expand_number_token(&num_str));
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
        assert_eq!(normalize("it's a test."), "it's a test");
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
}
