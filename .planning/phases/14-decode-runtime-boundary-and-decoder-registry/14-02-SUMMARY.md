---
phase: 14-decode-runtime-boundary-and-decoder-registry
plan: 02
subsystem: infra
tags: [rust, ffi, decoder-registry, libsigrokdecode, tdd]
requires:
  - phase: 14-01
    provides: Raw decode runtime loading, inspect snapshots, and error taxonomy in `dsview-sys`
provides:
  - Safe `dsview-sys` decode list/inspect wrappers with owned Rust decoder metadata
  - Typed `dsview-core` decoder descriptors for channels, options, annotations, and stack IO
  - Regression coverage for upstream id preservation and unknown-decoder handling
affects: [14-03, 15, decode-cli, config-validation]
tech-stack:
  added: []
  patterns: [owned ffi snapshot conversion, core metadata normalization]
key-files:
  created: []
  modified:
    - crates/dsview-sys/src/lib.rs
    - crates/dsview-sys/tests/boundary.rs
    - crates/dsview-core/src/lib.rs
    - crates/dsview-core/tests/device_options.rs
key-decisions:
  - "Keep upstream decoder, channel, and option ids canonical across both sys and core layers."
  - "Model required and optional channels plus inputs and outputs explicitly in the core descriptor so later phases can validate stacks without extra native queries."
patterns-established:
  - "Decode discovery in `dsview-sys` returns owned wrapper structs and frees raw bridge allocations immediately after conversion."
  - "Core normalization mirrors sys decoder metadata structurally instead of inventing a second decoder token namespace."
requirements-completed: [DEC-01, DEC-02]
duration: 3min
completed: 2026-04-21
---

# Phase 14 Plan 02: Decoder Registry Summary

**Owned decode discovery wrappers and typed core decoder descriptors now preserve upstream ids, channel structure, options, annotations, and stack IO for later CLI and config phases**

## Performance

- **Duration:** 3 min
- **Started:** 2026-04-21T04:56:21Z
- **Completed:** 2026-04-21T04:59:09Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- Added safe `decode_list` and `decode_inspect` entrypoints on `DecodeRuntimeBridge` with owned decoder wrappers in `crates/dsview-sys/src/lib.rs`
- Preserved canonical upstream decoder ids and labels through sys-layer list/inspect coverage in `crates/dsview-sys/tests/boundary.rs`
- Added typed core decoder descriptors for channels, options, annotations, required/optional channel groups, and stack IO in `crates/dsview-core/src/lib.rs`
- Added focused core tests for upstream id preservation, channel separation, and stack-relevant inputs/outputs in `crates/dsview-core/tests/device_options.rs`

## Task Commits

Each task was committed atomically:

1. **Task 1: Wrap raw decode discovery snapshots in safe sys-layer APIs** - `2441a3f` (test), `6d52486` (feat)
2. **Task 2: Introduce core decoder registry and inspect domain types** - `b87265b` (test), `ace12e4` (feat)

**Plan metadata:** pending summary commit

_Note: Both tasks followed TDD with failing test coverage committed before implementation._

## Files Created/Modified

- `crates/dsview-sys/src/lib.rs` - Adds `DecodeDecoder` plus owned `decode_list` and `decode_inspect` conversions
- `crates/dsview-sys/tests/boundary.rs` - Verifies canonical id preservation, inspect parity, and unknown-decoder errors
- `crates/dsview-core/src/lib.rs` - Defines typed decoder registry descriptors and normalization helpers for downstream use
- `crates/dsview-core/tests/device_options.rs` - Covers decoder id preservation, channel grouping, and stack IO exposure

## Decisions Made

- Reused the same decoder wrapper shape for sys list and inspect calls so canonical upstream ids stay identical across both entrypoints.
- Kept decoder inputs and outputs as explicit descriptor lists in the core model rather than plain strings so later config validation has a stable typed surface.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- `cargo test -p dsview-sys --test boundary -- --nocapture` prints existing upstream decoder import warnings from the vendored `DSView/libsigrokdecode4DSL/decoders` tree, but the targeted runtime metadata checks still passed and unknown-decoder errors stayed differentiated.
- The plan's literal `! rg -n "stable_id|token" crates/dsview-core/src/lib.rs` acceptance command remains false because `crates/dsview-core/src/lib.rs` already contained pre-plan `stable_id` device-option code; the new decoder registry additions did not introduce any new stable-id or token namespace.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- `DecodeRuntimeBridge` now exposes safe owned decoder discovery metadata for CLI rendering work in Plan 14-03.
- `DecoderDescriptor` now preserves required/optional channels and stack IO facts, so Phase 15 can validate decode configs without adding another native metadata path.

## Self-Check

PASSED

- Verified `.planning/phases/14-decode-runtime-boundary-and-decoder-registry/14-02-SUMMARY.md` exists.
- Verified task commits `2441a3f`, `6d52486`, `b87265b`, and `ace12e4` exist in git history.

---
*Phase: 14-decode-runtime-boundary-and-decoder-registry*
*Completed: 2026-04-21*
