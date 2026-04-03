---
phase: 01-native-integration-foundation
plan: 03
subsystem: testing
tags: [smoke, ffi, tests, validation]
requires: [01-01, 01-02]
provides:
  - Scoped dsview-sys smoke command through Cargo
  - Optional runtime smoke shim for sr_get_lib_version_string()
  - Documentation of what Phase 1 smoke proves and what remains deferred
affects: [testing, native-integration, planning]
tech-stack:
  added: [C smoke shim, cfg-gated runtime smoke]
  patterns: [environment-gated smoke, non-GUI validation, compile-time plus optional runtime proof]
key-files:
  created:
    - crates/dsview-sys/smoke_version_shim.c
  modified:
    - crates/dsview-sys/build.rs
    - crates/dsview-sys/src/lib.rs
    - .planning/phases/01-native-integration-foundation/01-RESEARCH.md
key-decisions:
  - "Use cargo test -p dsview-sys as the scoped Phase 1 smoke command."
  - "Make runtime smoke conditional on local native prerequisites instead of pretending the environment always supports it."
  - "Keep smoke coverage non-hardware and non-GUI for Phase 1."
patterns-established:
  - "Smoke Pattern: compile-time boundary checks always run, runtime symbol checks run only when the local environment can build the shim."
  - "Fallback Pattern: dsview-sys exposes a None-returning path when runtime smoke is unavailable."
requirements-completed: []
duration: 32 min
completed: 2026-04-03
---

# Phase 01 Plan 03: Add smoke coverage for native boundary Summary

**Added a scoped smoke path that validates the `dsview-sys` boundary through normal Cargo tests without requiring hardware or GUI launch**

## Performance

- **Duration:** 32 min
- **Completed:** 2026-04-03
- **Primary verification:** `cargo test -p dsview-sys`

## Accomplishments
- Added `crates/dsview-sys/smoke_version_shim.c`, a tiny C shim that returns `SR_LIB_VERSION_STRING` through the chosen public proof symbol.
- Expanded `crates/dsview-sys/build.rs` to optionally compile and archive the shim when the local machine has the required compiler and glib development headers.
- Updated `crates/dsview-sys/src/lib.rs` with cfg-gated extern declarations, runtime availability helpers, and smoke tests covering both enabled and skipped-runtime cases.
- Extended `.planning/phases/01-native-integration-foundation/01-RESEARCH.md` with explicit smoke expectations, limitations, and revalidation triggers.

## Files Created/Modified
- `crates/dsview-sys/smoke_version_shim.c` - Minimal runtime proof of the public version symbol.
- `crates/dsview-sys/build.rs` - Builds the shim when prerequisites exist and warns clearly when they do not.
- `crates/dsview-sys/src/lib.rs` - Adds smoke-oriented APIs and tests around runtime availability.
- `.planning/phases/01-native-integration-foundation/01-RESEARCH.md` - Documents what the smoke command proves and defers.

## Decisions Made
- Scoped smoke verification to `cargo test -p dsview-sys` so failures point directly at the native boundary instead of the broader application.
- Chose an environment-gated runtime shim because this machine does not currently provide the full native header setup needed for a broader direct native proof.
- Kept the smoke contract honest: Phase 1 proves boundary wiring, not device discovery, capture execution, or private lifecycle correctness.

## Deviations from Plan
- Used `sr_get_lib_version_string()` rather than `sr_init`/`sr_exit` because the latter are private in this DSView fork and were intentionally excluded from the supported public boundary.

## Issues Encountered
- `pkg-config` was unavailable, so the smoke path could not rely on standard pkg-config discovery.
- Local glib development headers were missing, so the runtime shim is skipped on this machine and the tests verify the documented skip behavior instead.
- The first test pass exposed an undefined-symbol link failure until the runtime-only externs were cfg-gated correctly.

## User Setup Required
- Install the required native headers if you want this machine to exercise the optional runtime shim instead of the documented skip path.

## Next Phase Readiness
- Phase 1 now has an automated smoke command that guards the chosen native boundary.
- Phase 2 can build on this with device/session work once a broader runtime path is chosen for lifecycle operations.

---
*Phase: 01-native-integration-foundation*
*Completed: 2026-04-03*
