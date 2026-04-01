# Multi-Language Support

shabda supports multiple languages for G2P conversion. Each language has its own rule engine and phoneme inventory.

## Supported Languages

| Language | Variant | Dictionary | Rules |
|----------|---------|------------|-------|
| English | General American | shabdakosh (10K+ entries) | Full (silent letters, morphology, magic-e) |
| Spanish | Castilian | None (rules-only) | Full (ch, ll, rr, qu, gu, c/g/z context) |

## Usage

```rust
use shabda::prelude::*;

// English (default — has dictionary + rules)
let en = G2PEngine::new(Language::English);
let events = en.convert("hello world").unwrap();

// Spanish (rules-only, no dictionary)
let es = G2PEngine::new(Language::Spanish);
let events = es.convert("hola mundo").unwrap();
```

## Spanish Rules

Spanish orthography is highly regular. Key rules:

- **Digraphs**: ch→/tʃ/, ll→/ʎ/, rr→/r̄/, qu→/k/
- **Context-sensitive**: c before e/i→/θ/, g before e/i→/x/, gu before e/i→/g/ (u silent)
- **Silent h**: always silent
- **b/v merger**: both→/b/
- **z→/θ/**: Castilian theta (not Latin American /s/)

## Phoneme Inventory (varna feature)

With the `varna` feature enabled, you can inspect the phoneme inventory:

```rust
let g2p = G2PEngine::new(Language::Spanish);
let inv = g2p.phoneme_inventory();
assert!(inv.has("θ"));  // Castilian theta
assert!(inv.has("ɲ"));  // ñ
```
