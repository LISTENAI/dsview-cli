---
phase: 11-device-option-validation-model
plan: 02
subsystem: validation
tags: [rust, validation, cli, error-codes, dslogic-plus]
requires:
  - phase: 11-device-option-validation-model
    provides: Phase 11 request/capability contracts and selected-device validation capability loading
provides:
  - Pure Rust validation of DSLogic Plus option combinations against selected-device capabilities
  - Stable validation code taxonomy for known pre-acquisition failures
  - CLI mapping that keeps current capture-config validation out of the generic runtime_error path
affects: [11-03 regression coverage, phase-12 cli-device-option-surface, phase-13 option application]
tech-stack:
  added: []
  patterns: [pure request-capability validation, shared sample-limit alignment math, typed validation-to-cli code mapping]
key-files:
  created:
    - .planning/phases/11-device-option-validation-model/11-02-SUMMARY.md
  modified:
    - crates/dsview-core/src/device_option_validation.rs
    - crates/dsview-core/src/capture_config.rs
    - crates/dsview-core/src/lib.rs
    - crates/dsview-core/tests/device_option_validation.rs
    - crates/dsview-cli/src/main.rs
key-decisions:
  - "Use Phase 10 stable IDs as the allowlist source, then carry native codes in the validated request for later apply-time phases."
  - "Reuse the existing capture sample-limit alignment helpers instead of duplicating the arithmetic in the Phase 11 validator."
  - "Map current capture-config validation failures to the Phase 11 taxonomy while preserving their existing human-readable messages."
patterns-established:
  - "DeviceOptionValidationCapabilities::validate_request is a pure Rust gate before any runtime apply or acquisition work."
  - "CLI validation responses use typed validation codes directly and reserve runtime_error for real runtime and bridge failures."
requirements-completed: [VAL-01, VAL-02]
duration: 13 min
completed: 2026-04-13
---

# Phase 11 Plan 02: Device Option Validation Model Summary

**Pure DSLogic Plus request validation with stable pre-acquisition error codes and CLI mapping that no longer collapses known validation failures into `runtime_error`**

## Performance

- **Duration:** 13 min
- **Started:** 2026-04-13T04:47:43Z
- **Completed:** 2026-04-13T05:00:59Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- Implemented `DeviceOptionValidationCapabilities::validate_request(...)` so unified Phase 11 requests validate entirely in pure Rust against selected-device capabilities.
- Reused the shared sample-limit alignment and capacity math from `capture_config.rs` so validation stays aligned with the shipped depth rules.
- Added `Discovery::validate_device_option_request(...)` as the selected-device entrypoint for future CLI option wiring.
- Updated the CLI to classify known capture-config validation failures through the Phase 11 taxonomy instead of sending them through the generic `runtime_error` path.

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement the pure DSLogic Plus option validator and stable error enum** - `5ac8648` (test, RED) and `5f2df7f` (feat, GREEN)
2. **Task 2: Map validation failures to stable CLI codes and keep the `v1.0` baseline intact** - `30d34dd` (test, RED) and `4a2e4eb` (feat, GREEN)

## Files Created/Modified

- `crates/dsview-core/tests/device_option_validation.rs` - Adds the first requirement-specific RED/GREEN validator cases for success, samplerate ceilings, channel ceilings, and aligned capacity overflow.
- `crates/dsview-core/src/device_option_validation.rs` - Implements pure request validation, exact Phase 11 code strings, threshold checks, and stop/filter/channel compatibility resolution.
- `crates/dsview-core/src/capture_config.rs` - Exposes shared sample-limit alignment helpers for reuse by the richer validator.
- `crates/dsview-core/src/lib.rs` - Adds `validate_device_option_request(...)` as the selected-device validation entrypoint.
- `crates/dsview-cli/src/main.rs` - Expands stable validation-code assertions and classifies capture-config validation failures through the typed validation taxonomy.

## Decisions Made

- Kept validation pure by resolving operation mode, channel mode, stop option, filter, and threshold rules entirely against the loaded capability snapshot.
- Preserved current capture-config error messages for the existing capture path while switching their machine-readable codes to the Phase 11 validation taxonomy.
- Used epsilon-tolerant threshold step checking and data-driven stop-option compatibility per the phase research decisions.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- `11-03` can now lock the validator and CLI taxonomy with broader DSView-rule regression coverage instead of building the core behavior from scratch.
- Phase 12 can call `Discovery::validate_device_option_request(...)` directly once CLI option flags are introduced.
- The current capture path already reports known validation failures with stable machine-readable codes, reducing risk when richer option validation is wired in later phases.

## Self-Check: PASSED

- Found summary file: `.planning/phases/11-device-option-validation-model/11-02-SUMMARY.md`
- Found commit: `5ac8648`
- Found commit: `5f2df7f`
- Found commit: `30d34dd`
- Found commit: `4a2e4eb`
