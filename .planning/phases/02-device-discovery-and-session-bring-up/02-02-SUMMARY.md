---
phase: 02-device-discovery-and-session-bring-up
plan: 02
subsystem: core
tags: [core, devices, session, lifecycle, dslogic-plus]
requires: [02-01]
provides:
  - Safe Rust discovery/session orchestration over the dsview-sys bring-up boundary
  - Explicit DSLogic Plus filtering and resource-directory validation
  - Deterministic teardown via opened-device drop and release flow
affects: [core, native-integration, diagnostics]
tech-stack:
  added: [thiserror]
  patterns: [safe wrapper over raw ABI, RAII teardown, explicit resource contract]
key-files:
  created: []
  modified:
    - crates/dsview-core/Cargo.toml
    - crates/dsview-core/src/lib.rs
key-decisions:
  - "Model DSLogic Plus support explicitly in dsview-core rather than treating all DSLogic-family devices as supported."
  - "Validate firmware/resource assets in dsview-core before runtime bring-up so setup failures are actionable and deterministic."
  - "Use RAII-style teardown for opened devices so callers cannot easily forget release."
patterns-established:
  - "Core Filtering Pattern: native discovery output is normalized into a supported-device domain before the CLI consumes it."
  - "Bring-Up Contract Pattern: resource-directory validation is part of connect/open flow rather than an optional post-step."
requirements-completed: [DEV-01, DEV-02, DEV-03]
duration: 45 min
completed: 2026-04-03
---

# Phase 02 Plan 02: Add safe device and session bring-up orchestration Summary

**Built the safe Rust orchestration layer that validates DSLogic Plus resources, filters supported devices explicitly, and guarantees release behavior around bring-up sessions**

## Performance

- **Duration:** 45 min
- **Completed:** 2026-04-03
- **Primary verification:** `cargo test -p dsview-core`

## Accomplishments
- Added `Discovery` as the safe Phase 2 entry point for connecting to a runtime, listing supported devices, and opening a selected handle through the `dsview-sys` boundary.
- Added explicit `SupportedDeviceKind`, `SupportedDevice`, and filtering helpers so `DSLogic Plus` is modeled intentionally rather than inferred ad hoc in the CLI.
- Added `ResourceDirectory` validation for `DSLogicPlus.bin` and `DSLogicPlus-pgl12.bin`, and accepted either `DSLogicPlus.fw` or the compatible fallback `DSLogic.fw` before bring-up, matching upstream firmware/bitstream evidence from `dsl.h` plus local validation.
- Added `OpenedDevice` with deterministic release-on-drop semantics and surfaced native init status / last-error details for diagnostics.

## Files Created/Modified
- `crates/dsview-core/src/lib.rs` - Safe bring-up domain types, resource validation, discovery API, teardown behavior, and tests.
- `crates/dsview-core/Cargo.toml` - Adds `thiserror` for structured bring-up errors.

## Decisions Made
- Kept the safe lifecycle API inside `dsview-core` so the CLI stays focused on argument parsing and output shaping.
- Chose explicit firmware/bitstream validation in core because open-time failures are otherwise ambiguous and difficult to automate against.
- Added `Discovery::connect_auto()` to pair naturally with the new source-built runtime path from `dsview-sys`.

## Deviations from Plan
- The plan mentioned splitting domain concerns into separate files, but the implementation kept the Phase 2 core surface in `crates/dsview-core/src/lib.rs` to avoid premature module sprawl while the bring-up API is still small.

## Issues Encountered
- Upstream DSView uses process-global runtime state, so the safe wrapper must preserve a clear single-owner lifecycle around runtime init/exit and active-device release.
- The upstream `DSLogic PLus` model string uses inconsistent capitalization, so core filtering normalizes support based on the upstream-known model spelling rather than corrected display text from native output.

## User Setup Required
- Provide a valid resource directory containing the required DSLogic Plus bitstream files and either `DSLogicPlus.fw` or the compatible fallback `DSLogic.fw` before attempting list/open bring-up flows.

## Next Phase Readiness
- The CLI can now build on a safe discovery/open/release API instead of raw sys calls.
- Native setup, unsupported selection, and empty-device conditions are already normalized into stable core errors for Phase 2 CLI diagnostics.

---
*Phase: 02-device-discovery-and-session-bring-up*
*Completed: 2026-04-03*
