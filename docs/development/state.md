# shabda — Current State

> Refreshed every release. CLAUDE.md is preferences/process/procedures
> (durable); this file is **state** (volatile).

## Version

**3.1.3** (2026-11-17): the accuracy sweep, over every module, with accuracy as the oracle
([ADR-0002](../adr/0002-accuracy-over-rust-parity.md)). No dependency change. A new English
letter-to-sound engine (NRL rules, tuned on CMUdict: 13.7% → 46.4% of held-out words exact), English
stress rules (70.9% → 87.9% of polysyllables), accurate Spanish, German, Hindi, Sanskrit and Arabic rules
(Swadesh exact words: es 16 → 72%, de 28 → 80%, hi 40 → 88%, ar 0 → 28%), per-language onsets, English
number and abbreviation reading, seven heteronym rules fixed, SSML 1.1 / XML handling. Four functions
added, none removed. Output changes in every language; conversions 8–14% slower.

**3.1.2** (2026-11-17): dependency release plus the allocator-taking conversion. shabdakosh 3.1.4,
svara 3.14.0, varna 2.6.0, with hisab 3.5.0 / naad 2.3.5 / goonj 2.3.2 coming in transitively and
sakshi 2.5.10 as the stdlib leaf, each its dep's newest tag; cyrius stays 6.7.18. Adds
`shabda_convert_a` / `shabda_convert_with_a` (hoosh's issue, now archived): every allocation from the
caller's Allocator, 0 bytes on the default heap. No function removed, no signature changed. Output
changes for ARPABET `IY`, now /iː/ (shabdakosh 3.1.3), and the rules and IPA tables drop the Rust's
`VOWEL_E` stand-ins and give Spanish its trill, under the new bar: accuracy is the oracle and
`rust-old/` the capability list ([ADR-0002](../adr/0002-accuracy-over-rust-parity.md)). Fixes a
builder overflow on a refused grow.

**3.1.1** (2026-11-07): toolchain and dependency release plus an audit cleanup. cyrius 6.7.18,
shabdakosh 3.1.1, svara 3.6.3, varna 2.5.2, with hisab 3.4.1 / naad 2.3.3 / goonj 2.3.1 /
sakshi 2.5.10 coming in transitively, each its dep's newest tag. No behavior, API or output
change. `cyrius audit` exits 0 again: fmt, lint (14 warnings in `tests/`) and docs (40 public fns
given doc comments) are clean. `dist/shabda.deps` is gone, because no stdlib leaf is left to name.

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
lines of Rust preserved at `rust-old/`, then the parity oracle. Since 3.1.2 it is the capability
reference and accuracy is the oracle ([ADR-0002](../adr/0002-accuracy-over-rust-parity.md)).

## Toolchain

- **Cyrius pin**: `6.7.18` (in `cyrius.cyml [package].cyrius`), still cyrius's newest tag at
  3.1.2. At 3.1.0, 3.1.1 and 3.1.2, `lib/` and `cyrius.lock` were regenerated from scratch, with
  `cyrius deps` run into an empty `lib/` and no lock. Re-resolving into an existing lock appends new
  commit rows beside the old ones instead of replacing them.
- **6.7.18 pin set** (3.1.2): svara 3.14.0, varna 2.6.0, hisab 3.5.0, naad 2.3.5 and goonj 2.3.2
  are each dep's newest tag, and all pin exactly 6.7.18. svara's manifest supplies the hisab, naad and
  goonj versions. shabdakosh 3.1.4 is built on this same set, and its lock has the same five commit
  pins. sakshi has no commit pin since svara 3.6.4 dropped its package: it is the stdlib leaf.
- **`cyrius tests` is gone** (removed in 6.7.12). Run every suite with `cyrius test tests`, or
  with the bare `cyrius test`, which CI runs.
- **6.7.x meaning changes**: `src/`, `tests/` and `fuzz/` hold none of the constructs whose meaning
  6.7.0–6.7.18 changed: lone `!`, unary `-` / `~`, `for` steps, `*T` derefs, slices, the newly
  reserved words, `struct` / `Str` / multi-value returns, `**T`, closures, `impl`, narrow integer
  annotations, duplicate globals or fns. See the 3.1.0 and 3.1.1 CHANGELOG entries. The one
  natively typed spot is the `f32` multiply in `src/ssml.cyr` (`<break time="Xs">`), checked
  bit-exact across 6.7.9 and 6.7.18 over 404,075 inputs. New code should avoid these constructs
  or be checked against the rules.
- **Never put `path =` beside `git`/`tag` in `[deps.*]`.** An existing path makes the tag inert
  and skips the commit pin. Every `cyrius build` / `test` resolves deps again, so just running
  the suites would vendor whatever the sibling has checked out.
- **`cyrius audit` exits 0** since 3.1.1, with fmt, lint, docs, tests and bench all clean. It had
  failed from 3.0.5 through 3.1.0. CI does not run audit, so check it before a release.

## Allocators (3.1.2)

- **`shabda_convert_a` / `shabda_convert_with_a`** take an Allocator first and draw every allocation
  from it, the events included. `shabda_convert` / `shabda_convert_with` are wrappers over
  `default_alloc()` and allocate exactly what 3.1.1 did. CLAUDE.md states the rule new code on the
  convert path must keep.
- **No `_a` form yet** for `shabda_convert_ssml`, `shabda_convert_streaming`, `shabda_speak` or
  `shabda_speak_with`. They allocate on the default heap.
- **Stand-ins for dependencies without an `_a` form**: `_shabda_dict_entry_a` (shabdakosh's lookup
  allocates its `Str` key), `_shabda_event_new_a` (svara's event constructor), `_shabda_str_cstr_a`
  and `_shabda_ascii_lower_a` (stdlib copies). An `_a` form upstream in shabdakosh or svara would let
  the first two go.
- **Shared tables are built in `shabda_g2p_new`**: the heteronym rules, varna's inventory and
  phonotactics, and (3.1.3) the English letter-to-sound rule table and its stress lists. A conversion reads the engine and writes only its Allocator and the event vec, so
  threads can share one engine, each with its own arena (`tests/allocator.tcyr`, four threads).
- **A refused allocation ends the process** on a vec or builder grow, as `vec_push` does. Callers
  should pass a growable arena.

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
- **Where the port is accurate and `rust-old/` is not** ([ADR-0002](../adr/0002-accuracy-over-rust-parity.md)
  is the standing reason; each release's CHANGELOG lists the output it changes):
  - 3.0.3, toward the zero-panic contract, where the Rust crashes: the SSML parser caps
    element nesting at `SHABDA_SSML_MAX_DEPTH = 256` instead of recursing until the stack
    is gone, and `shabda_select_phonemes` returns an empty vec on an empty pronunciation vec
    instead of reaching `_vec_die()`, which kills the host process rather than panicking.
  - 3.1.2, the rules and IPA tables: /iː/ is `SVARA_PH_VOWEL_LONG_I` (the Rust used
    `VOWEL_E`, svara's /e/), German ü / y are `SVARA_PH_VOWEL_Y`, and Spanish trills rr, a
    word-initial r and an r after n / l / s with `SVARA_PH_TRILL_ALVEOLAR` (the Rust gave rr
    the English /ɹ/ and every r the tap). `VOWEL_E` is /e/ in every IPA table, and the
    Spanish tap and trill are spelled `ɾ` and `r`.
  - 3.1.3, the accuracy sweep over every module: the English rules are NRL's, tuned on CMUdict;
    English stress follows English's rules; every language's rules use the phonemes its phonology has;
    the text, heteronym and SSML fixes. The 3.1.3 CHANGELOG entry lists every output change and how it
    was measured.
- **Still inaccurate** (the roadmap's *Accuracy sweep* item, which 3.1.3 left open): dictionary stress
  (shabdakosh keeps none), varna's inventory gaps, Spanish and German stress, non-English number
  reading, unvocalized Arabic, German lexical exceptions, and the rest listed there.
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

**1,342 assertions** across 11 `.tcyr` suites + the fuzz harness — all green:

| Suite | Assertions |
|-------|-----------|
| error | 25 |
| normalize | 120 |
| syllable | 96 |
| heteronym | 48 |
| ssml | 97 |
| rules | 294 |
| prosody | 72 |
| validate | 235 |
| engine | 309 |
| shabda (crate-level) | 37 |
| allocator | 9 |
| fuzz (`shabda.fcyr`) | corpus + 800 generated cases, through all six engines |

`allocator.tcyr` is its own process because its first group must run before any
conversion. It converts from four threads at once, and `thread` is declared in
`cyrius.cyml` for it alone. Since 3.1.2 every fuzz case also converts through
`shabda_convert_a` on a dirtied arena and must match `shabda_convert` and keep nothing
on the default heap; since 3.1.3 it does so on all six engines and feeds the raw bytes to all six rule
sets. `cyrius fuzz`, `--poison` and `--bounds` all pass.

**Accuracy measurement** (3.1.3) is not in the suites: the gold sets are the 10,617 CMUdict words of
shabdakosh's `data/cmudict-5k.txt` (English rules and stress, tuned on the even half and measured on the
odd one) and varna's Swadesh lists (25 words per language, the independent check elsewhere). The 3.1.3
CHANGELOG entry gives the method and the figures; rerun them before claiming an accuracy change.

Run one suite with `cyrius test tests/<mod>.tcyr`, all with `cyrius test tests`, the
fuzz corpus with `cyrius fuzz`. Every entry file exits through the target-resolved
`SYS_EXIT`; before 3.0.3 they used a raw `syscall(60, rc)`, which is `exit` only on
x86-64 Linux/macOS — on AGNOS the pass/fail result was discarded and a failing suite
read as green.

## Benchmarks

`tests/shabda.bcyr` (x86_64), run with `cyrius bench tests/shabda.bcyr`.

**3.1.3**, medians of 8 alternated runs of 3.1.2 and 3.1.3 in one session: g2p_hello_world 15.75 →
17.05 µs (+8.3%), g2p_sentence 62.88 → 71.50 µs (+13.7%), g2p_sentence_arena 59.56 → 67.90 µs (+14.0%),
speak_hello 3.56 → 3.74 ms (+4.9%); speak_sentence, construction and lookups within noise. The cost is
the new stress, syllable and normalization passes (not profiled further). A word through the English
rules costs 17–22 µs, from 3.6–4.9 µs; no bench row covers the rules.

**3.1.2**, medians of **8** runs of three builds alternated in one session: 3.1.1, the 3.1.2 deps with
3.1.1's source, and 3.1.2.
That session ran slow: 3.1.1's own rows read about 19% above the figures recorded at 3.1.1, so compare
within a table, never across sessions. The lookup rows are sub-µs, and the stdlib hashmap seeds itself
per process, so they move from run to run: use at least 5 alternated rounds before calling a lookup
row changed.

| Benchmark | 3.1.1 | 3.1.2 deps, 3.1.1 source | 3.1.2 |
|-----------|-------|--------------------------|-------|
| g2p_hello_world | 18.22 µs | 18.45 µs | 18.95 µs |
| g2p_sentence | 72.77 µs | 73.18 µs | 75.27 µs |
| g2p_sentence_arena | — | — | 69.69 µs |
| speak_hello | 6.32 ms | 4.21 ms | 4.18 ms |
| speak_sentence | 20.71 ms | 12.89 ms | 12.94 ms |
| dict_english_construction | 9.40 ms | 9.34 ms | 9.56 ms |
| dict_lookup_hit | 164 ns | 172 ns | 167 ns |
| dict_lookup_miss | 193 ns | 192 ns | 190 ns |

svara 3.8.x makes both speak rows 33–38% faster. Twelve more alternated runs of the last two builds
put the `_a` refactor at +0.5% (`g2p_hello_world`) and +2.9% (`g2p_sentence`), with overlapping
interquartile ranges, while `dict_lookup_hit`, the same code in both, moved +2.2%. `g2p_sentence_arena`
(`shabda_convert_a` into an arena reset per conversion) read 64.47 µs there, 3–6% under either
default-heap form.

## Distlib bundle

`cyrius distlib` → `dist/shabda.cyr` (v3.1.3). There is no `dist/shabda.deps` sidecar since
3.1.1. Module order is the `[lib].modules` list in `cyrius.cyml`. Consumers pull the bundle
rather than rebuilding from `src/`.

Since cyrius 6.6.18 the sidecar lists only what the compile-verify proves the bundle needs
beyond its named deps. That was 22 leaves on 6.5.36–6.6.3 and one, `unicode`, on 6.6.18–6.7.9.
On 6.7.18 it is none, so distlib removes the file. The `unicode` tables now arrive with `bayan`,
which svara's sidecar requires and which includes `lib/unicode/` at 6.7.18. hisab, goonj and naad
are not listed, so ADR-0001's resolver failure does not hit consumers of shabda.

`cyrius distlib` lists 10 `shbdk_*` names for the consumer to supply: `shbdk_dict_new`,
`shbdk_dict_english`, `shbdk_dict_lookup`, `shbdk_pronunciation_phonemes`, and, since 3.1.2's `_a`
lookup, `shbdk_is_lower_cstr`, `shbdk_dict_lookup_entry`, `shbdk_dict_entries`,
`shbdk_dict_user_entries`, `shbdk_dict_entry_primary_phonemes` and `shbdk_dict_entry_all`. They are
shabdakosh's dictionary surface, called from `engine.cyr` and `heteronym.cyr`, and the consumer's
chain provides them (`… → varna → shabdakosh → shabda`, as dhvani declares it). This is expected.

## Dependencies

Direct, declared git+tag only in `cyrius.cyml` (no `path`; see Toolchain):

- **shabdakosh** 3.1.4 (`dist/shabdakosh.cyr`) — pronunciation dictionary (`shbdk_*`). Its bundle needs svara and varna in scope; its sidecar lists only `mmap`. Declared `optional` (see [ADR-0001](../adr/0001-shabdakosh-declared-optional.md)), so `lib/shabdakosh.cyr` is refreshed by hand **from the tag**: `git -C ../shabdakosh show <tag>:dist/shabdakosh.cyr > lib/shabdakosh.cyr`, then `cyrius deps && cyrius deps --verify`. Check the tag against GitHub first (`git ls-remote`). Copy from the tag, not with ADR-0001's `cp ../shabdakosh/dist/...`, because the sibling checkout tracks `main`. On every resolve, 6.7.18 notes that the file is "outside the declared footprint". It keeps the file and its lock line, so the note is expected.
- **svara** 3.14.0 (`dist/svara.cyr`) — `SVARA_PH_*` phonemes, `PhonemeEvent`, sequence/voice/render for `speak()`. Its sidecar lists `bayan`. Its manifest carries the hisab 3.5.0 / naad 2.3.5 / goonj 2.3.2 git pins the whole chain folds. It sets the consumer floor at cyrius 6.7.2. shabda builds `SvPhonemeEvent` records itself on the `_a` path, through svara's public struct and setters (`_shabda_event_new_a`, compared byte for byte in `tests/prosody.tcyr`).
- **varna** 2.6.0 (`dist/varna.cyr`) — phoneme inventories, phonotactics, script detection. Self-contained on the stdlib folds. shabda calls 12 of its functions, all with their 2.5.2 signatures. 2.6.0 spells the Spanish tap `ɾ` and trill `r`, where shabda's Spanish IPA table keeps the oracle's `r` / `rr`, so `shabda_validate_phonemes_for` flags every Spanish trill (see the 3.1.2 CHANGELOG entry).
- **Lock**: 5 commit pins (svara, varna, hisab, naad, goonj) plus a `cyrius 6.7.18` toolchain row; 61 hashed files. shabdakosh has no commit pin because it is `optional`; the lock hashes `lib/shabdakosh.cyr` instead.
- **stdlib**: syscalls, string, alloc, str, fmt, vec, io, args, assert, fnptr, atomic, sakshi, math, ganita, tagged, hashmap, bayan, mmap, bench, slice, result, callback, **unicode**, **thread**.
  `thread` is new at 3.1.2, for `tests/allocator.tcyr` alone; nothing in `src/` creates a thread.
  `unicode` is new at 3.0.3 — `unicode_category` (GeneralCategory) and `unicode_to_lower`
  (casefold) back `_shabda_is_alpha` / `_shabda_is_upper` / `_shabda_to_lower_cp`.
  On 6.7.18 a consumer gets the leaf with `bayan`, through svara's sidecar, so
  `dist/shabda.cyr` no longer `include`s it itself.
- `lib/sakshi.cyr` is **2.5.10**, the 6.7.18 snapshot's copy. Since 3.1.2 no dep in the chain
  commit-pins sakshi, so the stdlib leaf is the only copy, and a resolve no longer prints the
  `refusing to overwrite stdlib leaf 'sakshi'` note it printed through 3.1.1.

## Consumers

dhvani (audio engine), vansh (voice AI shell) — will pull `dist/shabda.cyr`.
