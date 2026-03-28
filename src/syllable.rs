//! Syllabification of phoneme sequences.
//!
//! Splits a phoneme sequence into syllables using the Maximal Onset Principle
//! with sonority-based constraints. Each syllable has an onset (consonants),
//! nucleus (vowel/diphthong), and coda (consonants).

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use svara::phoneme::{Phoneme, PhonemeClass};

/// A syllable within a word.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Syllable {
    /// Onset consonant(s) before the nucleus.
    pub onset: Vec<Phoneme>,
    /// Nucleus vowel or diphthong (the stress-bearing element).
    pub nucleus: Phoneme,
    /// Coda consonant(s) after the nucleus.
    pub coda: Vec<Phoneme>,
}

impl Syllable {
    /// A syllable is "heavy" if it has a non-empty coda or a diphthong nucleus.
    ///
    /// Heavy syllables attract stress in English.
    #[must_use]
    pub fn is_heavy(&self) -> bool {
        !self.coda.is_empty() || self.nucleus.class() == PhonemeClass::Diphthong
    }

    /// Returns all phonemes in this syllable (onset + nucleus + coda).
    #[must_use]
    pub fn phonemes(&self) -> Vec<Phoneme> {
        let mut ph = self.onset.clone();
        ph.push(self.nucleus);
        ph.extend_from_slice(&self.coda);
        ph
    }
}

/// Syllabifies a phoneme sequence using the Maximal Onset Principle.
///
/// Returns an empty vector if the input contains no vowels.
#[must_use]
pub fn syllabify(phonemes: &[Phoneme]) -> Vec<Syllable> {
    if phonemes.is_empty() {
        return Vec::new();
    }

    // Find indices of all nuclei (vowels and diphthongs)
    let nuclei: Vec<usize> = phonemes
        .iter()
        .enumerate()
        .filter(|(_, p)| is_nucleus(p))
        .map(|(i, _)| i)
        .collect();

    if nuclei.is_empty() {
        return Vec::new();
    }

    let mut syllables = Vec::with_capacity(nuclei.len());

    for (syl_idx, &nucleus_idx) in nuclei.iter().enumerate() {
        let onset_start = if syl_idx == 0 {
            // First syllable: all preceding consonants are onset
            0
        } else {
            // Split consonants between previous coda and this onset
            let prev_nucleus = nuclei[syl_idx - 1];
            let cluster_start = prev_nucleus + 1;
            let cluster = &phonemes[cluster_start..nucleus_idx];

            // Apply Maximal Onset Principle: give as many consonants as
            // possible to the onset of this syllable
            let onset_len = max_legal_onset(cluster);
            nucleus_idx - onset_len
        };

        let coda_end = if syl_idx == nuclei.len() - 1 {
            // Last syllable: all following consonants are coda
            phonemes.len()
        } else {
            // Coda extends to just before the next syllable's onset
            // (will be determined when the next syllable is processed)
            // For now, include up to the next nucleus
            nuclei[syl_idx + 1]
        };

        let onset = phonemes[onset_start..nucleus_idx].to_vec();
        let nucleus = phonemes[nucleus_idx];
        let coda_slice = &phonemes[nucleus_idx + 1..coda_end];

        // For non-final syllables, we need to split the coda from the next onset.
        // The coda is what's left after the next syllable takes its onset.
        let coda = if syl_idx < nuclei.len() - 1 {
            let next_nucleus = nuclei[syl_idx + 1];
            let cluster = &phonemes[nucleus_idx + 1..next_nucleus];
            let next_onset_len = max_legal_onset(cluster);
            phonemes[nucleus_idx + 1..next_nucleus - next_onset_len].to_vec()
        } else {
            coda_slice.to_vec()
        };

        syllables.push(Syllable {
            onset,
            nucleus,
            coda,
        });
    }

    syllables
}

/// Returns true if a phoneme can serve as a syllable nucleus.
fn is_nucleus(ph: &Phoneme) -> bool {
    matches!(ph.class(), PhonemeClass::Vowel | PhonemeClass::Diphthong)
}

/// Returns the sonority level of a phoneme (higher = more sonorous).
fn sonority(ph: &Phoneme) -> u8 {
    match ph.class() {
        PhonemeClass::Vowel | PhonemeClass::Diphthong => 6,
        _ => match ph {
            Phoneme::ApproximantR | Phoneme::ApproximantW | Phoneme::ApproximantJ => 5,
            Phoneme::LateralL => 4,
            Phoneme::NasalM | Phoneme::NasalN | Phoneme::NasalNg => 3,
            Phoneme::FricativeF
            | Phoneme::FricativeV
            | Phoneme::FricativeS
            | Phoneme::FricativeZ
            | Phoneme::FricativeSh
            | Phoneme::FricativeZh
            | Phoneme::FricativeTh
            | Phoneme::FricativeDh
            | Phoneme::FricativeH => 2,
            _ => 1, // plosives, affricates, glottal stop, etc.
        },
    }
}

