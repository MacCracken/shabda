# shabda Roadmap

## Multi-Language (future)

**Boundary**: varna provides phoneme inventories and phonotactic constraints per language. shabda provides the G2P rules (orthography → sound). varna never does G2P — it's the reference data, shabda is the engine, shabdakosh is the cache.

- [ ] German G2P rules — validated against `varna::phoneme::german()`
- [ ] Hindi/Devanagari G2P (nearly 1:1) — validated against `varna::phoneme::hindi()`
- [ ] Sanskrit G2P (perfectly regular Devanagari) — validated against `varna::phoneme::sanskrit()`
- [ ] Arabic G2P rules (consonantal root + vowel patterns) — validated against `varna::phoneme::arabic()`
