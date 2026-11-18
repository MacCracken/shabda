# shabda — Roadmap

> Milestone plan through v1.0. State lives in [`state.md`](state.md);
> this file is the sequencing — what ships, in what order, against
> what dependency gates.

The port shipped straight to **v3.0.0** (full parity with the Rust 2.x surface), so the
original v0.x/v1.0 sequencing collapsed into a single parity milestone.

## Release criteria (v3.0.0)

- [x] Rust → Cyrius surface parity verified (function-level against `rust-old/`; every module ✅ or consciously collapsed)
- [x] Test coverage adequate for the surface area (653 assertions / 11 suites, all green)
- [x] Benchmarks captured (`tests/shabda.bcyr`)
- [x] Downstream consumption verified (`dist/shabda.cyr` bundle built + linked against the svara/shabdakosh/varna chain)
- [x] CHANGELOG complete (3.0.0 entry)

## Milestones

### M0 — Port scaffold (v0.1.0) — ✅ shipped 2026-07-06

- `cyrius port` scaffold landed
- Rust source moved to `rust-old/`
- Doc-tree per [first-party-documentation.md](https://github.com/MacCracken/agnosticos/blob/main/docs/development/applications/first-party-documentation.md)

### M1 — Full parity port (v3.0.0) — ✅ shipped 2026-07-06

Every Rust module ported function-for-function to CYRIUS and cross-checked against `rust-old/`:
error, normalize, syllable, heteronym, ssml, rules (English + Spanish/German/Hindi/Arabic/Sanskrit),
prosody, validate (varna), and the engine keystone (`convert` / `convert_with` / `convert_ssml` /
`convert_streaming` / `speak` / `speak_with`, plus `detect_language` / `phoneme_inventory`). The
Cargo feature flags collapsed (CYRIUS has none — varna validation is always compiled). distlib
bundle built + consumer-verified. See [`state.md`](state.md) for the per-module ledger.

## Out of scope (v3.0.0)

- **Feature flags** (`std` / `varna` / `json` / `logging` / `full`) — CYRIUS has no feature flags, so
  the gates collapse: varna phoneme-inventory validation, language detection, and every other
  capability are always compiled into the one bundle.
- **`no_std` / serde plumbing** — dropped with the Rust crate machinery; serialization is hand-written
  where needed.
- **The Rust `cli` / examples binaries and criterion harness** — replaced by `.tcyr` test suites and
  `tests/shabda.bcyr` benchmarks.

## Backlog

- **Security audit** — an adversarial audit of the untrusted-input paths (normalize, SSML parser,
  rules) on the shabdakosh model has not yet run for shabda; track before the next release.
- **Accuracy sweep** — done in 3.1.3 over all 9 modules, with `rust-old/` as the list of capabilities
  and accuracy as the oracle ([ADR-0002](../adr/0002-accuracy-over-rust-parity.md)); the 3.1.3
  CHANGELOG entry gives each change and its measurement. Left open, each checked on the 3.1.3 tree:
  - ~~**Dictionary stress**~~ — done in 3.2.0: shabdakosh 3.2.0 keeps CMUdict's digits
    (`shbdk_pronunciation_stress`), and a dictionary word takes them (87.8% → 100% of polysyllables
    with the right primary). Still open upstream: shabdakosh reads no stress from IPA (ˈ ˌ), so a
    dictionary loaded from IPA, PLS or a varna lexicon has none and its words take the stress rules.
  - **varna's inventory gaps** (upstream, varna), which validation reports: English /ɜ aɪ aʊ ɔɪ/,
    Hindi /aː ɽ z q x ɣ/, Sanskrit h.
  - **An allocator-taking constructor in svara and lookup in shabdakosh**, so shabda can drop its
    `_shabda_event_new_a` and `_shabda_dict_entry_a` copies (upstream).
  - **Stress outside English**: Spanish's orthographic rule (an accent wins; else a word ending in a
    vowel, n or s stresses the penult, others the last), German stem stress with unstressed prefixes.
  - **Number and abbreviation reading in the other languages** (it is English everywhere).
  - **English rules**: 46.4% of held-out CMUdict words exact; most remaining errors are stress guesses
    in the reduction pass. A word costs 17–22 µs through the rules.
  - **Unvocalized Arabic** (short vowels, shadda, diphthongs in monosyllables such as بيت).
  - **German lexical exceptions** (Mond, vielleicht, final-stress loans, compounds), Spanish x as the
    jota (México), English 2-letter acronyms and AT&T, inflections missing from the dictionary.
  - **SSML**: `<phoneme>`, `<sub>`, `<say-as>`, `<p>` / `<s>` breaks; intonation is computed and
    discarded; the pause durations (0.15 / 0.30 / 0.04 s) have no source.

---

## Moving the cyrius pin to 6.6.6

**Current pin:** `cyrius = "6.6.3"` (`cyrius.cyml`).

**shabda needs the pin bump and nothing else.** It is a pure text-transform
library — grapheme-to-phoneme rules, normalisation, SSML parsing — and does no
file I/O whatsoever: zero `sys_open` / `file_open` / `file_write_all` /
`file_read_all` / `file_exists` call sites in `src/`, `programs/` or `tests/`. The
Windows `O_APPEND` / `O_TRUNC` data corruption 6.6.6 fixes cannot reach it, and
there is no `CYRIUS_TARGET_WIN` guard in the tree; CI is `ubuntu-latest` only.

Checked and absent, so none of 6.6.6's new compile errors can fire: 0 structs
(no different-struct-copy site, no by-value deep-copy site), 0 `async fn`, 0
`operator` fns, 0 `ret2` / `rethi`, no SIMD intrinsics. The 24 `: cstring`
annotations are all **return** types (`shabda_phoneme_to_ipa` and its
per-language siblings in `src/validate.cyr`) — 6.6.6's rule is about a literal
int passed into a `: cstring` **param**, and shabda has none. 78 globals, none
redeclared. No `var` inside a top-level block (the `tests/heteronym.tcyr:93`
candidate is the body of a `fn` whose signature wraps across two lines). No own
`vec_*` definitions, so `assert.cyr`'s new transitive `vec.cyr` include cannot
collide. No raw `SYS_STATFS`. `lib/regression.cyr` is not vendored — the
"regression" grep hits are stale comments inside vendored `lib/syscalls*.cyr` and
`lib/hisab.cyr`, not calls — so 6.6.6's exec-deadline and `PR_SET_PDEATHSIG`
changes to that module do not reach shabda. `lib/` holds no symlinks and
`cyrius.lock` is writable.

After bumping: `cyrius test` and regenerate `dist/`.
