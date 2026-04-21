---
phase: 17-decode-output-and-workflow-reporting
plan: 02
subsystem: cli
tags: [rust, clap, serde_json, decode-reporting, artifacts]
requires:
  - phase: 17-01
    provides: canonical decode success and failure schema primitives
provides:
  - stable decode-run failure payloads with fine-grained error codes
  - optional `decode run --output` artifact writing for canonical JSON reports
  - regression coverage that locks stdout and file report parity
affects: [17-03, decode-run, automation]
tech-stack:
  added: []
  patterns: [shared decode report serialization, binary failure status with supplementary diagnostics]
key-files:
  created: []
  modified:
    - crates/dsview-cli/src/main.rs
    - crates/dsview-cli/src/lib.rs
    - crates/dsview-cli/tests/decode_cli.rs
    - crates/dsview-core/tests/decode_execute.rs
key-decisions:
  - "Decode-run failures now emit canonical failure payloads with stable error codes instead of falling back to the generic CLI error envelope."
  - "`decode run --output` always writes the canonical JSON report document so stdout's default schema and persisted artifacts stay aligned."
patterns-established:
  - "Decode reporting: use shared JSON serialization helpers for stdout and artifact writes."
  - "Failure semantics: keep `run.status` binary while surfacing partial diagnostics as additive fields."
requirements-completed: [DEC-06, PIPE-01]
duration: 8 min
completed: 2026-04-21
---

# Phase 17 Plan 02: Decode Failure Reporting and Artifact Output Summary

**Stable decode-run failure payloads, partial diagnostics exposure, and canonical `--output` artifact writing for offline decode reports**

## Performance

- **Duration:** 8 min
- **Started:** 2026-04-21T13:07:39Z
- **Completed:** 2026-04-21T13:16:11Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Decode runtime failures now map to stable session-stage codes and emit canonical failure JSON with additive diagnostics.
- `decode run` accepts optional `--output` and writes the same canonical JSON document that stdout emits by default.
- CLI and library regression tests now lock success, failure, partial-diagnostic, and stdout/file schema behavior.

## Task Commits

Each task was committed atomically:

1. **Task 1: Finalize decode failure reporting and partial diagnostics semantics** - `a7d86da` (fix)
2. **Task 2: Add optional decode report artifact writing and lock stdout/file contract behavior** - `fde3b61` (feat)

## Files Created/Modified
- `crates/dsview-cli/src/main.rs` - emits canonical decode failure payloads, adds `--output`, and routes JSON rendering through shared serializers
- `crates/dsview-cli/src/lib.rs` - provides shared decode report serialization and artifact-writing helpers
- `crates/dsview-cli/tests/decode_cli.rs` - verifies success/failure stdout output, artifact writing, and stdout/file schema parity
- `crates/dsview-core/tests/decode_execute.rs` - proves retained partial annotations never promote decode failures into a degraded-success state

## Decisions Made
- Reused the decode report schema on failure by embedding stable error metadata alongside the failure report rather than printing a separate generic CLI error payload.
- Kept `--output` JSON-only so persisted artifacts always match the default machine-readable stdout contract, even when text rendering is requested for humans.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 17 Plan 02 is ready for 17-03 contract-locking work.
- Decode reporting now has a single canonical JSON shape for stdout and optional artifacts, so follow-on work can focus on final polish instead of schema reconciliation.

## Known Stubs

None

## Self-Check: PASSED

- Verified `.planning/phases/17-decode-output-and-workflow-reporting/17-02-SUMMARY.md` exists.
- Verified task commits `a7d86da` and `fde3b61` are present in git history.

---
*Phase: 17-decode-output-and-workflow-reporting*
*Completed: 2026-04-21*
