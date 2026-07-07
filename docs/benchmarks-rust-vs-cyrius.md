# Benchmarks: Rust vs Cyrius

> shabda v3.0.0 parity benchmark — the same seven workloads run against the
> preserved Rust crate (`rust-old/`) and the CYRIUS port (`src/`), on the same
> machine, 2026-07-06.
>
> - **Rust**: criterion 0.5, `--release`. Deps from crates.io: shabdakosh 2.0.0,
>   svara 2.0.0, varna 1.0.0 (the 2.x-era stack the crate shipped against).
>   Default features (`std`) — so the `#[cfg(feature="varna")]` validation in
>   `convert_with` is `debug_assert!`-gated and **elided in release**.
> - **Cyrius**: cycc 6.4.12, `bench.cyr`. Deps: shabdakosh 3.0.2, svara 3.1.0,
>   varna 2.0.0 (the ported bundles). No feature flags — the varna inventory +
>   phonotactic **validation always runs** inside `shabda_convert`.
> - **Platform**: x86_64 Linux. Numbers are per-operation.
>
> This is an honest real-world comparison (old Rust stack vs current Cyrius
> stack), not a version-matched one — behaviour is full parity, dependency
> versions differ.

## Head-to-Head

| Operation | Rust | Cyrius | Ratio | Notes |
|-----------|------|--------|-------|-------|
| `g2p_hello_world` (convert 2 words) | 838 ns | 21.88 µs | **26×** | per-word heap vecs + always-on validation |
| `g2p_sentence` (convert 10 words) | 3.28 µs | 83.6 µs | **25×** | scales linearly with words |
| `speak_hello` (convert + render audio) | 1.47 ms | 7.65 ms | **5.2×** | svara **3.1.0** control-rate diphthong (was 22.4 ms / 15×); "hello" ends /oʊ/ |
| `speak_sentence` | 1.27 ms | 24.79 ms | **19×** | svara render; monophthong-heavy, so little diphthong benefit |
| `dict_english_construction` (load 10k) | 1.59 ms | 9.40 ms | **5.9×** | hashmap inserts; closest ratio |
| `dict_lookup` (hit) | ~10.6 ns/word¹ | 127 ns | **~12×** | O(1) hashmap both sides; alloc-free |
| `dict_lookup_miss` | 9.83 ns | 249 ns | **25×** | miss path |

¹ Rust `dict_lookup_5k` measures 6 mixed words per iteration (63.7 ns total ⇒
~10.6 ns/word); the Cyrius `dict_lookup_hit` measures a single `"the"` lookup
(127 ns). Not a perfectly matched harness — read it as order-of-magnitude.

## Full Rust set (criterion, release — [low  median  high])

| Benchmark | median | range |
|-----------|--------|-------|
| g2p_hello_world | 838.5 ns | [832.4, 845.3] |
| g2p_sentence | 3.277 µs | [3.263, 3.291] |
| speak_hello | 1.4733 ms | [1.4703, 1.4763] |
| speak_sentence | 1.2711 ms | [1.2681, 1.2742] |
| dict_english_construction | 1.5857 ms | [1.5800, 1.5916] |
| dict_lookup_5k (6 words) | 63.74 ns | [63.35, 64.18] |
| dict_lookup_miss | 9.834 ns | [9.758, 9.934] |

## Full Cyrius set (`cyrius bench tests/shabda.bcyr`, cycc 6.4.12)

| Benchmark | avg | iterations |
|-----------|-----|------------|
| g2p_hello_world | 21.876 µs | 5,000 |
| g2p_sentence | 83.595 µs | 2,000 |
| speak_hello | 7.651 ms (svara 3.1.0; was 22.4 ms) | 200 |
| speak_sentence | 24.788 ms | 100 |
| dict_english_construction | 9.395 ms | 50 |
| dict_lookup_hit | 127 ns | 200,000 |
| dict_lookup_miss | 249 ns | 200,000 |

## Analysis

### Why Cyrius is 5–26× slower per operation

| Factor | Cost | Where it bites |
|--------|------|----------------|
| **Validation always-on** | 2 varna passes / word | `convert_*` — Rust release `debug_assert!`-elides `validate_phonemes_for` + `validate_phonotactics`; Cyrius runs them every convert. A large slice of the 25× G2P gap. |
| **Heap allocation** | ~100–250 ns per `alloc` | G2P is alloc-heavy: per word a phoneme vec (lookup copy / rules output), a syllables vec (each syllable = struct + onset/coda vecs), an events vec (each event = an `SvPhonemeEvent` alloc). Rust reuses buffers / stacks / SmallVec. |
| **f64 vs f32** | ~1.5–2× | prosody durations + svara synthesis math widened f32→f64 (x87/SSE2, no SIMD). |
| **svara synthesis** | dominates `speak_*` | `shabda_speak` → `svara_sequence_render` is svara's (Cyrius-ported) formant synthesizer; the residual `speak` gap is mostly the dependency, not shabda logic. svara **3.1.0** control-rate glide coefficients already cut `speak_hello` 22.4→7.65 ms (diphthong words); the rest tracks svara-side. |
| **UTF-8 string work** | per char | `normalize` decodes UTF-8 into a code-point vec and rebuilds NUL-terminated cstrings; Rust operates on `&str`/`String` in place. |

The multiplier is smallest where the work is bounded by data movement, not
per-item allocation: `dict_english_construction` (5.9×, hashmap inserts) and the
alloc-free O(1) `dict_lookup` hit (~12×).

### Does it matter for the workload?

For real-time TTS, yes — it's comfortably fast. `speak_hello` renders a word of
audio (~0.5 s at 44.1 kHz) in **7.7 ms** (svara 3.1.0): a real-time factor around
**0.015** (≈65× faster than playback). Converting a 10-word sentence to phonemes
takes **84 µs**. Absolute latency, not the ratio to Rust, is what a speech pipeline
feels — and there is ample headroom.

### Where Cyrius wins

| Metric | Rust | Cyrius |
|--------|------|--------|
| Binary | dynamic, links libc + 9-crate tree | single static `.cyr` bundle |
| Dependencies | shabdakosh + svara + varna + serde + thiserror + tracing + … (crates.io) | shabdakosh + svara + varna (path/git), sakshi-based, sovereign — no crates.io |
| Build | cargo resolve + compile | `cyrius build`, near-instant; `cyrius distlib` → one bundle |
| Numeric precision | f32 (svara path) | f64 throughout |
| Governance | external registry | fully owned toolchain (AGNOS) |

### Optimization vectors (future versions)

1. **Gate the validation** — make the always-on `validate_*` passes in
   `convert_with` opt-in (a debug/verify build), matching Rust's release
   elision. Biggest single win for the G2P ratio.
2. **Arena / reused buffers** — a per-convert bump arena for the transient
   phoneme/syllable/event vecs would erase most of the allocation tax.
3. **Fewer intermediate vecs** — `assign_stress_syllabic` rebuilds syllable
   phoneme lists; streaming phonemes straight into the events vec avoids copies.
4. **svara render** — the `speak_*` gap closes wherever svara adopts SIMD / f32
   fast paths in its synthesis core (a svara-side change).

### Reproduce

```sh
# Rust baseline (needs the crates.io 2.x deps):
cd rust-old && cargo bench

# Cyrius:
cyrius bench tests/shabda.bcyr
```
