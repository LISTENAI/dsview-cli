---
phase: 16-offline-decode-execution
plan: 03
subsystem: cli
tags: [rust, clap, offline-decode, testing]
requires:
  - phase: 16-02
    provides: core offline decode execution over validated configs plus raw logic artifacts
provides:
  - `decode run` CLI wiring for validated config plus offline raw logic input
  - Minimal text/JSON execution summaries for Phase 16 offline decode runs
  - Fixture-backed CLI regressions for successful, invalid-input, and runtime-failure runs
affects: [phase-17-reporting, decode-cli, offline-decode]
tech-stack:
  added: []
  patterns:
    - config-and-artifact-driven decode execution
    - debug-only fixture runtimes for spawned CLI regression coverage
key-files:
  created: []
  modified:
    - crates/dsview-cli/src/main.rs
    - crates/dsview-cli/src/lib.rs
    - crates/dsview-core/src/lib.rs
    - crates/dsview-cli/tests/decode_cli.rs
    - crates/dsview-core/tests/decode_execute.rs
key-decisions:
  - "Keep `decode run` narrow by accepting a validated config file plus a JSON raw-logic artifact instead of expanding decoder-specific flags."
  - "Use debug-only fixture runtimes in CLI tests so spawned coverage exercises the real core executor without requiring packaged decode runtime assets."
patterns-established:
  - "CLI decode execution validates config and offline input before invoking the runtime and keeps failures on the stable error renderer."
  - "Phase 16 execution summaries stay intentionally coarse: root decoder, stack depth, sample count, and annotation totals."
requirements-completed: [DEC-05, DEC-07]
duration: 10m 15s
completed: 2026-04-21
---

# Phase 16 Plan 03: Offline Decode Execution Summary

**Offline `decode run` now validates config plus raw logic artifacts, executes the core decoder stack from the CLI, and locks binary success/failure behavior with fixture-backed regressions**

## Performance

- **Duration:** 10m 15s
- **Started:** 2026-04-21T11:46:59Z
- **Completed:** 2026-04-21T11:57:14Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- Added `decode run` to the CLI with `--config` and `--input` paths, reusing the validated config flow and the canonical raw logic input contract.
- Added minimal Phase 16 execution rendering in `json` and `text` with root decoder, config version, stack depth, sample count, and annotation totals.
- Locked successful runs, malformed offline input, and runtime execution failures with spawned CLI regressions and matching core fixture coverage.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add the offline decode execution CLI command and minimal result rendering** - `9c5e8b6` (test), `dd2718b` (feat)
2. **Task 2: Lock offline decode run success and failure behavior with CLI regressions** - `debe972` (test)

**Plan metadata:** pending summary commit

_Note: Task 1 followed TDD with a failing response/rendering test commit before the implementation commit._

## Files Created/Modified

- `crates/dsview-cli/src/main.rs` - Added `decode run`, offline input loading, runtime/error classification, and debug-only execution fixtures for spawned CLI tests.
- `crates/dsview-cli/src/lib.rs` - Added `DecodeRunResponse` plus minimal Phase 16 text/JSON rendering helpers and unit coverage.
- `crates/dsview-core/src/lib.rs` - Exposed the decode runtime bridge from `DecodeDiscovery` so the CLI can validate and execute within one discovery session.
- `crates/dsview-cli/tests/decode_cli.rs` - Added success, malformed-input, and runtime-failure CLI regressions for offline decode execution.
- `crates/dsview-core/tests/decode_execute.rs` - Added fixture-style successful execution coverage aligned with the CLI regression seam.

## Decisions Made

- Accepted a single JSON offline input artifact file for raw sample execution to keep `decode run` aligned with the canonical Phase 16 contract instead of inventing new sample flags.
- Kept the user-visible Phase 16 execution response intentionally minimal so Phase 17 can still define the final reporting schema without back-compat pressure.
- Used fixture runtimes only in debug/test builds; release builds still route `decode run` through the real decode runtime discovery path.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Completed decode runtime error-code mapping while compiling the new binary path**
- **Found during:** Task 2 (Lock offline decode run success and failure behavior with CLI regressions)
- **Issue:** The binary compile surfaced unhandled `DecodeRuntimeErrorCode::InputShape` and `DecodeRuntimeErrorCode::SessionState` variants, which blocked the spawned CLI test target.
- **Fix:** Added stable CLI error-code mapping for those runtime variants while preserving the existing machine-readable error path.
- **Files modified:** `crates/dsview-cli/src/main.rs`
- **Verification:** `cargo test -p dsview-cli --test decode_cli -- --nocapture`
- **Committed in:** `debe972`

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** The auto-fix was required to compile and verify the new CLI execution path. No scope creep.

## Issues Encountered

- The plan's blanket `! rg -n "VCD" crates/dsview-cli/src/main.rs crates/dsview-cli/src/lib.rs` acceptance check cannot pass literally because `crates/dsview-cli/src/main.rs` already contains shipped capture-path VCD help and error text. I verified instead that the new `decode run` command path and rendering additions do not introduce VCD-based execution assumptions.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 17 can now build richer decode reporting on top of a real CLI execution path without changing the validated config contract or Phase 16's binary success/failure semantics.
- Offline decode success and failure behavior is regression-locked for representative raw artifact inputs.

## Self-Check

PASSED

- Verified `.planning/phases/16-offline-decode-execution/16-03-SUMMARY.md` exists.
- Verified task commits `9c5e8b6`, `dd2718b`, and `debe972` exist in git history.

---
*Phase: 16-offline-decode-execution*
*Completed: 2026-04-21*
