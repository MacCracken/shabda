# Changelog

## [3.1.1] — 2026-11-07

Toolchain and dependency release, plus an audit cleanup. `cyrius` 6.7.9 -> **6.7.18**,
`shabdakosh` 3.1.0 -> **3.1.1**, `svara` 3.6.0 -> **3.6.3**, `varna` 2.5.0 -> **2.5.2**, and
transitively `hisab` 3.4.0 -> **3.4.1**, `naad` 2.3.0 -> **2.3.3**, `goonj` 2.1.0 -> **2.3.1**,
`sakshi` 2.5.8 -> **2.5.10**. Each is its dep's newest tag and pins 6.7.18, and the lock's six
commit pins are the ones shabdakosh 3.1.1 ships. `lib/` and `cyrius.lock` were regenerated from
scratch. Every vendored dep is byte-identical to its tag's `dist/` bundle, every stdlib leaf is
byte-identical to the 6.7.18 snapshot, `cyrius deps --verify` passes 53 of 53, and
`cyrius deps --locked` is clean.

No behavior, API or output change. **727 assertions / 10 suites**, the same as 3.1.0, and
`cyrius fuzz` passes. svara 3.6.3 keeps the consumer floor at cyrius 6.7.2. **`cyrius audit` exits
0**, which it last did at 3.0.3.

### Fixed — `cyrius audit` is clean

- **fmt**: `cyrius fmt` re-indented 3 continuation lines in `tests/engine.tcyr` and
  `tests/syllable.tcyr`, from 4 spaces to 2.
- **lint**: the 14 warnings in `tests/` are gone. 12 assert calls over 120 columns are split before
  their message argument, and a doubled blank line in `tests/engine.tcyr` is removed. The
  271-column expected string for `expand i64::MAX` in `tests/normalize.tcyr` is now built from
  three literals with `str_builder` at run time, the way `tests/engine.tcyr` and `tests/ssml.tcyr`
  already build their nesting inputs. Cyrius cannot split one literal across lines yet:
  adjacent-literal concatenation is still a 6.7.x proposal. The assertion compares the same string.
- **docs**: the 40 undocumented public fns now have doc comments. 37 are in `src/`: engine 12,
  ssml 10, error 4, heteronym 4, normalize 4, syllable 3. Most are ported from the Rust doc comment
  of the same item, and each states the Cyrius encoding: f64 bit patterns, sentinels, and the
  fallback on an unknown name or code. The other 3 are entry points: `main` in `tests/shabda.bcyr`,
  and `fuzz_main` and `main` in `fuzz/shabda.fcyr`.

### Changed

- **`lib/`**: 34 files change, and `simd.cyr` is added because goonj 2.3.1 includes it. shabda
  already vendored `lib/unicode/`, which the 6.7.18 `bayan` now includes.
- **`lib/shabdakosh.cyr` was copied by hand from the 3.1.1 tag**, per ADR-0001, after checking that
  the local tag is the commit GitHub has (`7642b9b`). The 6.7.18 resolver now notes on every resolve
  that this file is outside the declared footprint, because shabdakosh is `optional`. It keeps the
  file and its lock line, so the note is expected.
- **`dist/shabda.deps` is gone**, and `dist/shabda.cyr` no longer opens with the
  `include "lib/unicode/*.cyr"` lines. The 6.7.18 compile-verify finds no stdlib leaf left to name:
  the `unicode` tables now arrive with `bayan`, which svara's sidecar requires and which includes
  `lib/unicode/` at 6.7.18. A consumer gets them through svara, which shabda needs anyway.
  Otherwise the bundle changes only in its version header and the new doc comments.
- **`build/shabda` grew from 4,010,864 to 4,842,184 bytes.** The default build keeps unreachable
  functions, and the unreachable share grew from 3.02 MB (6,920 fns) to 3.62 MB (7,715 fns).
- **`cyrius tests` was removed in cyrius 6.7.12.** CLAUDE.md, README and `state.md` now give
  `cyrius test tests` for running every suite. CI already runs the bare `cyrius test`, which finds
  the same 10 suites.

### Checked — the 6.7.10–6.7.18 meaning changes do not reach shabda

varna 2.5.1 lists what 6.7.10–6.7.18 changed in meaning: struct values, `Str`, multi-value returns,
declaration zones, `f64`, `**T`, closures, `impl`, narrow scalar annotations, duplicate globals and
fns, and include spellings. With comments and strings stripped, `src/`, `tests/` and `fuzz/` have
no `struct`, `enum`, `#derive`, `Str` annotation, `ret2`, `: stack`, `**T`, closure, `impl` or
narrow integer annotation. They have no duplicate global or fn, no fn that shadows a `lib/` one,
and one spelling per include. `f64` goes only through the `f64_*` calls on i64 bit patterns.

