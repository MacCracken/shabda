# 0001 — shabdakosh is declared `optional` to work around the dep-sidecar resolver

**Status**: Accepted
**Date**: 2026-09-01

## Context

shabdakosh ships `dist/shabdakosh.deps`, the sidecar `cyrius distlib` writes beside a
bundle to name the leaves a consumer must have in scope. Its header calls those
"stdlib leaves", but the generator also emits shabdakosh's non-stdlib folds — `hisab`,
`goonj`, `naad`. On the consuming side `cyrius deps` resolves a sidecar name **only**
against the pinned toolchain's stdlib snapshot, so all three come back as errors:

```
error: dep shabdakosh requires 'hisab' but it is not in the cyrius stdlib
       (looked in ~/.cyrius/versions/6.5.36/lib)
```

The three are svara's own git deps. shabda resolves them transitively through svara's
manifest, and they are sitting in `lib/` at the moment the sidecar is read — including
when `[deps.shabdakosh]` is moved last so they resolve first. The resolver simply does
not look there. Measured, none of these silences it:

- declaring `[deps.hisab]` / `[deps.goonj]` / `[deps.naad]` explicitly, at svara's tags
- ordering those declarations ahead of `[deps.shabdakosh]`
- `requires = [...]` on `[deps.shabdakosh]` — the sidecar is unioned, not overridden
- `requires = []` — same
- `cyrius build --no-deps` — the manifest is still validated

At the previous `6.4.12` pin the three errors were reported but non-fatal, which is why
3.0.1 shipped with them. From **6.5.36** they exit 3, and that takes down `cyrius deps`,
`build`, `test`, `bench` and `distlib` alike — shabda cannot build at all on the pin
this release moves to.

## Decision

Mark `[deps.shabdakosh]` `optional = true`. The resolver then skips the dep — and with
it the sidecar read — while the entry keeps documenting the git URL, the local `path`,
and the `3.0.5` tag.

`lib/shabdakosh.cyr` is committed (all of `lib/` is), so the bundle is present for local
builds and for CI's fresh clone, and `cyrius.lock` still hashes it. This is not a claim
that shabdakosh is optional: it is mandatory, and `src/main.cyr` includes it
unconditionally.

Refreshing it is now manual:

```sh
cp ../shabdakosh/dist/shabdakosh.cyr lib/shabdakosh.cyr
cyrius deps --lock && cyrius deps --verify
```

and the `tag` in `cyrius.cyml` is bumped to match.

svara and varna are unaffected — their sidecars name only real stdlib leaves — and stay
plain required deps.

## Consequences

- **Positive** — shabda builds, tests, benches and bundles on the 6.5.36 pin. Every
  other dep still auto-resolves, and `cyrius.lock` regained the commit pins it had lost
  while the resolver was erroring (hisab 2.11.2, naad 2.2.1, goonj 2.0.4, sakshi 2.4.11).
- **Negative** — `cyrius deps` no longer refreshes `lib/shabdakosh.cyr`. A `tag` bump
  without the copy above leaves the manifest claiming a version `lib/` does not hold;
  `cyrius deps --verify` catches drift against the lock, not against the tag.
- **Neutral** — the fix belongs upstream: `cyrius distlib` should not write non-stdlib
  names into a sidecar documented as stdlib leaves, or `cyrius deps` should satisfy a
  sidecar name from the declared `[deps.*]` and from `lib/`. When either lands, delete
  `optional = true` and this ADR is superseded. dhvani reaches the same place from the
  other direction — it declines to wire sibling bundles as `[deps]` at all and vendors
  them into `lib/` by hand.
- **Neutral** — shabda's own regenerated `dist/shabda.deps` leads with the same three
  names, so a consumer that wires `dist/shabda.cyr` as a `[deps]` entry on 6.5.36 —
  dhvani, vansh — hits this from shabda and needs the same workaround. Noted in
  `README.md` beside the bundle instructions.
