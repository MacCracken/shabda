# shabda — Claude Code Instructions

## Project Identity

**shabda** (Sanskrit: word / sound) — Grapheme-to-phoneme conversion for AGNOS

- **Type**: Flat library crate
- **License**: GPL-3.0
- **MSRV**: 1.89
- **Version**: SemVer 0.1.0

## Consumers

dhvani (audio engine), vansh (voice AI shell), and any AGNOS component needing text-to-speech.

## Dependencies

- **shabdakosh**: Pronunciation dictionary (10K+ entries, ARPABET/IPA, user overlay)
- **svara**: Phoneme types, synthesis engine, voice profiles

## Work Loop

1. Read the relevant code before proposing changes
2. Make the change
3. `cargo fmt`
4. `cargo clippy --all-features --all-targets -- -D warnings`
5. `cargo test --all-features`
6. `cargo test --doc`
7. `cargo check --no-default-features` (no_std verification)
8. `cargo bench` (if performance-relevant)
9. Update CHANGELOG.md if user-facing
10. Update docs/development/roadmap.md if completing a roadmap item

## Task Sizing

- **Small**: Single-rule addition, test fix, doc tweak
- **Medium**: New rule pattern set (e.g., silent letters), prosody feature
- **Large**: Syllabification algorithm, morphological awareness, new language

## Key Principles

- Never skip benchmarks
- `#[non_exhaustive]` on ALL public enums
- `#[must_use]` on all pure functions
- Every type must be Serialize + Deserialize (serde)
- Zero unwrap/panic in library code
- All types must have serde roundtrip tests
- Dictionary-first, rules as fallback — accuracy over speed
- Phoneme output must be compatible with svara's PhonemeEvent

## Module Structure

- `engine.rs` — G2PEngine, Language, convert(), speak()
- `rules.rs` — English letter-to-sound rules (fallback when dictionary misses)
- `normalize.rs` — Text normalization, sentence type detection
- `prosody.rs` — Stress assignment, intonation mapping
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
