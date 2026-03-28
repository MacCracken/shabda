# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

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
