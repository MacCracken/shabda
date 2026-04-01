//! Heteronym disambiguation using preceding-word context.
//!
//! Heteronyms are words spelled the same but pronounced differently based on
//! meaning or part of speech (e.g., "read" as /riːd/ vs /rɛd/, "live" as
//! /lɪv/ vs /laɪv/). This module provides a simple context-based heuristic:
//! if a trigger word precedes the heteronym, select the non-default variant.

use serde::{Deserialize, Serialize};
use svara::phoneme::Phoneme;

/// A heteronym disambiguation rule.
///
/// Each rule maps a word to two pronunciation variants: a default
/// and a context-triggered alternative. When a trigger word appears
/// in the preceding context, the alternative pronunciation is used.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct HeteronymRule {
    /// The heteronym word (lowercase).
    pub word: &'static str,
    /// Index of the default pronunciation in the dictionary entry.
    pub default_variant: usize,
    /// Index of the context-triggered pronunciation variant.
    pub context_variant: usize,
    /// Words that trigger the non-default pronunciation when preceding.
    pub triggers: &'static [&'static str],
}

/// Serializable version of a heteronym rule for API consumers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[non_exhaustive]
pub struct HeteronymRuleOwned {
    /// The heteronym word.
    pub word: alloc::string::String,
    /// Index of the default pronunciation in the dictionary entry.
    pub default_variant: usize,
    /// Index of the context-triggered pronunciation variant.
    pub context_variant: usize,
    /// Words that trigger the non-default pronunciation.
    pub triggers: alloc::vec::Vec<alloc::string::String>,
}

impl From<&HeteronymRule> for HeteronymRuleOwned {
    fn from(rule: &HeteronymRule) -> Self {
        Self {
            word: alloc::string::String::from(rule.word),
            default_variant: rule.default_variant,
            context_variant: rule.context_variant,
            triggers: rule
                .triggers
                .iter()
                .map(|s| alloc::string::String::from(*s))
                .collect(),
        }
    }
}

/// Returns the heteronym rule for a word, if one exists.
#[must_use]
pub fn lookup(word: &str) -> Option<&'static HeteronymRule> {
    HETERONYMS.iter().find(|r| r.word == word)
}

/// Selects the correct pronunciation variant index based on context.
///
/// Checks if any of the preceding words match a trigger for this heteronym.
/// Returns the variant index to use when calling `DictEntry::all()[index]`.
#[must_use]
pub fn select_variant(rule: &HeteronymRule, preceding_words: &[&str]) -> usize {
    for prev in preceding_words.iter().rev().take(3) {
        let lower = prev.to_lowercase();
        if rule.triggers.contains(&lower.as_str()) {
            return rule.context_variant;
        }
    }
    rule.default_variant
}

/// Selects the phonemes for a heteronym given context and available pronunciations.
///
/// If the dictionary has the selected variant, returns those phonemes.
/// Otherwise falls back to the first available pronunciation.
#[must_use]
pub fn select_phonemes<'a>(
    rule: &HeteronymRule,
    preceding_words: &[&str],
    pronunciations: &'a [shabdakosh::Pronunciation],
) -> &'a [Phoneme] {
    let idx = select_variant(rule, preceding_words);
    if idx < pronunciations.len() {
        pronunciations[idx].phonemes()
    } else {
        pronunciations[0].phonemes()
    }
}

