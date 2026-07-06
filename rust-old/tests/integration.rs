//! Integration tests for shabda.

use shabda::engine::ConvertOptions;
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

#[test]
fn test_phrase_pause_comma() {
    let g2p = G2PEngine::new(Language::English);
    let events = g2p.convert("hello, world").unwrap();
    // Should contain a longer silence for the comma (0.15s)
    let pauses: Vec<_> = events
        .iter()
        .filter(|e| e.phoneme == svara::phoneme::Phoneme::Silence && e.duration > 0.1)
        .collect();
    assert!(!pauses.is_empty(), "comma should produce a phrase pause");
}

#[test]
fn test_phrase_pause_period() {
    let g2p = G2PEngine::new(Language::English);
    let events = g2p.convert("hello. world").unwrap();
    // Should contain a longer silence for the period (0.30s)
    let pauses: Vec<_> = events
        .iter()
        .filter(|e| e.phoneme == svara::phoneme::Phoneme::Silence && e.duration > 0.2)
        .collect();
    assert!(
        !pauses.is_empty(),
        "period should produce a longer phrase pause"
    );
}

#[test]
fn test_number_expansion_in_pipeline() {
    let g2p = G2PEngine::new(Language::English);
    // "42" should be expanded to "forty two" and produce phonemes
    let events = g2p.convert("42").unwrap();
    assert!(
        events.len() > 4,
        "number should expand to words with phonemes"
    );
}

// --- ConvertOptions tests ---

#[test]
fn test_convert_with_default_matches_convert() {
    let g2p = G2PEngine::new(Language::English);
    let events_default = g2p.convert("hello world").unwrap();
    let events_with = g2p
        .convert_with("hello world", &ConvertOptions::default())
        .unwrap();
    assert_eq!(events_default.len(), events_with.len());
}

#[test]
fn test_convert_with_emphasis() {
    let g2p = G2PEngine::new(Language::English);
    let opts = ConvertOptions::new().with_emphasis(true);
    let events = g2p.convert_with("HELLO world", &opts).unwrap();
    assert!(!events.is_empty());
    // Emphasized word should have primary stress on all vowels
    let primary_count = events
        .iter()
        .filter(|e| e.stress == svara::prosody::Stress::Primary)
        .count();
    assert!(primary_count >= 1, "emphasis should produce primary stress");
}

#[test]
fn test_convert_with_speaking_rate() {
    let g2p = G2PEngine::new(Language::English);
    let normal = g2p.convert("hello world").unwrap();
    let slow = g2p
        .convert_with(
            "hello world",
            &ConvertOptions::new().with_speaking_rate(75.0),
        )
        .unwrap();
    // Slow speech should have longer total duration
    let normal_dur: f32 = normal.iter().map(|e| e.duration).sum();
    let slow_dur: f32 = slow.iter().map(|e| e.duration).sum();
    assert!(
        slow_dur > normal_dur * 1.5,
        "75 WPM should be ~2x longer than 150 WPM default"
    );
}

#[test]
fn test_serde_roundtrip_convert_options() {
    let opts = ConvertOptions::new()
        .with_emphasis(true)
        .with_speaking_rate(120.0);
    let json = serde_json::to_string(&opts).unwrap();
    let roundtripped: ConvertOptions = serde_json::from_str(&json).unwrap();
    assert_eq!(opts, roundtripped);
}

// --- Accuracy feature tests ---

#[test]
fn test_abbreviation_in_pipeline() {
    let g2p = G2PEngine::new(Language::English);
    // "Dr." should be expanded to "doctor" and produce phonemes
    let events = g2p.convert("Dr. Smith").unwrap();
    assert!(!events.is_empty());
}

#[test]
fn test_acronym_in_pipeline() {
    let g2p = G2PEngine::new(Language::English);
    // "FBI" spelled out should produce phonemes for each letter
    let events = g2p.convert("the FBI").unwrap();
    assert!(!events.is_empty());
}

#[test]
fn test_foreign_word_in_pipeline() {
    let g2p = G2PEngine::new(Language::English);
    // "café" should still produce phonemes (diacritics stripped)
    let events = g2p.convert("café").unwrap();
    assert!(!events.is_empty());
}

#[test]
fn test_heteronym_in_pipeline() {
    let g2p = G2PEngine::new(Language::English);
    // Both contexts should produce phonemes (even if same variant for now)
    let events1 = g2p.convert("I read a book").unwrap();
    let events2 = g2p.convert("please read this").unwrap();
    assert!(!events1.is_empty());
    assert!(!events2.is_empty());
}

// --- SSML tests ---

#[test]
fn test_ssml_basic() {
    let g2p = G2PEngine::new(Language::English);
    let events = g2p.convert_ssml("<speak>hello world</speak>").unwrap();
    assert!(!events.is_empty());
}

