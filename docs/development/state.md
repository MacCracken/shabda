# shabda — Current State

> Refreshed every release. CLAUDE.md is preferences/process/procedures
> (durable); this file is **state** (volatile).

## Version

**3.0.0** (RELEASED 2026-07-06) — Rust→Cyrius port. Full behavioral parity with
the Rust 2.0.0 surface. ~5,000 lines of Rust preserved at `rust-old/` as the
parity oracle.

## Toolchain

- **Cyrius pin**: `6.4.12` (in `cyrius.cyml [package].cyrius`).

## Port decisions (Rust → Cyrius)

- **Errors**: **sakshi** packed-i64 codes (`[ctx][category][code]`, `0 == ok`), not
  `thiserror`. The Rust `ShabdaError` variants → `SHABDA_ERR_*` codes + `shabda_err_name()`;
  the `From<ShabdakoshError>` impl → a `SHBDK_ERR_*` → `SHABDA_ERR_*` code translation.
- **Returns**: `Option<T>` → sentinels; `Result<T>` → pointer-or-0 / out-param. Fallible
  `convert*` / `speak*` return a packed error (`0 == ok`, test with `shabda_is_err`) and write
  their result vec (events / samples) to an out-param slot; `convert_streaming` has no result
  vec, so it returns the packed error directly and drives its work through an `fnptr` callback.
- **Traits**: → function-pointer / enum-tag dispatch. No serde derive — serialization is
  hand-written where needed.
- **Numerics**: `f32` prosody/rate scalars widened to `f64`.
- **Feature flags COLLAPSE**: CYRIUS has no feature flags, so the Rust `std` / `varna` / `json` /
  `logging` / `full` gates are gone — varna phoneme-inventory validation, language detection, and
  every other capability are always compiled into the one bundle.
- **Naming**: every symbol prefixed `shabda_` / `SHABDA_` / `Shabda` (flat link namespace;
  coexists with shabdakosh's `shbdk_`, svara's `SVARA_PH_*`, and varna's bare symbols).

## Modules (9 of 9 ported + green)

Leaf-first order (`src/main.cyr` / `[lib].modules`):

| Rust module | Cyrius | Status | Tests |
|-------------|--------|--------|-------|
| error.rs | src/error.cyr | ✅ ported | 25 |
| normalize.rs | src/normalize.cyr | ✅ ported | 40 |
| syllable.rs | src/syllable.cyr | ✅ ported | 47 |
| heteronym.rs | src/heteronym.cyr | ✅ ported | 21 |
| ssml.rs | src/ssml.cyr | ✅ ported | 45 |
| rules.rs | src/rules.cyr | ✅ ported | 126 |
| prosody.rs | src/prosody.cyr | ✅ ported | 50 |
| validate.rs | src/validate.cyr | ✅ ported | 127 |
| engine.rs | src/engine.cyr | ✅ ported | 133 |

- **error.cyr** — `ShabdaError` → sakshi packed codes + `shabda_err_name`; `From<ShabdakoshError>` map.
- **normalize.cyr** — abbreviation/acronym/number expansion, foreign-word diacritic detection, emphasis markers.
- **syllable.cyr** — syllabify via Maximal Onset Principle with sonority constraints.
- **heteronym.cyr** — context-trigger heteronym disambiguation over shabdakosh `Pronunciation`s.
- **ssml.cyr** — SSML subset parser (`<break>` / `<emphasis>` / `<prosody>`).
- **rules.cyr** — letter-to-sound rules: English + Spanish / German / Hindi / Arabic / Sanskrit (native scripts + romanized fallback).
- **prosody.cyr** — stress, emphasis, speaking-rate clamp (50–300 WPM), timing profiles, intonation mapping.
- **validate.cyr** — phoneme→IPA (per-language), varna inventory + phonotactics validation (always compiled).
- **engine.cyr** — G2PEngine / Language / ConvertOptions / TimingProfile; `convert` / `convert_with` / `convert_ssml` / `convert_streaming` / `speak` / `speak_with`; `detect_language`, `phoneme_inventory`.

## Tests

**653 assertions** across 11 `.tcyr` suites — all green:

| Suite | Assertions |
|-------|-----------|
| error | 25 |
| normalize | 40 |
| syllable | 47 |
| heteronym | 21 |
| ssml | 45 |
| rules | 126 |
| prosody | 50 |
| validate | 127 |
| engine | 133 |
| shabda (crate-level) | 29 |
| fuzz (`shabda.fcyr`) | — |

Run one suite with `cyrius test tests/<mod>.tcyr`, all with `cyrius tests tests`.

## Benchmarks

`tests/shabda.bcyr` (x86_64), run with `cyrius bench tests/shabda.bcyr`:

| Benchmark | Result |
|-----------|--------|
| g2p_hello_world | 21.6 µs |
| g2p_sentence | 82 µs |
| speak_hello | 22.7 ms |
| speak_sentence | 25.5 ms |
| dict_english_construction | ~10 ms |
| dict_lookup_hit | 127 ns |
| dict_lookup_miss | 258 ns |

## Distlib bundle

`cyrius distlib` → `dist/shabda.cyr` (v3.0.0) + `dist/shabda.deps` sidecar (folds
hisab/goonj/naad). Module order is the `[lib].modules` list in `cyrius.cyml`.
Consumers pull the bundle rather than rebuilding from `src/`.

## Dependencies

Direct (path for local dev + git+tag for CI, declared in `cyrius.cyml`):

- **shabdakosh** 3.0.1 (`dist/shabdakosh.cyr`) — pronunciation dictionary (`shbdk_*`). Folds hisab/goonj/naad (its `.deps` sidecar).
- **svara** 3.0.1 (`dist/svara.cyr`) — `SVARA_PH_*` phonemes, `PhonemeEvent`, sequence/voice/render for `speak()`. Folds hashmap/bayan (its `.deps` sidecar).
- **varna** 2.0.0 (`dist/varna.cyr`) — phoneme inventories, phonotactics, script detection. Self-contained on the stdlib folds.
- **stdlib**: syscalls, string, alloc, str, fmt, vec, io, args, assert, fnptr, atomic, sakshi, math, ganita, tagged, hashmap, bayan, mmap, bench.

## Consumers

dhvani (audio engine), vansh (voice AI shell) — will pull `dist/shabda.cyr`.
