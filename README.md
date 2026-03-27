# shabda

**shabda** (Sanskrit: शब्द — word / sound) — Grapheme-to-phoneme (G2P) conversion for Rust.

The bridge between text and vocal synthesis. Converts English text to phoneme sequences ready for [svara](https://crates.io/crates/svara)'s synthesis engine. Dictionary-based with rule-based fallback, automatic stress assignment, and prosody mapping from punctuation.

## Features

- **G2P engine**: Text → phoneme events in one call
- **Pronunciation dictionary**: Built-in English irregular words (the, one, colonel, etc.)
- **Rule-based fallback**: Letter-to-sound rules for unknown words (~90% accuracy)
- **Stress assignment**: Content words get primary stress, function words reduced
- **Prosody mapping**: Question/exclamation/statement detection from punctuation
- **`speak()` method**: Text → audio samples in one call (G2P + svara rendering)
- **Custom dictionaries**: Add application-specific pronunciations
- **Serde support**: All types serialize/deserialize
- **`no_std` compatible**

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

```
"hello world" → Normalize → Tokenize → Dictionary/Rules → Stress → PhonemeEvents
                                                                        |
                                                                      svara
                                                                        |
                                                                   audio samples
```

## Consumers

- **dhvani** — AGNOS audio engine (text-to-speech pipeline)
- **vansh** — Voice AI shell
- Any application needing text-to-speech with svara

## License

GPL-3.0-only
