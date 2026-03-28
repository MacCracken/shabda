# ADR-001: Dictionary-First, Rules as Fallback

## Status

Accepted (2026-03-27)

## Context

English pronunciation is highly irregular. A purely rule-based system achieves ~90% accuracy. A purely dictionary-based system covers only known words. We need a strategy that maximizes accuracy while handling novel/unknown words.

## Decision

Use a **dictionary-first, rules-as-fallback** architecture:

1. Look up every word in shabdakosh's 10,000+ entry dictionary (O(1) via HashMap)
2. Only if the dictionary misses, apply the rule engine
3. User overlay entries take highest precedence

## Consequences

- **Accuracy**: Irregular words (colonel, psychology, one) are always correct
- **Coverage**: Unknown words still get reasonable pronunciation via rules
- **Performance**: Dictionary lookup is O(1); rules only fire on misses (~5-10% of common text)
- **Extensibility**: Users can add domain terms via overlay without modifying rules
- **Dependency**: shabda depends on shabdakosh as a separate crate