// Heteronym table.
//
// Convention: variant 0 = most common / default pronunciation,
// variant 1 = alternative triggered by context words.
//
// Trigger words are words that typically precede the non-default form.
// For verb/noun pairs: "to", "will", "can", "could" etc. trigger the verb form.
static HETERONYMS: &[HeteronymRule] = &[
    // read: default = past tense /rɛd/, triggers = present tense /riːd/
    HeteronymRule {
        word: "read",
        default_variant: 0,
        context_variant: 1,
        triggers: &[
            "to", "will", "can", "could", "should", "would", "may", "might", "must", "please",
            "let's", "shall",
        ],
    },
    // lead: default = metal /lɛd/, triggers = verb /liːd/
    HeteronymRule {
        word: "lead",
        default_variant: 0,
        context_variant: 1,
        triggers: &[
            "to", "will", "can", "could", "should", "would", "may", "might", "must", "shall",
        ],
    },
    // live: default = verb /lɪv/, triggers = adjective /laɪv/
    HeteronymRule {
        word: "live",
        default_variant: 0,
        context_variant: 1,
        triggers: &["a", "the", "is", "was", "go", "went", "broadcast", "on"],
    },
    // wind: default = air /wɪnd/, triggers = verb /waɪnd/
    HeteronymRule {
        word: "wind",
        default_variant: 0,
        context_variant: 1,
        triggers: &["to", "will", "can", "could", "should", "would", "must"],
    },
    // tear: default = eye /tɪər/, triggers = rip /tɛər/
    HeteronymRule {
        word: "tear",
        default_variant: 0,
        context_variant: 1,
        triggers: &["to", "will", "can", "could", "don't", "didn't"],
    },
    // bow: default = weapon /boʊ/, triggers = bend /baʊ/
    HeteronymRule {
        word: "bow",
        default_variant: 0,
        context_variant: 1,
        triggers: &["to", "will", "take", "took", "a"],
    },
    // close: default = verb /kloʊz/, triggers = adjective /kloʊs/
    HeteronymRule {
        word: "close",
        default_variant: 0,
        context_variant: 1,
        triggers: &["a", "the", "too", "very", "so", "how", "is", "was", "get"],
    },
    // record: default = noun /rɛkərd/, triggers = verb /rɪkɔːrd/
    HeteronymRule {
        word: "record",
        default_variant: 0,
        context_variant: 1,
        triggers: &[
            "to", "will", "can", "could", "should", "would", "must", "please",
        ],
    },
    // present: default = noun /prɛzənt/, triggers = verb /prɪzɛnt/
    HeteronymRule {
        word: "present",
        default_variant: 0,
        context_variant: 1,
        triggers: &[
            "to", "will", "can", "could", "should", "would", "must", "shall",
        ],
    },
    // refuse: default = noun /rɛfjuːs/, triggers = verb /rɪfjuːz/
    HeteronymRule {
        word: "refuse",
        default_variant: 0,
        context_variant: 1,
        triggers: &["to", "will", "can", "could", "should", "would", "must", "i"],
    },
    // produce: default = noun /proʊduːs/, triggers = verb /prədjuːs/
    HeteronymRule {
        word: "produce",
        default_variant: 0,
        context_variant: 1,
        triggers: &["to", "will", "can", "could", "should", "would", "must"],
    },
    // object: default = noun /ɒbdʒɛkt/, triggers = verb /əbdʒɛkt/
    HeteronymRule {
        word: "object",
        default_variant: 0,
        context_variant: 1,
        triggers: &[
            "to", "will", "can", "could", "should", "would", "must", "i", "we", "they",
        ],
    },
    // project: default = noun /prɒdʒɛkt/, triggers = verb /prədʒɛkt/
    HeteronymRule {
        word: "project",
        default_variant: 0,
        context_variant: 1,
        triggers: &["to", "will", "can", "could", "should", "would", "must"],
    },
    // permit: default = noun /pɜːrmɪt/, triggers = verb /pərmɪt/
    HeteronymRule {
        word: "permit",
        default_variant: 0,
        context_variant: 1,
        triggers: &[
            "to", "will", "can", "could", "should", "would", "must", "not",
        ],
    },
    // desert: default = noun /dɛzərt/, triggers = verb /dɪzɜːrt/
    HeteronymRule {
        word: "desert",
        default_variant: 0,
        context_variant: 1,
        triggers: &[
            "to", "will", "can", "could", "should", "would", "must", "don't",
        ],
    },
    // minute: default = time /mɪnɪt/, triggers = adjective /maɪnjuːt/
    HeteronymRule {
        word: "minute",
        default_variant: 0,
        context_variant: 1,
        triggers: &["a", "the", "very", "most", "so", "how", "extremely"],
    },
    // bass: default = music /beɪs/, triggers = fish /bæs/
    HeteronymRule {
        word: "bass",
        default_variant: 0,
        context_variant: 1,
        triggers: &[
            "a",
            "the",
            "caught",
            "fishing",
            "lake",
            "sea",
            "striped",
            "largemouth",
        ],
    },
    // wound: default = injury /wuːnd/, triggers = past tense of wind /waʊnd/
    HeteronymRule {
        word: "wound",
        default_variant: 0,
        context_variant: 1,
        triggers: &["he", "she", "they", "i", "we", "had", "was", "were"],
    },
    // dove: default = bird /dʌv/, triggers = past tense of dive /doʊv/
    HeteronymRule {
        word: "dove",
        default_variant: 0,
        context_variant: 1,
        triggers: &["he", "she", "they", "i", "we", "then", "and"],
    },
    // sow: default = verb plant /soʊ/, triggers = female pig /saʊ/
    HeteronymRule {
        word: "sow",
        default_variant: 0,
        context_variant: 1,
        triggers: &["a", "the", "old", "fat"],
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_known_heteronym() {
        assert!(lookup("read").is_some());
        assert!(lookup("lead").is_some());
        assert!(lookup("live").is_some());
    }

    #[test]
    fn test_lookup_unknown_word() {
        assert!(lookup("hello").is_none());
        assert!(lookup("world").is_none());
    }

    #[test]
    fn test_select_variant_default() {
        let rule = lookup("read").unwrap();
        // No trigger words → default variant
        assert_eq!(select_variant(rule, &["the", "book"]), 0);
    }

    #[test]
    fn test_select_variant_triggered() {
        let rule = lookup("read").unwrap();
        // "to" triggers non-default variant
        assert_eq!(select_variant(rule, &["want", "to"]), 1);
    }

    #[test]
    fn test_select_variant_trigger_in_window() {
        let rule = lookup("read").unwrap();
        // "will" within 3-word lookback
        assert_eq!(select_variant(rule, &["i", "will"]), 1);
    }

    #[test]
    fn test_serde_roundtrip() {
        let rule = lookup("read").unwrap();
        let owned = HeteronymRuleOwned::from(rule);
        let json = serde_json::to_string(&owned).unwrap();
        let roundtripped: HeteronymRuleOwned = serde_json::from_str(&json).unwrap();
        assert_eq!(owned, roundtripped);
    }

    #[test]
    fn test_all_heteronyms_have_triggers() {
        for rule in HETERONYMS {
            assert!(
                !rule.triggers.is_empty(),
                "heteronym {:?} has no triggers",
                rule.word
            );
        }
    }
}
