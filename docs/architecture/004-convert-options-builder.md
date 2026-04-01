# ADR-004: ConvertOptions Builder Pattern

**Status**: Accepted (2026-04-01)

## Context

New prosody features (emphasis, speaking rate, timing profiles) need parameters passed to the G2P pipeline. Adding parameters to `convert()` would break the existing API.

## Decision

Introduce a `ConvertOptions` struct with a builder pattern. The existing `convert()` delegates to `convert_with()` with default options. `ConvertOptions` is `#[non_exhaustive]` so new fields can be added without breaking changes.

- `convert()` — unchanged, uses defaults
- `convert_with(text, &options)` — full control
- `speak()` / `speak_with()` — same pattern for audio output

Builder methods: `with_emphasis()`, `with_speaking_rate()`, `with_timing()`.

## Consequences

- Existing callers unaffected
- New features opt-in via builder
- `#[non_exhaustive]` prevents external struct literal construction — builder is the only way
- Same-crate code (SSML renderer) can still use struct literals
