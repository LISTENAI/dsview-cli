---
phase: 16-offline-decode-execution
plan: 01
subsystem: api
tags: [decode, libsigrokdecode, offline-execution, rust-ffi, validation]
requires:
  - phase: 15-decode-config-model-and-validation
    provides: validated decode config ids, channels, options, and linear stack rules
provides:
  - canonical offline decode input types for raw split/cross logic artifacts
  - decode session lifecycle and chunk send wrappers over libsigrokdecode
  - boundary coverage for packet alignment and absolute sample progression
affects: [offline decode executor, decode run cli, annotation reporting]
tech-stack:
  added: []
  patterns: [validated raw artifact contract, linear decode session bridge, absolute sample cursor enforcement]
key-files:
  created: []
  modified:
    - crates/dsview-sys/wrapper.h
    - crates/dsview-sys/bridge_runtime.c
    - crates/dsview-sys/src/lib.rs
    - crates/dsview-sys/tests/boundary.rs
    - crates/dsview-core/src/lib.rs
key-decisions:
  - "Keep the sys-layer execution bridge minimal: create session, set samplerate, build a strict linear stack, send absolute-numbered logic chunks, then end/destroy."
  - "Validate packet boundaries and absolute sample progression in Rust before calling into the C bridge so malformed raw artifacts fail before touching runtime state."
patterns-established:
  - "Offline decode input stays raw-data-first and format-explicit instead of introducing a VCD-centered execution model."
  - "Root decoder channel bindings define the input channel envelope; stacked decoders are instantiated without direct logic-channel binding."
requirements-completed: [DEC-05]
duration: 20min
completed: 2026-04-21
---

# Phase 16 Plan 01: Offline Decode Execution Summary

**Canonical raw offline decode input types plus dsview-sys linear decode session wrappers for chunked absolute-sample execution**

## Performance

- **Duration:** 20 min
- **Started:** 2026-04-21T10:00:08Z
- **Completed:** 2026-04-21T10:19:46Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments
- Added an owned decode-session bridge in `crates/dsview-sys/src/lib.rs` that can create a session, inject samplerate metadata, build a root-plus-stack decoder chain, send raw logic chunks, and end cleanly.
- Wired new C bridge primitives in `crates/dsview-sys/bridge_runtime.c` and `crates/dsview-sys/wrapper.h` to `libsigrokdecode4DSL` session and stacking APIs while enforcing root-only channel binding.
- Defined `OfflineDecodeInput`, `OfflineDecodeDataFormat`, and `OfflineDecodeExecutionRequest` in `crates/dsview-core/src/lib.rs` so later execution code can pair validated configs with raw artifact payloads.
- Added regression coverage for empty payload rejection, misaligned packet lengths, absolute sample progression, and split/cross raw input validation.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add raw decode session lifecycle and chunked send wrappers in dsview-sys** - `6ef227f` (feat)
2. **Task 2: Define the canonical offline decode input contract in core** - `46fda45` (feat)

## Files Created/Modified
- `crates/dsview-sys/wrapper.h` - Declares raw decode execution structs, status codes, and session lifecycle/send APIs.
- `crates/dsview-sys/bridge_runtime.c` - Loads `srd_*` execution symbols and builds the linear stack/send bridge over raw split/cross logic chunks.
- `crates/dsview-sys/src/lib.rs` - Exposes safe Rust session specs, logic format types, packet validation, and absolute sample cursor enforcement.
- `crates/dsview-sys/tests/boundary.rs` - Adds offline decode boundary regressions on malformed payloads and chunk progression.
- `crates/dsview-core/src/lib.rs` - Defines the canonical raw offline decode input contract and basic shape/sample-count helpers.

## Decisions Made

- Keep packet-boundary validation in safe Rust so empty or misaligned packet shapes fail before FFI calls mutate runtime state.
- Treat cross-logic payloads as channel-count-aware 64-sample blocks and split-logic payloads as unitsize-aligned sample rows so both formats share one canonical raw input contract.
- Preserve execution layering: core owns the input contract, while sys owns the runtime session primitives and format-specific chunk marshaling.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- The C shim could not include `libsigrokdecode-internal.h` because it pulls in `Python.h` during build; the bridge now uses the public `libsigrokdecode.h` decoder-instance layout instead, which already exposes the fields needed for chunk marshaling.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- The next execution plan can consume `ValidatedDecodeConfig` plus `OfflineDecodeInput` to drive offline decode runs with packet-aware chunking.
- CLI/reporting phases now have stable raw input and session-bridge seams without needing to invent a VCD-first execution model.

## Self-Check

PASSED

---
*Phase: 16-offline-decode-execution*
*Completed: 2026-04-21*
