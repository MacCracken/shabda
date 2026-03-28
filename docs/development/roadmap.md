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

### Rule Engine Improvements
- [ ] Context-sensitive rules (vowel before consonant cluster affects pronunciation)
- [ ] Silent letter handling (knight, gnome, write, psychology)
- [ ] Morphological awareness (un-prefix, -tion suffix, -ed ending: /t/ vs /d/ vs /ɪd/)
- [ ] Stress from syllable weight (heavy syllables attract stress)
- [ ] Syllabification algorithm (needed for stress rules and hyphenation)

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
- [ ] Number-to-words (42 → "forty two")
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

- [ ] English dictionary with 5,000+ entries
- [ ] Syllabification algorithm
- [ ] Morphological awareness (-tion, -ed, un-, re-)
- [ ] Silent letter handling
- [ ] Context-sensitive vowel rules
- [ ] Phrase-level prosody (commas, periods)
- [ ] Number-to-words conversion
- [ ] All public types: Serialize + Deserialize + roundtrip tested
- [ ] Benchmarks baselined
- [ ] Comprehensive documentation
