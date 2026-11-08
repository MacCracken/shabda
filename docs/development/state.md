# shabda — Current State

> Refreshed every release. CLAUDE.md is preferences/process/procedures
> (durable); this file is **state** (volatile).

## Version

**3.1.0** (2026-11-07): toolchain and dependency release onto the cyrius 6.7 language minor.
cyrius 6.7.9, shabdakosh 3.1.0, svara 3.6.0, varna 2.5.0, with hisab 3.4.0 / naad 2.3.0 /
goonj 2.1.0 / sakshi 2.5.8 coming in transitively. No source or API change. A minor because
consumers now need cyrius 6.7.2 or later (svara 3.6.0's floor). One output change, inherited from
shabdakosh 3.1.0: `portuguese` now comes from the dictionary instead of the rules, as in
`rust-old/`.

**3.0.6** (2026-11-07): toolchain and dependency refresh. cyrius 6.6.20, shabdakosh 3.0.8,
svara 3.5.6, varna 2.4.3, with hisab 3.3.7 / naad 2.2.5 / goonj 2.0.6 / sakshi 2.5.7 coming in
transitively. No source or API change. Dropped the `path` overrides that had made the svara and
varna tags inert. 3.0.4 and 3.0.5 were toolchain pins (6.5.37, 6.6.3).

**3.0.3** (2026-09-01) — priority-1 audit sweep. 13 confirmed defects fixed
(quadratic syllable allocation, the SSML nesting SIGSEGV, five ASCII-for-Unicode
predicates, the `<break>` u32 domain, a UTF-8 over-read, `ConvertOptions` aliasing,
five allocate-instead-of-mutate loops, 8 unguarded allocations, a host-process kill on
an empty pronunciation vec), the test entry files' discarded exit status, and a fuzz
driver that was two literals. 727 assertions (was 653); `cyrius audit` exits 0.

**3.0.2** (2026-09-01) — dependency + toolchain pin refresh over 3.0.1 (svara 3.5.4,
shabdakosh 3.0.5, varna 2.4.1, cyrius 6.5.36), plus [ADR-0001](../adr/0001-shabdakosh-declared-optional.md):
shabdakosh is declared `optional` to work around the dep-sidecar resolver.
**3.0.1** (2026-07-06) was a dependency + toolchain pin refresh over the 3.0.0 port
(svara 3.1.0, shabdakosh 3.0.2, cyrius 6.4.12). **3.0.0** (RELEASED 2026-07-06) was
the Rust→Cyrius port: full behavioral parity with the Rust 2.0.0 surface, ~5,000
lines of Rust preserved at `rust-old/` as the parity oracle.

## Toolchain

- **Cyrius pin**: `6.7.9` (in `cyrius.cyml [package].cyrius`). At 3.1.0, `lib/` and
  `cyrius.lock` were regenerated from scratch: `cyrius deps` into an empty `lib/`, then again
  with no lock. Re-resolving into an existing lock appends new commit rows beside the old ones
  instead of replacing them.
- **6.7.9 pin set**: svara 3.6.0, varna 2.5.0, hisab 3.4.0, naad 2.3.0 and goonj 2.1.0 are the
  tags pinned to exactly 6.7.9; the next tag of each needs 6.7.18. svara's manifest supplies the
  hisab, naad and goonj versions. shabdakosh 3.1.0 is built on this same set, and its lock has
  the same six commit pins.
- **6.7.x meaning changes**: `src/`, `tests/` and `fuzz/` hold none of the constructs whose meaning
  6.7.x changed (lone `!`, unary `-` / `~`, `for` steps, `*T` derefs, slices, the newly reserved
  words); see the 3.1.0 CHANGELOG entry. New code should keep it that way or be checked against
  those rules.
- **Never put `path =` beside `git`/`tag` in `[deps.*]`.** An existing path makes the tag inert
  and skips the commit pin. Every `cyrius build` / `test` / `tests` resolves deps again, so just
  running the suites would vendor whatever the sibling has checked out.
- **`cyrius audit` exits 1**: it failed by 3.0.5, and the 3.0.3 "exits 0" no longer holds. fmt
  (`tests/engine.tcyr`, `tests/syllable.tcyr`), 14 lint warnings in `tests/`, and 40
  undocumented public fns on 6.6.20 and 6.7.9 (3 on 6.6.3, because 6.6.20's cyrdoc no longer
  counts `#must_use` as documentation). `src/` is fmt- and lint-clean, and CI does not run audit.

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
- **Numerics**: `f32` prosody/rate scalars widened to `f64`. ⚠ **This was never a
  decision** — Cyrius had no scalar f32 when the port was written. It does now
  (`f32_from` / `f32_to` plus native `addss`/`mulss` on `f32`-typed params). 3.0.3
  restored native f32 where shabda owns the whole computation — the
  `<break time="Xs">` multiply. The rest is **svara's to undo, not shabda's**:
  `SvPhonemeEvent.duration` is `f64` and svara's base durations are f64 constants
  (0.08 / 0.12 / 0.18) where the Rust svara used f32 throughout, so shabda rounding its
  own prosody arithmetic to f32 would match neither the oracle nor the current stack.
  Restoring f32 end to end is a svara-first change, and breaking for
  `shabda_convert_options_with_speaking_rate` / `shabda_timing_new`, which take f64 bit
  patterns.
- **Deliberate divergences from `rust-old/`** (both from 3.0.3, both toward the
  zero-panic contract; the oracle crashes in each case): the SSML parser caps element
  nesting at `SHABDA_SSML_MAX_DEPTH = 256` instead of recursing until the stack is gone,
  and `shabda_select_phonemes` returns an empty vec on an empty pronunciation vec instead
  of reaching `_vec_die()`, which kills the host process rather than panicking.
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

**727 assertions** across 10 `.tcyr` suites + the fuzz harness — all green:

| Suite | Assertions |
|-------|-----------|
| error | 25 |
| normalize | 60 |
| syllable | 55 |
| heteronym | 23 |
| ssml | 66 |
| rules | 133 |
| prosody | 50 |
| validate | 127 |
| engine | 151 |
| shabda (crate-level) | 37 |
| fuzz (`shabda.fcyr`) | corpus + 800 generated cases |

Run one suite with `cyrius test tests/<mod>.tcyr`, all with `cyrius tests tests`, the
fuzz corpus with `cyrius fuzz`. Every entry file exits through the target-resolved
`SYS_EXIT`; before 3.0.3 they used a raw `syscall(60, rc)`, which is `exit` only on
x86-64 Linux/macOS — on AGNOS the pass/fail result was discarded and a failing suite
read as green.

## Benchmarks

`tests/shabda.bcyr` (x86_64), run with `cyrius bench tests/shabda.bcyr`. Medians of 3 runs,
with 3.0.6 (cyrius 6.6.20, its committed `lib/`) and 3.1.0 (cyrius 6.7.9, the pins below)
alternated in one session. Each delta is toolchain and dependencies together, not either one
alone. `g2p_hello_world` and `speak_hello` were slower in every 3.1.0 run than in every 3.0.6
run; the other rows overlap. The lookup rows are sub-µs and noisy: 3.0.6's `dict_lookup_miss`
alone ranged 141–219 ns across the three runs.

| Benchmark | 3.0.6 | 3.1.0 |
|-----------|-------|-------|
| g2p_hello_world | 15.05 µs | 15.18 µs |
| g2p_sentence | 58.84 µs | 59.72 µs |
| speak_hello | 5.17 ms | 5.32 ms |
| speak_sentence | 16.94 ms | 17.16 ms |
| dict_english_construction | 7.35 ms | 7.47 ms |
| dict_lookup_hit | 135 ns | 133 ns |
| dict_lookup_miss | 147 ns | 133 ns |

## Distlib bundle

`cyrius distlib` → `dist/shabda.cyr` (v3.1.0) + `dist/shabda.deps` sidecar. Module
order is the `[lib].modules` list in `cyrius.cyml`. Consumers pull the bundle rather
than rebuilding from `src/`.

Since cyrius 6.6.18 the sidecar lists only what the compile-verify proves the bundle
needs beyond its named deps, which here is one leaf, `unicode` (it was 22 on 6.5.36–6.6.3).
hisab, goonj and naad are no longer listed, so ADR-0001's resolver failure no longer hits
consumers of shabda.

`cyrius distlib` lists 5 `shbdk_*` names (`shbdk_dict_new`, `shbdk_dict_english`,
`shbdk_dict_lookup`, `shbdk_dict_lookup_all`, `shbdk_pronunciation_phonemes`) for the
consumer to supply. They are shabdakosh's dictionary surface, called from `engine.cyr` and
`heteronym.cyr`, and the consumer's chain provides them (`… → varna → shabdakosh → shabda`,
as dhvani declares it). This is expected.

