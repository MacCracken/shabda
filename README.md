# shabda

**shabda** (Sanskrit: word / sound) — Grapheme-to-phoneme (G2P) conversion for Rust.

The bridge between text and vocal synthesis. Converts English and Spanish text to phoneme sequences ready for [svara](https://crates.io/crates/svara)'s synthesis engine. Uses [shabdakosh](https://crates.io/crates/shabdakosh)'s 10,000+ entry pronunciation dictionary with intelligent rule-based fallback.

## Features

- **Multi-language G2P**: English (General American) and Spanish (Castilian)
- **10,000+ word dictionary**: Via shabdakosh — CMUdict-derived, O(1) lookup, variant pronunciations
- **Rule-based fallback**: Context-sensitive letter-to-sound rules for unknown words
- **Silent letter handling**: knight, gnome, write, psychology, lamb, etc.
- **Morphological awareness**: -tion/-sion suffixes, -ed endings (/t/ vs /d/ vs /id/), un-/re-/dis- prefixes
- **Magic-e and r-colored vowels**: make→/meik/, car→/kar/, bird→/berd/
- **Syllabification**: Maximal Onset Principle with sonority constraints
- **Syllable-weight stress**: Heavy penult rule for polysyllabic words
- **Prosody control**: Phrase pauses, emphasis markers (CAPS, \*asterisks\*), speaking rate (WPM), timing profiles
- **SSML support**: `<break>`, `<emphasis>`, `<prosody>` tags via `convert_ssml()`
- **Streaming API**: Word-by-word callback via `convert_streaming()`
- **Accuracy**: Abbreviation expansion (Dr.→doctor), acronym handling (NASA vs FBI), foreign word detection, heteronym disambiguation
- **Number expansion**: 42→"forty two", 3.14→"three point one four"
- **`speak()` method**: Text to audio samples in one call (G2P + svara rendering)
- **varna integration**: Phoneme inventory validation and language detection (optional)
- **Serde support**: All types serialize/deserialize
- **`no_std` compatible**: Works with alloc, no standard library required

## Quick Start

```rust
use shabda::prelude::*;

// Convert text to phonemes
let g2p = G2PEngine::new(Language::English);
let events = g2p.convert("hello world").unwrap();

// With options: emphasis + slow rate
let opts = ConvertOptions::new()
    .with_emphasis(true)
    .with_speaking_rate(100.0);
let events = g2p.convert_with("HELLO world", &opts).unwrap();

// SSML input
let events = g2p.convert_ssml(
    r#"Hello <break time="300ms"/> <emphasis level="strong">world</emphasis>"#
).unwrap();

// Spanish
let g2p_es = G2PEngine::new(Language::Spanish);
let events = g2p_es.convert("hola mundo").unwrap();

// Direct to audio
let voice = svara::voice::VoiceProfile::new_male();
let samples = g2p.speak("hello world", &voice, 44100.0).unwrap();
```

## Architecture

```text
Input text
    |
    v
Abbreviation Expansion (Dr. → doctor)
    |
    v
Acronym Handling (FBI → f b i, NASA → nasa)
    |
    v
Number Expansion (42 → "forty two")
    |
    v
Normalizer (lowercase, punctuation → phrase markers, emphasis markers)
    |
    v
G2P Engine
    |-- Heteronym check (read/read, live/live — context-based)
    |-- Dictionary lookup (shabdakosh, 10K+ entries)
    |-- Foreign word detection (strip diacritics, retry)
    |-- Rule-based fallback (English or Spanish rules)
    v
Syllabifier (Maximal Onset Principle)
    |
    v
Prosody Mapper (stress, emphasis, rate, timing)
    |
    v
Vec<PhonemeEvent> (ready for svara)
```

## Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `std` | Yes | Standard library support. Disable for `no_std` + `alloc` |
| `logging` | No | Structured logging via tracing-subscriber |
| `json` | No | JSON dictionary import/export via serde_json |
| `varna` | No | Phoneme inventory validation and language detection |
| `full` | No | All of the above |

## Consumers

- **dhvani** — AGNOS audio engine (text-to-speech pipeline)
- **vansh** — Voice AI shell
- Any application needing text-to-speech with svara

## License

GPL-3.0-only