The one natively typed spot is the `f32` multiply behind `<break time="Xs">`
(`_shabda_f32_mul` in `src/ssml.cyr`), and 6.7.x changed how an `f32` local is stored. A
differential over 404,075 inputs to `_shabda_secs_to_ms_u32` was built under 3.1.0 (cyrius 6.7.9)
and 3.1.1, and the two outputs are byte-identical. The inputs cover every millisecond from −2 s to
200 s, the same range in sevenths of a second, 64 powers of two divided by three, NaN, ±inf, ±0,
the smallest subnormal and the u32 boundary. The 10,647-input conversion differential from 3.1.0
is byte-identical too.

varna 2.5.2's breaking change is to its `-D MCP` tool surface, and `allophone_realize` now compares
by value. shabda calls neither, and all 12 varna functions it does call exist in 2.5.2.

### Benchmarks

`tests/shabda.bcyr`, x86_64. Medians of **8** runs, with 3.1.0 (cyrius 6.7.9, its `lib/`) and
3.1.1 alternated in one session. Each delta is toolchain and dependencies together. Every row's
range overlaps between versions, so none of these is a measured change. The first three rounds
alone made `dict_lookup_miss` look 27 % slower. The stdlib hashmap seeds itself per process, so a
lookup row moves from run to run, and five more rounds put its medians 3 ns apart.

| Benchmark | 3.1.0 | 3.1.1 | Δ (all overlap) |
|-----------|-------|-------|-----------------|
| g2p_hello_world | 15.29 µs | 15.32 µs | +0.2% |
| g2p_sentence | 60.58 µs | 60.34 µs | −0.4% |
| speak_hello | 5.30 ms | 5.18 ms | −2.4% |
| speak_sentence | 17.39 ms | 16.95 ms | −2.6% |
| dict_english_construction | 7.53 ms | 7.41 ms | −1.6% |
| dict_lookup_hit | 136 ns | 136 ns | noise |
| dict_lookup_miss | 152 ns | 155 ns | noise (3.1.1 ranged 148–525 ns) |

## [3.1.0] — 2026-11-07

Toolchain and dependency release, moving to the cyrius 6.7 language minor. `cyrius` 6.6.20 ->
**6.7.9**, `shabdakosh` 3.0.8 -> **3.1.0**, `svara` 3.5.6 -> **3.6.0**, `varna` 2.4.3 -> **2.5.0**,
and transitively `hisab` 3.3.7 -> **3.4.0**, `naad` 2.2.5 -> **2.3.0**, `goonj` 2.0.6 -> **2.1.0**,
`sakshi` 2.5.7 -> **2.5.8**. This is the 6.7.9 batch. svara 3.6.0, varna 2.5.0, hisab 3.4.0, naad
2.3.0 and goonj 2.1.0 are the tags pinned to exactly 6.7.9, and the next tag of each needs 6.7.18.
shabdakosh 3.1.0 ships the same set, and its lock carries the same six commit pins. `lib/` and
`cyrius.lock` were regenerated from scratch. Every vendored dep is byte-identical to its tag's
`dist/` bundle, every stdlib leaf is byte-identical to the 6.7.9 snapshot, `cyrius deps --verify`
passes 52 of 52, and `cyrius deps --locked` confirms that `lib/` and the lock are exactly what the
tags resolve.

It is a minor because the floor for consumers moves: svara 3.6.0 needs cyrius 6.7.2 or later.
Output changes for one word, which comes from shabdakosh (see *Fixed*).

No source change, no API change. **727 assertions / 10 suites**, the same as 3.0.6; `cyrius fuzz`
passes. `src/` stays fmt- and lint-clean, and `cyrius vet` and `cyrius deny` are clean.

### Fixed

- **`portuguese` now comes from the dictionary.** shabdakosh 3.0.8's generator stored the
  one mixed-case CMUdict entry as `Portuguese`, which a lowercasing lookup never reaches, so shabda
  fell back to the English letter-to-sound rules. Rust's `build.rs` lowercases, so the oracle
  always took the dictionary path. shabdakosh 3.1.0 fixes the generator. A differential over the
  10,617 words of shabdakosh's `data/cmudict-5k.txt`, plus 30 sentences and other-language inputs,
  was built under 3.0.6 (cyrius 6.6.20) and under 3.1.0. Only the 3 `portuguese` rows differ: the
  9 rule-derived events become the dictionary's 8. Every other row is byte-identical, durations
  included.

