---
phase: 10-device-option-bridge-and-discovery
plan: 02
subsystem: api
tags: [rust, serde, dslogic-plus, discovery, normalization]
requires:
  - phase: 10-01
    provides: owned DSLogic Plus option snapshots from dsview-sys
provides:
  - Stable Rust device-option snapshot model for selected DSLogic Plus devices
  - Discovery API that opens a selected device and returns normalized option data
  - Regression coverage for code-backed IDs, grouped channel modes, and threshold shape
affects: [10-03, dsview-core, dsview-cli]
tech-stack:
  added: []
  patterns:
    - code-backed stable ids
    - per-operation-mode channel grouping
    - discovery-model separation from capture validation
key-files:
  created:
    - crates/dsview-core/src/device_options.rs
    - crates/dsview-core/tests/device_options.rs
  modified:
    - crates/dsview-core/src/lib.rs
key-decisions:
  - "Normalize automation IDs from raw native codes with fixed prefixes instead of relying on labels."
  - "Keep threshold as a fixed voltage-range capability rooted at `threshold:vth-range` and carry legacy threshold data only as raw metadata."
  - "Expose a dedicated discovery snapshot in dsview-core rather than extending Phase 9 capture capability types."
patterns-established:
  - "Sort enum options by native code and channel-mode groups by operation-mode code before exposing them to CLI consumers."
  - "Keep current values explicit alongside supported values so later phases can validate without changing the discovery contract."
requirements-completed: [OPT-01]
duration: 6 min
completed: 2026-04-10
---

# Phase 10 Plan 02: Device option core normalization summary

**Stable code-backed DSLogic Plus discovery snapshots in `dsview-core` with truthful threshold facts and grouped channel-mode output**

## Performance

- **Duration:** 6 min
- **Started:** 2026-04-10T10:24:04Z
- **Completed:** 2026-04-10T10:29:54Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- Added `crates/dsview-core/src/device_options.rs` with a serde-ready normalized device-option snapshot model for selected devices.
- Exported `Discovery::inspect_device_options()` so later CLI work can consume stable current values, grouped channel modes, and threshold facts directly from `dsview-core`.
- Locked the public contract with regression coverage for native-code-backed IDs, deterministic ordering, and the `threshold:vth-range` voltage-range schema.

## Task Commits

Each task was committed atomically:

1. **Task 1: Build the normalized core model and selected-device discovery API**
   - `8e411e8` (`test`) - failing TDD coverage for stable IDs and grouped channel modes
   - `e3a8116` (`feat`) - normalized snapshot model plus `inspect_device_options()`
2. **Task 2: Finalize threshold, ordering, and compatibility boundaries in the core snapshot**
   - `ddcc8a0` (`fix`) - threshold-contract and ordering regression coverage with library baseline verification

## Files Created/Modified
- `crates/dsview-core/src/device_options.rs` - normalized discovery structs and conversion helpers from `dsview-sys`
- `crates/dsview-core/src/lib.rs` - public exports plus `Discovery::inspect_device_options()`
- `crates/dsview-core/tests/device_options.rs` - regression coverage for stable IDs, grouped channel modes, and threshold semantics

## Decisions Made
- Used raw native codes as the source of truth for stable option IDs so CLI automation is decoupled from human labels.
- Kept threshold discovery separate from enum-style options and normalized it to a fixed voltage-range contract for later CLI rendering.
- Preserved the existing Phase 9 capture/export surface by adding a separate discovery model instead of mutating `CaptureCapabilities`.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- `dsview-cli` can now render a selected device's supported option surface without understanding native snapshot details.
- Phase 11 can build validation rules on top of explicit current values, raw codes, and per-operation-mode channel groups without changing the discovery schema.
- The shipped capture/export API remains intact and green after the core discovery expansion.

## Self-Check: PASSED

- Found `.planning/phases/10-device-option-bridge-and-discovery/10-02-SUMMARY.md`
- Found task commits `8e411e8`, `e3a8116`, and `ddcc8a0`
