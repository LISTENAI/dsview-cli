---
phase: 10-device-option-bridge-and-discovery
plan: 01
subsystem: api
tags: [rust, ffi, glib, dsview, libsigrok4dsl]
requires: []
provides:
  - Fixed-size C ABI for owned DSLogic Plus device-option discovery snapshots
  - Safe Rust `RuntimeBridge::device_options()` wrapper for current and supported option facts
  - Sys-level regression coverage for restore-on-exit discovery and truthful threshold reporting
affects: [10-02, 10-03, dsview-core, dsview-cli]
tech-stack:
  added: []
  patterns:
    - owned ffi snapshots
    - mode-scoped channel discovery
    - restore-on-exit native enumeration
key-files:
  created:
    - crates/dsview-sys/tests/device_options.rs
  modified:
    - crates/dsview-sys/wrapper.h
    - crates/dsview-sys/bridge_runtime.c
    - crates/dsview-sys/src/lib.rs
key-decisions:
  - "Keep option discovery as a fixed-size owned C snapshot so Rust never touches live GVariant-backed pointers."
  - "Enumerate channel modes per operation mode and derive max enabled channels from DSView state, not label parsing."
  - "Expose threshold discovery as a VTH voltage-range fact with optional legacy threshold metadata."
patterns-established:
  - "Bridge current values with ds_get_actived_device_config() and supported values with ds_get_actived_device_config_list()."
  - "Restore original operation mode and channel mode before returning from any multi-mode discovery path."
requirements-completed: [OPT-01]
duration: 8 min
completed: 2026-04-10
---

# Phase 10 Plan 01: Device option sys bridge summary

**Owned DSLogic Plus option snapshots with mode-scoped channel discovery and truthful VTH threshold facts inside `dsview-sys`**

## Performance

- **Duration:** 8 min
- **Started:** 2026-04-10T10:10:29Z
- **Completed:** 2026-04-10T10:18:45Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Added `dsview_bridge_ds_get_device_options` and fixed-size ABI structs so the native bridge returns owned device-option snapshots.
- Added `RuntimeBridge::device_options()` plus safe Rust decoding for operation modes, stop options, filters, threshold facts, and grouped channel modes.
- Hardened channel-mode discovery to restore original operation/channel modes on success and failure while treating `SR_ERR_NA` option data as optional.
- Added sys integration coverage for owned snapshot semantics, grouped channel modes, restore-on-exit behavior, and truthful threshold reporting.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add the owned sys discovery snapshot for DSLogic Plus options**
   - `1c6553c` (`test`) - failing TDD coverage for option snapshots
   - `9c5ae48` (`feat`) - native ABI and safe Rust snapshot wrapper
2. **Task 2: Harden mode-scoped channel discovery and threshold truthfulness**
   - `0488fd4` (`fix`) - restore-path hardening and `SR_ERR_NA` regression coverage

## Files Created/Modified
- `crates/dsview-sys/wrapper.h` - fixed-size device-option snapshot structs and the new discovery entrypoint declaration
- `crates/dsview-sys/bridge_runtime.c` - owned list copying, mode-scoped channel enumeration, restore-on-exit logic, and sys test mocks
- `crates/dsview-sys/src/lib.rs` - safe Rust snapshot models plus `RuntimeBridge::device_options()`
- `crates/dsview-sys/tests/device_options.rs` - integration coverage for current/supported option reads, grouped channel modes, restore behavior, and threshold truthfulness

## Decisions Made
- Kept all list walking, pointer handling, and `GVariant` ownership in `bridge_runtime.c` so later core/CLI work can stay fully safe Rust.
- Preserved the Phase 9 `capture_capabilities()` surface and added a parallel discovery API instead of mutating the existing capability type.
- Modeled threshold as the authoritative `SR_CONF_VTH` range contract with legacy `SR_CONF_THRESHOLD` metadata carried only as optional supplementary data.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- `dsview-core` can now normalize a safe, lossless sys snapshot without expanding the unsafe boundary.
- `dsview-cli` can build discovery output on top of grouped channel modes and truthful threshold facts.
- Existing `capture_capabilities()` and Phase 9 boundary coverage remain green, so the shipped `v1.0` baseline stays intact for the next plans.

## Self-Check: PASSED

- Found `.planning/phases/10-device-option-bridge-and-discovery/10-01-SUMMARY.md`
- Found task commits `1c6553c`, `9c5ae48`, and `0488fd4`
