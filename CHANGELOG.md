# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **varna integration**: Optional `varna` feature flag for phoneme inventory validation and language detection
- **Phoneme validation**: `validate` module maps svara phonemes to IPA and validates against varna's `PhonemeInventory`; debug assertions in `convert()` catch invalid phoneme output during development
- **Language detection**: `detect_language()` auto-detects language from text using varna's script/Unicode range data
- **ConvertOptions**: New `convert_with()` and `speak_with()` methods with builder-pattern options for emphasis, speaking rate, and timing control
- **Emphasis markers**: ALL-CAPS (3+ chars) and `*asterisk*` words receive emphatic stress (1.3x duration, primary stress) via `ConvertOptions::with_emphasis(true)`
- **Speaking rate**: `ConvertOptions::with_speaking_rate(wpm)` scales durations, clamped 50–300 WPM
- **Timing profiles**: `TimingProfile` for independent vowel/consonant/pause duration scaling
- **Abbreviation expansion**: 25-entry table (Dr.→doctor, Mr.→mister, etc.) with word-boundary detection
- **Acronym handling**: 3–5 char ALL-CAPS detection with pronounceability heuristic (NASA→word, FBI→spelled out)
- **Foreign word detection**: Diacritic detection with `strip_diacritics()` fallback (café→cafe)
- **Heteronym disambiguation**: 20-entry table with preceding-word context triggers (read, lead, live, wind, etc.)
- **SSML parser**: Hand-rolled no_std parser supporting `<speak>`, `<break>`, `<emphasis>`, `<prosody>` tags
- **SSML engine support**: `convert_ssml()` method applies SSML markup to G2P pipeline
- **Streaming API**: `convert_streaming()` for word-by-word callback-based G2P processing
- **Rule compilation**: Static pattern slices eliminate per-match Vec allocations in rule engine
- **Silent letter handling**: kn, gn, wr, ps, pn, mn (word-initial), mb, bt, mn, gn (word-final), igh, eigh, augh, ough patterns
- **Context-sensitive vowel rules**: Magic-e (CVCe), r-colored vowels (ar, er, ir, or, ur, air, ear, our)
- **Morphological decomposition**: -tion/-sion/-cian suffixes, -ed post-processing (/t/ vs /d/ vs /ɪd/), prefix stripping (un-, re-, dis-, pre-, mis-)
- **Number-to-words**: `expand_numbers()` in normalize — integers 0–999,999,999, decimals, negatives
- **Syllabification**: New `syllable` module with `Syllable` type and `syllabify()` using Maximal Onset Principle with sonority constraints
- **Syllable-weight stress**: `assign_stress_syllabic()` in prosody — heavy penult rule, antepenult fallback
- **Phrase-level prosody**: Commas insert 150ms pause, periods/semicolons insert 300ms pause
- **Dictionary extracted** to standalone `shabdakosh` crate (10,000+ entries, O(1) lookup, IPA, PLS, SSML)

### Changed

- Engine pipeline now uses syllabification for stress assignment instead of simple first-vowel rule
- `normalize()` now expands numbers before text normalization

## [0.1.0] - 2026-03-27

### Added

- Initial scaffold of the shabda crate
- `G2PEngine`: Text-to-phoneme conversion with dictionary lookup + rule-based fallback
- `PronunciationDict`: Built-in English dictionary (~30 common/irregular words), extensible
- `english_rules()`: Letter-to-sound rules with digraph handling (sh, ch, th, ph, ng, etc.)
- `normalize()`: Text normalization (lowercase, punctuation strip, space collapse)
- `detect_intonation()`: Sentence type detection from punctuation (statement/question/exclamation)
- `assign_stress()`: Automatic stress assignment (content words primary, function words unstressed)
- `speak()`: One-call text-to-audio via G2P + svara rendering
- Integration tests: hello world, dictionary lookup, rule fallback, speak output, custom entries, stress assignment, serde roundtrips
- Criterion benchmarks: G2P conversion, full speak pipeline
- `no_std` support, strict `deny.toml`, Send/Sync assertions
