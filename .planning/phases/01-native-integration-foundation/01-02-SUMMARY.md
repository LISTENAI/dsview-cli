---
phase: 01-native-integration-foundation
plan: 02
subsystem: native-integration
tags: [ffi, libsigrok4dsl, build, linking, documentation]
requires: [01-01]
provides:
  - Narrow public-header native boundary rooted at DSView/libsigrok4DSL
  - Build-script validation for DSView submodule and libsigrok4DSL public header presence
  - Phase-local documentation of supported native dependency path and revalidation rules
affects: [native-integration, build, planning]
tech-stack:
  added: [Cargo build script, C header wrapper]
  patterns: [public frontend boundary, read-only submodule integration, isolated sys FFI]
key-files:
  created:
    - crates/dsview-sys/build.rs
    - crates/dsview-sys/wrapper.h
  modified:
    - crates/dsview-sys/Cargo.toml
    - crates/dsview-sys/src/lib.rs
    - .planning/phases/01-native-integration-foundation/01-RESEARCH.md
key-decisions:
  - "Use DSView/libsigrok4DSL/libsigrok.h as the supported Phase 1 Rust-facing boundary."
  - "Scope Phase 1 to sr_get_lib_version_string() instead of internal lifecycle calls such as sr_init/sr_exit."
  - "Do not treat the DSView GUI executable target as a reusable integration path for the CLI."
patterns-established:
  - "Boundary Pattern: dsview-sys owns the only raw FFI declarations."
  - "Build Validation Pattern: build.rs fails early when the DSView submodule or public headers are missing."
requirements-completed: []
duration: 35 min
completed: 2026-04-03
---

# Phase 01 Plan 02: Validate native build and link strategy Summary

**Documented and encoded the lowest-risk Phase 1 native boundary around the public `libsigrok4DSL` frontend surface**

## Performance

- **Duration:** 35 min
- **Completed:** 2026-04-03
- **Primary verification:** `cargo check -p dsview-sys`

## Accomplishments
- Added `crates/dsview-sys/build.rs` to validate the `DSView/` submodule layout, check for `libsigrok4DSL/libsigrok.h`, and emit explicit boundary metadata to Cargo.
- Added `crates/dsview-sys/wrapper.h` so the Rust sys crate points at the public `DSView/libsigrok4DSL/libsigrok.h` header instead of broader DSView application headers.
- Updated `crates/dsview-sys/src/lib.rs` to expose only the minimal raw FFI boundary needed for the current proof point.
- Recorded the Phase 1 boundary decision and revalidation triggers in `.planning/phases/01-native-integration-foundation/01-RESEARCH.md`.

## Files Created/Modified
- `crates/dsview-sys/build.rs` - Validates DSView/native prerequisites and encodes the Phase 1 boundary assumptions.
- `crates/dsview-sys/wrapper.h` - Narrow C header entrypoint for the Rust sys layer.
- `crates/dsview-sys/Cargo.toml` - Enables the build script.
- `crates/dsview-sys/src/lib.rs` - Declares the minimal raw symbol boundary.
- `.planning/phases/01-native-integration-foundation/01-RESEARCH.md` - Documents the supported native path and revalidation rules.

## Decisions Made
- Chose `sr_get_lib_version_string()` as the public proof symbol because it is exported from `libsigrok.h` and avoids binding Phase 1 to DSView-private lifecycle APIs.
- Treated `DSView/` as a read-only upstream dependency and explicitly rejected DSView GUI target reuse as the CLI integration strategy.
- Kept the boundary intentionally narrow so later phases can add a shim or a standalone native library path without expanding unsafe Rust surface prematurely.

## Deviations from Plan
- Did not claim a finished standalone `libsigrok4DSL` library artifact exists in-tree; the plan closed with a documented proof boundary rather than a reusable packaged native library.

## Issues Encountered
- The DSView CMake graph builds `libsigrok4DSL` sources into the DSView application target, so a direct reusable library artifact is not yet available for the CLI.
- The executor worktree path could not see the populated `DSView/` submodule, so execution continued in the main repository via the user-approved bypass path.

## User Setup Required
- Keep the `DSView/` submodule initialized before building `dsview-sys`.

## Next Phase Readiness
- Phase 1 now has a real, documented native boundary for smoke validation.
- Later device/session work still needs either a standalone native library path or a tiny repo-owned shim for broader runtime behavior.

---
*Phase: 01-native-integration-foundation*
*Completed: 2026-04-03*
