# ADR-003: Phrase Boundary Markers in Normalization

## Status

Accepted (2026-03-27)

## Context

The G2P pipeline normalizes text (lowercasing, stripping punctuation) before the engine processes words. But commas and periods carry prosodic meaning — they represent phrase boundaries with pauses. We need a way to preserve this information through normalization.

## Decision

Insert **sentinel tokens** during normalization that survive tokenization:

- Comma → `,pause` token (150ms silence in output)
- Period/semicolon → `.pause` token (300ms silence in output)

The engine recognizes these tokens during word processing and emits `Phoneme::Silence` events with the appropriate duration instead of attempting dictionary lookup.

## Alternatives Considered

- **Pre-parse punctuation positions**: Track character offsets before normalization. Rejected — adds complexity to the engine and couples it to the normalizer's behavior.
- **Two-pass normalization**: First pass extracts punctuation, second pass normalizes. Rejected — more complex than sentinel tokens.

## Consequences

- **Simple**: One-pass normalization, engine just checks for known tokens
- **Extensible**: Additional punctuation (ellipsis, em-dash) can be added as new marker tokens
- **Limitation**: Marker tokens could theoretically collide with real words (`,pause` is not an English word, so this is safe)
