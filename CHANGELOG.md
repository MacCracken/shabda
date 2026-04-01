# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.0.0] — 2026-04-01

Multi-language G2P, prosody control, SSML, accuracy, and varna integration.

- **Breaking**: `Language` enum expanded with `Spanish`, `German`, `Hindi`, `Arabic`, `Sanskrit` variants
- **Feature**: Spanish G2P rules — Castilian orthography (ch, ll, rr, qu, gu, c/g/z context)
- **Feature**: German G2P rules — sch, ch (ich/ach-Laut), ei/ie/eu digraphs, umlauts, final devoicing, ß
- **Feature**: Hindi G2P rules — Devanagari with inherent schwa deletion, virama/matra, romanized fallback
- **Feature**: Arabic G2P rules — 28 consonants, diacritics, shadda, tanween, hamza, romanized fallback
- **Feature**: Sanskrit G2P rules — perfectly regular Devanagari (no schwa deletion), 36C + 14V, romanized fallback
- **Feature**: `ConvertOptions` with builder pattern — `convert_with()` and `speak_with()` for emphasis, rate, timing
- **Feature**: Emphasis markers — ALL-CAPS and `*asterisk*` words receive emphatic stress
- **Feature**: Speaking rate control — `with_speaking_rate(wpm)`, clamped 50–300 WPM
- **Feature**: `TimingProfile` — independent vowel/consonant/pause duration scaling
- **Feature**: SSML subset parser — `<speak>`, `<break>`, `<emphasis>`, `<prosody>` tags
- **Feature**: `convert_ssml()` — applies SSML markup to G2P pipeline
- **Feature**: `convert_streaming()` — word-by-word callback API for real-time G2P
- **Feature**: Abbreviation expansion — 25-entry table (Dr.→doctor, Mr.→mister, etc.)
- **Feature**: Acronym handling — pronounceability heuristic (NASA→word, FBI→spelled out)
- **Feature**: Foreign word detection — diacritic detection with `strip_diacritics()` fallback
- **Feature**: Heteronym disambiguation — 20-entry context table (read, lead, live, wind, etc.)
- **Feature**: Optional `varna` feature — phoneme inventory validation, `detect_language()`, `phoneme_inventory()`
- **Feature**: Phonotactic constraint validation via varna (debug assertions)
- **Feature**: Language-aware IPA mapping — `validate_phonemes_for()` and `phoneme_to_ipa_for()`
- **Performance**: Rule compilation — static pattern slices eliminate per-match Vec allocations

## [1.0.0] — 2026-03-28

Full English G2P pipeline with dictionary, rules, syllabification, and prosody.

- **Feature**: Dictionary extracted to standalone `shabdakosh` crate (10,000+ entries)
- **Feature**: Context-sensitive vowel rules (magic-e, r-colored vowels)
- **Feature**: Silent letter handling (kn, gn, wr, ps, pn, mn, mb, bt, igh, eigh, ough)
- **Feature**: Morphological decomposition (-tion/-sion/-cian, -ed, un-/re-/dis-/pre-/mis-)
- **Feature**: Syllabification (Maximal Onset Principle with sonority constraints)
- **Feature**: Syllable-weight stress (heavy penult rule, antepenult fallback)
- **Feature**: Number-to-words expansion (0–999,999,999, decimals, negatives)
- **Feature**: Phrase-level prosody (commas 150ms, periods 300ms)
- **Changed**: Engine pipeline uses syllabification for stress instead of simple first-vowel rule
- **Changed**: `normalize()` expands numbers before text normalization

## [0.1.0] — 2026-03-27

- **Feature**: Initial scaffold — G2PEngine, dictionary lookup, rule-based fallback
- **Feature**: Built-in English dictionary (~30 entries), extensible
- **Feature**: Text normalization, sentence type detection, stress assignment
- **Feature**: `speak()` one-call text-to-audio via svara
- **Feature**: Integration tests, benchmarks, no_std, serde
