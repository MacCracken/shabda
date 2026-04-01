# shabda — Claude Code Instructions

## Project Identity

**shabda** (Sanskrit: word / sound) — Grapheme-to-phoneme conversion for AGNOS

- **Type**: Flat library crate
- **License**: GPL-3.0
- **MSRV**: 1.89
- **Version**: SemVer — `VERSION` file is source of truth; use `scripts/version-bump.sh <ver>` to update

## Consumers

dhvani (audio engine), vansh (voice AI shell), and any AGNOS component needing text-to-speech.

## Dependencies

- **shabdakosh**: Pronunciation dictionary (10K+ entries, ARPABET/IPA, user overlay)
- **svara**: Phoneme types, synthesis engine, voice profiles
- **varna** (optional): Phoneme inventories, script detection, language data (50+ languages)

## Work Loop

1. Read the relevant code before proposing changes
2. Make the change
3. Cleanliness check:
   - `cargo fmt`
   - `cargo clippy --all-features --all-targets -- -D warnings`
   - `cargo audit`
   - `cargo deny check`
   - `RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps`
4. `cargo test --all-features`
5. `cargo test --doc`
6. `cargo check --no-default-features` (no_std verification)
7. `cargo bench` or `./scripts/bench-history.sh` (if performance-relevant)
8. Update CHANGELOG.md if user-facing
9. Update docs/development/roadmap.md if completing a roadmap item

## Task Sizing

- **Small**: Single-rule addition, test fix, doc tweak
- **Medium**: New rule pattern set, prosody feature, varna inventory gap fixes
- **Large**: New language G2P module (German, Hindi, Sanskrit, Arabic)

## Key Principles

- Never skip benchmarks
- `#[non_exhaustive]` on ALL public enums
- `#[must_use]` on all pure functions
- Every type must be Serialize + Deserialize (serde)
- Zero unwrap/panic in library code
- All types must have serde roundtrip tests
- Dictionary-first, rules as fallback — accuracy over speed
- Phoneme output must be compatible with svara's PhonemeEvent
- When `varna` feature is active, debug assertions validate phoneme output against varna's inventory

## Feature Flags

- `std` (default) — Standard library; disable for `no_std` + `alloc`
- `logging` — Structured logging via tracing-subscriber
- `json` — JSON serialization via serde_json
- `varna` — Phoneme inventory validation and language detection via varna
- `full` — All of the above

## Module Structure

- `engine.rs` — G2PEngine, Language, ConvertOptions, TimingProfile, convert/convert_with/convert_ssml/convert_streaming/speak/speak_with, detect_language (varna), phoneme_inventory (varna)
- `rules.rs` — English and Spanish letter-to-sound rules (fallback when dictionary misses)
- `normalize.rs` — Text normalization, abbreviation/acronym expansion, foreign word detection, emphasis markers
- `prosody.rs` — Stress assignment, emphasis, speaking rate, timing profiles, intonation mapping
- `syllable.rs` — Syllabification using Maximal Onset Principle with sonority constraints
- `ssml.rs` — SSML subset parser (break, emphasis, prosody)
- `heteronym.rs` — Heteronym disambiguation with context triggers
- `validate.rs` — Phoneme→IPA mapping (per-language), inventory + phonotactic validation (varna feature)
- `error.rs` — ShabdaError (wraps ShabdakoshError via From impl)
- Re-exports from shabdakosh: `arpabet`, `dictionary`

## DO NOT

- **Do not commit or push** — the user handles all git operations
- **NEVER use `gh` CLI** — use `curl` to GitHub API only
- Do not add unnecessary dependencies
- Do not skip benchmarks before claiming performance improvements

## Documentation

- CHANGELOG.md: Keep a Changelog format (Added/Changed/Fixed/Removed)
- README.md: Quick start, feature list, architecture
- docs/development/roadmap.md: Completed versions + backlog

## CHANGELOG Format

```
## [version] — YYYY-MM-DD

Description.

- **Feature**: description
- **Breaking**: description
```
