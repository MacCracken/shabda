# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [3.0.3] — 2026-09-01

Priority-1 audit sweep: correctness, hardening, security, parity and performance,
plus the two `cyrius audit` gaps 3.0.2 shipped with. **13 confirmed defects fixed**,
each found by reading against `rust-old/`, adversarially re-verified, and pinned by a
regression test. **737 assertions across 10 suites** (was 653), fuzz corpus rebuilt,
`cyrius audit` exits 0. No API removals; three public functions changed behaviour and
are called out below.

### Fixed — CRITICAL: a 6 KB word allocated 858 MB, permanently

`_shabda_max_legal_onset` (`src/syllable.cyr`) walked `start` from 0 to `len` and
materialised a **fresh vec on every iteration** where the oracle borrows a sub-slice:

```
rust-old/src/syllable.rs:165   for start in 0..cluster.len() {
rust-old/src/syllable.rs:166       let candidate = &cluster[start..];      // O(1), no alloc
src/syllable.cyr:171                var candidate = _shabda_slice(cluster, start, len);
```

The bump allocator never frees, so that is ~21.6·L² bytes retained for a consonant run
of length L. Measured end to end on `shabda_convert(eng, "a" + "b"×6000 + "a")` — one
6,002-byte argument, no length ceiling anywhere between the public entry and here:

| consonant run | 3.0.2 | 3.0.3 |
|---|---|---|
| 500 | 5,461,304 B | 150,600 B |
| 1,000 | 21,702,424 B | 291,944 B |
| 2,000 | 86,559,424 B | 574,608 B |
| 4,000 | 345,772,560 B | 1,139,936 B |
| **6,000** | **858,241,376 B** | **2,188,592 B** |

Quadratic (4× per doubling) → linear (2× per doubling); **392× less at 6,000**. Output
is unchanged — 6,002 events either way. The scan is now range-based
(`_shabda_max_legal_onset_range` / `_shabda_is_legal_onset_range` take bounds instead of
a copy), which also removes the two `_shabda_slice` copies in `shabda_syllabify` where
Rust borrows. Reachable from `convert` / `convert_with` / `convert_ssml` /
`convert_streaming` / `speak*` and from `shabda_syllabify` directly. Streaming does not
help: the blow-up is *within* one whitespace token.

### Fixed — HIGH: nested SSML took the process down with SIGSEGV

`_shabda_parse_until` ↔ `_shabda_parse_tag` (`src/ssml.cyr`) are mutually recursive once
per element, with no ceiling — depth is whatever the caller's string says. The
unknown-tag arm means any tag name recurses, not just the four known ones.

```
shabda_ssml_parse(<emphasis> ×19,437 + "x" + </emphasis> ×19,437)   # ~408 KB
  -> SIGSEGV (exit 139) on the default 8 MB stack, ~432 B of frame per level
```

Bisected to the byte: depth 19,375 survives, 19,437 does not. `shabda_convert_ssml`
reaches it too, and `shabda_ssml_node_clone` / `_shabda_clone_children` recurse over the
same tree. `rust-old/src/ssml.rs:239` has the identical shape and the identical crash,
so **capping is a deliberate divergence from the oracle** rather than a parity fix.
`SHABDA_SSML_MAX_DEPTH = 256` — what libxml2 defaults to, and ~50× deeper than any real
SSML document. Past it the parse fails as `SHABDA_SSML_PARSE_ERR`, which
`shabda_ssml_parse` reports as 0 and `shabda_convert_ssml` as `InvalidInput`. A document
nested exactly 256 deep parses; 257 does not. A 4.2 MB, 200,000-deep document now
returns an error instead of killing the host.

### Fixed — the port was ASCII-only everywhere Rust was Unicode

Five hand-rolled predicates approximated Unicode with ASCII (or ASCII + Latin-1). Each
is now the real property, and the three byte-identical whitespace copies collapsed into
one L0 definition (`shabda_is_ws_cp` / `shabda_ws_len_at` / `shabda_ws_len_before` in
`src/error.cyr`).

