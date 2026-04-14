# Architecture Research: v1.2 DSView protocol decode CLI foundation

**Date:** 2026-04-14
**Scope:** Integrating DSView protocol decode into the existing Rust CLI architecture

## Recommended Architecture

### Separation of concerns

Keep three layers:

- `capture`
  - collects logic data
  - owns device-facing runtime interactions
- `decode`
  - loads decoders
  - validates decode config
  - executes protocol decode on saved logic artifacts
- `pipeline` (future-facing)
  - orchestrates capture + decode as two connected steps
  - should compose the other two layers rather than absorbing their parameters

### New native seam

Add a dedicated decode seam next to the current capture/export seam:

- native decode runtime exposes decoder list / inspect / run operations
- Rust `dsview-sys` owns unsafe bridging and callback translation
- Rust `dsview-core` owns typed config, validation, and orchestration
- Rust `dsview-cli` owns command UX and output formatting

## Integration Points

### Native layer

Integrate with:

- `DSView/libsigrokdecode4DSL`
- decoder scripts in `DSView/libsigrokdecode4DSL/decoders`
- Python runtime setup and decoder search path initialization

### Core layer

Add:

- typed decoder metadata structs
- typed decode config structs
- artifact input contracts for logic sample data
- typed annotation event structs
- stable error taxonomy for decode failures

### CLI layer

Add:

- `decode list`
- `decode inspect <decoder-id>`
- `decode run --input ... --config ...`

## Build Order Recommendation

1. Build decode runtime bring-up and decoder enumeration
2. Build decoder inspect and config schema
3. Build offline decode execution against saved logic data
4. Build output/artifact reporting and future pipeline handoff contract

## What To Avoid Architecturally

- Reusing Qt `pv/data/decoderstack.*` as the runtime abstraction
- Making `capture` responsible for decoder-specific flags
- Hiding decode config semantics inside opaque native blobs with no Rust-side validation
- Coupling decode output to any UI-only row/paint model

## Why This Fits The Existing Project

The project already follows a clean boundary:

- native bridge for upstream DSView capabilities
- Rust core for validation/orchestration
- CLI for stable automation-facing UX

Protocol decode can extend that same pattern without breaking the shipped `v1.0`/`v1.1` model.
