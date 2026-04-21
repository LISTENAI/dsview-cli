---
phase: 17-decode-output-and-workflow-reporting
plan: 01
subsystem: reporting
tags: [rust, serde, decode, cli, reporting]
requires:
  - phase: 16-offline-decode-execution
    provides: Offline decode execution results and retained diagnostics from Phase 16
provides:
  - Final core decode success and failure report domain types
  - CLI builders and text renderers for canonical run plus flat events output
affects: [17-02 failure reporting, 17-03 output contract lock, decode CLI]
tech-stack:
  added: []
  patterns: [core-owned report projection, canonical flat event schema]
key-files:
  created: []
  modified:
    [
      crates/dsview-core/src/lib.rs,
      crates/dsview-core/tests/decode_execute.rs,
      crates/dsview-cli/src/lib.rs,
      crates/dsview-cli/src/main.rs,
    ]
key-decisions:
  - Keep report projection in dsview-core so CLI reporting builds directly from Phase 16 execution results.
  - Keep failure diagnostics additive via DecodeFailureReport while preserving binary success and failure status.
patterns-established:
  - OfflineDecodeResult::to_report projects successful runs into the canonical run plus flat events schema.
  - OfflineDecodeRunError::to_failure_report exposes retained diagnostics without introducing partial-success semantics.
requirements-completed: [DEC-06]
duration: 3min
completed: 2026-04-21
---

# Phase 17 Plan 01: Final decode report schema Summary

**Canonical decode reporting now ships as core-owned `run + flat events` types with CLI builders and text rendering aligned to that schema.**

## Performance

- **Duration:** 3 min
- **Started:** 2026-04-21T12:52:07Z
- **Completed:** 2026-04-21T12:55:06Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- Added `DecodeReport`, `DecodeRunSummary`, `DecodeEvent`, `DecodeFailureReport`, and supporting diagnostics in `crates/dsview-core/src/lib.rs`.
- Added projection coverage proving successful runs map to `run + flat events` and failure reports retain partial events without a degraded-success state in `crates/dsview-core/tests/decode_execute.rs`.
- Replaced the temporary CLI decode-run summary path with builders and renderers around the final report schema in `crates/dsview-cli/src/lib.rs` and `crates/dsview-cli/src/main.rs`.

## Task Commits

Each task was committed atomically:

1. **Task 1: Define final success/failure decode reporting types in core** - `abb3e57` (feat)
2. **Task 2: Build CLI response/rendering helpers around the final report schema** - `857bb6c` (feat)

## Files Created/Modified

- `crates/dsview-core/src/lib.rs` - Adds canonical decode success/failure report types and projections from Phase 16 execution results.
- `crates/dsview-core/tests/decode_execute.rs` - Verifies success reports stay flat and failure reports keep retained diagnostics without changing run status.
- `crates/dsview-cli/src/lib.rs` - Builds and renders the final decode report schema for CLI JSON/text output.
- `crates/dsview-cli/src/main.rs` - Routes `decode run` success output through the new report builders and prepares failure report construction for later phases.

## Decisions Made

- Keep `DecodeReport` and `DecodeFailureReport` in core so reporting remains an execution-derived domain projection instead of a CLI-only reinterpretation.
- Keep text output compact and summary-oriented while leaving the JSON `run + events` document authoritative.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- Git writes required escalated permissions because `.git/index.lock` could not be created inside the sandbox; both task commits succeeded after escalation.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- The core and CLI now share a stable decode result schema for follow-on failure-envelope and artifact-output work in Plans 17-02 and 17-03.
- Failure report builders exist, but the CLI still emits the established error envelope on command failure until the later Phase 17 plans wire in the finalized failure contract.

## Self-Check: PASSED

- Verified `.planning/phases/17-decode-output-and-workflow-reporting/17-01-SUMMARY.md` exists.
- Verified task commits `abb3e57` and `857bb6c` exist in git history.

---
*Phase: 17-decode-output-and-workflow-reporting*
*Completed: 2026-04-21*