### Changed

- **`lib/`**: 31 of its 52 files change and the file set does not: nothing is added or dropped.
- **`lib/shabdakosh.cyr` was copied by hand from the 3.1.0 tag**
  (`git -C ../shabdakosh show 3.1.0:dist/shabdakosh.cyr`), per ADR-0001, after checking that the
  local tag is the commit GitHub has (`88cea94`).
- **`cyrius.lock`**: the 6 commit pins move to the 6.7.9 batch, and the toolchain row is
  `cyrius 6.7.9`. Re-resolving into the existing lock appends the new commit rows beside the old
  ones (it held 12), so the lock was regenerated from an empty file.
- **`lib/sakshi.cyr` is 2.5.8**, the 6.7.9 snapshot's copy, byte-identical to the sakshi 2.5.8
  artifact the dep chain pins, so nothing is split. The `refusing to overwrite stdlib leaf 'sakshi'`
  note still prints on every resolve, as expected.
- `dist/shabda.cyr` changes only in its version header. `dist/shabda.deps` still names only
  `unicode`.

### Checked — the 6.7.x meaning changes do not reach shabda

6.7.x changes what some existing code *means*, not just what compiles. As varna 2.5.0 did,
`src/`, `tests/` and `fuzz/` were swept with comments and string literals stripped. They contain no
lone `!` (a real operator since 6.7.3; before that `if (!ok)` meant `if (ok)`), no unary `-` or `~`
(the 6.7.9 binding change), no `for` loops (the 6.7.8–6.7.9 step rule), no `*T` derefs or slices
(6.7.8's element width), and none of the newly reserved words (`trait`, `const`, `true`, `false`,
`do`, `dyn`). shabda negates as `0 - x` and loops with `while`.

### Note — `cyrius distlib` lists 5 `shbdk_*` names for the consumer to supply

Unchanged from 3.0.6: `shbdk_dict_new`, `shbdk_dict_english`, `shbdk_dict_lookup`,
`shbdk_dict_lookup_all` and `shbdk_pronunciation_phonemes`, supplied by the consumer's
`… → varna → shabdakosh → shabda` chain. Expected, and not a defect.

### Known issue — `cyrius audit` exits 1

Unchanged from 3.0.6, leg for leg: **fmt** (`tests/engine.tcyr`, `tests/syllable.tcyr`), **lint**
(14 warnings, all in `tests/`), **docs** (40 undocumented public fns). CI does not run
`cyrius audit`.

### Benchmarks

`tests/shabda.bcyr`, x86_64. Medians of 3 runs, with 3.0.6 (cyrius 6.6.20, its committed `lib/`)
and 3.1.0 alternated in one session. Each delta is toolchain and dependencies together, not either
one alone. `g2p_hello_world` and `speak_hello` were slower in every 3.1.0 run than in every 3.0.6
run. The other rows overlap between versions, and the lookup rows are sub-µs noise (3.0.6's
`dict_lookup_miss` alone ranged 141–219 ns).

| Benchmark | 3.0.6 | 3.1.0 | Δ |
|-----------|-------|-------|---|
| g2p_hello_world | 15.05 µs | 15.18 µs | +0.9% |
| g2p_sentence | 58.84 µs | 59.72 µs | +1.5% (overlaps) |
| speak_hello | 5.17 ms | 5.32 ms | +2.8% |
| speak_sentence | 16.94 ms | 17.16 ms | +1.3% (overlaps) |
| dict_english_construction | 7.35 ms | 7.47 ms | +1.7% (overlaps) |
| dict_lookup_hit | 135 ns | 133 ns | noise |
| dict_lookup_miss | 147 ns | 133 ns | noise |

## [3.0.6] — 2026-11-07

Toolchain and dependency release. `cyrius` 6.6.3 -> **6.6.20**, `shabdakosh` 3.0.5 -> **3.0.8**,
`svara` 3.5.5 -> **3.5.6**, `varna` 2.4.2 -> **2.4.3**, and transitively `hisab` 2.22.1 -> **3.3.7**,
`naad` 2.2.2 -> **2.2.5**, `goonj` 2.0.4 -> **2.0.6**, `sakshi` 2.5.2 -> **2.5.7**. This is the set
matched to 6.6.20, the same one shabdakosh 3.0.8 ships: svara 3.5.6 and varna 2.4.3 are the last
tags on 6.6.x, and their next tags need 6.7.x. svara's manifest supplies the hisab / naad / goonj
versions. `lib/` and `cyrius.lock` were regenerated from scratch (`cyrius deps` into an empty
`lib/`). Every vendored dep is byte-identical to its tag's `dist/` bundle, every stdlib leaf is
byte-identical to the 6.6.20 snapshot, and `cyrius deps --verify` passes 52 of 52.

