# Pitfalls Research: v1.2 DSView protocol decode CLI foundation

**Date:** 2026-04-14
**Scope:** Common implementation mistakes when adding protocol decode to `DSView CLI`

## Major Pitfalls

### 1. Reusing GUI-facing DSView classes as the decode engine

Risk:

- Pulls Qt, view models, and snapshot assumptions into CLI code
- Makes the implementation harder to package and test

Prevention:

- Treat `libsigrokdecode4DSL` as the engine
- Treat `DSView/DSView/pv/...` decode classes only as reference material for execution flow and config semantics

### 2. Letting `capture` absorb decode configuration

Risk:

- Command surface explodes as decoders and options grow
- User workflows become brittle and hard to script

Prevention:

- Keep decode configuration file-based
- Reserve `pipeline` orchestration for future composition instead of flattening flags into `capture`

### 3. Getting the input sample layout wrong

Risk:

- Decoder runtime expects ordered absolute sample ranges and per-channel bit-packed buffers
- A VCD-like abstraction can hide layout bugs until late

Prevention:

- Validate sample layout explicitly at the native boundary
- Start with offline decode against controlled sample artifacts and fixtures

### 4. Underestimating Python runtime packaging

Risk:

- Decode may work in development but fail in packaged environments because decoder scripts or Python runtime paths are missing

Prevention:

- Make runtime path errors explicit and testable
- Keep decoder search-path setup visible in the Rust/native API contract

### 5. Designing the config too far away from DSView concepts

Risk:

- Harder migration from DSView
- Harder mental model for users already familiar with decoder ids, channel bindings, and stacked decoders

Prevention:

- Reuse upstream concepts and ids where practical
- Normalize only where automation value is clear

### 6. Shipping decode execution without stable failure categories

Risk:

- Automation cannot distinguish malformed config from decoder runtime failure or missing prerequisites

Prevention:

- Define stable error groups early:
  - runtime unavailable
  - decoder not found
  - config invalid
  - input invalid
  - decode execution failed
  - artifact write failed

## Recommended Phase Ownership

- Phase 14 should address runtime loading, decoder registry, and Python/decoder-path prerequisites.
- Phase 15 should address config schema drift and inspect/config validation.
- Phase 16 should address sample-layout correctness and stacked decoder execution.
- Phase 17 should address output contracts, artifact reporting, and stable failure taxonomy.
