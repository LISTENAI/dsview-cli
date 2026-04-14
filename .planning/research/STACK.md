# Stack Research: v1.2 DSView protocol decode CLI foundation

**Date:** 2026-04-14
**Scope:** Protocol decode support for the existing Rust-based `DSView CLI`

## Existing Baseline

Already shipped and should remain unchanged unless a decode requirement explicitly needs extension:

- Rust workspace split across `dsview-cli`, `dsview-core`, and `dsview-sys`
- Source-backed `DSView/libsigrok4DSL` runtime bridge for device discovery, capture, and export
- `DSView/` treated as a read-only upstream dependency
- JSON-first CLI contracts with text rendering as a secondary surface

## Stack Additions Needed

### 1. Native decode runtime

Recommended addition:

- A separate native runtime target for `DSView/libsigrokdecode4DSL`
- Link against:
  - `glib-2.0`
  - Python 3 development/runtime libraries
  - `common/log`
- Reuse the upstream decoder search path layout under `DSView/libsigrokdecode4DSL/decoders`

Why:

- `libsigrokdecode4DSL` already exposes a stable C-facing session API for initialization, decoder loading, session construction, sample streaming, and callback-based annotation output.
- Reusing this layer avoids reimplementing 149 Python protocol decoders.

### 2. Rust FFI boundary for decode

Recommended addition:

- Extend the native boundary with a decode-focused module or sibling crate that owns:
  - runtime loading / unloading
  - decoder enumeration
  - decoder metadata inspection
  - decode session execution
  - annotation callback marshaling into owned Rust data

Why:

- This keeps all unsafe FFI and callback bridging inside the existing native boundary pattern.
- It prevents CLI/core layers from depending on raw Python or GLib structures.

### 3. Config-driven decoder model

Recommended addition:

- A Rust-owned decode config model that captures:
  - root decoder id
  - stacked decoder ids
  - channel bindings
  - decoder options
  - input samplerate
  - input channel layout / unitsize assumptions

Why:

- Decoder option surfaces vary widely; a config file is safer and more scalable than flattening them into capture flags.
- DSView already serializes decoder stack concepts in JSON, so a config-driven CLI can stay conceptually aligned with upstream behavior.

## Runtime Boundary Recommendation

Recommended shape:

- Keep capture runtime and decode runtime logically separate.
- Share packaging/discovery helpers where helpful, but do not force Python dependencies into the existing capture-only path.

Why:

- Capture today only needs the `libsigrok4DSL` path.
- Decode adds embedded Python and decoder script packaging concerns.
- Separation keeps bring-up/capture workflows lightweight and reduces failure surface for users who do not need decode.

## Packaging Notes

Need to package or discover:

- `libsigrokdecode4DSL`-backed runtime library
- Python 3 runtime compatibility
- decoder scripts directory

Recommended behavior:

- Prefer caller-supplied decode runtime / decoder path overrides for development.
- Support bundled runtime + bundled decoders for shipped CLI builds.

## What Not To Add

Avoid:

- Porting Qt/PulseView `DecoderStack` UI classes into Rust
- Flattening decoder-specific options into `capture` command flags
- Tying decode implementation to GUI snapshot/view models
- Requiring a VCD round-trip when raw logic sample artifacts are already available

## Summary

The lowest-risk stack change is to add a dedicated decode runtime around `libsigrokdecode4DSL`, then wrap it in Rust with a config-driven execution model that stays separate from the current `capture` command surface.
