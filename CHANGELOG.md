# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
