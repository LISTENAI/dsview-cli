---
phase: 04-acquisition-execution
plan: 03
subsystem: test
tags: [tests, integration, sys, cli, verification, acquisition]
requires:
  - phase: 04-02
    provides: failure classification, cleanup precedence, and bounded wait behavior
provides:
  - Hardware-free acquisition lifecycle coverage across core, sys, and cli layers
  - Verifier-ready Phase 4 manual UAT and coverage matrix updates
  - Narrow boundary tests that preserve stable diagnostic shape without requiring DSLogic hardware
affects: [phase-04, tests, verification, cli-diagnostics]
tech-stack:
  added: []
  patterns: [synthetic acquisition fixtures, boundary smoke assertions, preflight-first hardware UAT]
key-files:
  created:
    - crates/dsview-sys/tests/boundary.rs
    - crates/dsview-cli/tests/capture_cli.rs
    - .planning/phases/04-acquisition-execution/04-03-SUMMARY.md
  modified:
    - crates/dsview-core/tests/acquisition.rs
    - crates/dsview-sys/src/lib.rs
    - crates/dsview-cli/src/main.rs
    - .planning/phases/04-acquisition-execution/04-RESEARCH.md
key-decisions:
  - "Keep default acquisition verification hardware-free by asserting synthetic result shapes at each layer."
  - "Add a narrow sys boundary test without touching DSView upstream code or requiring a live runtime session."
  - "Make manual UAT explicitly preflight-first and call out LIBUSB_ERROR_ACCESS as a blocker, not an implementation failure."
patterns-established:
  - "Verification Pattern: core/unit, sys/boundary, and cli/diagnostic tests all encode the same Phase 4 failure vocabulary."
  - "UAT Pattern: hardware validation begins with preflight and only claims acquisition lifecycle truth, not Phase 5 export correctness."
requirements-completed: [RUN-01, RUN-02, RUN-03]
duration: TO_BE_FILLED
completed: 2026-04-07
---

# Phase 04 Plan 03: Add smoke and integration validation for the acquisition lifecycle Summary

**Completed the Phase 4 validation layer by adding hardware-free acquisition coverage across core, sys, and CLI seams, then documenting a verifier-ready preflight-first manual UAT path for DSLogic Plus.**

## Performance

- **Duration:** TO_BE_FILLED
- **Started:** TO_BE_FILLED
- **Completed:** 2026-04-07
- **Tasks:** 4
- **Files modified:** 7

## Accomplishments
- Expanded `crates/dsview-core/tests/acquisition.rs` to cover the Phase 4 case matrix shape for `clean_success`, `preflight_blocked`, `start_failure`, `run_failure`, `detach`, `incomplete`, `timeout`, and `cleanup_failure` without requiring hardware.
- Added a narrow sys boundary test in `crates/dsview-sys/tests/boundary.rs` and one extra raw-summary unit assertion in `crates/dsview-sys/src/lib.rs` so boundary coverage includes header/runtime-path shape and detach summary translation.
- Added CLI-level tests in `crates/dsview-cli/tests/capture_cli.rs` and extended `crates/dsview-cli/src/main.rs` tests so stable non-zero diagnostic shape is checked for preflight-not-ready, run failure, detach, timeout, incomplete, start failure, and cleanup failure.
- Updated `.planning/phases/04-acquisition-execution/04-RESEARCH.md` with a verifier-ready coverage matrix and manual DSLogic Plus UAT steps that begin with preflight and explicitly call out `LIBUSB_ERROR_ACCESS` risk.

## Files Created/Modified
- `crates/dsview-core/tests/acquisition.rs` - Adds hardware-free acquisition fixture coverage for all Phase 4 lifecycle classes and cleanup outcomes.
- `crates/dsview-sys/src/lib.rs` - Adds one more raw acquisition summary translation test for detach boundary behavior.
- `crates/dsview-sys/tests/boundary.rs` - Adds narrow smoke checks for upstream header presence and source-runtime path shape.
- `crates/dsview-cli/src/main.rs` - Exposes CLI error classification to test modules and adds more stable diagnostic-shape assertions.
- `crates/dsview-cli/tests/capture_cli.rs` - Adds CLI-layer failure-shape tests, including preflight-not-ready behavior.
- `.planning/phases/04-acquisition-execution/04-RESEARCH.md` - Refreshes the 04-03 coverage matrix and preflight-first manual UAT notes.
- `.planning/phases/04-acquisition-execution/04-03-SUMMARY.md` - Records plan 04-03 completion context.

## Decisions Made
- The default verifier path remains synthetic for automated coverage; hardware is reserved for explicit manual UAT.
- CLI integration-style tests reuse the checked-in command classification logic rather than trying to stand up a fake DSView runtime.
- Phase 4 validation remains scoped to acquisition lifecycle truth and deliberately avoids claiming export correctness.

## Issues Encountered
- The repository does not expose a runnable `/gsd:execute-phase` entrypoint in this worktree, so execution proceeded against the checked-in 04-03 plan artifact as the GSD source of truth.
- The CLI test surface needed small visibility changes so integration-style tests could assert stable diagnostic shape without duplicating classification logic.

## User Setup Required
- None for automated tests.
- Manual hardware UAT still requires the existing source-runtime prerequisites, the DSView resource directory, and working USB permissions for the connected `DSLogic Plus`.

## Next Phase Readiness
- Phase 4 now has a verifier-readable automated coverage story for each promised failure class plus a preflight-first manual UAT path for hardware confirmation.
- Remaining truth outside automated scope is still hardware-backed rerunability after a representative failure on the current machine.

---
*Phase: 04-acquisition-execution*
*Completed: 2026-04-07*