#[test]
fn test_ssml_break() {
    let g2p = G2PEngine::new(Language::English);
    let events = g2p
        .convert_ssml("hello <break time=\"500ms\"/> world")
        .unwrap();
    // Should contain a 500ms silence
    let pauses: Vec<_> = events
        .iter()
        .filter(|e| e.phoneme == svara::phoneme::Phoneme::Silence && e.duration > 0.4)
        .collect();
    assert!(!pauses.is_empty(), "should have a 500ms break");
}

#[test]
fn test_ssml_emphasis() {
    let g2p = G2PEngine::new(Language::English);
    let events = g2p
        .convert_ssml("<emphasis level=\"strong\">important</emphasis>")
        .unwrap();
    assert!(!events.is_empty());
    // Strong emphasis should have primary stress
    let has_primary = events
        .iter()
        .any(|e| e.stress == svara::prosody::Stress::Primary);
    assert!(has_primary, "strong emphasis should produce primary stress");
}

#[test]
fn test_ssml_prosody_rate() {
    let g2p = G2PEngine::new(Language::English);
    let normal = g2p.convert("hello world").unwrap();
    let slow = g2p
        .convert_ssml("<prosody rate=\"slow\">hello world</prosody>")
        .unwrap();
    let normal_dur: f32 = normal.iter().map(|e| e.duration).sum();
    let slow_dur: f32 = slow.iter().map(|e| e.duration).sum();
    assert!(
        slow_dur > normal_dur,
        "slow prosody should be longer than normal"
    );
}

#[test]
fn test_ssml_empty_errors() {
    let g2p = G2PEngine::new(Language::English);
    assert!(g2p.convert_ssml("").is_err());
}

// --- Timing + Streaming tests ---

#[test]
fn test_timing_profile() {
    use shabda::engine::TimingProfile;
    let g2p = G2PEngine::new(Language::English);
    let normal = g2p.convert("hello").unwrap();
    let stretched = g2p
        .convert_with(
            "hello",
            &ConvertOptions::new().with_timing(TimingProfile::new(2.0, 1.0, 1.0)),
        )
        .unwrap();
    let normal_dur: f32 = normal.iter().map(|e| e.duration).sum();
    let stretched_dur: f32 = stretched.iter().map(|e| e.duration).sum();
    assert!(
        stretched_dur > normal_dur,
        "2x vowel scale should increase total duration"
    );
}

#[test]
fn test_convert_streaming() {
    let g2p = G2PEngine::new(Language::English);
    let mut words_seen = Vec::new();
    g2p.convert_streaming("hello world", |word, events| {
        words_seen.push(String::from(word));
        assert!(!events.is_empty());
    })
    .unwrap();
    assert!(words_seen.len() >= 2);
}

#[test]
fn test_convert_streaming_empty_errors() {
    let g2p = G2PEngine::new(Language::English);
    assert!(g2p.convert_streaming("", |_, _| {}).is_err());
}

#[test]
fn test_serde_roundtrip_timing_profile() {
    use shabda::engine::TimingProfile;
    let profile = TimingProfile::new(1.2, 0.9, 1.5);
    let json = serde_json::to_string(&profile).unwrap();
    let roundtripped: TimingProfile = serde_json::from_str(&json).unwrap();
    assert_eq!(profile, roundtripped);
}

// --- Spanish G2P tests ---

#[test]
fn test_spanish_hola_mundo() {
    let g2p = G2PEngine::new(Language::Spanish);
    let events = g2p.convert("hola mundo").unwrap();
    assert!(!events.is_empty(), "Spanish G2P should produce phonemes");
}

#[test]
fn test_spanish_convert_with_options() {
    let g2p = G2PEngine::new(Language::Spanish);
    let opts = ConvertOptions::new().with_speaking_rate(100.0);
    let events = g2p.convert_with("buenos días", &opts).unwrap();
    assert!(!events.is_empty());
}

#[test]
fn test_spanish_speak() {
    let g2p = G2PEngine::new(Language::Spanish);
    let voice = svara::voice::VoiceProfile::new_male();
    let samples = g2p.speak("hola", &voice, 44100.0).unwrap();
    assert!(!samples.is_empty());
}

#[test]
fn test_serde_roundtrip_spanish_language() {
    let lang = Language::Spanish;
    let json = serde_json::to_string(&lang).unwrap();
    let roundtripped: Language = serde_json::from_str(&json).unwrap();
    assert_eq!(lang, roundtripped);
}

// --- German tests ---

#[test]
fn test_german_hallo_welt() {
    let g2p = G2PEngine::new(Language::German);
    let events = g2p.convert("hallo welt").unwrap();
    assert!(!events.is_empty());
}

