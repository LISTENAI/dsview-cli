---
phase: 11-device-option-validation-model
plan: 01
subsystem: validation
tags: [rust, dslogic-plus, dsview-sys, ffi, validation]
requires:
  - phase: 10-device-option-bridge-and-discovery
    provides: Stable device-option discovery IDs and restore-safe option probing
provides:
  - Internal Phase 11 validation request, capability, and error contracts
  - Restore-safe native capability probing for operation-mode and channel-mode validation facts
  - Selected-device core loading for validation capabilities without changing Phase 10 discovery output
affects: [11-02 validator, 11-03 regression tests, phase-12 cli-device-option-surface]
tech-stack:
  added: []
  patterns: [owned native validation snapshots, mode-scoped samplerate probing, stable validation error codes]
key-files:
  created:
    - crates/dsview-core/src/device_option_validation.rs
    - crates/dsview-core/tests/device_option_validation.rs
    - .planning/phases/11-device-option-validation-model/11-01-SUMMARY.md
  modified:
    - crates/dsview-core/src/lib.rs
    - crates/dsview-cli/src/main.rs
    - crates/dsview-sys/wrapper.h
    - crates/dsview-sys/bridge_runtime.c
    - crates/dsview-sys/src/lib.rs
    - crates/dsview-sys/tests/device_options.rs
key-decisions:
  - "Keep Phase 11 validation capabilities additive and internal rather than extending the shipped Phase 10 discovery schema."
  - "Probe per-mode validation facts entirely inside dsview-sys and restore original operation/channel modes on all exit paths."
  - "Advertise stop-option compatibility only for Buffer Mode in the internal validation model."
patterns-established:
  - "Validation contracts reuse Phase 10 stable IDs while carrying native codes for later apply-time work."
  - "Mode-aware validation probing snapshots owned data across the sys boundary so safe Rust never depends on mutable native state."
requirements-completed: [VAL-01, VAL-02]
duration: 45 min
completed: 2026-04-13
---

# Phase 11 Plan 01: Device Option Validation Model Summary

**Internal DSLogic Plus validation contracts with restore-safe mode probing and selected-device capability loading for later validator work**

## Performance

- **Duration:** 45 min
- **Started:** 2026-04-13T04:02:22Z
- **Completed:** 2026-04-13T04:47:43Z
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments

- Added the Phase 11-only request, capability, validated-request, and typed-error contracts in `crates/dsview-core/src/device_option_validation.rs`.
- Reserved the CLI validation-code mapping surface and dedicated core validation test target before validator behavior work begins.
- Added a new `dsview-sys` validation-capability snapshot path that probes operation-mode and channel-mode samplerates while restoring device state on every exit path.
- Exposed selected-device validation capability loading through `Discovery::load_device_option_validation_capabilities(...)` without changing the Phase 10 public discovery schema.

## Task Commits

Each task was committed atomically:

1. **Task 1: Create the internal validation contracts and predecessor test scaffolding** - `571804e` (feat)
2. **Task 2: Load selected-device validation capabilities through `dsview-sys` without acquisition side effects** - `7a09b6e` (feat)

## Files Created/Modified

- `crates/dsview-core/src/device_option_validation.rs` - Phase 11 validation request/capability/error contracts plus native-to-stable capability normalization.
- `crates/dsview-core/tests/device_option_validation.rs` - Dedicated predecessor test target for validation contracts and stable error-code scaffolding.
- `crates/dsview-core/src/lib.rs` - Exports validation model types and adds `load_device_option_validation_capabilities(...)`.
- `crates/dsview-cli/src/main.rs` - Adds `classify_validation_error(...)` and stable CLI assertions for validation error codes.
- `crates/dsview-sys/wrapper.h` - Declares fixed-size native validation-capability snapshot structs and bridge entrypoint.
- `crates/dsview-sys/bridge_runtime.c` - Implements restore-safe validation probing across operation and channel modes.
- `crates/dsview-sys/src/lib.rs` - Adds Rust decoding and safe wrapper for validation-capability snapshots.
- `crates/dsview-sys/tests/device_options.rs` - Covers mode-scoped samplerate probing and restore-on-failure behavior for the new snapshot path.

## Decisions Made

- Kept validation capabilities internal to Phase 11 by adding a parallel loader path instead of mutating `DeviceOptionsSnapshot`.
- Reused Phase 10 stable IDs in the validation contracts while also preserving native codes for future apply-time phases.
- Applied the phase research rule that explicit stop-option compatibility is Buffer Mode only for this milestone.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- `11-02` can now implement the pure validator against a dedicated request/capability contract instead of inventing the model late.
- The CLI already has a stable validation error-code mapping surface, so later phases can wire real validation failures without changing the taxonomy.
- The sys boundary now returns owned mode-aware validation facts without starting acquisition or altering the Phase 10 discovery response shape.

## Self-Check: PASSED

- Found summary file: `.planning/phases/11-device-option-validation-model/11-01-SUMMARY.md`
- Found commit: `571804e`
- Found commit: `7a09b6e`