/// Determines the maximum number of consonants from the END of a cluster
/// that form a legal English onset (rising sonority toward nucleus).
fn max_legal_onset(cluster: &[Phoneme]) -> usize {
    if cluster.is_empty() {
        return 0;
    }

    // Try from the full cluster down to 1 consonant
    for start in 0..cluster.len() {
        let candidate = &cluster[start..];
        if is_legal_onset(candidate) {
            return candidate.len();
        }
    }

    // Single consonant is always a legal onset
    1
}

/// Checks if a consonant cluster is a legal English onset.
fn is_legal_onset(cluster: &[Phoneme]) -> bool {
    if cluster.is_empty() {
        return true;
    }
    if cluster.len() == 1 {
        return true;
    }

    // Check sonority rises from left to right
    for i in 1..cluster.len() {
        if sonority(&cluster[i]) <= sonority(&cluster[i - 1]) {
            // Exception: /s/ + plosive clusters (sp, st, sk, spl, spr, str, skr)
            if cluster[0] == Phoneme::FricativeS && sonority(&cluster[i]) <= 2 {
                continue;
            }
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syllabify_monosyllable() {
        let phonemes = alloc::vec![Phoneme::PlosiveK, Phoneme::VowelAsh, Phoneme::PlosiveT];
        let syls = syllabify(&phonemes);
        assert_eq!(syls.len(), 1);
        assert_eq!(syls[0].onset, alloc::vec![Phoneme::PlosiveK]);
        assert_eq!(syls[0].nucleus, Phoneme::VowelAsh);
        assert_eq!(syls[0].coda, alloc::vec![Phoneme::PlosiveT]);
        assert!(syls[0].is_heavy()); // has coda
    }

    #[test]
    fn test_syllabify_two_syllables() {
        // "hello" = h + ɛ + l + oʊ
        let phonemes = alloc::vec![
            Phoneme::FricativeH,
            Phoneme::VowelOpenE,
            Phoneme::LateralL,
            Phoneme::DiphthongOU,
        ];
        let syls = syllabify(&phonemes);
        assert_eq!(syls.len(), 2);
        // hɛ.loʊ — l goes to onset of second syllable (MOP)
        assert_eq!(syls[0].nucleus, Phoneme::VowelOpenE);
        assert_eq!(syls[1].nucleus, Phoneme::DiphthongOU);
    }

    #[test]
    fn test_syllabify_empty() {
        assert!(syllabify(&[]).is_empty());
    }

    #[test]
    fn test_syllabify_no_vowels() {
        let phonemes = alloc::vec![Phoneme::PlosiveK, Phoneme::PlosiveT];
        assert!(syllabify(&phonemes).is_empty());
    }

    #[test]
    fn test_heavy_syllable_with_coda() {
        let syl = Syllable {
            onset: alloc::vec![Phoneme::PlosiveK],
            nucleus: Phoneme::VowelAsh,
            coda: alloc::vec![Phoneme::PlosiveT],
        };
        assert!(syl.is_heavy());
    }

    #[test]
    fn test_light_syllable() {
        let syl = Syllable {
            onset: alloc::vec![Phoneme::PlosiveK],
            nucleus: Phoneme::VowelAsh,
            coda: alloc::vec![],
        };
        assert!(!syl.is_heavy());
    }

    #[test]
    fn test_heavy_with_diphthong() {
        let syl = Syllable {
            onset: alloc::vec![],
            nucleus: Phoneme::DiphthongAI,
            coda: alloc::vec![],
        };
        assert!(syl.is_heavy()); // diphthong = heavy
    }

    #[test]
    fn test_serde_roundtrip() {
        let syl = Syllable {
            onset: alloc::vec![Phoneme::PlosiveK],
            nucleus: Phoneme::VowelAsh,
            coda: alloc::vec![Phoneme::PlosiveT],
        };
        let json = serde_json::to_string(&syl).unwrap();
        let syl2: Syllable = serde_json::from_str(&json).unwrap();
        assert_eq!(syl, syl2);
    }

    #[test]
    fn test_syllable_phonemes() {
        let syl = Syllable {
            onset: alloc::vec![Phoneme::PlosiveK],
            nucleus: Phoneme::VowelAsh,
            coda: alloc::vec![Phoneme::PlosiveT],
        };
        assert_eq!(
            syl.phonemes(),
            alloc::vec![Phoneme::PlosiveK, Phoneme::VowelAsh, Phoneme::PlosiveT]
        );
    }
}