#[test]
fn test_german_speak() {
    let g2p = G2PEngine::new(Language::German);
    let voice = svara::voice::VoiceProfile::new_male();
    let samples = g2p.speak("guten tag", &voice, 44100.0).unwrap();
    assert!(!samples.is_empty());
}

#[test]
fn test_serde_roundtrip_german_language() {
    let lang = Language::German;
    let json = serde_json::to_string(&lang).unwrap();
    let roundtripped: Language = serde_json::from_str(&json).unwrap();
    assert_eq!(lang, roundtripped);
}

// --- Hindi tests ---

#[test]
fn test_hindi_namaste() {
    let g2p = G2PEngine::new(Language::Hindi);
    let events = g2p.convert("नमस्ते").unwrap();
    assert!(!events.is_empty());
}

#[test]
fn test_hindi_romanized() {
    let g2p = G2PEngine::new(Language::Hindi);
    let events = g2p.convert("namaste").unwrap();
    assert!(!events.is_empty());
}

#[test]
fn test_hindi_speak() {
    let g2p = G2PEngine::new(Language::Hindi);
    let voice = svara::voice::VoiceProfile::new_male();
    let samples = g2p.speak("namaste", &voice, 44100.0).unwrap();
    assert!(!samples.is_empty());
}

#[test]
fn test_serde_roundtrip_hindi_language() {
    let lang = Language::Hindi;
    let json = serde_json::to_string(&lang).unwrap();
    let roundtripped: Language = serde_json::from_str(&json).unwrap();
    assert_eq!(lang, roundtripped);
}

#[cfg(feature = "varna")]
#[test]
fn test_phoneme_inventory_german() {
    let g2p = G2PEngine::new(Language::German);
    let inv = g2p.phoneme_inventory();
    assert!(inv.has("p"));
    assert!(inv.has("ʃ"));
}

#[cfg(feature = "varna")]
#[test]
fn test_phoneme_inventory_hindi() {
    let g2p = G2PEngine::new(Language::Hindi);
    let inv = g2p.phoneme_inventory();
    assert!(inv.has("p"));
    assert!(inv.has("ə"));
}

// --- Arabic tests ---

#[test]
fn test_arabic_convert() {
    let g2p = G2PEngine::new(Language::Arabic);
    let events = g2p.convert("كتاب").unwrap();
    assert!(!events.is_empty());
}

#[test]
fn test_arabic_romanized() {
    let g2p = G2PEngine::new(Language::Arabic);
    let events = g2p.convert("salam").unwrap();
    assert!(!events.is_empty());
}

#[test]
fn test_serde_roundtrip_arabic_language() {
    let lang = Language::Arabic;
    let json = serde_json::to_string(&lang).unwrap();
    let roundtripped: Language = serde_json::from_str(&json).unwrap();
    assert_eq!(lang, roundtripped);
}

// --- Sanskrit tests ---

#[test]
fn test_sanskrit_convert() {
    let g2p = G2PEngine::new(Language::Sanskrit);
    let events = g2p.convert("धर्म").unwrap();
    assert!(!events.is_empty());
}

#[test]
fn test_sanskrit_romanized() {
    let g2p = G2PEngine::new(Language::Sanskrit);
    let events = g2p.convert("dharma").unwrap();
    assert!(!events.is_empty());
}

#[test]
fn test_serde_roundtrip_sanskrit_language() {
    let lang = Language::Sanskrit;
    let json = serde_json::to_string(&lang).unwrap();
    let roundtripped: Language = serde_json::from_str(&json).unwrap();
    assert_eq!(lang, roundtripped);
}

#[cfg(feature = "varna")]
#[test]
fn test_phoneme_inventory_arabic() {
    let g2p = G2PEngine::new(Language::Arabic);
    let inv = g2p.phoneme_inventory();
    assert!(inv.has("b"));
    assert!(inv.has("ʔ"));
}

#[cfg(feature = "varna")]
#[test]
fn test_phoneme_inventory_sanskrit() {
    let g2p = G2PEngine::new(Language::Sanskrit);
    let inv = g2p.phoneme_inventory();
    assert!(inv.has("p"));
    assert!(inv.has("ɐ"));
}

#[cfg(feature = "varna")]
#[test]
fn test_phoneme_inventory_english() {
    let g2p = G2PEngine::new(Language::English);
    let inv = g2p.phoneme_inventory();
    assert!(inv.has("p"));
    assert!(inv.has("ʃ"));
    assert!(inv.has("ə"));
}

#[cfg(feature = "varna")]
#[test]
fn test_phoneme_inventory_spanish() {
    let g2p = G2PEngine::new(Language::Spanish);
    let inv = g2p.phoneme_inventory();
    assert!(inv.has("p"));
    assert!(inv.has("θ"));
    assert!(inv.has("a"));
}
