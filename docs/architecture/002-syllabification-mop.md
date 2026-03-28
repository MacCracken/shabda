# ADR-002: Syllabification via Maximal Onset Principle

## Status

Accepted (2026-03-27)

## Context

Syllabification is needed for weight-based stress assignment. English has complex syllable structure with many valid consonant clusters. We need an algorithm that handles the full range of English phonotactics.

## Decision

Use the **Maximal Onset Principle (MOP)** with **sonority sequencing constraints**:

1. Identify all vowel/diphthong nuclei in the phoneme sequence
2. For consonant clusters between nuclei, assign as many consonants as possible to the onset of the following syllable
3. Onset must have rising sonority (plosives < fricatives < nasals < laterals < approximants < vowels)
4. Exception: /s/ + plosive clusters (sp, st, sk) are legal onsets despite sonority plateau

## Consequences

- **Accuracy**: Handles most English words correctly (he.llo, ba.na.na, strength)
- **Simplicity**: Operates on the phoneme sequence, not graphemes
- **Limitation**: Does not handle morphological boundaries (un.happy vs u.nhappy) — but the morphological decomposition in rules.rs handles prefixes before syllabification sees the phonemes
