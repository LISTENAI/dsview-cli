---
phase: 13-option-aware-capture-reporting
plan: 02
subsystem: capture
tags: [rust, serde, metadata, cli, dslogic-plus]
requires:
  - phase: 13-option-aware-capture-reporting
    provides: Option-aware capture apply readback and validated-request plumbing from Plan 13-01
  - phase: 12-cli-device-option-surface
    provides: Stable DSLogic Plus option IDs and CLI validation contracts
provides:
  - Shared requested/effective device-option facts for capture metadata and CLI success JSON
  - Schema-versioned metadata sidecars with explicit device option reporting for automation
  - Effective-only CLI success text that reuses the core facts block instead of ad hoc rendering
affects: [13-03, capture, metadata, cli, reporting]
tech-stack:
  added: []
  patterns: [shared-capture-reporting-facts, schema-versioned-metadata-contract, effective-only-cli-success-text]
key-files:
  created: [.planning/phases/13-option-aware-capture-reporting/13-02-SUMMARY.md]
  modified: [crates/dsview-core/src/lib.rs, crates/dsview-core/tests/export_artifacts.rs, crates/dsview-cli/src/main.rs]
key-decisions:
  - Build requested/effective device-option facts once in `dsview-core` from validated requests plus device snapshots, then reuse that block for both metadata and CLI JSON.
  - Keep baseline captures on the legacy execution path while still emitting explicit requested/effective facts by mirroring inherited runtime state into both blocks.
patterns-established:
  - "Capture reporting pattern: schema v2 metadata sidecars and CLI success JSON share one serde-ready `CaptureDeviceOptionFacts` model."
  - "CLI success text pattern: print only the effective option values used for the run, in fixed D-08 order, immediately before artifact paths."
requirements-completed: [RUN-05]
duration: 9m
completed: 2026-04-13
---

# Phase 13 Plan 02: Option-aware capture reporting Summary

**Schema-v2 capture metadata and CLI success responses now share one requested/effective DSLogic Plus option facts model, while text mode stays concise and reports only the effective values used for the run.**

## Performance

- **Duration:** 9 min
- **Started:** 2026-04-13T11:10:07Z
- **Completed:** 2026-04-13T11:19:36Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Added serde-ready `CaptureDeviceOptionFacts` and `CaptureDeviceOptionSnapshot` structs in `dsview-core` so metadata and CLI success output cannot drift.
- Bumped capture metadata sidecars to schema version `2` and locked the requested/effective reporting contract with option-aware plus inherited-baseline regressions.
- Updated CLI success rendering to expose the shared facts block in JSON and print only effective values in text immediately before artifact paths.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add a shared requested/effective device-option facts model to capture metadata**
   - `cdf7042` (`test`) RED: failing export-artifact coverage for schema v2 and requested/effective facts
   - `8b205a2` (`feat`) GREEN: shared core reporting structs, schema v2 metadata, and export-path fact construction
2. **Task 2: Reuse the shared facts in CLI JSON and keep text output focused on effective values**
   - `e938de3` (`test`) RED: failing CLI success-response coverage for JSON and effective-only text output
   - `5e90be9` (`feat`) GREEN: CLI success JSON/text reuse of the shared core device-option facts

## Files Created/Modified

- `crates/dsview-core/src/lib.rs` - defines the shared requested/effective reporting structs, builds baseline and option-aware fact snapshots, and writes schema-v2 metadata sidecars.
- `crates/dsview-core/tests/export_artifacts.rs` - verifies schema versioning plus requested/effective metadata behavior for both validated-option and inherited-baseline captures.
- `crates/dsview-cli/src/main.rs` - passes the snapshot/request context needed for export reporting, exposes shared facts in JSON success output, and renders effective-only text summaries.

## Decisions Made

- Core owns requested/effective fact construction so metadata sidecars and CLI JSON always serialize the same machine-readable structure.
- Baseline captures mirror inherited current option state into both `requested` and `effective` blocks so automation never has to special-case the legacy path.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan `13-03` can now focus on spawned CLI integration coverage and validation-document refresh against the stabilized metadata/CLI reporting contract.
- Schema version `2` and the shared facts model make downstream verification straightforward for both automation consumers and human reviewers.

## Self-Check: PASSED

- Found summary file: `.planning/phases/13-option-aware-capture-reporting/13-02-SUMMARY.md`
- Found commit: `cdf7042`
- Found commit: `8b205a2`
- Found commit: `e938de3`
- Found commit: `5e90be9`
