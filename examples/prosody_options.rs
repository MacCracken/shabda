//! Prosody control via ConvertOptions, SSML, and streaming.

use shabda::prelude::*;

fn main() {
    let g2p = G2PEngine::new(Language::English);

    // Emphasis detection (CAPS and *asterisks*)
    let opts = ConvertOptions::new().with_emphasis(true);
    let events = g2p.convert_with("This is VERY *important*", &opts).unwrap();
    let primary = events
        .iter()
        .filter(|e| e.stress == svara::prosody::Stress::Primary)
        .count();
    println!("Emphasis: {primary} primary-stressed phonemes");

    // Slow speaking rate
    let normal = g2p.convert("hello world").unwrap();
    let slow = g2p
        .convert_with(
            "hello world",
            &ConvertOptions::new().with_speaking_rate(75.0),
        )
        .unwrap();
    let normal_dur: f32 = normal.iter().map(|e| e.duration).sum();
    let slow_dur: f32 = slow.iter().map(|e| e.duration).sum();
    println!("Normal: {normal_dur:.3}s, Slow (75 WPM): {slow_dur:.3}s");

    // Timing profile (stretch vowels, compress pauses)
    let opts = ConvertOptions::new().with_timing(TimingProfile::new(1.5, 1.0, 0.5));
    let events = g2p.convert_with("hello world", &opts).unwrap();
    let dur: f32 = events.iter().map(|e| e.duration).sum();
    println!("Custom timing: {dur:.3}s");

    // SSML
    let events = g2p
        .convert_ssml(r#"Hello <break time="500ms"/> <emphasis level="strong">world</emphasis>"#)
        .unwrap();
    println!("SSML: {} events", events.len());

    // Streaming (word-by-word callback)
    print!("Streaming: ");
    g2p.convert_streaming("hello beautiful world", |word, events| {
        print!("{word}({}) ", events.len());
    })
    .unwrap();
    println!();
}
