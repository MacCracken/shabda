# 0002 — Accuracy over Rust parity: `rust-old/` is the capability list, not the oracle

**Status**: Accepted
**Date**: 2026-11-17

## Context

Through 3.1.1, CLAUDE.md made `rust-old/` the port's correctness bar: a result was right if the
Rust produced it, and the port could differ only with an ADR. That bar got the port to full
parity, but it also meant copying the Rust's mistakes, including ones made because a dependency
could not express the right answer at the time.

3.1.2's dependency bump showed what that costs. The Rust spelled every /iː/ `VowelE`, svara's /e/,
in its rules, its IPA tables and shabdakosh, although svara 1.0 already had `VowelLongI`. Its German
ü and its Spanish trill were stand-ins too, `VowelE` and the English /ɹ/, because svara 1.0 had no
/y/ and no trill. svara has both now. shabdakosh 3.1.3
fixed its side (ARPABET `IY` is `VOWEL_LONG_I`, because svara said "see" as "say"). A dictionary word
and an unknown word with the same spelling then got different vowels through shabda. varna 2.6.0
spelled the Spanish tap and trill as IPA does, `ɾ` and `r`, where the Rust had the spelling `r` /
`rr`. So shabda's Rust-faithful Spanish table flagged every trill. In both cases the Rust was the
inaccurate side, and parity meant keeping a defect that the rest of the stack had already removed.

shabdakosh made the same call in its ADR 005. The maintainer set the rule for shabda too:
*rust-old is functionality, not the oracle; accuracy is the oracle.*

## Decision

**`rust-old/` defines what the port must be able to do. Accuracy decides whether a result is
right.** Accuracy means:

- the linguistics: IPA as defined, each language's phonology, ARPABET;
- the relevant specification: SSML, Unicode, IEEE 754;
- the documented contract;
- what a consumer would reasonably expect, such as a phoneme that the synthesizer renders as the
  sound the text spells.

Each difference from the Rust falls in one of three classes:

1. **A capability the port lacks** → port it.
2. **The port is inaccurate** → fix it.
3. **The Rust is inaccurate** → the port is accurate, and the CHANGELOG records the output change.

A consumer-visible output change is never silent. The release's CHANGELOG lists it, and this ADR is
the standing reason the port's output differs from the Rust's. A divergence does not need an ADR of
its own.

### The output changes this decision made (3.1.2)

| Area | Rust / 3.1.1 | Now | Why |
|---|---|---|---|
| English /iː/ (ee, ea, magic-e e, word-final y, re- / pre-) | `VOWEL_E` | `VOWEL_LONG_I` | `VOWEL_E` is /e/; the dictionary gives `IY` as `VOWEL_LONG_I` |
| German ie | `VOWEL_E` | `VOWEL_LONG_I` | /iː/ |
| German ü, y | `VOWEL_E` | `VOWEL_Y` | /yː/; svara 1.0 had no /y/, svara now does |
| Hindi / Sanskrit ई ी, Sanskrit ॠ ॄ | `VOWEL_E` | `VOWEL_LONG_I` | /iː/ |
| Spanish rr | `APPROX_R` (/ɹ/) | `TRILL_ALVEOLAR` | the alveolar trill; /ɹ/ is English |
| Spanish r, word-initially and after n / l / s | `TAP_FLAP` | `TRILL_ALVEOLAR` | Spanish trills it there: rojo, honra, alrededor, Israel |
| IPA of `VOWEL_E` (English, German, Hindi) | `iː` | `e` | its sound; `/iː/` is `VOWEL_LONG_I` |
| IPA of the Spanish tap / trill | `r` / `rr` | `ɾ` / `r` | IPA, and varna 2.6.0's inventory |

## Consequences

- **Positive**:
  - shabda agrees with shabdakosh, svara and varna on what each phoneme is.
  - Validation passes accurate phonemes: varna's Spanish inventory finds the trill and the tap,
    German's finds /iː/ and /yː/.
  - A consumer can read `VOWEL_E` as /e/ everywhere.
- **Negative**:
  - Output changes for consumers. Put through each language's rules, 2,094 of the 10,617 words of
    shabdakosh's `data/cmudict-5k.txt` change in English, 1,855 in German and 428 in Spanish. An
    English conversion changes only where a word reaches the rules, because the dictionary already
    gives `VOWEL_LONG_I`.
  - Tests that pinned the Rust's strings were rewritten to pin the accurate ones.
- **Neutral**:
  - 3.1.3 applied the decision to every module: the English rules, stress, every language's rules
    and IPA tables, syllabification, normalization, heteronyms and SSML. Its CHANGELOG entry lists
    each output change and how it was measured against gold data. What it left inaccurate is the
    roadmap's *Accuracy sweep* item.

## Alternatives considered

- **Keep Rust parity and record each defect as a divergence** (the rule through 3.1.1). It was
  rejected because it ships the Rust's mistakes to every consumer, and now disagrees with shabda's
  own dependencies, which have dropped the same mistakes.
- **Change only the IPA spellings to match varna and leave the rules' phonemes alone.** It was
  rejected because validation would pass while svara still synthesized /e/ for /iː/ and an English
  /ɹ/ for a Spanish trill. The phoneme is what a listener hears.
