//! Integration tests for shabda.

use shabda::dictionary::PronunciationDict;
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
    // Should not error even with punctuation
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

// --- Expanded dictionary tests ---

#[test]
fn test_expanded_dictionary_size() {
    let dict = PronunciationDict::english();
    assert!(
        dict.len() >= 5000,
        "expanded dictionary should have 5000+ entries, got {}",
        dict.len()
    );
}

#[test]
fn test_expanded_dictionary_common_words() {
    let g2p = G2PEngine::new(Language::English);
    // These words should all be in the dictionary now
    let words = [
        "people", "because", "through", "enough", "beautiful", "colonel", "psychology",
        "knight", "thought", "language", "world", "hello", "the", "computer", "science",
        "music", "water", "friend", "school", "house",
    ];
    for word in words {
        let events = g2p.convert(word).unwrap();
        assert!(
            !events.is_empty(),
            "'{word}' should produce phoneme events"
        );
    }
}

#[test]
fn test_minimal_dictionary_still_works() {
    let minimal = PronunciationDict::english_minimal();
    assert_eq!(minimal.len(), 29);
    assert!(minimal.lookup("the").is_some());
    assert!(minimal.lookup("hello").is_some());
    assert!(minimal.lookup("computer").is_none());
}

// --- User overlay tests ---

#[test]
fn test_user_overlay_precedence() {
    let mut dict = PronunciationDict::english();
    let original = dict.lookup("hello").unwrap().to_vec();

    // Insert a different pronunciation in the user overlay
    let custom = &[svara::phoneme::Phoneme::VowelA];
    dict.insert_user("hello", custom);

    // User entry should take precedence
    assert_eq!(dict.lookup("hello").unwrap(), custom);

    // Remove user entry, base should come back
    assert!(dict.remove_user("hello"));
    assert_eq!(dict.lookup("hello").unwrap(), original.as_slice());
}

#[test]
fn test_user_overlay_new_word() {
    let mut dict = PronunciationDict::english();
    assert!(dict.lookup("xyzzy").is_none());

    dict.insert_user(
        "xyzzy",
        &[
            svara::phoneme::Phoneme::FricativeZ,
            svara::phoneme::Phoneme::VowelNearI,
            svara::phoneme::Phoneme::FricativeZ,
            svara::phoneme::Phoneme::VowelE,
        ],
    );
    assert!(dict.lookup("xyzzy").is_some());
    assert_eq!(dict.user_len(), 1);
}

#[test]
fn test_user_overlay_serde_roundtrip() {
    let mut dict = PronunciationDict::english_minimal();
    dict.insert_user("custom", &[svara::phoneme::Phoneme::VowelA]);

    let json = serde_json::to_string(&dict).unwrap();
    let dict2: PronunciationDict = serde_json::from_str(&json).unwrap();

    assert_eq!(dict2.lookup("custom").unwrap(), &[svara::phoneme::Phoneme::VowelA]);
    assert_eq!(dict2.user_len(), 1);
    assert_eq!(dict2.len(), dict.len());
}

// --- Format tests ---

#[test]
fn test_cmudict_parse_roundtrip() {
    use shabda::dictionary::format;

    let input = ";;; test dict\nhello  HH AH0 L OW1\nworld  W ER1 L D\n";
    let dict = format::parse_cmudict(input).unwrap();
    assert_eq!(dict.len(), 2);
    assert!(dict.lookup("hello").is_some());
    assert!(dict.lookup("world").is_some());
}

#[test]
fn test_cmudict_export() {
    use shabda::dictionary::format;

    let mut dict = PronunciationDict::new();
    dict.insert(
        "cat",
        &[
            svara::phoneme::Phoneme::PlosiveK,
            svara::phoneme::Phoneme::VowelAsh,
            svara::phoneme::Phoneme::PlosiveT,
        ],
    );
    let output = format::to_cmudict(&dict);
    assert!(output.contains("cat  K AE1 T"));
}

#[test]
fn test_cmudict_parse_error_missing_separator() {
    use shabda::dictionary::format;

    let input = "badline\n";
    let result = format::parse_cmudict(input);
    assert!(result.is_err());
}

#[test]
fn test_cmudict_parse_error_unknown_symbol() {
    use shabda::dictionary::format;

    let input = "word  XX1\n";
    let result = format::parse_cmudict(input);
    assert!(result.is_err());
}

#[test]
fn test_serde_roundtrip_dict_parse_error() {
    let err = ShabdaError::DictParseError("test parse error".into());
    let json = serde_json::to_string(&err).unwrap();
    let e2: ShabdaError = serde_json::from_str(&json).unwrap();
    assert_eq!(err.to_string(), e2.to_string());
}