- **`char::is_whitespace`** — was `{space, tab, LF, CR, FF, VT}` in `normalize.cyr`,
  `ssml.cyr` and `engine.cyr`; now the full White_Space property (25 code points,
  written out — no table needed). Consequences, all measured:
  - `shabda_normalize("hello\u{00A0}world")` returned `hello\u{00A0}world` as **one
    token** and sent it to the letter-to-sound rules whole. Now `hello world`.
  - `shabda_convert(eng, "\u{3000}")` returned **Ok with 0 events** where Rust's
    `if text.trim().is_empty()` returns `Err(InvalidInput)`. Same for `convert_with`,
    `convert_ssml`, `convert_streaming` and the `render_ssml_nodes` text guard.
  - `shabda_detect_intonation("really?\u{00A0}")` read as a **Statement** — the
    ASCII-only `trim_end` left `0xA0` as the last byte and never saw the `?`.
  - An SSML tag whose name was separated from its attributes by U+00A0 was parsed as a
    tag named `break\u{00A0}time=...` and silently dropped.
- **`char::is_alphabetic`** — was five hard-coded ranges (ASCII, Latin-1 Supplement +
  Extended-A/B, Devanagari, two Arabic blocks). Every other script hit **no arm**, so
  `shabda_normalize` **deleted it**: Greek, Cyrillic, Hebrew, Thai, Han and Hangul input
  normalized to empty and produced zero phonemes. Now L\* + Nl from the stdlib's Unicode
  17 GeneralCategory table, plus the Devanagari and Arabic mark ranges for the
  Other_Alphabetic segments the table does not carry. The scripts that already worked are
  unchanged — that is asserted, because dropping them is the regression the module header
  warns about.
- **`char::to_lowercase().next()`** — was ASCII-only, so `"CAFÉ"` lowered to `"cafÉ"` and
  every accented capital reached the rules in the wrong case. Now the stdlib casefold
  table.
- **`char::is_uppercase`** — was ASCII + U+00C0..U+00DE, so ALL-CAPS emphasis was
  silently not applied to Greek, Cyrillic, or even Latin Extended-A. Now GeneralCategory
  `Lu`. (Residual: `Uppercase` is `Lu` + Other_Uppercase; the ~130 Other_Uppercase points
  are Roman-numeral forms and circled capitals, none of which shabda pronounces.)

`[deps].stdlib` gains **`unicode`** for the two tables. It does not grow
`dist/shabda.cyr` — stdlib leaves are named in the sidecar, not bundled — but consumers
must have the leaf in scope.

### Fixed — `<break time=...>` left Rust's `u32` domain

`_shabda_parse_break_duration` (`src/ssml.cyr`) returns a duration through the same
channel as its failure sentinel, and neither bound nor saturated it.

- **`Nms` had no `u32` ceiling.** Rust does `ms_str.parse::<u32>()`; the port accumulated
  into an i64 with no limit, so `<break time="5000000000ms">` returned 5,000,000,000
  where Rust returns `Err`, and a 20-digit run **wrapped the i64**. Now
  `_shabda_atoi_u32`, which rejects empty, non-digit and `> u32::MAX`.
- **`Xs` did not saturate.** Rust's `(secs * 1000.0) as u32` is a saturating cast:
  NaN → 0, negative → 0, above `u32::MAX` → `u32::MAX`. The port used a bare `f64_to`,
  so `time="-5s"` produced −5000 — and `time="-0.001s"` produced exactly **−1, which IS
  `SHABDA_SSML_PARSE_ERR`**, making a legal parse indistinguishable from a failure.
