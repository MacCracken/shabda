# Multi-Language Support

shabda supports multiple languages for G2P conversion. Each language has its own rule engine and phoneme inventory.

## Supported Languages

| Language | Variant | Dictionary | Rules |
|----------|---------|------------|-------|
| English | General American | shabdakosh (10K+ entries) | Full (silent letters, morphology, magic-e) |
| Spanish | Castilian | None (rules-only) | Full (ch, ll, rr, qu, gu, c/g/z context) |
| German | Standard High German | None (rules-only) | Full (sch, ch, ei/ie/eu, umlauts, final devoicing) |
| Hindi | Modern Standard | None (rules-only) | Devanagari + romanized fallback |

## Usage

```rust
use shabda::prelude::*;

// English (has dictionary + rules)
let en = G2PEngine::new(Language::English);
let events = en.convert("hello world").unwrap();

// Spanish (rules-only)
let es = G2PEngine::new(Language::Spanish);
let events = es.convert("hola mundo").unwrap();

// German (rules-only)
let de = G2PEngine::new(Language::German);
let events = de.convert("guten tag").unwrap();

// Hindi (Devanagari or romanized)
let hi = G2PEngine::new(Language::Hindi);
let events = hi.convert("नमस्ते").unwrap();
let events = hi.convert("namaste").unwrap(); // romanized also works
```

## German Rules

Key features:
- **Digraphs**: sch→/ʃ/, ch→/ç/ or /x/, ei→/aɪ/, ie→/iː/, eu/äu→/ɔɪ/
- **Affricates**: pf→/pf/, z→/ts/
- **Umlauts**: ä→/ɛ/, ö→/œ/, ü→/y/
- **Final devoicing**: b→p, d→t, g→k at word end
- **ß**: always /s/
- **v→/f/**, **w→/v/** (unlike English)
- **Double consonants**: collapsed to single phoneme

## Hindi Rules

Hindi/Devanagari has near-1:1 grapheme-to-phoneme mapping:
- Each consonant carries an inherent schwa (अ)
- Virama (्) suppresses the inherent schwa
- Vowel matras (ा, ि, ी, etc.) replace the schwa
- Word-final schwa deletion (standard Hindi rule)
- Supports romanized input as fallback

## Phoneme Inventory (varna feature)

With the `varna` feature enabled:

```rust
let g2p = G2PEngine::new(Language::German);
let inv = g2p.phoneme_inventory();
assert!(inv.has("ʃ"));  // sch
assert!(inv.has("ç"));  // ich-Laut
```
