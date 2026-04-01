//! Basic grapheme-to-phoneme conversion example.

use shabda::prelude::*;

fn main() {
    let g2p = G2PEngine::new(Language::English);

    // Simple conversion
    let events = g2p.convert("hello world").unwrap();
    println!("hello world -> {} phoneme events", events.len());

    // Numbers are expanded automatically
    let events = g2p.convert("I have 42 cats").unwrap();
    println!("I have 42 cats -> {} phoneme events", events.len());

    // Abbreviations are expanded
    let events = g2p.convert("Dr. Smith is here").unwrap();
    println!("Dr. Smith -> {} phoneme events", events.len());

    // Phrase pauses at commas and periods
    let events = g2p.convert("first, second. third").unwrap();
    let pauses = events
        .iter()
        .filter(|e| e.phoneme == svara::phoneme::Phoneme::Silence && e.duration > 0.1)
        .count();
    println!("Phrase pauses detected: {pauses}");

    // Silent letters handled correctly
    let events = g2p.convert("knight").unwrap();
    println!(
        "knight -> {} phoneme events (silent k stripped)",
        events.len()
    );

    // Spanish
    let g2p_es = G2PEngine::new(Language::Spanish);
    let events = g2p_es.convert("hola mundo").unwrap();
    println!("hola mundo -> {} phoneme events (Spanish)", events.len());
}
