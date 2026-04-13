---
phase: 12-cli-device-option-surface
plan: 03
subsystem: testing
tags: [rust, clap, integration-tests, device-options, dslogic-plus]
requires:
  - phase: 12-cli-device-option-surface
    provides: Capture-facing token contract plus pre-acquisition validation from Plans 12-01 and 12-02
  - phase: 11-device-option-validation-model
    provides: Stable validation taxonomy and selected-device semantic rules used by spawned CLI assertions
provides:
  - Spawned `capture` CLI regression coverage for help, friendly token acceptance, and stable parse/validation failures
  - Final `devices options` contract coverage that mirrors the same capture flag order and token vocabulary
  - A debug-only test fixture seam that lets integration tests exercise selected-device validation without live hardware
affects: [13-01, capture, devices-options, cli-contracts]
tech-stack:
  added: []
  patterns: [spawned-cli-contract-tests, debug-only-validation-fixture]
key-files:
  created: [.planning/phases/12-cli-device-option-surface/12-03-SUMMARY.md]
  modified: [crates/dsview-cli/src/main.rs, crates/dsview-cli/tests/capture_cli.rs, crates/dsview-cli/tests/device_options_cli.rs]
key-decisions:
  - Keep the new selected-device test seam debug-only and env-gated so release behavior stays unchanged while spawned tests remain deterministic.
  - Reuse the same friendly tokens in `capture_cli` and `device_options_cli` assertions so discovery and execution cannot drift independently.
patterns-established:
  - "Spawned capture contract tests: assert public help, parser acceptance, and stable validation codes from the compiled binary instead of unit-only helpers."
  - "Debug-only validation fixture: enable integration coverage for selected-device semantics through an explicit env gate rather than a new user-facing CLI switch."
requirements-completed: [OPT-02, OPT-03, OPT-04, OPT-05, OPT-06, OPT-07]
duration: 6m
completed: 2026-04-13
---

# Phase 12 Plan 03: CLI device option surface Summary

**The shipped CLI contract is now locked by spawned tests that keep `capture` help, friendly token validation, and `devices options` token guidance aligned for copy-paste device-option workflows.**

## Performance

- **Duration:** 6 min
- **Started:** 2026-04-13T08:43:25Z
- **Completed:** 2026-04-13T08:49:33Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Added spawned `capture_cli` coverage for the new device-option help text, accepted-token parsing, clap failures for malformed threshold input, and stable selected-device validation diagnostics.
- Added a debug-only env-gated fixture path in `crates/dsview-cli/src/main.rs` so integration tests can reach selected-device option resolution and validation without depending on live DSLogic hardware.
- Finalized `device_options_cli` contract checks so text output lists capture flag examples in the real flag order and JSON exposes the exact acceptance tokens used by the `capture` contract tests.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add spawned-process coverage for help text, token acceptance, and parse failures**
   - `bb44c67` (`test`) RED: failing spawned `capture_cli` coverage for help, acceptance, and validation diagnostics
   - `ecc4a83` (`feat`) GREEN: debug-only validation fixture seam plus final spawned capture contract assertions
2. **Task 2: Finalize the inspection contract so `devices options` and `capture` stay aligned**
   - `16165ab` (`test`) RED: failing alignment tests for ordered capture guidance and shared acceptance tokens
   - `fd7b492` (`test`) GREEN: final text/JSON alignment assertions for `devices options`

## Files Created/Modified

- `crates/dsview-cli/src/main.rs` - adds the debug-only env-gated device-option fixture used to drive selected-device validation in spawned CLI tests.
- `crates/dsview-cli/tests/capture_cli.rs` - locks `capture --help`, accepted token parsing, clap numeric failures, and stable semantic validation error codes from the compiled binary.
- `crates/dsview-cli/tests/device_options_cli.rs` - adds final alignment checks for ordered capture flag examples and the exact acceptance tokens shared with `capture_cli`.
- `.planning/phases/12-cli-device-option-surface/12-03-SUMMARY.md` - records Plan 12-03 execution, deviations, and verification evidence.

## Decisions Made

- Used an env-gated debug-only fixture in the binary instead of adding a public testing command or depending on live hardware, because Plan 12-03 requires spawned selected-device validation coverage but the local runtime cannot enumerate hardware reliably in CI-like execution.
- Bound the new `device_options_cli` expectations to the same friendly tokens used in the `capture_cli` acceptance case so Phase 13 can treat `devices options` as the single authoritative discovery surface.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added a debug-only selected-device fixture seam for spawned CLI validation tests**
- **Found during:** Task 1 (Add spawned-process coverage for help text, token acceptance, and parse failures)
- **Issue:** Spawned `capture` integration tests could not reach selected-device inspection or validation because the local DSView runtime failed during device initialization without accessible hardware/libusb support, causing `native_call_failed` before the new parser and validation assertions ran.
- **Fix:** Added an env-gated debug-only fixture path in `crates/dsview-cli/src/main.rs` that supplies a deterministic DSLogic Plus option snapshot and validation capabilities to the existing resolver/validator path for integration tests only.
- **Files modified:** `crates/dsview-cli/src/main.rs`
- **Verification:** `cargo test -p dsview-cli --test capture_cli -- --nocapture` and the full Plan 12-03 verification bundle now reach and assert the intended stable validation codes.
- **Committed in:** `ecc4a83`

---

**Total deviations:** 1 auto-fixed (Rule 3: 1)
**Impact on plan:** The extra seam was necessary to execute the planned spawned CLI coverage in a deterministic local environment. User-facing release behavior and scope stayed unchanged.

## Issues Encountered

- The local DSView runtime initialized far enough to report `native_call_failed` from libusb before selected-device option inspection, so the new spawned capture tests needed an internal debug-only fixture path to exercise the intended contract without live hardware.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 12 now has full regression coverage for the user-facing option vocabulary, so Phase 13 can focus on applying validated options at runtime and reporting effective values without re-litigating token/help contracts.
- The `devices options` and `capture` surfaces now share locked acceptance tokens, reducing drift risk when Phase 13 threads those selections into real runtime application.

## Self-Check: PASSED

- Found summary file: `.planning/phases/12-cli-device-option-surface/12-03-SUMMARY.md`
- Found commit: `bb44c67`
- Found commit: `ecc4a83`
- Found commit: `16165ab`
- Found commit: `fd7b492`
