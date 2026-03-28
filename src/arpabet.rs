//! ARPABET-to-svara phoneme mapping.
//!
//! The CMU Pronouncing Dictionary uses ARPABET notation for phonemes.
//! This module maps ARPABET symbols to svara [`Phoneme`] variants.
//!
//! Vowels in ARPABET carry a stress digit (0 = no stress, 1 = primary, 2 = secondary).
//! The stress digit is stripped before phoneme lookup and extracted separately.
//!
//! Note: the mapping logic is duplicated in `build.rs` because build scripts
//! cannot import from the crate being built. Keep both in sync.

use svara::phoneme::Phoneme;

/// Converts an ARPABET symbol (without stress digit) to a svara [`Phoneme`].
///
/// Returns `None` if the symbol is not recognized.
#[must_use]
pub fn arpabet_to_phoneme(symbol: &str) -> Option<Phoneme> {
    match symbol {
        // Vowels
        "AA" => Some(Phoneme::VowelOpenA),
        "AE" => Some(Phoneme::VowelAsh),
        "AH" => Some(Phoneme::VowelCupV), // stressed AH; see `arpabet_to_phoneme_with_stress` for schwa
        "AO" => Some(Phoneme::VowelOpenO),
        "AW" => Some(Phoneme::DiphthongAU),
        "AY" => Some(Phoneme::DiphthongAI),
        "EH" => Some(Phoneme::VowelOpenE),
        "ER" => Some(Phoneme::VowelBird),
        "EY" => Some(Phoneme::DiphthongEI),
        "IH" => Some(Phoneme::VowelNearI),
        "IY" => Some(Phoneme::VowelE),
        "OW" => Some(Phoneme::DiphthongOU),
        "OY" => Some(Phoneme::DiphthongOI),
        "UH" => Some(Phoneme::VowelNearU),
        "UW" => Some(Phoneme::VowelU),

        // Consonants
        "B" => Some(Phoneme::PlosiveB),
        "CH" => Some(Phoneme::AffricateCh),
        "D" => Some(Phoneme::PlosiveD),
        "DH" => Some(Phoneme::FricativeDh),
        "F" => Some(Phoneme::FricativeF),
        "G" => Some(Phoneme::PlosiveG),
        "HH" => Some(Phoneme::FricativeH),
        "JH" => Some(Phoneme::AffricateJ),
        "K" => Some(Phoneme::PlosiveK),
        "L" => Some(Phoneme::LateralL),
        "M" => Some(Phoneme::NasalM),
        "N" => Some(Phoneme::NasalN),
        "NG" => Some(Phoneme::NasalNg),
        "P" => Some(Phoneme::PlosiveP),
        "R" => Some(Phoneme::ApproximantR),
        "S" => Some(Phoneme::FricativeS),
        "SH" => Some(Phoneme::FricativeSh),
        "T" => Some(Phoneme::PlosiveT),
        "TH" => Some(Phoneme::FricativeTh),
        "V" => Some(Phoneme::FricativeV),
        "W" => Some(Phoneme::ApproximantW),
        "Y" => Some(Phoneme::ApproximantJ),
        "Z" => Some(Phoneme::FricativeZ),
        "ZH" => Some(Phoneme::FricativeZh),

        _ => None,
    }
}

/// Strips the trailing stress digit (0/1/2) from an ARPABET symbol.
///
/// Returns `(base_symbol, stress_digit)`. If no digit is present, stress is `None`.
#[must_use]
pub fn strip_stress(symbol: &str) -> (&str, Option<u8>) {
    if let Some(last) = symbol.as_bytes().last() {
        match last {
            b'0' => (&symbol[..symbol.len() - 1], Some(0)),
            b'1' => (&symbol[..symbol.len() - 1], Some(1)),
            b'2' => (&symbol[..symbol.len() - 1], Some(2)),
            _ => (symbol, None),
        }
    } else {
        (symbol, None)
    }
}

/// Converts an ARPABET symbol (possibly with stress digit) to a svara [`Phoneme`],
/// handling the AH/schwa distinction: unstressed AH (stress 0) maps to [`Phoneme::VowelSchwa`],
/// while stressed AH (1 or 2) maps to [`Phoneme::VowelCupV`].
#[must_use]
pub fn arpabet_to_phoneme_with_stress(symbol: &str) -> Option<Phoneme> {
    let (base, stress) = strip_stress(symbol);

    // Special case: unstressed AH is schwa
    if base == "AH" && stress == Some(0) {
        return Some(Phoneme::VowelSchwa);
    }

    arpabet_to_phoneme(base)
}

