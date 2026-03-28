//! Basic grapheme-to-phoneme conversion example.

fn main() {
    let g2p = shabda::engine::G2PEngine::new(shabda::engine::Language::English);

    // Simple conversion
    let events = g2p.convert("hello world").unwrap();
    println!("hello world -> {} phoneme events", events.len());

    // Numbers are expanded automatically
    let events = g2p.convert("I have 42 cats").unwrap();
    println!("I have 42 cats -> {} phoneme events", events.len());

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
}
