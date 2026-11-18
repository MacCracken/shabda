# A conversion keeps everything it allocates: shabda has no allocator-taking forms — RESOLVED

**Status:** ✅ **RESOLVED** in 3.1.2 (see *Resolution* at the end).
**Placement:** 3.1.2.
**Where the fix would live:** `shabda_convert` / `shabda_convert_with` (`src/engine.cyr:469`, `:478` at 3.1.1) and
everything they call per conversion: normalizing, the dictionary lookup's temporaries, the rules, syllables, prosody,
the events vector and the events.
**Discovered:** 2026-11-11, building hoosh 2.12.2's `POST /v1/audio/speech` (hoosh's ADR 016).
**Reporter:** hoosh 2.12.2
**Severity:** Medium.
- **Why it matters.** Cyrius's default allocator never frees. A long-running consumer converts on every request, so
  every conversion it ever made stays: about 0.66 KB per input byte. hoosh serves speech in its gateway process, and
  this is now the largest thing a speech request keeps.
- **Who can reach it.** Any server that converts per request. A one-shot tool (convert, render, exit) never notices.

**Affects:** shabda 3.1.1 (tag `3.1.1`, `e2ebfd8`), built with cyrius 6.7.18.

## Summary

One English engine (`shabda_g2p_new(SHABDA_LANG_ENGLISH)`, 6,139,024 bytes, built once), then each input converted
twice. The second conversion is the steady state, and it keeps:

| input | bytes | events | bytes kept by one conversion |
|---|---:|---:|---:|
| `Hello.` | 6 | 6 | 4,608 |
| an 87-byte sentence | 87 | 81 | 60,744 |
| a 378-byte paragraph | 378 | 343 | 250,760 |

A consumer can bound everything else it does per request: hoosh renders speech into per-thread buffers and builds
its own per-request data in an arena it releases when the thread's next request starts. This is the part it cannot
reach. In hoosh 2.12.2 the conversion is 60% of what a speech request keeps; svara's renders are the rest, filed
there.

## Reproduction

[`repros/2026-11-11-conversion-allocates-on-the-default-heap.cyr`](repros/2026-11-11-conversion-allocates-on-the-default-heap.cyr)
prints the table above and exits 1 while a conversion keeps anything. Since 3.1.2 it also measures `shabda_convert_a`,
and its exit code follows that form (see *Resolution*):

```sh
cyrius build docs/development/issues/archived/repros/2026-11-11-conversion-allocates-on-the-default-heap.cyr /tmp/conv
/tmp/conv; echo "exit=$?"
```

## What would fix it

Allocator-taking forms of the conversion, in the stdlib's `_a` convention (`vec_new_a`, `str_builder_new_a`,
`alloc_via`):

```
shabda_convert_a(a, e, text, out_events)
shabda_convert_with_a(a, e, text, options, out_events)
```

- Every allocation the conversion makes comes from `a`, the events vector and the events included, so a consumer
  passing an arena it resets gets all of it back. The engine stays where `shabda_g2p_new` put it.
- The existing forms become `_a` with `default_alloc()`, so nothing changes for current callers.
- It would help to say whether a conversion writes the engine. hoosh assumes it may, and holds a lock around every
  conversion. If the engine is read-only during `convert`, two threads could convert at once.

## References

- hoosh `docs/decisions/016-in-process-speech-through-svara.md` (the consumer, and how it measures:
  `scripts/memory-growth.py --speech`).
- svara `docs/development/issues/2026-11-11-a-server-render-keeps-what-each-phoneme-allocates.md` (the other half).

## Resolution

**Shipped in 3.1.2** as the two forms asked for, `shabda_convert_a(a, e, text, out_events)` and
`shabda_convert_with_a(a, e, text, options, out_events)`. Every allocation a conversion makes comes from `a`: the
events vector, the events, and everything normalizing, the dictionary lookup, the rules, syllables, prosody and
validation build on the way. `shabda_convert` and `shabda_convert_with` are now wrappers over `default_alloc()`,
and they allocate exactly what 3.1.1 did.

Re-run against 3.1.2, the repro prints the same table for `shabda_convert`, which still keeps its bytes by design. For
`shabda_convert_a` into one growable arena, reset before each conversion, it prints:

| input | bytes | events | default-heap bytes kept | arena bytes used |
|---|---:|---:|---:|---:|
| `Hello.` | 6 | 6 | 0 | 4,608 |
| an 87-byte sentence | 87 | 81 | 0 | 60,744 |
| a 378-byte paragraph | 378 | 343 | 0 | 250,760 |

The repro now exits 0. The arena holds exactly what the default heap kept, so nothing was dropped on the way, and a
differential over 10,808 inputs (10,617 dictionary words, sentences under every option, and the five other
languages) gave the same result for each one: the same events, nothing on the default heap, and an arena holding
exactly the default form's bytes.

- **What shabda had to replace.** Four things below shabda allocated on the default heap with no allocator to pass:
  shabdakosh's lookup (a 16-byte `Str` key on every call), svara's `svara_phoneme_event_new`, and the stdlib's
  `str_cstr` and `str_lower_cstr`. shabda now does each one itself through `a`: the lookup through shabdakosh's
  public accessors, and the event through svara's public struct, compared byte for byte with svara's own in a test.
  An allocator-taking lookup in shabdakosh, or `_a` constructors in svara, would let shabda drop its copies.
- **Use an arena that grows.** A conversion has no error for a refused allocation: a refused vector or builder grow
  ends the process, as `vec_push` does. `arena_allocator_growable()` chains chunks from the default heap until it
  reaches the largest request it has served, and from then on a request keeps nothing.
- **Reuse is safe.** An arena hands back the last request's bytes after a reset; the default heap never does. Every
  allocation the conversion makes is written before it is read, and the tests convert on an arena filled with 0xA5
  first.
- **Not covered.** `shabda_convert_ssml`, `shabda_convert_streaming`, `shabda_speak` and `shabda_speak_with` still
  allocate on the default heap. hoosh converts with `shabda_convert`, so none of them is on its path.

**Does a conversion write the engine? No.** A conversion reads the engine and the dictionary and writes nothing
outside its Allocator and the event vector it returns. Through 3.1.1 the first conversion in a process built three
shared tables (the heteronym rules, and varna's inventory and phonotactics for the language). 3.1.2 builds them in
`shabda_g2p_new`, so not even the first conversion writes shared state. Two threads can convert at once on one
engine, each with its own Allocator, because an arena is not thread-safe, and the lock around each conversion can
go. `tests/allocator.tcyr` pins this: four threads, 600 conversions on their own arenas and 160 through the default
heap, each the events one thread alone gives. A scratch soak of 13,600 found no difference. Changing the
dictionary while another thread converts (`shbdk_dict_insert*` through `shabda_g2p_dictionary_mut`) is still the
caller's to serialize.

The default forms cost at most a few percent more, from one indirect call per allocation. The arena form is
3–6% faster than either: 64.47 µs against 68.43 µs for the bench sentence.
