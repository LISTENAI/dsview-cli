---
phase: 17-decode-output-and-workflow-reporting
plan: 03
subsystem: testing
tags: [rust, clap, json, decode, reporting]
requires:
  - phase: 17-01
    provides: Flat decode report and binary failure-report projection
  - phase: 17-02
    provides: Stable decode failure codes and stdout/output report alignment
provides:
  - Final end-to-end CLI regression coverage for decode success, failure, and text reporting
  - Core regression coverage for binary failure status with partial diagnostics
  - Canonical failure-envelope rendering for validated decode input failures
affects: [decode-run, automation-contracts, workflow-reporting]
tech-stack:
  added: []
  patterns:
    - Canonical decode failures use a run-plus-error envelope once validated config context exists
    - Text decode output remains summary-focused while JSON stays authoritative
key-files:
  created: [.planning/phases/17-decode-output-and-workflow-reporting/17-03-SUMMARY.md]
  modified:
    - crates/dsview-cli/tests/decode_cli.rs
    - crates/dsview-core/tests/decode_execute.rs
    - crates/dsview-cli/src/lib.rs
    - crates/dsview-cli/src/main.rs
key-decisions:
  - "Wrap validated decode input failures in the canonical failure envelope so automation always sees run metadata plus stable error fields."
  - "Keep the reusable contract-only failure report helper in dsview-cli so summary-focused text rendering and JSON serialization share one failure shape."
patterns-established:
  - "Decode CLI contract tests should assert both field presence and field absence for success and failure envelopes."
  - "Failure diagnostics remain supplementary: no public partial_success state, no events list on failure."
requirements-completed: [DEC-06, PIPE-01]
duration: 8 min
completed: 2026-04-21
---

# Phase 17 Plan 03: Decode reporting contract lock Summary

**Final decode reporting coverage now locks success and failure JSON envelopes, summary-only text output, and binary failure diagnostics behavior end to end.**

## Performance

- **Duration:** 8 min
- **Started:** 2026-04-21T13:55:00Z
- **Completed:** 2026-04-21T14:03:06Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Added final CLI regressions for success JSON, failure JSON, and summary-focused text decode output.
- Added core regression coverage proving partial diagnostics never weaken the binary failure status.
- Finalized the decode CLI failure contract so validated input failures now return the same canonical run-plus-error envelope as runtime decode failures.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add final end-to-end decode reporting contract regressions** - `46299b9` (test)
2. **Task 2: Perform final report-shape cleanup without changing semantics** - `e5220a8` (fix)

_Note: Task 1 followed TDD RED with the failing regressions committed before the cleanup landed in Task 2._

## Files Created/Modified
- `crates/dsview-cli/tests/decode_cli.rs` - Locks the final decode success/failure JSON contract and concise text output behavior.
- `crates/dsview-core/tests/decode_execute.rs` - Locks binary failure status with partial diagnostics and no `partial_success` drift.
- `crates/dsview-cli/src/lib.rs` - Adds a reusable contract failure-report helper and unit coverage for omitted supplementary fields.
- `crates/dsview-cli/src/main.rs` - Routes validated decode input failures through the canonical failure envelope without broadening decode semantics.

## Decisions Made
- Validated decode input failures now render as `run + error` using the already-known decoder id and stack depth from the validated config.
- Partial diagnostics stay supplementary and only appear when they exist; failure reports never expose `events` or invent a degraded-success state.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 17's final decode reporting contract is locked with regression coverage for success, failure, and text rendering paths.
- Ready for orchestrator-managed phase wrap-up and any downstream verification that depends on a stable decode workflow result.

## Self-Check: PASSED

- Found summary file: `.planning/phases/17-decode-output-and-workflow-reporting/17-03-SUMMARY.md`
- Found task commit: `46299b9`
- Found task commit: `e5220a8`

---
*Phase: 17-decode-output-and-workflow-reporting*
*Completed: 2026-04-21*
