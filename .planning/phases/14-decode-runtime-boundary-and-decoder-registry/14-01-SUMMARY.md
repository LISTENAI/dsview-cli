---
phase: 14-decode-runtime-boundary-and-decoder-registry
plan: 01
subsystem: ffi
tags: [libsigrokdecode4dsl, ffi, cmake, runtime, decoder-registry]
requires: []
provides:
  - "Separate source-built decode runtime artifact and path contract"
  - "Raw decode runtime FFI for load/init/list/inspect/free operations"
  - "Owned decoder metadata snapshots for later safe registry/CLI layers"
affects: [phase-15, phase-16, decode-list, decode-inspect]
tech-stack:
  added: []
  patterns:
    - "Sibling capture/decode runtime artifacts built from the same native CMake entrypoint"
    - "Deep-copy nested libsigrokdecode metadata into bridge-owned snapshots before Rust conversion"
key-files:
  created: []
  modified:
    - crates/dsview-sys/native/CMakeLists.txt
    - crates/dsview-sys/build.rs
    - crates/dsview-sys/wrapper.h
    - crates/dsview-sys/bridge_runtime.c
    - crates/dsview-sys/src/lib.rs
    - crates/dsview-sys/tests/runtime_packaging.rs
    - crates/dsview-sys/tests/boundary.rs
key-decisions:
  - "Keep decode runtime packaging separate from capture runtime packaging so Python/decoder prerequisites stay isolated."
  - "Expose list/inspect results as bridge-owned snapshots with explicit free helpers instead of leaking GSList or Python-backed pointers into Rust."
  - "Classify decode bring-up failures into loader, decoder-directory, python/init, decoder-load, and unknown-decoder buckets at the sys boundary."
patterns-established:
  - "Decode runtime discovery mirrors capture runtime discovery with dedicated filename and source-artifact helpers."
  - "Decode metadata inspection is safe-to-wrap because Rust copies C snapshots immediately and the bridge owns all nested allocations."
requirements-completed: [DEC-01, DEC-02]
duration: 13 min
completed: 2026-04-21
---
# Phase 14 Plan 01: Decode Runtime Boundary Summary

**Separate decode runtime packaging plus owned list/inspect metadata snapshots for `libsigrokdecode4DSL` decoder discovery**

## Performance

- **Duration:** 13 min
- **Started:** 2026-04-21T04:10:07Z
- **Completed:** 2026-04-21T04:23:12Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments

- Added a sibling `dsview_decode_runtime` native target and build-script/env contract so decode discovery can ship independently of `dsview_runtime`.
- Added raw decode loader/init/list/inspect/free FFI plus Rust `DecodeRuntimeBridge` wrappers and explicit decode error categories.
- Deep-copied decoder channels, options, annotations, annotation rows, inputs, outputs, and tags into bridge-owned memory before safe Rust conversion.
- Locked the runtime contract and decode boundary with integration coverage in `runtime_packaging` and `boundary`.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add a separate decode runtime build/discovery contract** - `8f69a51` (`feat`)
2. **Task 2: Add raw decode-runtime FFI and owned metadata snapshot primitives** - `cddad71` (`feat`)

No separate docs commit was created here; the orchestrator owns follow-up `.planning/` state/roadmap writes.

## Files Created/Modified

- `crates/dsview-sys/native/CMakeLists.txt` - Adds optional sibling capture/decode runtime targets and Python-aware decode linking.
- `crates/dsview-sys/build.rs` - Builds/export capture and decode source artifacts independently and publishes decode runtime env helpers.
- `crates/dsview-sys/wrapper.h` - Declares the decode runtime C ABI and owned snapshot structs for list/inspect metadata.
- `crates/dsview-sys/bridge_runtime.c` - Implements decode runtime loading, init/exit, decoder list/inspect, deep-copy helpers, and decode error storage.
- `crates/dsview-sys/src/lib.rs` - Adds decode runtime filename/path helpers, raw FFI declarations, `DecodeRuntimeBridge`, decode data models, and conversions.
- `crates/dsview-sys/tests/runtime_packaging.rs` - Locks the decode runtime filename/path contract.
- `crates/dsview-sys/tests/boundary.rs` - Verifies decode loader/init error shaping and live list/inspect entrypoints.

## Decisions Made

- Built capture and decode runtimes from the same native entrypoint but behind separate CMake toggles so capture builds do not depend on decode prerequisites.
- Kept decode runtime list/inspect focused on metadata discovery only; no decode execution/session API was exposed in this plan.
- Returned option defaults and allowed values as stringified `GVariant` text in the raw snapshot so later safe layers can preserve upstream values without raw pointer access.
- Let `DecodeRuntimeBridge` drop call `srd_exit()` but leave actual shared-library unload explicit, avoiding unsafe `dlclose` churn during repeated boundary tests.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added `libsigrokdecode4DSL` include wiring to the bridge build**
- **Found during:** Task 2 (decode FFI implementation)
- **Issue:** `bridge_runtime.c` could not compile after `wrapper.h` started including `libsigrokdecode.h` because the bridge shim compile flags only exposed capture headers.
- **Fix:** Extended `crates/dsview-sys/build.rs` bridge include flags to add the `DSView/libsigrokdecode4DSL` include root.
- **Files modified:** `crates/dsview-sys/build.rs`
- **Verification:** `cargo test -p dsview-sys --test boundary -- --nocapture`
- **Committed in:** `cddad71`

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Required for the planned decode ABI to compile; no scope creep beyond the intended boundary work.

## Issues Encountered

- Several upstream decoder scripts fail to import cleanly in this local Python environment, but `srd_decoder_load_all()` still leaves a non-empty registry. The bridge therefore treats empty-registry results as the decoder-load failure boundary instead of surfacing every upstream script warning as a hard init failure.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Safe Rust and CLI layers can now build `decode list` / `decode inspect` features on top of stable low-level discovery primitives.
- Phase 15 can reuse the owned metadata schema for config validation without touching Python-backed libsigrokdecode internals.
- Phase 16 can layer decode execution separately; this plan intentionally stops at discovery/inspection.

## Self-Check: PASSED

- Verified `.planning/phases/14-decode-runtime-boundary-and-decoder-registry/14-01-SUMMARY.md` exists.
- Verified task commits `8f69a51` and `cddad71` exist in `git log`.

---
*Phase: 14-decode-runtime-boundary-and-decoder-registry*
*Completed: 2026-04-21*