No source change, no API change. **727 assertions / 10 suites**, the same as 3.0.5; `cyrius fuzz`
passes.

### Fixed

- **The svara and varna tag pins were inert.** `[deps.svara]` and `[deps.varna]` set `path`
  beside `git` and `tag`, and an existing `path` makes both inert: `cyrius deps` copies from the
  sibling checkout with no clone and no commit pin, and every `cyrius build` / `test` / `tests`
  resolves again. So 3.0.5 vendored svara 3.5.5 and varna 2.4.2 while the manifest said 3.5.4
  and 2.4.1, and the lock had no svara or varna row. The siblings now sit at 3.6.3 and 2.5.2 on
  cyrius 6.7.18, so running the suites would have pulled the 6.7.x chain into `lib/`.
  `path` is dropped from all three deps, and svara and varna now resolve from their tags with a
  commit pin. shabdakosh is `optional` and never resolved through its `path`; that line goes for
  consistency.
- **3.0.4's sakshi split is gone.** The 6.6.20 resolver keeps the stdlib snapshot's
  `lib/sakshi.cyr` and skips the dep artifact. Both copies are 2.5.7 here, so nothing is split.
  `cyrius build` / `test` / `deps` print a `refusing to overwrite stdlib leaf 'sakshi'` note on
  every resolve. That note is expected, and shabdakosh 3.0.8 prints it too.
- **Assertion count.** README and `state.md` said 737, but the per-suite counts they list add up
  to 727, which is what the suites report. Both now say 727.

### Changed

- **`dist/shabda.deps`: 22 leaves -> 1 (`unicode`).** Since cyrius 6.6.18 a sidecar lists only
  what the compile-verify proves the bundle needs beyond its named deps. `dist/shabda.cyr` now
  opens with the `include "lib/unicode/*.cyr"` lines for that leaf.
- **`lib/`**: `hashmap_fast.cyr` dropped (not declared; nothing includes it). `alloc_cx.cyr`,
  `chrono.cyr` and `poison.cyr` added, because the 6.6.x stdlib leaves now include them. The lock
  gains the svara and varna commit pins and a `cyrius 6.6.20` toolchain row.
- **`lib/shabdakosh.cyr` was copied by hand from the 3.0.8 tag**
  (`git -C ../shabdakosh show 3.0.8:dist/shabdakosh.cyr`), per ADR-0001. It was not copied from
  `../shabdakosh/dist/`: that checkout is on 3.1.1 / cyrius 6.7.18, so the `cp` that ADR-0001
  documents would vendor the wrong version.
- svara's bundle no longer prints `#inline ignored: body exceeds 32 tokens` warnings.

### Note — `cyrius distlib` lists 5 `shbdk_*` names for the consumer to supply

`shbdk_dict_new`, `shbdk_dict_english`, `shbdk_dict_lookup`, `shbdk_dict_lookup_all` and
`shbdk_pronunciation_phonemes` are shabdakosh's dictionary surface, which `src/engine.cyr` and
`src/heteronym.cyr` call (dictionary-first lookup, heteronym pronunciations). `dist/shabda.cyr`
does not carry shabdakosh. A consumer supplies it from the chain it declares and includes in
dependency order, `… → svara → varna → shabdakosh → shabda`, as dhvani does. This is expected
and not a defect.

### Known issue — `cyrius audit` exits 1

This predates the bump: 3.0.5 on 6.6.3 already failed three legs. **fmt**: `tests/engine.tcyr`
and `tests/syllable.tcyr` need reformatting. **lint**: 14 warnings, all in `tests/`, mostly lines
over 120 columns. **docs**: 3 undocumented public fns on 6.6.3, now **40** on 6.6.20, because
cyrdoc stopped counting `#must_use` as documentation (shabdakosh 3.0.8 hit the same thing). On
both, `src/` is fmt- and lint-clean. CI does not run `cyrius audit`.

### Benchmarks

`tests/shabda.bcyr`, x86_64. Medians of 3 runs, with 3.0.5 (cyrius 6.6.3, its committed `lib/`)
and 3.0.6 alternated in one session:

