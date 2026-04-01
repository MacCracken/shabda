# ADR-007: Multi-Language Dispatch via Language Enum

**Status**: Accepted (2026-04-01)

## Context

shabda needs to support multiple languages. Each language has different G2P rules, phoneme inventories, and potentially different dictionaries.

## Decision

The `Language` enum (already `#[non_exhaustive]`) gains new variants per language. The engine dispatches to language-specific rule functions and dictionaries via `match self.language`. Languages without dictionaries (e.g., Spanish) use an empty `PronunciationDict` and rely entirely on rules.

The `varna` feature provides per-language phoneme inventory validation via `phoneme_to_ipa_for(phoneme, language)`, since the same svara `Phoneme` variant maps to different IPA strings in different languages (e.g., `VowelO` is "ou" in English but "o" in Spanish).

## Consequences

- Adding a Language variant is a breaking change (exhaustive match arms break downstream)
- Each new language needs: rule function, IPA mapping, inventory_for entry
- Spanish is rules-only (no dictionary); English has dictionary + rules
- `is_content_word()` in prosody is currently English-only — needs per-language function word lists eventually
