# Error Handling Guide

shabda uses a single error enum `ShabdaError` for all fallible operations. Errors are descriptive, serializable, and composable with the `?` operator.

## Error Types

```rust
pub enum ShabdaError {
    InvalidInput(String),       // empty or invalid text
    UnknownWord(String),        // word could not be converted (available for strict mode)
    UnsupportedLanguage(String),// language not supported
    RuleError(String),          // rule application or audio synthesis failed
    DictParseError(String),     // dictionary parsing/IO failed
}
```

### InvalidInput

Returned when `convert()` or `speak()` receives empty or whitespace-only text.

```rust
use shabda::prelude::*;

let g2p = G2PEngine::new(Language::English);

match g2p.convert("") {
    Err(ShabdaError::InvalidInput(msg)) => {
        println!("Invalid: {msg}"); // "invalid input: empty text"
    }
    _ => unreachable!(),
}
```

### RuleError

Returned when svara audio synthesis fails (e.g., invalid voice profile or sample rate).

```rust
match g2p.speak("hello", &voice, sample_rate) {
    Err(ShabdaError::RuleError(msg)) => {
        // msg includes context: "audio synthesis failed: ..."
        eprintln!("Synthesis error: {msg}");
    }
    Ok(samples) => { /* use samples */ }
    Err(e) => { /* other errors */ }
}
```

### DictParseError

Returned when parsing dictionary files (CMUdict, IPA, PLS, JSON) via shabdakosh. The error message includes line numbers and the specific parse failure.

```rust
use shabda::dictionary::format;

match format::parse_cmudict("bad input") {
    Err(e) => {
        // e wraps shabdakosh::ShabdakoshError via From impl
        eprintln!("Parse error: {e}");
    }
    Ok(dict) => { /* use dict */ }
}
```

### UnknownWord

Available for callers implementing strict mode. The engine itself does not emit this error — it gracefully skips words that produce no phonemes (logging a `warn!` trace). Callers who want strict behavior can check phoneme counts:

```rust
let events = g2p.convert("xyzzy").unwrap();
if events.is_empty() {
    // No phonemes produced — word was unknown to both dict and rules
}
```

## Error Conversion

`ShabdakoshError` automatically converts to `ShabdaError` via `From`:

```rust
// This works because ShabdakoshError -> ShabdaError::DictParseError
let dict = shabda::dictionary::format::parse_cmudict(input)?;
```

## Graceful Degradation

The G2P pipeline is designed to be resilient:

| Situation | Behavior |
|-----------|----------|
| Word not in dictionary | Falls back to rule engine (trace log) |
| Rules produce empty phonemes | Skips word, logs `warn!` |
| Number in text | Expanded to words automatically |
| Unknown punctuation | Stripped silently |
| Comma/period | Converted to phrase pause |

The only hard error is empty input text (`InvalidInput`). Everything else degrades gracefully with logging.

## Serde Support

All error variants are `Serialize + Deserialize`:

```rust
let err = ShabdaError::InvalidInput("empty text".into());
let json = serde_json::to_string(&err).unwrap();
let err2: ShabdaError = serde_json::from_str(&json).unwrap();
assert_eq!(err.to_string(), err2.to_string());
```

## Best Practices

1. **Enable tracing in development** to see dictionary hit/miss patterns
2. **Check `warn!` logs** for words that produce no phonemes — add them to the user dictionary
3. **Use `?` propagation** — all errors implement `std::error::Error` (with `std` feature)
4. **Pattern match** on specific variants when you need different recovery strategies
