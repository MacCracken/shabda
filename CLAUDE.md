# shabda — Claude Code Instructions

> **Core rule**: this file is **preferences, process, and procedures** —
> durable rules that change rarely. Volatile state (current version,
> module line counts, port progress, test counts, consumers) lives in
> [`docs/development/state.md`](docs/development/state.md).
> Do not inline state here.

## Project Identity

**shabda** (Sanskrit: *word / sound*) — grapheme→phoneme (G2P) engine for AGNOS.
Cyrius port of a ~5,000-line Rust library (preserved at `rust-old/`).

- **Type**: Port (Rust → Cyrius), flat library
- **License**: GPL-3.0-only
- **Language**: Cyrius (toolchain pinned in `cyrius.cyml [package].cyrius`)
- **Version**: `VERSION` at the project root is the source of truth — do not inline the number here
- **Standards**: [First-Party Standards](https://github.com/MacCracken/agnosticos/blob/main/docs/development/applications/first-party-standards.md) · [First-Party Documentation](https://github.com/MacCracken/agnosticos/blob/main/docs/development/applications/first-party-documentation.md)

## Goal

shabda OWNS **grapheme→phoneme conversion** in the AGNOS text-to-speech stack: it
turns text into a sequence of **svara** `SVARA_PH_*` `PhonemeEvent`s, driving a
normalize → dictionary lookup (shabdakosh) / letter-to-sound rules fallback →
syllabify → prosody pipeline, with an SSML subset, heteronym disambiguation, and
varna inventory validation. Dictionary-first, rules as fallback; accuracy over
speed.

## Current State

> Volatile state lives in [`docs/development/state.md`](docs/development/state.md) —
> port progress, surface parity, in-flight work. Refreshed every release.

This file (`CLAUDE.md`) is durable rules.

## Consumers

dhvani (audio engine), vansh (voice AI shell), and any AGNOS component needing
text-to-speech. Consumers pull `dist/shabda.cyr` (the distlib bundle).

## Dependencies

- **shabdakosh** (`shbdk_*`) — pronunciation dictionary (dict lookup, ARPABET/IPA, heteronym pronunciations, `SHBDK_ERR_*` codes). Consumed by engine/heteronym/error. Folds hisab/goonj/naad.
- **svara** (`SVARA_PH_*`) — phoneme identities (the `PhonemeEvent`-compat contract), phoneme class/duration, stress, intonation, and the sequence/voice/render surface for `speak()`. Folds hashmap/bayan.
- **varna** — phoneme inventories, phonotactics, and script detection. Consumed by validate and `detect_language`.

## Scaffolding

Project was scaffolded with `cyrius port`. Original Rust at `rust-old/` is the reference oracle — do not modify it; cross-check the port against it.

## Quick Start

```sh
cyrius deps                              # resolve dependencies
cyrius build src/main.cyr build/shabda   # compile the smoke binary
cyrius test tests/<mod>.tcyr             # run one suite
cyrius tests tests                       # run all .tcyr
cyrius bench tests/shabda.bcyr           # run benchmarks
cyrius distlib                           # regenerate dist/shabda.cyr
```

## Key Principles (durable)

- **Cross-check against `rust-old/`** — the port's correctness bar is "matches what Rust did". Diverge only with an ADR.
- **Correctness over cleverness** — if the Cyrius behavior diverges silently from Rust, the bugs win.
- **Dictionary-first, rules as fallback** — accuracy over speed.
- Test after every change, not after the feature is "done"; ONE change at a time — never bundle unrelated changes.
- Build with `cyrius build`, not raw `cat file | cc5` — the manifest auto-resolves deps.
- Source files only need project includes — stdlib auto-resolves from `cyrius.cyml`.
- `var buf[N]` = N **bytes**, not N entries.
- **Prefix everything** `shabda_`/`SHABDA_`/`Shabda` — the distlib links flat (coexists with `shbdk_` from shabdakosh, `SVARA_PH_*` from svara, and varna's bare symbols). shabda OWNS the `shabda_` namespace.

## Port Invariants (carried from the Rust crate)

- `#[non_exhaustive]` on public enums → keep additive; give every dispatch a catch-all / default arm.
- `#[must_use]` on pure functions → `#must_use`.
- **Zero unwrap/panic** in library code → errors build on **sakshi** (packed i64, `0 == ok`). A fallible fn returns a packed shabda error (test with `shabda_is_err`) and writes its payload to an out-param pointer; `convert_streaming` has no result vec, so it returns the packed error directly. `shabda_err_name()` gives diagnostic text. The Rust `From<ShabdakoshError>` map is a `SHBDK_ERR_*` → `SHABDA_ERR_*` code translation.
- Phoneme output compatible with svara's `PhonemeEvent` — a conversion result is a vec of svara `PhonemeEvent` handles built from `SVARA_PH_*` ordinals.
- **No feature flags** — CYRIUS has none, so varna validation / detection and every other capability are always compiled into the one bundle (the Rust `std`/`varna`/`json`/`logging`/`full` gates collapse).
- **Never skip benchmarks** before claiming a performance change.

## Module Structure

Nine modules under `src/` (leaf-first include order in `src/main.cyr` / `[lib].modules`):

- `error.cyr` — sakshi-backed error surface (`shabda_err_*`, `shabda_is_err`); `From<ShabdakoshError>` code map
- `normalize.cyr` — text normalization, abbreviation/acronym/number expansion, foreign-word detection, emphasis markers
- `syllable.cyr` — syllabify via Maximal Onset Principle with sonority constraints
- `heteronym.cyr` — heteronym disambiguation with context triggers
- `ssml.cyr` — SSML subset parser (break / emphasis / prosody)
- `rules.cyr` — letter-to-sound rules (English, Spanish, German, Hindi, Arabic, Sanskrit)
- `prosody.cyr` — stress, emphasis, speaking rate, timing profiles, intonation mapping
- `validate.cyr` — phoneme→IPA mapping (per-language), varna inventory + phonotactic validation
- `engine.cyr` — G2PEngine, Language, ConvertOptions, TimingProfile; convert*/speak*; detect_language, phoneme_inventory

## Rules (Hard Constraints)

- **Do not commit or push** — the user handles all git operations
- **Never use `gh` CLI** — use `curl` to the GitHub API if needed
- Do not modify `rust-old/` — it's the parity oracle
- Do not skip tests before claiming changes work
- Do not modify `lib/` files (vendored stdlib / dep symlinks)
- Do not hardcode toolchain versions in CI YAML — `cyrius = "X.Y.Z"` in `cyrius.cyml` is the source of truth

## CHANGELOG Format (Keep a Changelog + SemVer)

```
## [version] — YYYY-MM-DD

Description.

- **Feature**: description
- **Breaking**: description
```

## Documentation

- [`docs/adr/`](docs/adr/) — Architecture Decision Records (*why X over Y?*)
- [`docs/architecture/`](docs/architecture/) — Non-obvious constraints
- [`docs/guides/`](docs/guides/) — Task-oriented how-tos
- [`docs/examples/`](docs/examples/) — Runnable examples
- [`docs/development/state.md`](docs/development/state.md) — Live state
- [`docs/development/roadmap.md`](docs/development/roadmap.md) — Milestones through v1.0