- **The multiply is now native single precision**, because Rust's is (`parse::<f32>()`
  then an f32 multiply). Typing the params `f32` puts the patterns in xmm lanes and emits
  `mulss`; the result returns as its bit pattern through an `i64`. Same shape
  [ganita 1.2.0](https://github.com/MacCracken/ganita) uses — see the note at the end.

### Fixed — the UTF-8 decoder in `rules.cyr` read past the terminator

`_shabda_decode_chars` loaded continuation bytes at `word+i+1..+3` with **no bound**, so
a lead byte at the end of a string ran up to 3 bytes past the NUL into whatever the bump
allocator held there — a heap over-read whose phoneme output depended on neighbouring
allocations. Its sibling `_shabda_utf8_next` (`normalize.cyr`) always bailed at the NUL;
the crate's two decoders disagreed. Both now degrade a truncated sequence to its lead
byte and resume at the next, and `tests/shabda.tcyr` asserts they agree byte for byte
across ASCII, Devanagari, Arabic and all three truncation shapes.

### Fixed — `ConvertOptions` builders aliased the caller's block

Rust's `with_emphasis` / `with_speaking_rate` / `with_timing` take `mut self` and return
`Self` **by value**, so the receiver is moved and cannot be reused. The port mutated the
caller's block and returned the same pointer:

```
var base = shabda_convert_options_new();
var fast = shabda_convert_options_with_speaking_rate(base, wpm200);
var slow = shabda_convert_options_with_speaking_rate(base, wpm100);
# base, fast and slow were ONE block, all at 100 wpm
```

That program does not compile in Rust, so there was no oracle behaviour to preserve.
Each builder now clones first. All three are `#must_use`, so discarding the returned
handle was already an error. **Behaviour change for any consumer that relied on the
in-place mutation and ignored the return value.** The engine's internal
`_shabda_engine_opts_with_emphasis` / `_with_rate` already cloned, so nested SSML was
never affected.

### Fixed — five loops allocated where the oracle mutates in place

`shabda_apply_emphasis`, `shabda_apply_rate`, `shabda_apply_timing` (`prosody.cyr`) and
`_shabda_engine_apply_emphasis_range`, `_shabda_engine_destress_range` (`engine.cyr`)
minted a replacement `SvPhonemeEvent` per mutated element, out of a bump allocator that
never frees, where Rust does `event.stress = …; event.duration *= …` over a
`&mut [PhonemeEvent]`. A comment claimed this was forced ("immutable-by-handle"); svara
has had `SvPhonemeEvent_set_duration` / `_set_stress` all along.

It is worst under nested `<emphasis>`, where every level re-walks the whole event tail
its children produced — so the allocations are depth × events. Measured on a 26-event
body:

| nesting depth | 3.0.2 | 3.0.3 |
|---|---|---|
| 16 | 35,320 B | 31,224 B |
| 32 | 44,152 B | 35,960 B |
| 64 | 61,816 B | 45,432 B |
| 128 | 97,144 B | 64,376 B |

### Fixed — hardening

- **Every input-proportional `alloc()` is checked for the 0 sentinel** (8 sites across
  `engine.cyr`, `normalize.cyr`, `ssml.cyr`). `alloc` returns 0 for a request over
  `ALLOC_MAX` (2 GiB) or on exhaustion; each was written through immediately afterwards.
- **`shabda_select_phonemes` no longer kills the host process on an empty pronunciation
  vec.** Rust's fallback arm is `pronunciations[0]`, which panics; here `vec_get(v, 0)`
  reaches `lib/vec.cyr`'s `_vec_die()` → `syscall(SYS_EXIT, 1)`, which does not unwind and
  does not return a code — it terminates the **host**, so a consumer like dhvani would
  lose its audio daemon. Reachable: `shbdk_dict_lookup_all` returns
  `shbdk_dict_entry_all(e)` verbatim and a caller-built entry can hold zero
  pronunciations. It now returns an empty vec, which the caller's rules fallback already
  handles. A deliberate divergence, in the direction of the crate's zero-panic contract.

### Fixed — the test suites could not report failure on AGNOS

Every `.tcyr`, the `.bcyr` and the `.fcyr` exited with a raw `syscall(60, rc)`. #60 is
`exit` only on x86-64 Linux/macOS; on AGNOS it is a different call, so
`assert_summary()`'s pass/fail result was **discarded** and CI would read the suite green
while assertions failed. All twelve entry files now use the target-resolved `SYS_EXIT`,
matching `src/main.cyr`. Verified by breaking an assertion on purpose: the suite exits 1.

### Changed — the fuzz harness now fuzzes

`fuzz/shabda.fcyr`'s entry point was fine; its driver was **two string literals**, and
could not reach a single defect in this release. It now runs three things: a fixed
**corpus** of every shape that produced a defect here (kept as permanent regressions —
truncated UTF-8, Unicode whitespace, the `<break>` domain edges, malformed markup, deep
nesting on both sides of the cap, long single tokens, i64-boundary numbers), 400 rounds
of **random bytes** including invalid UTF-8, and 400 rounds of **random markup**
assembled from SSML fragments so the parser meets half-formed and mismatched structure
rather than only noise. The generator is a seeded xorshift, so a CI failure is
reproducible from its iteration number.

### Changed — tests

**737 assertions, 10 suites** (was 653): ssml 45 → 66, engine 133 → 151, normalize
40 → 60, rules 126 → 133, syllable 47 → 55, shabda 29 → 37, heteronym 21 → 23. Every fix
above has a regression that fails on 3.0.2, including the two that previously killed the
process — reaching those assertions at all is the test.

### Changed — `cyrius audit` exits 0

3.0.2 shipped with two gaps. `src/ssml.cyr` and `src/heteronym.cyr` needed
`cyrius fmt` (continuation indent), and `main` in `src/main.cyr` was undocumented.

### Performance

Flat. `cyrius bench tests/shabda.bcyr`: `g2p_hello_world` 15.75 µs, `g2p_sentence`
61.4 µs, `speak_hello` 7.40 ms, `speak_sentence` 23.7 ms, `dict_english_construction`
7.62 ms, `dict_lookup_hit` 123 ns, `dict_lookup_miss` 258 ns — all within the ±3-4%
run-to-run band established in 3.0.2. The wins above are in *allocation*, which the
benchmark set does not probe; they were measured with `alloc_used()` deltas instead.

### Note — the f32→f64 widening is svara's to undo, not shabda's

`docs/development/state.md` has listed "f32 prosody/rate scalars widened to f64" as a
port decision since 3.0.0. It was not a decision — it was a toolchain limitation, and
that limitation is gone. This release restores native f32 where shabda **owns** the whole
computation (the `<break time="Xs">` multiply above), and files the rest honestly:

`svara`'s `SvPhonemeEvent.duration` is `f64` and its base durations are f64 constants
(`0.08`, `0.12`, `0.18`), where the Rust svara used `f32` throughout. So the widening is
**chain-wide and owned by svara**. shabda rounding its own prosody arithmetic to f32
would produce a third set of numbers — matching neither the oracle nor the current stack
— so it is deliberately NOT done here. The public encoding (`with_speaking_rate` and
`shabda_timing_new` take f64 bit patterns) is unchanged; restoring f32 end to end is a
svara-first change and a breaking one for this surface.

The adjacent gap **was** fixed upstream: `ganita_f32_add` / `_sub` / `_mul` / `_div`
shipped in **ganita 1.2.0**, and the cyrius issue that recorded "no f32 tier can be
written in native single-precision arithmetic today" has been amended — a function
parameter can simply be typed `f32`, which is what the original filing missed.

## [3.0.2] — 2026-09-01

Dependency + toolchain pin refresh over 3.0.1. No shabda source change — the
distlib bundle is byte-identical apart from its version line, and the
653-assertion suite (10 `.tcyr` files) plus the fuzz harness pass unchanged.

- **Changed**: toolchain pin 6.4.12 → **6.5.36**, aligning shabda with the rest of
  the chain (shabdakosh and varna are already there; svara is on 6.5.35).
- **Changed**: shabdakosh 3.0.2 → **3.0.5** (a HIGH-severity corrupt-text-export
  fix, a `prefix_search` stack-overflow DoS, deep-cloning `merge`, O(n log n)
  dictionary sorts, and two `rust-old/` parity gaps), svara 3.1.0 → **3.5.4** (two
  memory-safety repairs that made every sample rate in (1000, 7500] abort), varna
  2.0.0 → **2.4.1** (tone/features/numerals modules upstream; four data-accuracy
  fixes). The transitive folds move with them: hisab 2.6.7 → **2.11.2**, naad
  2.1.1 → **2.2.1**, goonj 2.0.0 → **2.0.4**, sakshi 2.4.2 → **2.4.11**.
- **Changed**: `[deps].stdlib` gains `slice` + `result` (varna 2.4.1's sidecar) and
  `callback` (new at hisab 2.11.2 / naad 2.2.1). None is reached from `src/` — they
  are leaves the folds need in scope.
- **Changed**: `[deps.shabdakosh]` is declared `optional = true`. shabdakosh's
  `dist/shabdakosh.deps` names hisab / goonj / naad, which `cyrius deps` resolves
  only against the stdlib snapshot; at 6.4.12 the three errors were non-fatal, at
  6.5.36 they exit 3 and take down `deps` / `build` / `test` / `bench` / `distlib`.
  shabdakosh is still mandatory and `lib/shabdakosh.cyr` is still committed —
  refreshing it is now a manual copy. Rationale, the five workarounds that do *not*
  work, and the upstream fix that retires this:
  [ADR-0001](docs/adr/0001-shabdakosh-declared-optional.md).
- **Fixed**: `cyrius.lock` regained its commit pins. The resolver had been erroring
  out before writing them since at least 3.0.1, so the committed lock still named
  hisab 2.6.7 / naad 2.1.1 / goonj 2.0.0 / sakshi 2.4.2 — svara **3.1.0**'s pins,
  left behind by the 3.0.0-era resolve. It now pins the four commits actually in
  `lib/`, and `cyrius deps --verify` passes 41/41.
- **Changed**: `dist/shabda.deps` carries the full 22-leaf stdlib set instead of
  just the three folds — a 6.5.36 distlib-generator change, the same one shabdakosh
  saw at 3.0.3. hisab/goonj/naad still lead the list, so consumers on 6.5.36
  (dhvani, vansh) need the ADR-0001 workaround against shabda in turn.
- **Performance**: flat to slightly better, measured on one machine in one session
  (3.0.1 on cycc 6.4.12, 3.0.2 on 6.5.36 — each delta is toolchain and dependencies
  together). `dict_lookup_hit` 135 → **123 ns**, `dict_lookup_miss` 258 → **237 ns**,
  `dict_english_construction` 7.47 → **7.17 ms**; `g2p_hello_world` 15.79 → 15.37 µs,
  `g2p_sentence` 61.5 → 60.0 µs, `speak_hello` 7.29 → 7.28 ms, `speak_sentence`
  23.69 → 23.44 ms. Only the dictionary rows are clearly outside run-to-run variance,
  which a repeat run put at ±3-4% on the g2p/speak rows.
  `docs/benchmarks-rust-vs-cyrius.md` keeps its 2026-07-06 numbers
  and now says so — its ratios are a ceiling until the Rust half is re-run.

## [3.0.1] — 2026-07-06

Dependency + toolchain pin refresh over the 3.0.0 port. No shabda API or behavior
change; the 653-assertion suite passes unchanged.

- **Changed**: svara pin 3.0.1 → **3.1.0** (control-rate glide synthesis) and
  shabdakosh 3.0.1 → **3.0.2** (which re-pins svara 3.1.0). shabda picks up svara's
  faster diphthong synthesis end to end — **`speak` of a diphthong word (e.g.
  "hello") drops ~3× (22.4 ms → 7.65 ms)**; convert/g2p is unchanged (render-independent).
- **Changed**: toolchain pin 6.4.11 → **6.4.12** (current release; aligns the whole
  shabda/shabdakosh/svara chain on one toolchain, removes drift).

## [3.0.0] — 2026-07-06

Complete port of shabda from Rust to the **CYRIUS** language (AGNOS toolchain). A
full-parity port: every Rust module reproduced against the preserved `rust-old/`
oracle and cross-checked by a 653-assertion suite across 11 test suites, plus a
consumer-verified distlib bundle (`dist/shabda.cyr`).

- **Breaking**: Language change — shabda is now a CYRIUS (`.cyr`) library, not a Rust crate. The API is flat, `shabda_`-prefixed C-style functions (`shabda_g2p_new`, `shabda_convert`, `shabda_speak`, …) rather than Rust methods/traits/generics. Consumers pull `dist/shabda.cyr`.
- **Breaking**: Errors are **sakshi** packed-i64 codes (`0 == ok`) instead of `thiserror` enums; the Rust `ShabdaError` variants became `SHABDA_ERR_*` codes with `shabda_err_name()` diagnostics, and the `From<ShabdakoshError>` map became a code translation. Fallible functions return a packed error (test with `shabda_is_err`) and write their result to an out-param pointer.
- **Breaking**: Traits → dispatch. `Option<T>` → sentinels; `Result<T>` → pointer-or-0 / out-param; `convert_streaming` takes an `fnptr` callback instead of a Rust closure; `f32` prosody scalars widened to `f64`. No serde derive — serialization is hand-written where needed.
- **Breaking**: Cargo feature flags (`std`, `varna`, `json`, `logging`, `full`) **collapse** — CYRIUS has no feature flags, so varna phoneme-inventory validation, language detection, and every other capability are always compiled into the one bundle.
- **Feature**: full G2P pipeline — normalize → dictionary lookup (shabdakosh) / letter-to-sound rules fallback → syllabify (Maximal Onset Principle) → prosody (stress / rate / timing) → svara `PhonemeEvent`s. `shabda_convert` / `convert_with` / `convert_ssml` / `convert_streaming`, plus `shabda_speak` / `speak_with` (renders audio via svara).
- **Feature**: languages — English (dictionary + letter-to-sound rules), Spanish / German / Hindi / Arabic / Sanskrit (rules; native scripts + romanized fallback). `shabda_detect_language` picks a language from script over varna's ranges.
- **Feature**: normalization — abbreviation expansion (Dr.→doctor), acronym pronounceability heuristic (NASA→word, FBI→spelled out), number-to-words, foreign-word diacritic detection, and emphasis markers (ALL-CAPS / `*asterisk*`).
- **Feature**: prosody & SSML — syllable-weight stress, emphasis, speaking-rate clamp (50–300 WPM), a `TimingProfile` for independent vowel/consonant/pause scaling, intonation mapping, and an SSML subset parser (`<break>`, `<emphasis>`, `<prosody>`).
- **Feature**: heteronym disambiguation with context triggers, and varna-backed phoneme-inventory + phonotactics validation (`shabda_g2p_phoneme_inventory`, per-language IPA mapping) — always compiled, not feature-gated.
- **Changed**: dependencies are CYRIUS distlib bundles pulled via path (local dev) + git+tag (CI) — shabdakosh 3.0.2 (`shbdk_*`), svara 3.1.0 (`SVARA_PH_*`), varna 2.0.0. The transitive stack folds hisab/goonj/naad (shabdakosh) + hashmap/bayan (svara).
- **Removed**: the Rust `cli`/examples binaries, the criterion harness, and `no_std`/serde plumbing — replaced by `.tcyr` test suites and `tests/shabda.bcyr` benchmarks.

## [2.0.0] — 2026-04-01

Multi-language G2P, prosody control, SSML, accuracy, and varna integration.

- **Breaking**: `Language` enum expanded with `Spanish`, `German`, `Hindi`, `Arabic`, `Sanskrit` variants
- **Feature**: Spanish G2P rules — Castilian orthography (ch, ll, rr, qu, gu, c/g/z context)
- **Feature**: German G2P rules — sch, ch (ich/ach-Laut), ei/ie/eu digraphs, umlauts, final devoicing, ß
- **Feature**: Hindi G2P rules — Devanagari with inherent schwa deletion, virama/matra, romanized fallback
- **Feature**: Arabic G2P rules — 28 consonants, diacritics, shadda, tanween, hamza, romanized fallback
- **Feature**: Sanskrit G2P rules — perfectly regular Devanagari (no schwa deletion), 36C + 14V, romanized fallback
- **Feature**: `ConvertOptions` with builder pattern — `convert_with()` and `speak_with()` for emphasis, rate, timing
- **Feature**: Emphasis markers — ALL-CAPS and `*asterisk*` words receive emphatic stress
- **Feature**: Speaking rate control — `with_speaking_rate(wpm)`, clamped 50–300 WPM
- **Feature**: `TimingProfile` — independent vowel/consonant/pause duration scaling
- **Feature**: SSML subset parser — `<speak>`, `<break>`, `<emphasis>`, `<prosody>` tags
- **Feature**: `convert_ssml()` — applies SSML markup to G2P pipeline
- **Feature**: `convert_streaming()` — word-by-word callback API for real-time G2P
- **Feature**: Abbreviation expansion — 25-entry table (Dr.→doctor, Mr.→mister, etc.)
- **Feature**: Acronym handling — pronounceability heuristic (NASA→word, FBI→spelled out)
- **Feature**: Foreign word detection — diacritic detection with `strip_diacritics()` fallback
- **Feature**: Heteronym disambiguation — 20-entry context table (read, lead, live, wind, etc.)
- **Feature**: Optional `varna` feature — phoneme inventory validation, `detect_language()`, `phoneme_inventory()`
- **Feature**: Phonotactic constraint validation via varna (debug assertions)
- **Feature**: Language-aware IPA mapping — `validate_phonemes_for()` and `phoneme_to_ipa_for()`
- **Performance**: Rule compilation — static pattern slices eliminate per-match Vec allocations

## [1.0.0] — 2026-03-28

Full English G2P pipeline with dictionary, rules, syllabification, and prosody.

- **Feature**: Dictionary extracted to standalone `shabdakosh` crate (10,000+ entries)
- **Feature**: Context-sensitive vowel rules (magic-e, r-colored vowels)
- **Feature**: Silent letter handling (kn, gn, wr, ps, pn, mn, mb, bt, igh, eigh, ough)
- **Feature**: Morphological decomposition (-tion/-sion/-cian, -ed, un-/re-/dis-/pre-/mis-)
- **Feature**: Syllabification (Maximal Onset Principle with sonority constraints)
- **Feature**: Syllable-weight stress (heavy penult rule, antepenult fallback)
- **Feature**: Number-to-words expansion (0–999,999,999, decimals, negatives)
- **Feature**: Phrase-level prosody (commas 150ms, periods 300ms)
- **Changed**: Engine pipeline uses syllabification for stress instead of simple first-vowel rule
- **Changed**: `normalize()` expands numbers before text normalization

## [0.1.0] — 2026-03-27

- **Feature**: Initial scaffold — G2PEngine, dictionary lookup, rule-based fallback
- **Feature**: Built-in English dictionary (~30 entries), extensible
- **Feature**: Text normalization, sentence type detection, stress assignment
- **Feature**: `speak()` one-call text-to-audio via svara
- **Feature**: Integration tests, benchmarks, no_std, serde