/// Converts a svara [`Phoneme`] back to an ARPABET symbol (without stress digit).
///
/// Returns `None` for phonemes that have no ARPABET equivalent (e.g., `Silence`).
///
/// Note: `VowelSchwa` maps to `AH` (the stress distinction is lost without context).
/// `VowelCupV` also maps to `AH`.
#[must_use]
pub fn phoneme_to_arpabet(phoneme: &Phoneme) -> Option<&'static str> {
    match phoneme {
        // Vowels
        Phoneme::VowelOpenA => Some("AA1"),
        Phoneme::VowelAsh => Some("AE1"),
        Phoneme::VowelSchwa => Some("AH0"),
        Phoneme::VowelCupV => Some("AH1"),
        Phoneme::VowelOpenO => Some("AO1"),
        Phoneme::DiphthongAU => Some("AW1"),
        Phoneme::DiphthongAI => Some("AY1"),
        Phoneme::VowelOpenE => Some("EH1"),
        Phoneme::VowelBird => Some("ER1"),
        Phoneme::DiphthongEI => Some("EY1"),
        Phoneme::VowelNearI => Some("IH1"),
        Phoneme::VowelE => Some("IY1"),
        Phoneme::DiphthongOU => Some("OW1"),
        Phoneme::DiphthongOI => Some("OY1"),
        Phoneme::VowelNearU => Some("UH1"),
        Phoneme::VowelU => Some("UW1"),

        // Consonants
        Phoneme::PlosiveB => Some("B"),
        Phoneme::AffricateCh => Some("CH"),
        Phoneme::PlosiveD => Some("D"),
        Phoneme::FricativeDh => Some("DH"),
        Phoneme::FricativeF => Some("F"),
        Phoneme::PlosiveG => Some("G"),
        Phoneme::FricativeH => Some("HH"),
        Phoneme::AffricateJ => Some("JH"),
        Phoneme::PlosiveK => Some("K"),
        Phoneme::LateralL => Some("L"),
        Phoneme::NasalM => Some("M"),
        Phoneme::NasalN => Some("N"),
        Phoneme::NasalNg => Some("NG"),
        Phoneme::PlosiveP => Some("P"),
        Phoneme::ApproximantR => Some("R"),
        Phoneme::FricativeS => Some("S"),
        Phoneme::FricativeSh => Some("SH"),
        Phoneme::PlosiveT => Some("T"),
        Phoneme::FricativeTh => Some("TH"),
        Phoneme::FricativeV => Some("V"),
        Phoneme::ApproximantW => Some("W"),
        Phoneme::ApproximantJ => Some("Y"),
        Phoneme::FricativeZ => Some("Z"),
        Phoneme::FricativeZh => Some("ZH"),

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vowel_mappings() {
        assert_eq!(arpabet_to_phoneme("AA"), Some(Phoneme::VowelOpenA));
        assert_eq!(arpabet_to_phoneme("AE"), Some(Phoneme::VowelAsh));
        assert_eq!(arpabet_to_phoneme("AH"), Some(Phoneme::VowelCupV));
        assert_eq!(arpabet_to_phoneme("AO"), Some(Phoneme::VowelOpenO));
        assert_eq!(arpabet_to_phoneme("AW"), Some(Phoneme::DiphthongAU));
        assert_eq!(arpabet_to_phoneme("AY"), Some(Phoneme::DiphthongAI));
        assert_eq!(arpabet_to_phoneme("EH"), Some(Phoneme::VowelOpenE));
        assert_eq!(arpabet_to_phoneme("ER"), Some(Phoneme::VowelBird));
        assert_eq!(arpabet_to_phoneme("EY"), Some(Phoneme::DiphthongEI));
        assert_eq!(arpabet_to_phoneme("IH"), Some(Phoneme::VowelNearI));
        assert_eq!(arpabet_to_phoneme("IY"), Some(Phoneme::VowelE));
        assert_eq!(arpabet_to_phoneme("OW"), Some(Phoneme::DiphthongOU));
        assert_eq!(arpabet_to_phoneme("OY"), Some(Phoneme::DiphthongOI));
        assert_eq!(arpabet_to_phoneme("UH"), Some(Phoneme::VowelNearU));
        assert_eq!(arpabet_to_phoneme("UW"), Some(Phoneme::VowelU));
    }

    #[test]
    fn test_consonant_mappings() {
        assert_eq!(arpabet_to_phoneme("B"), Some(Phoneme::PlosiveB));
        assert_eq!(arpabet_to_phoneme("CH"), Some(Phoneme::AffricateCh));
        assert_eq!(arpabet_to_phoneme("D"), Some(Phoneme::PlosiveD));
        assert_eq!(arpabet_to_phoneme("DH"), Some(Phoneme::FricativeDh));
        assert_eq!(arpabet_to_phoneme("F"), Some(Phoneme::FricativeF));
        assert_eq!(arpabet_to_phoneme("G"), Some(Phoneme::PlosiveG));
        assert_eq!(arpabet_to_phoneme("HH"), Some(Phoneme::FricativeH));
        assert_eq!(arpabet_to_phoneme("JH"), Some(Phoneme::AffricateJ));
        assert_eq!(arpabet_to_phoneme("K"), Some(Phoneme::PlosiveK));
        assert_eq!(arpabet_to_phoneme("L"), Some(Phoneme::LateralL));
        assert_eq!(arpabet_to_phoneme("M"), Some(Phoneme::NasalM));
        assert_eq!(arpabet_to_phoneme("N"), Some(Phoneme::NasalN));
        assert_eq!(arpabet_to_phoneme("NG"), Some(Phoneme::NasalNg));
        assert_eq!(arpabet_to_phoneme("P"), Some(Phoneme::PlosiveP));
        assert_eq!(arpabet_to_phoneme("R"), Some(Phoneme::ApproximantR));
        assert_eq!(arpabet_to_phoneme("S"), Some(Phoneme::FricativeS));
        assert_eq!(arpabet_to_phoneme("SH"), Some(Phoneme::FricativeSh));
        assert_eq!(arpabet_to_phoneme("T"), Some(Phoneme::PlosiveT));
        assert_eq!(arpabet_to_phoneme("TH"), Some(Phoneme::FricativeTh));
        assert_eq!(arpabet_to_phoneme("V"), Some(Phoneme::FricativeV));
        assert_eq!(arpabet_to_phoneme("W"), Some(Phoneme::ApproximantW));
        assert_eq!(arpabet_to_phoneme("Y"), Some(Phoneme::ApproximantJ));
        assert_eq!(arpabet_to_phoneme("Z"), Some(Phoneme::FricativeZ));
        assert_eq!(arpabet_to_phoneme("ZH"), Some(Phoneme::FricativeZh));
    }

    #[test]
    fn test_unknown_symbol() {
        assert_eq!(arpabet_to_phoneme("XX"), None);
        assert_eq!(arpabet_to_phoneme(""), None);
    }

    #[test]
    fn test_strip_stress() {
        assert_eq!(strip_stress("AH0"), ("AH", Some(0)));
        assert_eq!(strip_stress("AH1"), ("AH", Some(1)));
        assert_eq!(strip_stress("AH2"), ("AH", Some(2)));
        assert_eq!(strip_stress("B"), ("B", None));
        assert_eq!(strip_stress(""), ("", None));
    }

    #[test]
    fn test_ah_schwa_distinction() {
        // Unstressed AH -> schwa
        assert_eq!(
            arpabet_to_phoneme_with_stress("AH0"),
            Some(Phoneme::VowelSchwa)
        );
        // Stressed AH -> cup-v
        assert_eq!(
            arpabet_to_phoneme_with_stress("AH1"),
            Some(Phoneme::VowelCupV)
        );
        assert_eq!(
            arpabet_to_phoneme_with_stress("AH2"),
            Some(Phoneme::VowelCupV)
        );
    }

    #[test]
    fn test_with_stress_strips_digits() {
        assert_eq!(
            arpabet_to_phoneme_with_stress("AA1"),
            Some(Phoneme::VowelOpenA)
        );
        assert_eq!(
            arpabet_to_phoneme_with_stress("IY0"),
            Some(Phoneme::VowelE)
        );
        // Consonants have no stress digit
        assert_eq!(
            arpabet_to_phoneme_with_stress("B"),
            Some(Phoneme::PlosiveB)
        );
    }

    #[test]
    fn test_serde_roundtrip() {
        // Verify all mapped phonemes survive serde roundtrip
        let symbols = [
            "AA", "AE", "AH", "AO", "AW", "AY", "EH", "ER", "EY", "IH", "IY", "OW", "OY",
            "UH", "UW", "B", "CH", "D", "DH", "F", "G", "HH", "JH", "K", "L", "M", "N", "NG",
            "P", "R", "S", "SH", "T", "TH", "V", "W", "Y", "Z", "ZH",
        ];
        for sym in symbols {
            let phoneme = arpabet_to_phoneme(sym).unwrap();
            let json = serde_json::to_string(&phoneme).unwrap();
            let roundtripped: Phoneme = serde_json::from_str(&json).unwrap();
            assert_eq!(phoneme, roundtripped, "serde roundtrip failed for {sym}");
        }
    }
}
