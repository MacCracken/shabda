//! Integration tests for shabda.

use shabda::prelude::*;

#[test]
fn test_hello_world() {
    let g2p = G2PEngine::new(Language::English);
    let events = g2p.convert("hello world").unwrap();
    assert!(!events.is_empty(), "should produce phoneme events");
    // Should have phonemes + silence between words
    assert!(events.len() > 4, "hello world needs multiple phonemes");
}

#[test]
fn test_empty_input_errors() {
    let g2p = G2PEngine::new(Language::English);
    assert!(g2p.convert("").is_err());
    assert!(g2p.convert("   ").is_err());
}

#[test]
fn test_dictionary_lookup() {
    let g2p = G2PEngine::new(Language::English);
    // "the" is in the dictionary
    let events = g2p.convert("the").unwrap();
    assert!(!events.is_empty());
}

#[test]
fn test_rule_fallback() {
    let g2p = G2PEngine::new(Language::English);
    // "flurb" is not in any dictionary — rules handle it
    let events = g2p.convert("flurb").unwrap();
    assert!(!events.is_empty());
}

#[test]
fn test_speak_produces_audio() {
    let g2p = G2PEngine::new(Language::English);
    let voice = svara::voice::VoiceProfile::new_male();
    let samples = g2p.speak("hello", &voice, 44100.0).unwrap();
    assert!(!samples.is_empty());
    assert!(samples.iter().all(|s| s.is_finite()));
    let max_amp: f32 = samples.iter().map(|s| s.abs()).fold(0.0, f32::max);
    assert!(max_amp > 0.001, "should produce audible output");
}

#[test]
fn test_speak_sentence() {
    let g2p = G2PEngine::new(Language::English);
    let voice = svara::voice::VoiceProfile::new_female();
    let samples = g2p.speak("hello world", &voice, 44100.0).unwrap();
    assert!(!samples.is_empty());
    assert!(samples.iter().all(|s| s.is_finite()));
}

#[test]
fn test_question_intonation() {
    let g2p = G2PEngine::new(Language::English);
    let voice = svara::voice::VoiceProfile::new_male();
    let samples = g2p.speak("hello?", &voice, 44100.0).unwrap();
    assert!(!samples.is_empty());
}

#[test]
fn test_custom_dictionary_entry() {
    let mut g2p = G2PEngine::new(Language::English);
    g2p.dictionary_mut().insert(
        "agnos",
        &[
            svara::phoneme::Phoneme::VowelAsh,
            svara::phoneme::Phoneme::PlosiveG,
            svara::phoneme::Phoneme::NasalN,
            svara::phoneme::Phoneme::VowelO,
            svara::phoneme::Phoneme::FricativeS,
        ],
    );
    let events = g2p.convert("agnos").unwrap();
    assert!(events.len() >= 5);
}

#[test]
fn test_content_vs_function_word_stress() {
    let g2p = G2PEngine::new(Language::English);
    let events = g2p.convert("the cat").unwrap();
    // "the" should be unstressed, "cat" should have primary stress
    let the_events: Vec<_> = events.iter().take(2).collect(); // "the" = 2 phonemes
    assert_eq!(the_events[0].stress, svara::prosody::Stress::Unstressed);
}

#[test]
fn test_serde_roundtrip_language() {
    let json = serde_json::to_string(&Language::English).unwrap();
    let l2: Language = serde_json::from_str(&json).unwrap();
    assert_eq!(l2, Language::English);
}

#[test]
fn test_serde_roundtrip_engine() {
    let engine = G2PEngine::new(Language::English);
    let json = serde_json::to_string(&engine).unwrap();
    let e2: G2PEngine = serde_json::from_str(&json).unwrap();
    assert_eq!(e2.language(), Language::English);
    assert_eq!(e2.dictionary().len(), engine.dictionary().len());
}

#[test]
fn test_serde_roundtrip_error() {
    let err = ShabdaError::UnknownWord("test".into());
    let json = serde_json::to_string(&err).unwrap();
    let e2: ShabdaError = serde_json::from_str(&json).unwrap();
    assert_eq!(err.to_string(), e2.to_string());
}
