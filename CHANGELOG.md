# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