## Dependencies

Direct, declared git+tag only in `cyrius.cyml` (no `path`; see Toolchain):

- **shabdakosh** 3.1.0 (`dist/shabdakosh.cyr`) — pronunciation dictionary (`shbdk_*`). Its bundle needs svara and varna in scope; its sidecar lists only `mmap`. Declared `optional` (see [ADR-0001](../adr/0001-shabdakosh-declared-optional.md)), so `lib/shabdakosh.cyr` is refreshed by hand **from the tag**: `git -C ../shabdakosh show <tag>:dist/shabdakosh.cyr > lib/shabdakosh.cyr`, then `cyrius deps && cyrius deps --verify`. Do not use ADR-0001's `cp ../shabdakosh/dist/...`: the sibling checkout tracks `main`, which is ahead of the pin (3.1.1 on cyrius 6.7.18 at 3.1.0).
- **svara** 3.6.0 (`dist/svara.cyr`) — `SVARA_PH_*` phonemes, `PhonemeEvent`, sequence/voice/render for `speak()`. Its sidecar lists `bayan`. Its manifest carries the hisab 3.4.0 / naad 2.3.0 / goonj 2.1.0 git pins the whole chain folds. It sets the consumer floor at cyrius 6.7.2.
- **varna** 2.5.0 (`dist/varna.cyr`) — phoneme inventories, phonotactics, script detection. Self-contained on the stdlib folds.
- **Lock**: 6 commit pins (svara, varna, hisab, naad, goonj, sakshi) plus a `cyrius 6.7.9` toolchain row; 52 hashed files. shabdakosh has no commit pin because it is `optional`; the lock hashes `lib/shabdakosh.cyr` instead.
- **stdlib**: syscalls, string, alloc, str, fmt, vec, io, args, assert, fnptr, atomic, sakshi, math, ganita, tagged, hashmap, bayan, mmap, bench, slice, result, callback, **unicode**.
  `unicode` is new at 3.0.3 — `unicode_category` (GeneralCategory) and `unicode_to_lower`
  (casefold) back `_shabda_is_alpha` / `_shabda_is_upper` / `_shabda_to_lower_cp`.
  `dist/shabda.cyr` `include`s the leaf rather than inlining it, so consumers must have
  it in scope.
- `lib/sakshi.cyr` is **2.5.8**, the 6.7.9 snapshot's copy, byte-identical to the 2.5.8 artifact
  the dep chain pins. On every resolve `cyrius` prints `refusing to overwrite stdlib leaf 'sakshi'`; the
  note is expected. The 3.0.4 split, with `cyrius deps` reverting the snapshot copy, is gone.

## Consumers

dhvani (audio engine), vansh (voice AI shell) — will pull `dist/shabda.cyr`.
