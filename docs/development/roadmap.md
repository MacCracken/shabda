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
- [ ] Emphasis markers (CAPS = emphatic stress, *asterisks* = focus)
- [ ] Speaking rate control (words per minute)
- [ ] SSML subset support (<break>, <emphasis>, <prosody>)

### Multi-Language (lipi integration)

**Boundary**: lipi provides phoneme inventories and phonotactic constraints per language. shabda provides the G2P rules (orthography → sound). lipi never does G2P — it's the reference data, shabda is the engine, shabdakosh is the cache.

**v1.x (non-breaking, optional `lipi` feature)**:
- [ ] Optional `lipi` feature flag — inventory validation during rule development
- [ ] Debug assertions verify rule output phonemes exist in lipi's inventory for target language
- [ ] Language detection (auto-detect from text using lipi's script/character set data)

**v2.0 (breaking — expand Language enum)**:
- [ ] `Language` enum expands with variants mapped to lipi inventories + per-language rule modules
- [ ] Spanish G2P rules — validated against `lipi::phoneme::spanish()`
- [ ] German G2P rules — validated against `lipi::phoneme::german()`
- [ ] Hindi/Devanagari G2P (nearly 1:1) — validated against `lipi::phoneme::hindi()`
- [ ] Sanskrit G2P (perfectly regular Devanagari) — validated against `lipi::phoneme::sanskrit()`
- [ ] Arabic G2P rules (consonantal root + vowel patterns) — validated against `lipi::phoneme::arabic()`
- [ ] `G2PEngine::phoneme_inventory()` — returns lipi `PhonemeInventory` for active language
- [ ] Phonotactic constraint checking on rule output (reject impossible sequences per language)

## Backlog — Medium Priority

### Accuracy
- [ ] Heteronym disambiguation (read/read, live/live, wind/wind — needs POS tagging)
- [ ] Abbreviation expansion (Dr., St., etc.)
- [x] Number-to-words (42 → "forty two") — done in rule engine improvements
- [ ] Acronym handling (NASA, FBI — spell out vs pronounce)
- [ ] Foreign word detection and passthrough

### Performance
- [ ] Dictionary trie for O(1) lookup instead of BTreeMap
- [ ] Rule compilation (precompute pattern matching tables)
- [ ] Lazy dictionary loading (load on first use, not construction)

### Integration
- [ ] SSML parser
- [ ] Phoneme-level timing control (explicit duration overrides)
- [ ] Callback API for streaming G2P (word-by-word for real-time)

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
- [ ] Comprehensive documentation
