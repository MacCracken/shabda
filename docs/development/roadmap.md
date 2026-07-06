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
- **Parity audit** — a systematic function-level parity pass over the 9 modules against `rust-old/`
  (the shabdakosh port surfaced low-severity divergences this way).
