---
phase: 02-device-discovery-and-session-bring-up
plan: 01
subsystem: native-integration
tags: [ffi, shim, device-discovery, dsview, dslogic]
requires: [01-03]
provides:
  - Narrow dynamic ds_* runtime bridge isolated in dsview-sys
  - Source-built shared runtime path from DSView/libsigrok4DSL sources
  - Typed raw sys boundary for init, list, open, release, and last-error bring-up
affects: [native-integration, build, testing, cli]
tech-stack:
  added: [dynamic loader shim, source-built shared library target, thiserror]
  patterns: [repo-owned runtime bridge, source-backed native artifact, typed sys error mapping]
key-files:
  created:
    - crates/dsview-sys/bridge_runtime.c
    - crates/dsview-sys/native/CMakeLists.txt
  modified:
    - crates/dsview-sys/Cargo.toml
    - crates/dsview-sys/build.rs
    - crates/dsview-sys/src/lib.rs
    - crates/dsview-sys/wrapper.h
    - .planning/phases/02-device-discovery-and-session-bring-up/02-RESEARCH.md
key-decisions:
  - "Keep Rust bound to a repo-owned dynamic bridge over the DSView ds_* facade instead of binding broad DSView-private internals directly."
  - "Add a source-built libdsview_runtime.so path so Phase 2 no longer depends on an externally prepared shared library."
  - "Preserve DSView/ as read-only upstream code and build the shared runtime from repo-owned native build wiring."
patterns-established:
  - "Runtime Artifact Pattern: dsview-sys can consume either an explicit runtime library path or an automatically built source runtime artifact."
  - "Bridge Pattern: all loader and raw ownership behavior stays in dsview-sys while higher crates consume typed handles and errors."
requirements-completed: [DEV-01, DEV-02, DEV-03]
duration: 95 min
completed: 2026-04-03
---

# Phase 02 Plan 01: Extend native boundary for discovery and bring-up Summary

**Extended the native boundary from a proof-only symbol into a real DSView-backed bring-up bridge and added a source-built shared runtime so Phase 2 can run without a preexisting external `.so` artifact**

## Performance

- **Duration:** 95 min
- **Completed:** 2026-04-03
- **Primary verification:** `cargo test -p dsview-sys`

## Accomplishments
- Added `crates/dsview-sys/bridge_runtime.c` and expanded `crates/dsview-sys/wrapper.h` so Rust can dynamically load the DSView `ds_*` facade and call init, firmware-resource setup, device list, open, release, last-error, and init-status entry points through a narrow repo-owned bridge.
- Reworked `crates/dsview-sys/src/lib.rs` to expose typed raw sys-layer concepts such as `DeviceHandle`, `DeviceSummary`, `NativeErrorCode`, `RuntimeBridge`, and bridge/source-runtime availability helpers while keeping unsafe FFI confined to `dsview-sys`.
- Upgraded `crates/dsview-sys/build.rs` to build both the bridge shim and a source-backed `libdsview_runtime.so` from `DSView/libsigrok4DSL` plus `DSView/common`, gated by explicit native prerequisite checks.
- Added `crates/dsview-sys/native/CMakeLists.txt` as a repo-owned native build target that compiles the minimum Phase 2 shared runtime without depending on the DSView GUI executable target.

## Files Created/Modified
- `crates/dsview-sys/bridge_runtime.c` - Dynamic loader bridge over the DSView `ds_*` facade.
- `crates/dsview-sys/native/CMakeLists.txt` - Source-backed shared runtime build for `libdsview_runtime.so`.
- `crates/dsview-sys/build.rs` - Builds the bridge, optional smoke shim, and source runtime when native prerequisites are available.
- `crates/dsview-sys/src/lib.rs` - Adds typed Phase 2 raw boundary APIs and tests.
- `crates/dsview-sys/wrapper.h` - Declares the narrow C-side bridge ABI.
- `crates/dsview-sys/Cargo.toml` - Adds `thiserror` for structured sys-layer runtime errors.

## Decisions Made
- Chose a dynamic bridge over direct Rust bindings to DSView-private lifecycle/session internals so the unsafe ABI surface stays stable and repo-owned.
- Added a source-built runtime path because the upstream DSView tree does not ship a standalone reusable `libsigrok4DSL` artifact for the CLI.
- Kept the runtime build independent from the DSView GUI target by compiling the relevant `libsigrok4DSL` and `common` sources directly into a shared library.

## Deviations from Plan
- The plan originally assumed callers would always provide an external compatible shared library path; implementation improved this by also producing a local source-built runtime artifact automatically when native prerequisites are present.

## Issues Encountered
- Earlier environment probing suggested `pkg-config`, `glib`, and `libusb` prerequisites were missing, but the current machine does provide enough native tooling for the source runtime build path.
- The DSView source tree assumes broad application-level build wiring, so the minimal runtime target required explicit source selection and dependency shaping to avoid GUI coupling.

## User Setup Required
- Keep the `DSView/` submodule initialized.
- Ensure native build prerequisites remain available (`cmake`, `pkg-config`, `glib-2.0`, `libusb-1.0`, `fftw3`, `zlib`) if you want the source-built runtime artifact to be generated locally.

## Next Phase Readiness
- `dsview-core` can now wrap a real bring-up-capable runtime boundary instead of a proof-only symbol.
- The CLI can either accept an explicit runtime library path or use the source-built runtime artifact for Phase 2 workflows.
- Manual hardware validation now has a runnable source-built runtime path for list/open/release checks.

---
*Phase: 02-device-discovery-and-session-bring-up*
*Completed: 2026-04-03*
