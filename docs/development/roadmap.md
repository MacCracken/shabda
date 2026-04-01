# shabda Roadmap

## Multi-Language (future)

**Boundary**: varna provides phoneme inventories and phonotactic constraints per language. shabda provides the G2P rules (orthography → sound). varna never does G2P — it's the reference data, shabda is the engine, shabdakosh is the cache.

- [ ] Sanskrit G2P (perfectly regular Devanagari) — validated against `varna::phoneme::sanskrit()`
- [ ] Arabic G2P rules (consonantal root + vowel patterns) — validated against `varna::phoneme::arabic()`
