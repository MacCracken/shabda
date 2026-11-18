# shabda

[![version](https://img.shields.io/badge/version-3.1.2-blue.svg)](VERSION)
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
- **Server use** — `shabda_convert_a` / `shabda_convert_with_a` draw every allocation from a caller's Allocator, so an arena reset per request gets all of it back. A conversion only reads the engine, so threads can share one, each converting into its own arena.
- **Sakshi errors** — no panics; fallible functions return a sakshi packed-i64 code (`0 == ok`, test with `shabda_is_err`) and write their result to an out-param pointer.

## Quick Start

```sh
cyrius deps                              # resolve dependencies (shabdakosh, svara, varna, stdlib)
cyrius build src/main.cyr build/shabda   # compile the smoke binary
cyrius test tests/engine.tcyr            # run one suite
cyrius test tests                        # run all .tcyr suites
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

# In a server: convert into an arena and reset it per request. Nothing reaches
# the default heap, which never frees. One engine serves every thread; give
# each thread its own arena.
var arena = arena_allocator_growable(65536);
reset_via(arena);                        # at the start of each request
shabda_convert_a(arena, g, "hello world", slot);

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
concatenated bundle `dist/shabda.cyr` rather than rebuilding from `src/`. Point
`cyrius deps` at it as a git+tag dependency. Since 3.1.1 there is no
`dist/shabda.deps` sidecar: the one stdlib leaf the bundle uses (`unicode`)
arrives with `bayan`, which svara's sidecar requires. The bundle's module order
is the `[lib].modules` list in [`cyrius.cyml`](cyrius.cyml). Regenerate it with
`cyrius distlib`.

> **Consumers**: shabda resolves as a plain `[deps]` entry. Declare and include
> the chain in dependency order: svara (with hisab → goonj → naad), varna,
> **shabdakosh**, then shabda. shabda's engine and heteronym modules call
> shabdakosh's dictionary surface directly. Since 3.1.0 the chain needs
> cyrius 6.7.2 or later (svara's floor). shabda itself pins 6.7.18.

Direct dependencies (git+tag; no `path`, which would make the tag inert):

- **shabdakosh** 3.1.4 — pronunciation dictionary (`shbdk_*`; dict lookup, ARPABET/IPA, heteronym pronunciations).
- **svara** 3.14.0 — `SVARA_PH_*` phoneme identities, `PhonemeEvent`, and the sequence/voice/render surface for `speak()`. Brings hisab 3.5.0 / naad 2.3.5 / goonj 2.3.2.
- **varna** 2.6.0 — phoneme inventories, phonotactics, and script detection (self-contained on the stdlib folds).

## Tests & Benchmarks

- **840 assertions** across 11 `.tcyr` suites — error (25), normalize (65), syllable (55), heteronym (23), ssml (66), rules (133), prosody (54), validate (127), engine (246), crate-level shabda (37), allocator (9, including four threads converting at once). All green, plus a fuzz corpus (`cyrius fuzz`) of every shape that produced a defect, 400 random-byte rounds and 400 random-markup rounds, each also checking that `shabda_convert_a` matches `shabda_convert` and keeps nothing on the default heap.
- **Benchmarks** (`tests/shabda.bcyr`, x86_64, cyrius 6.7.18, median of 12): `g2p_hello_world` 17.0 µs, `g2p_sentence` 68.4 µs, `g2p_sentence_arena` (through `shabda_convert_a`) 64.5 µs, `speak_hello` 3.83 ms, `speak_sentence` 12.0 ms, `dict_english_construction` 8.64 ms, `dict_lookup_hit` 160 ns, `dict_lookup_miss` 176 ns. Figures from different sessions on this machine differ by up to about 20%; compare builds only within one session (see `docs/development/state.md`).

## Consumers

- [dhvani](https://github.com/MacCracken/dhvani) — AGNOS audio engine (text-to-speech pipeline)
- [vansh](https://github.com/MacCracken/vansh) — voice AI shell
- Any AGNOS component needing text-to-speech with svara

## License

GPL-3.0-only
