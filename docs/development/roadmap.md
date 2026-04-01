# shabda Roadmap

## Multi-Language v2.0 (breaking — expand Language enum)

**Boundary**: varna provides phoneme inventories and phonotactic constraints per language. shabda provides the G2P rules (orthography → sound). varna never does G2P — it's the reference data, shabda is the engine, shabdakosh is the cache.

- [ ] `Language` enum expands with variants mapped to varna inventories + per-language rule modules
- [ ] Spanish G2P rules — validated against `varna::phoneme::spanish()`
- [ ] German G2P rules — validated against `varna::phoneme::german()`
- [ ] Hindi/Devanagari G2P (nearly 1:1) — validated against `varna::phoneme::hindi()`
- [ ] Sanskrit G2P (perfectly regular Devanagari) — validated against `varna::phoneme::sanskrit()`
- [ ] Arabic G2P rules (consonantal root + vowel patterns) — validated against `varna::phoneme::arabic()`
- [ ] `G2PEngine::phoneme_inventory()` — returns varna `PhonemeInventory` for active language
- [ ] Phonotactic constraint checking on rule output (reject impossible sequences per language)
