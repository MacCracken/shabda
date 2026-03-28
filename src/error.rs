//! Error types for the shabda crate.

use alloc::string::String;
use serde::{Deserialize, Serialize};

/// Errors that can occur during grapheme-to-phoneme conversion.
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[non_exhaustive]
pub enum ShabdaError {
    /// Input text is empty or invalid.
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// A word could not be converted to phonemes.
    #[error("unknown word: {0}")]
    UnknownWord(String),

    /// The requested language is not supported.
    #[error("unsupported language: {0}")]
    UnsupportedLanguage(String),

    /// A conversion rule failed.
    #[error("rule error: {0}")]
    RuleError(String),

    /// Dictionary parsing or I/O failed.
    #[error("dictionary parse error: {0}")]
    DictParseError(String),
}

/// Convenience type alias for shabda results.
pub type Result<T> = core::result::Result<T, ShabdaError>;

impl From<shabdakosh::ShabdakoshError> for ShabdaError {
    fn from(e: shabdakosh::ShabdakoshError) -> Self {
        ShabdaError::DictParseError(alloc::format!("{e}"))
    }
}
