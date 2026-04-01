# ADR-005: Hand-Rolled SSML Subset Parser

**Status**: Accepted (2026-04-01)

## Context

SSML (Speech Synthesis Markup Language) support is needed for TTS control. Full SSML 1.1 is complex. The crate must remain `no_std + alloc` compatible, ruling out most XML parsing crates.

## Decision

Implement a minimal hand-rolled parser in `ssml.rs` supporting only:

- `<speak>` — root wrapper (optional)
- `<break time="500ms"/>` — insert pause (ms or seconds, or strength keywords)
- `<emphasis level="strong|moderate|reduced">` — stress control
- `<prosody rate="slow|medium|fast|x-fast">` — speaking rate

Unknown tags are ignored (text content preserved). The parser outputs `Vec<SsmlNode>` which `convert_ssml()` walks recursively, applying `ConvertOptions` overrides per element.

## Alternatives Considered

- **xml-rs / quick-xml**: Require `std`, too heavy for subset needs
- **Auto-detect SSML in convert()**: Performance penalty on every call, surprising behavior

## Consequences

- No external XML dependency
- `no_std` compatible
- Limited to the supported subset — no `<say-as>`, `<sub>`, `<voice>`, etc.
- Nested elements work (e.g., emphasis inside prosody)
- Easy to extend with new tags
