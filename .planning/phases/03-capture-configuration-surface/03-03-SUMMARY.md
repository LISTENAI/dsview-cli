---
phase: 03-capture-configuration-surface
plan: 03
subsystem: test
tags: [tests, validation, capture-config, dslogic-plus, sys, core]
requires: [03-02]
provides:
  - Automated coverage for valid and invalid DSLogic Plus configuration combinations
  - Regression checks for enabled-channel-dependent depth rules and sample-rate validation
  - Verified no-hardware config validation baseline plus source-runtime discovery regression check
affects: [testing, core, native-integration]
tech-stack:
  added: [table-driven config validation tests]
  patterns: [config validation regression suite, no-hardware verification, source-runtime smoke reuse]
key-files:
  created: []
  modified:
    - crates/dsview-core/src/capture_config.rs
    - crates/dsview-core/src/lib.rs
    - crates/dsview-sys/src/lib.rs
key-decisions:
  - "Use no-hardware table-driven validation to cover most Phase 3 correctness while keeping manual hardware checks focused on real active-device behavior."
  - "Keep the source-runtime list smoke as a regression check for Phase 2/3 integration while config-only APIs evolve."
patterns-established:
  - "Regression Pattern: config validation cases assert error categories and effective normalized values, not only free-form messages."
  - "Integration Smoke Pattern: Phase 3 keeps re-running source-runtime device discovery to catch regressions in the active-device bridge environment."
requirements-completed: [CAP-01, CAP-02, CAP-03, CAP-04]
duration: 30 min
completed: 2026-04-03
---

# Phase 03 Plan 03: Add tests for valid, invalid, and device-specific capture configuration cases Summary

**Added focused Phase 3 validation coverage so DSLogic Plus configuration rules are exercised without hardware and the source-runtime integration path keeps a live regression check**

## Performance

- **Duration:** 30 min
- **Completed:** 2026-04-03
- **Primary verification:** `cargo test -p dsview-core && cargo test -p dsview-sys`

## Accomplishments
- Added table-driven unit coverage in `crates/dsview-core/src/capture_config.rs` for valid configurations, unsupported sample rates, zero or out-of-range channels, too many enabled channels, and enabled-channel-dependent sample-limit failures.
- Added a core regression test proving the Phase 3 default DSLogic Plus capability snapshot is internally consistent and exposed as expected.
- Re-ran the source-runtime CLI discovery path with `DSView/DSView/res/` to confirm Phase 3 bridge changes did not regress Phase 2 list behavior.
- Kept all automated coverage no-hardware by default while preserving a clear manual validation path for real DSLogic Plus config apply behavior.

## Files Created/Modified
- `crates/dsview-core/src/capture_config.rs` - Expanded validation test coverage for Phase 3 rules.
- `crates/dsview-core/src/lib.rs` - Adds a focused capability snapshot regression test.
- `crates/dsview-sys/src/lib.rs` - Retains bridge-level regression coverage while the config surface grows.

## Decisions Made
- Reused no-hardware validation fixtures for most CAP-01 through CAP-04 checks to keep the default test path fast and deterministic.
- Kept the existing source-runtime CLI list smoke in the verification loop because it catches end-to-end bridge regressions without requiring config-apply hardware access yet.

## Deviations from Plan
- The current coverage lives in module tests rather than a dedicated `crates/dsview-core/tests/capture_config.rs` file because the new config model remains compact and tightly coupled to its implementation.

## Issues Encountered
- The first depth-capacity assertions used unrealistic sample sizes after switching to real DSLogic Plus hardware-depth assumptions, so the failure fixtures had to be adjusted to reflect actual per-channel capacity math.

## User Setup Required
- For manual hardware validation, open a real `DSLogic Plus`, inspect capabilities, apply one valid config, verify one invalid config is rejected before acquisition, and release the device cleanly.
- Real config apply verification may still require USB permission setup on this machine because Phase 2 source-runtime discovery logs `LIBUSB_ERROR_ACCESS` during profile checks.

## Next Phase Readiness
- Phase 4 can build acquisition start/stop behavior on top of a tested pre-run configuration model and active-device apply path.
- Remaining manual work is focused on real hardware config-apply confirmation rather than basic validation correctness.

---
*Phase: 03-capture-configuration-surface*
*Completed: 2026-04-03*
