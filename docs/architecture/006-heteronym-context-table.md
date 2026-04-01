# ADR-006: Heteronym Disambiguation via Context Table

**Status**: Accepted (2026-04-01)

## Context

Heteronyms (read/read, live/live, wind/wind) need different pronunciations based on meaning. Full POS tagging is complex and requires external dependencies.

## Decision

Use a static 20-entry table of heteronym rules with preceding-word context triggers. Each rule maps a word to two pronunciation variant indices and a list of trigger words. The engine checks the previous 1-3 words for triggers before dictionary lookup.

Example: "read" defaults to variant 0 (past tense /red/). If preceded by "to", "will", "can", etc., selects variant 1 (present tense /ri:d/).

## Alternatives Considered

- **POS tagger**: Accurate but heavy dependency, not `no_std` compatible
- **Always default pronunciation**: Simple but incorrect for many common words
- **Machine learning**: Overkill for 20 words

## Consequences

- Covers the most common heteronyms with reasonable accuracy
- No external dependencies
- False positives possible with unusual sentence structures
- Only works when shabdakosh has multiple pronunciation variants for the word
- Easy to extend by adding entries to the table
