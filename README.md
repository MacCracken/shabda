# shabda

[![version](https://img.shields.io/badge/version-3.1.0-blue.svg)](VERSION)
[![license](https://img.shields.io/badge/license-GPL--3.0--only-green.svg)](LICENSE)
[![language](https://img.shields.io/badge/language-CYRIUS-orange.svg)](cyrius.cyml)

**shabda** (Sanskrit: *word / sound*) — the grapheme→phoneme (G2P) engine for
[AGNOS](https://github.com/MacCracken) text-to-speech. A **CYRIUS** (`.cyr`)
library: it turns text into a sequence of [svara](https://github.com/MacCracken/svara)
`SVARA_PH_*` `PhonemeEvent`s ready for synthesis, driving a
normalize → dictionary lookup ([shabdakosh](https://github.com/MacCracken/shabdakosh))
/ letter-to-sound rules fallback → syllabify → prosody pipeline, with an SSML
subset, heteronym disambiguation, and [varna](https://github.com/MacCracken/varna)
phoneme-inventory validation.

> shabda is a full-parity **CYRIUS port** of a ~5,000-line Rust library. It is no
> longer a Rust crate: the API is flat, `shabda_`-prefixed C-style functions
> (`shabda_g2p_new`, `shabda_convert`, `shabda_speak`, …) — no methods, traits,
> generics, `Cargo.toml`, or crates.io. Consumers pull `dist/shabda.cyr`.

## Features

- **G2P pipeline** — normalize → dictionary lookup (shabdakosh, 10k+ entries) / letter-to-sound rules fallback → syllabify (Maximal Onset Principle) → prosody → svara `PhonemeEvent`s. One call, `shabda_convert`.
- **Languages** — English (dictionary + rules) and Spanish / German / Hindi / Arabic / Sanskrit (rules; native scripts + romanized fallback). `shabda_detect_language` picks a language from script.
- **Normalization** — abbreviation expansion (Dr.→doctor), acronym heuristic (NASA→word, FBI→spelled out), number-to-words (42→"forty two"), foreign-word diacritic detection, and emphasis markers (ALL-CAPS / `*asterisk*`).
- **Prosody & SSML** — syllable-weight stress, emphasis, speaking-rate clamp (50–300 WPM), a `TimingProfile` for independent vowel/consonant/pause scaling, intonation mapping, and an SSML subset (`<break>`, `<emphasis>`, `<prosody>`) via `shabda_convert_ssml`.
- **Heteronym disambiguation** — context triggers pick the right pronunciation for `read`, `lead`, `live`, `wind`, …
- **Validation** — varna-backed phoneme-inventory and phonotactics validation with per-language IPA mapping (always compiled — no feature gate).
- **Streaming & audio** — `shabda_convert_streaming` runs a word-by-word `fnptr` callback; `shabda_speak` renders straight to audio samples through svara in one call.
- **Sakshi errors** — no panics; fallible functions return a sakshi packed-i64 code (`0 == ok`, test with `shabda_is_err`) and write their result to an out-param pointer.

## Quick Start

```sh
cyrius deps                              # resolve dependencies (shabdakosh, svara, varna, stdlib)
cyrius build src/main.cyr build/shabda   # compile the smoke binary
cyrius test tests/engine.tcyr            # run one suite
cyrius tests tests                       # run all .tcyr suites
cyrius bench tests/shabda.bcyr           # run benchmarks
cyrius distlib                           # regenerate dist/shabda.cyr
```

## Usage

Names are flat and `shabda_`-prefixed. Fallible functions follow the CYRIUS
out-param convention: they return a packed shabda error (`0 == ok`) and write the
result vec to a caller-provided slot. `shabda_convert` writes the events vec.

```cyrius
# Build an engine for a language (English loads the shabdakosh dictionary).
var g = shabda_g2p_new(SHABDA_LANG_ENGLISH);

# Convert text -> vec of svara PhonemeEvents (out-param convention).
var slot = alloc(8);
var err = shabda_convert(g, "hello world", slot);
if (shabda_is_err(err) == 1) {
    println(shabda_err_name(err));
} else {
    var events = load64(slot);          # vec of svara PhonemeEvent handles
    println_int(vec_len(events));
}

# With options: emphasis + a slower rate.
var opts = shabda_convert_options_new();
opts = shabda_convert_options_with_emphasis(opts, 1);
opts = shabda_convert_options_with_speaking_rate(opts, 100.0);
shabda_convert_with(g, "HELLO world", opts, slot);

# SSML input.
shabda_convert_ssml(g, "Hello <break time=\"300ms\"/> <emphasis level=\"strong\">world</emphasis>", slot);

# Straight to audio through svara.
var voice = svara_voice_new_male();
var samples = alloc(8);
shabda_speak(g, "hello world", voice, 44100.0, samples);
```

## Architecture

The pipeline (`src/engine.cyr`) ties the modules together:

```text
Input text
    |
    v
Normalize (src/normalize.cyr)
    |-- abbreviation expansion (Dr. -> doctor)
    |-- acronym handling (FBI -> f b i, NASA -> nasa)
    |-- number expansion (42 -> "forty two")
    |-- lowercase, punctuation -> phrase markers, emphasis markers
    v
G2P core (src/engine.cyr)
    |-- heteronym check          (src/heteronym.cyr — context-based)
    |-- dictionary lookup        (shabdakosh, 10k+ entries)
    |-- foreign-word detection   (strip diacritics, retry)
    |-- letter-to-sound rules    (src/rules.cyr — per-language fallback)
    v
Syllabify (src/syllable.cyr — Maximal Onset Principle)
    |
    v
Prosody (src/prosody.cyr — stress, emphasis, rate, timing, intonation)
    |
    v
vec of svara PhonemeEvents  ->  shabda_speak renders audio via svara
```

`src/validate.cyr` (varna inventory + phonotactics) and `src/ssml.cyr` (the SSML
subset parser) feed the pipeline; `src/error.cyr` carries the sakshi-backed error
surface.

## Module Overview

Include order from `src/main.cyr` (modules never include each other — the entry
orders them; stdlib + shabdakosh/svara/varna auto-resolve from `cyrius.cyml`):

```text
src/
├── error.cyr        sakshi-backed error surface (shabda_err_*, shabda_is_err); From<ShabdakoshError> map
├── normalize.cyr    text normalization, abbreviation/acronym/number expansion, foreign-word + emphasis markers
├── syllable.cyr     syllabify() via Maximal Onset Principle with sonority constraints
├── heteronym.cyr    heteronym disambiguation with context triggers
├── ssml.cyr         SSML subset parser (break / emphasis / prosody)
├── rules.cyr        letter-to-sound rules — English, Spanish, German, Hindi, Arabic, Sanskrit
├── prosody.cyr      stress assignment, emphasis, speaking rate, timing profiles, intonation mapping
├── validate.cyr     phoneme -> IPA mapping (per-language), varna inventory + phonotactic validation
└── engine.cyr       G2PEngine, Language, ConvertOptions, TimingProfile; convert*/speak*; detect_language, phoneme_inventory
```

## Consuming the distlib

Downstream AGNOS components (dhvani, vansh, and any TTS consumer) pull the
concatenated bundle `dist/shabda.cyr` and its `dist/shabda.deps` sidecar rather
than rebuilding from `src/`. Point `cyrius deps` at it as a git+tag dependency;
the sidecar names the one stdlib leaf the bundle needs (`unicode`). The
bundle's module order is the `[lib].modules` list in
[`cyrius.cyml`](cyrius.cyml). Regenerate it with `cyrius distlib`.

> **Consumers**: since 3.0.6 (cyrius 6.6.20), `dist/shabda.deps` names only
> `unicode`, so shabda resolves as a plain `[deps]` entry. Declare and include
> the chain in dependency order: svara (with hisab → goonj → naad), varna,
> **shabdakosh**, then shabda. shabda's engine and heteronym modules call
> shabdakosh's dictionary surface directly. Since 3.1.0 the chain needs
> cyrius 6.7.2 or later (svara 3.6.0's floor); shabda itself pins 6.7.9.

Direct dependencies (git+tag; no `path`, which would make the tag inert):

- **shabdakosh** 3.1.0 — pronunciation dictionary (`shbdk_*`; dict lookup, ARPABET/IPA, heteronym pronunciations).
- **svara** 3.6.0 — `SVARA_PH_*` phoneme identities, `PhonemeEvent`, and the sequence/voice/render surface for `speak()`. Brings hisab 3.4.0 / naad 2.3.0 / goonj 2.1.0.
- **varna** 2.5.0 — phoneme inventories, phonotactics, and script detection (self-contained on the stdlib folds).

## Tests & Benchmarks

- **727 assertions** across 10 `.tcyr` suites — error (25), normalize (60), syllable (55), heteronym (23), ssml (66), rules (133), prosody (50), validate (127), engine (151), crate-level shabda (37). All green, plus a fuzz corpus (`cyrius fuzz`) of every shape that produced a defect, 400 random-byte rounds and 400 random-markup rounds.
- **Benchmarks** (`tests/shabda.bcyr`, x86_64, cyrius 6.7.9, median of 3): `g2p_hello_world` 15.2 µs, `g2p_sentence` 59.7 µs, `speak_hello` 5.32 ms, `speak_sentence` 17.2 ms, `dict_english_construction` 7.47 ms, `dict_lookup_hit` 133 ns, `dict_lookup_miss` 133 ns.

## Consumers

- [dhvani](https://github.com/MacCracken/dhvani) — AGNOS audio engine (text-to-speech pipeline)
- [vansh](https://github.com/MacCracken/vansh) — voice AI shell
- Any AGNOS component needing text-to-speech with svara

## License

GPL-3.0-only