| Benchmark | 3.0.5 | 3.0.6 | Δ |
|-----------|-------|-------|---|
| g2p_hello_world | 16.51 µs | 15.42 µs | −6.6% |
| g2p_sentence | 64.35 µs | 61.50 µs | −4.4% |
| speak_hello | 5.51 ms | 5.38 ms | −2.4% |
| speak_sentence | 17.94 ms | 17.64 ms | −1.7% |
| dict_english_construction | 7.84 ms | 7.64 ms | −2.6% |
| dict_lookup_hit | 138 ns | 141 ns | noise |
| dict_lookup_miss | 167 ns | 185 ns | noise (3.0.5 alone ranged 147–298 ns) |

## [3.0.5] - Unblocked by cyrius 6.6.3

Toolchain-pin release. `cyrius` 6.6.2 -> **6.6.3**; no source change.

6.6.2 could not compile this project. `#inline` became a real directive at cyrius
v6.5.63, which taught the compiler's PASS 2 to arm the directive — but PASS 1, the
declaration-collection scan, was never taught to CONSUME the token. An unconsumed
directive there falls through to the catchall and **terminates the scan**, so every
declaration after the first `#inline` went unregistered and the next `#derive`d
struct reached the parser as an unknown top-level token. The error named the struct,
which was innocent, hundreds of lines from the real trigger.

cyrius 6.6.3 closes both halves in all seven per-target compiler forks. Re-vendored
and rebuilt against it; tests green.

## [3.0.4] — 2026-09-01

Toolchain pin to cyrius **6.5.37**, a sidecar correction that unblocks consumers, and a repair
to this repo's own release tooling.

### Fixed — `dist/shabda.deps` named three VENDORED deps as stdlib leaves

The sidecar listed `hisab`, `goonj` and `naad` among its stdlib requirements. None is a cyrius
stdlib module — they are vendored dependencies reached through `src/main.cyr`, the umbrella
`cyrius distlib` scans for `include "lib/X.cyr"` lines. The scan correctly captured "these must
be in scope" but tagged them as *stdlib*, so a consumer's `cyrius deps` hunted for them in the
stdlib and failed outright:

```
error: dep shabda requires 'hisab' but it is not in the cyrius stdlib
```

⭐ **Over-reporting is the fatal direction.** An under-reported sidecar fails silently at the
consumer's compile step; an over-reported one is a HARD resolver error *before* compilation is
reached — the same shape that left `crab` and `puka` unable to resolve `dhancha` at all.

Fixed upstream in cyrius 6.5.37, which validates every captured leaf against the stdlib snapshot
at generation time, so the names are dropped by construction. Regenerating picks that up: the
sidecar is now **23 leaves, all of which resolve**.

⚠ No source change was needed or made. `src/main.cyr`'s includes are correct — they carry the
dependency chain the bundled modules require. Only the published metadata was wrong.

### Fixed — `scripts/version-bump.sh` had been BROKEN since the Rust port

It was still the Rust-era script: after writing `VERSION` it ran `sed -i … "$REPO_ROOT/Cargo.toml"`
followed by `cargo generate-lockfile`. `Cargo.toml` now lives under `rust-old/`, not the repo
root, so under `set -euo pipefail` the sed failed and the script **died at that line** — writing
`VERSION` and never regenerating the bundle. Any bump run through it left `dist/shabda.cyr`
stamped with the *previous* version while `VERSION` claimed the new one.

It failed loudly, which is the only reason this was caught; a `|| true` on that line would have
made it silent. Rewritten cyrius-native (write `VERSION`, then `cyrius distlib --all`), matching
shabdakosh's. `--all` rather than bare `distlib` so profile bundles cannot be left behind — the
sankoch 2.7.6 shape.

### Changed

- `cyrius` pin **6.5.36 → 6.5.37**.
- `lib/` re-synced from the 6.5.37 snapshot (42 files).
- `dist/shabda.cyr` + `dist/shabda.deps` regenerated.

### Known issue — `lib/sakshi.cyr` stays at 2.4.11

The pinned 6.5.37 snapshot ships sakshi **2.4.12** (fixing a negative-depth buffer underflow in
`sakshi_span_enter`), and `cyrius lib sync` writes it correctly — but the next `cyrius deps`
reverts it to **2.4.11**, byte-identical to a named dep's own vendored copy. The resolver pulls a
declared *stdlib* leaf from the dep's `lib/` instead of the pinned snapshot, so the two commands
fight and `deps` wins.

A cyrius resolver defect, not a shabda one, filed upstream as
`cyrius/docs/development/issues/2026-09-01-deps-pulls-stdlib-leaf-from-dep-lib.md`. shabda is the
**second** independent repo confirmed hitting it (shabdakosh 3.0.6 is the first). It does not
affect this release: the build is clean and all 198 tests pass.


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
