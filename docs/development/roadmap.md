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
- [ ] Phrase-level prosody (comma = continuation rise, period = falling)
- [ ] Emphasis markers (CAPS = emphatic stress, *asterisks* = focus)
- [ ] Speaking rate control (words per minute)
- [ ] SSML subset support (<break>, <emphasis>, <prosody>)

### Multi-Language
- [ ] Language detection (auto-detect from text)
- [ ] Spanish G2P rules (highly regular orthography)
- [ ] German G2P rules
- [ ] Hindi/Devanagari G2P (direct phoneme mapping — nearly 1:1)
- [ ] Language-specific phoneme inventories

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
- [ ] Phrase-level prosody (commas, periods)
- [x] Number-to-words conversion
- [x] All public types: Serialize + Deserialize + roundtrip tested
- [x] Benchmarks baselined
- [ ] Comprehensive documentation
