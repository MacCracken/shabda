# Logging Guide

shabda uses [tracing](https://crates.io/crates/tracing) for structured logging throughout the G2P pipeline. Logs are emitted at `trace` and `warn` levels — no logs are emitted by default unless a subscriber is configured.

## Enabling Logging

### With the `logging` feature (tracing-subscriber)

```toml
[dependencies]
shabda = { version = "1", features = ["logging"] }
```

```rust
// Initialize the subscriber (once, at program start)
tracing_subscriber::fmt()
    .with_env_filter("shabda=trace")
    .init();

let g2p = shabda::engine::G2PEngine::new(shabda::engine::Language::English);
let events = g2p.convert("hello world").unwrap();
```

### With your own tracing subscriber

shabda emits standard `tracing` spans and events. Any subscriber works:

```rust
// Use your existing subscriber — shabda logs will appear automatically
let g2p = shabda::engine::G2PEngine::new(shabda::engine::Language::English);
let events = g2p.convert("42 knights").unwrap();
```

### Environment variable

With the `logging` feature, control verbosity via `RUST_LOG`:

```sh
RUST_LOG=shabda=trace cargo run    # all trace-level detail
RUST_LOG=shabda=warn cargo run     # only warnings (empty phonemes)
RUST_LOG=shabda=info cargo run     # quiet (no shabda output currently at info)
```

## What Gets Logged

### engine.rs — `trace` level

| Event | Fields | When |
|-------|--------|------|
| `converting text to phonemes` | `input`, `normalized`, `intonation` | Start of `convert()` |
| `dictionary hit` | `word`, `phoneme_count` | Word found in shabdakosh dictionary |
| `dictionary miss, falling back to rules` | `word` | Word not in dictionary, rules engine used |
| `syllabified` | `word`, `syllable_count`, `is_content` | After syllabification |
| `no syllables (consonant-only), using simple stress` | `word` | Fallback for consonant-only sequences |

### engine.rs — `warn` level

| Event | Fields | When |
|-------|--------|------|
| `no phonemes produced, skipping word` | `word` | Both dictionary and rules returned empty |

### rules.rs — `trace` level

| Event | Fields | When |
|-------|--------|------|
| `silent letter preprocessing complete` | `word` | After stripping silent letters |
| `prefix detected and stripped` | `word` | Morphological prefix (un-, re-, dis-) found |

### normalize.rs — `trace` level

| Event | Fields | When |
|-------|--------|------|
| `expanded number` | `num_str`, `expanded` | Integer/decimal expanded to words |
| `expanded negative number` | `num_str`, `expanded` | Negative number expanded |

## Example Output

```
TRACE shabda::engine: converting text to phonemes input="42 knights" normalized="forty two nights" intonation=Statement
TRACE shabda::normalize: expanded number num_str="42" expanded="forty two"
TRACE shabda::engine: dictionary hit word="forty" phoneme_count=5
TRACE shabda::engine: dictionary hit word="two" phoneme_count=3
TRACE shabda::rules: silent letter preprocessing complete word="nights"
TRACE shabda::engine: dictionary miss, falling back to rules word="nights"
TRACE shabda::engine: syllabified word="nights" syllable_count=1 is_content=true
```

## Performance

Tracing is zero-cost when no subscriber is active. The `trace!` and `warn!` macros compile to no-ops unless a subscriber is registered. There is no runtime overhead in production builds without a subscriber.
