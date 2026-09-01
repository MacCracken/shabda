# shabda — Current State

> Refreshed every release. CLAUDE.md is preferences/process/procedures
> (durable); this file is **state** (volatile).

## Version

**3.0.2** (2026-09-01) — dependency + toolchain pin refresh over 3.0.1 (svara 3.5.4,
shabdakosh 3.0.5, varna 2.4.1, cyrius 6.5.36), plus [ADR-0001](../adr/0001-shabdakosh-declared-optional.md):
shabdakosh is declared `optional` to work around the dep-sidecar resolver.
**3.0.1** (2026-07-06) was a dependency + toolchain pin refresh over the 3.0.0 port
(svara 3.1.0, shabdakosh 3.0.2, cyrius 6.4.12). **3.0.0** (RELEASED 2026-07-06) was
the Rust→Cyrius port: full behavioral parity with the Rust 2.0.0 surface, ~5,000
lines of Rust preserved at `rust-old/` as the parity oracle.

## Toolchain

- **Cyrius pin**: `6.5.36` (in `cyrius.cyml [package].cyrius`).

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

Both columns measured on the same machine in one session — 3.0.1 with cycc 6.4.12
and its own pins, 3.0.2 with cycc 6.5.36 and the pins above — so each delta is
toolchain + dependencies together, not one of them. A repeat run puts run-to-run
variance at ±3-4% on the g2p/speak rows, so only the dictionary rows are clearly
outside the noise.

| Benchmark | 3.0.1 | 3.0.2 |
|-----------|-------|-------|
| g2p_hello_world | 15.788 µs | 15.373 µs |
| g2p_sentence | 61.471 µs | 60.031 µs |
| speak_hello | 7.291 ms | 7.281 ms |
| speak_sentence | 23.691 ms | 23.437 ms |
| dict_english_construction | 7.466 ms | 7.168 ms |
| dict_lookup_hit | 135 ns | 123 ns |
| dict_lookup_miss | 258 ns | 237 ns |

## Distlib bundle

`cyrius distlib` → `dist/shabda.cyr` (v3.0.2) + `dist/shabda.deps` sidecar. Module
order is the `[lib].modules` list in `cyrius.cyml`. Consumers pull the bundle rather
than rebuilding from `src/`. The bundle is byte-identical to 3.0.1 apart from its
version line — no shabda source changed this release.

The 6.5.36 generator writes the full stdlib leaf set into the sidecar (22 entries)
where 6.4.12 wrote only the three folds. hisab/goonj/naad still lead the list, so a
consumer on 6.5.36 hits the same resolver failure shabda hits on shabdakosh —
[ADR-0001](../adr/0001-shabdakosh-declared-optional.md) has the workaround.

## Dependencies

Direct (path for local dev + git+tag for CI, declared in `cyrius.cyml`):

- **shabdakosh** 3.0.5 (`dist/shabdakosh.cyr`) — pronunciation dictionary (`shbdk_*`). Folds hisab/goonj/naad (its `.deps` sidecar). Declared `optional` — see [ADR-0001](../adr/0001-shabdakosh-declared-optional.md); `lib/shabdakosh.cyr` is refreshed by hand.
- **svara** 3.5.4 (`dist/svara.cyr`) — `SVARA_PH_*` phonemes, `PhonemeEvent`, sequence/voice/render for `speak()`. Folds hashmap/bayan (its `.deps` sidecar). Its manifest carries the hisab 2.11.2 / goonj 2.0.4 / naad 2.2.1 git pins the whole chain folds.
- **varna** 2.4.1 (`dist/varna.cyr`) — phoneme inventories, phonotactics, script detection. Self-contained on the stdlib folds.
- **stdlib**: syscalls, string, alloc, str, fmt, vec, io, args, assert, fnptr, atomic, sakshi, math, ganita, tagged, hashmap, bayan, mmap, bench, slice, result, callback.

## Consumers

dhvani (audio engine), vansh (voice AI shell) — will pull `dist/shabda.cyr`.
