//! Adding custom pronunciations to the G2P engine.

fn main() {
    let mut g2p = shabda::engine::G2PEngine::new(shabda::engine::Language::English);

    // Add a custom word to the user overlay
    g2p.dictionary_mut().insert_user(
        "agnos",
        &[
            svara::phoneme::Phoneme::VowelAsh,
            svara::phoneme::Phoneme::PlosiveG,
            svara::phoneme::Phoneme::NasalN,
            svara::phoneme::Phoneme::VowelO,
            svara::phoneme::Phoneme::FricativeS,
        ],
    );

    let events = g2p.convert("welcome to agnos").unwrap();
    println!("welcome to agnos -> {} phoneme events", events.len());

    // User entries override dictionary entries
    g2p.dictionary_mut().insert_user(
        "the",
        &[
            svara::phoneme::Phoneme::FricativeDh,
            svara::phoneme::Phoneme::VowelE,
        ],
    );

    // The override takes effect immediately
    let events = g2p.convert("the").unwrap();
    println!("overridden 'the' -> {} phoneme events", events.len());

    // Remove the override — base dictionary pronunciation returns
    g2p.dictionary_mut().remove_user("the");
}
