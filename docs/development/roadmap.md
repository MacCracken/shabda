# shabda Roadmap

## Completed

### v0.1.0 — Initial Scaffold (2026-03-27)

- G2P engine with dictionary lookup + English rule-based fallback
- Built-in English pronunciation dictionary (~30 entries)
- Text normalization, tokenization, sentence type detection
- Automatic stress assignment (content vs function words)
- `speak()` one-call text-to-audio via svara
- Integration tests, benchmarks, no_std, serde

## Backlog — High Priority

### Dictionary Expansion — Extracted to shabdakosh
- [x] Expand English dictionary to 5,000+ common words (CMU Pronouncing Dictionary format)
- [x] Import/export dictionary format (JSON, CMUdict-compatible)
- [x] User dictionary overlay (application-specific terms)
- [x] Extracted into standalone `shabdakosh` crate

### Rule Engine Improvements — Completed
- [x] Context-sensitive rules (magic-e, r-colored vowels)
- [x] Silent letter handling (kn, gn, wr, ps, pn, mn, mb, bt, igh, eigh, ough)
- [x] Morphological awareness (un-/re-/dis- prefixes, -tion/-sion suffixes, -ed /t/d/ɪd/)
- [x] Stress from syllable weight (heavy penult rule)
- [x] Syllabification algorithm (Maximal Onset Principle with sonority)
- [x] Number-to-words expansion (0–999,999,999, decimals, negatives)

### Prosody
- [x] Phrase-level prosody (comma = 150ms pause, period = 300ms pause)
- [x] Emphasis markers (CAPS = emphatic stress, *asterisks* = focus)
- [x] Speaking rate control (words per minute)
- [x] SSML subset support (<break>, <emphasis>, <prosody>)

### Multi-Language (varna integration)

**Boundary**: varna provides phoneme inventories and phonotactic constraints per language. shabda provides the G2P rules (orthography → sound). varna never does G2P — it's the reference data, shabda is the engine, shabdakosh is the cache.

**v1.x (non-breaking, optional `varna` feature)**:
- [x] Optional `varna` feature flag — inventory validation during rule development
- [x] Debug assertions verify rule output phonemes exist in varna's inventory for target language
- [x] Language detection (auto-detect from text using varna's script/character set data)

**v2.0 (breaking — expand Language enum)**:
- [ ] `Language` enum expands with variants mapped to varna inventories + per-language rule modules
- [ ] Spanish G2P rules — validated against `varna::phoneme::spanish()`
- [ ] German G2P rules — validated against `varna::phoneme::german()`
- [ ] Hindi/Devanagari G2P (nearly 1:1) — validated against `varna::phoneme::hindi()`
- [ ] Sanskrit G2P (perfectly regular Devanagari) — validated against `varna::phoneme::sanskrit()`
- [ ] Arabic G2P rules (consonantal root + vowel patterns) — validated against `varna::phoneme::arabic()`
- [ ] `G2PEngine::phoneme_inventory()` — returns varna `PhonemeInventory` for active language
- [ ] Phonotactic constraint checking on rule output (reject impossible sequences per language)

## Backlog — Medium Priority

### Accuracy
- [x] Heteronym disambiguation (read/read, live/live, wind/wind — context-based heuristic with 20-entry table)
- [x] Abbreviation expansion (Dr., St., etc.)
- [x] Number-to-words (42 → "forty two") — done in rule engine improvements
- [x] Acronym handling (NASA, FBI — spell out vs pronounce)
- [x] Foreign word detection and passthrough

### Performance
- [ ] Dictionary trie for O(1) lookup instead of BTreeMap — shabdakosh scope
- [x] Rule compilation (precompute pattern matching tables — static slices, zero allocation)
- [ ] Lazy dictionary loading (load on first use, not construction) — shabdakosh scope

### Integration
- [x] SSML parser
- [x] Phoneme-level timing control (explicit duration overrides)
- [x] Callback API for streaming G2P (word-by-word for real-time)

## v1.0 Criteria

- [x] English dictionary with 10,000+ entries (via shabdakosh)
- [x] Syllabification algorithm
- [x] Morphological awareness (-tion, -ed, un-, re-)
- [x] Silent letter handling
- [x] Context-sensitive vowel rules
- [x] Phrase-level prosody (commas, periods)
- [x] Number-to-words conversion
- [x] All public types: Serialize + Deserialize + roundtrip tested
- [x] Benchmarks baselined
- [x] Comprehensive documentation
