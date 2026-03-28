# shabda

**shabda** (Sanskrit: word / sound) — Grapheme-to-phoneme (G2P) conversion for Rust.

The bridge between text and vocal synthesis. Converts English text to phoneme sequences ready for [svara](https://crates.io/crates/svara)'s synthesis engine. Uses [shabdakosh](https://crates.io/crates/shabdakosh)'s 10,000+ entry pronunciation dictionary with intelligent rule-based fallback.

## Features

- **G2P engine**: Text to phoneme events in one call
- **10,000+ word dictionary**: Via shabdakosh — CMUdict-derived, O(1) lookup, variant pronunciations
- **Rule-based fallback**: Context-sensitive letter-to-sound rules for unknown words
- **Silent letter handling**: knight, gnome, write, psychology, lamb, etc.
- **Morphological awareness**: -tion/-sion suffixes, -ed endings (/t/ vs /d/ vs /ɪd/), un-/re-/dis- prefixes
- **Magic-e and r-colored vowels**: make→/meɪk/, car→/kɑr/, bird→/bɜrd/
- **Syllabification**: Maximal Onset Principle with sonority constraints
- **Syllable-weight stress**: Heavy penult rule for polysyllabic words
- **Phrase-level prosody**: Commas insert 150ms pauses, periods insert 300ms pauses
- **Number expansion**: 42→"forty two", 3.14→"three point one four"
- **`speak()` method**: Text to audio samples in one call (G2P + svara rendering)
- **Custom dictionaries**: Add application-specific pronunciations via user overlay
- **Serde support**: All types serialize/deserialize
- **`no_std` compatible**: Works with alloc, no standard library required

## Quick Start

```rust
use shabda::prelude::*;

// Convert text to phonemes
let g2p = G2PEngine::new(Language::English);
let events = g2p.convert("hello world").unwrap();

// Or go directly to audio
let voice = svara::voice::VoiceProfile::new_male();
let samples = g2p.speak("hello world", &voice, 44100.0).unwrap();
```

## Architecture

```text
Input text
    |
    v
Number Expansion (42 → "forty two")
    |
    v
Normalizer (lowercase, punctuation → phrase markers)
    |
    v
Tokenizer (split into words, detect phrase boundaries)
    |
    v
G2P Engine (dictionary lookup → rule-based fallback)
    |  Rules: silent letters → prefix strip → pattern match → suffix post-process
    v
Syllabifier (Maximal Onset Principle)
    |
    v
Prosody Mapper (syllable-weight stress, phrase pauses)
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

## Consumers

- **dhvani** — AGNOS audio engine (text-to-speech pipeline)
- **vansh** — Voice AI shell
- Any application needing text-to-speech with svara

## License

GPL-3.0-only
