//! Text normalization: lowercasing, punctuation handling, number expansion.

use alloc::string::String;

/// Normalizes input text for G2P processing.
///
/// - Converts to lowercase
/// - Strips non-alphabetic characters (preserving spaces, apostrophes, hyphens)
/// - Collapses multiple spaces
#[must_use]
pub fn normalize(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut prev_space = false;

    for ch in text.chars() {
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
}
