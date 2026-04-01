# Prosody Control

shabda provides several ways to control speech prosody (emphasis, rate, timing).

## ConvertOptions

Use `ConvertOptions` with `convert_with()` for per-call control:

```rust
use shabda::prelude::*;

let g2p = G2PEngine::new(Language::English);

// Enable emphasis detection
let opts = ConvertOptions::new().with_emphasis(true);
let events = g2p.convert_with("This is IMPORTANT", &opts).unwrap();
// "IMPORTANT" gets primary stress + 1.3x duration

// Slow speech (75 WPM, default is 150)
let opts = ConvertOptions::new().with_speaking_rate(75.0);
let events = g2p.convert_with("slow and clear", &opts).unwrap();
```

## Emphasis Markers

When `emphasis: true`, two patterns are detected:

- **ALL-CAPS** (3+ letters): `"HELLO"` → emphatic stress
- **\*asterisks\***: `"*important*"` → emphatic stress

Emphasis boosts vowels to Primary stress and scales duration by 1.3x.

## Speaking Rate

`with_speaking_rate(wpm)` scales all phoneme durations. Clamped to 50-300 WPM. Default is 150 WPM. Minimum durations enforced (30ms consonants, 50ms vowels).

## Timing Profiles

For fine-grained control, use `TimingProfile` to scale vowels, consonants, and pauses independently:

```rust
use shabda::prelude::*;

let crisp = TimingProfile::new(0.8, 1.0, 0.7);   // shorter vowels + pauses
let slow = TimingProfile::new(1.3, 1.0, 1.5);     // longer vowels + pauses

let opts = ConvertOptions::new().with_timing(crisp);
```

## SSML

For markup-based control, use `convert_ssml()`:

```rust
let events = g2p.convert_ssml(r#"
    Hello <break time="500ms"/>
    <emphasis level="strong">world</emphasis>
    <prosody rate="slow">how are you</prosody>
"#).unwrap();
```

Supported tags: `<speak>`, `<break>`, `<emphasis>`, `<prosody>`.
