---
phase: 03-capture-configuration-surface
plan: 02
subsystem: native-integration
tags: [ffi, sys, core, capture-config, active-device, dslogic-plus]
requires: [03-01]
provides:
  - Extended active-device config bridge for reading and applying DSLogic Plus capture settings
  - Rust-safe capability snapshots from DSView active-device configuration APIs
  - Core apply path for validated sample rate, sample limit, and enabled channels
affects: [native-integration, core, validation]
tech-stack:
  added: [GLib-backed config bridge helpers]
  patterns: [active-device config bridge, safe capability snapshot, validated apply flow]
key-files:
  created: []
  modified:
    - crates/dsview-sys/bridge_runtime.c
    - crates/dsview-sys/build.rs
    - crates/dsview-sys/src/lib.rs
    - crates/dsview-sys/wrapper.h
    - crates/dsview-core/src/lib.rs
key-decisions:
  - "Keep GVariant and pointer-valued config-list handling inside dsview-sys rather than exposing them to dsview-core."
  - "Apply validated channel state before sample limit and samplerate so configuration ordering stays explicit."
  - "Expose capability snapshots from the active device session instead of re-reading raw config state at every call site."
patterns-established:
  - "Bridge Pattern: active-device configuration reads and writes are normalized into Rust-safe helper methods in dsview-sys."
  - "Apply Pattern: dsview-core only sends validated/effective configs into native apply calls."
requirements-completed: [CAP-01, CAP-02, CAP-03, CAP-04]
duration: 55 min
completed: 2026-04-03
---

# Phase 03 Plan 02: Wire validated capture settings into the native session layer Summary

**Extended the Phase 2 runtime bridge so an opened DSLogic Plus session can expose capture capabilities and accept only validated sample rate, sample limit, and channel-enable settings**

## Performance

- **Duration:** 55 min
- **Completed:** 2026-04-03
- **Primary verification:** `cargo test -p dsview-sys && cargo test -p dsview-core`

## Accomplishments
- Extended `crates/dsview-sys/bridge_runtime.c` and `crates/dsview-sys/wrapper.h` with active-device config helpers for samplerate, sample limit, total/valid channel counts, active channel mode, hardware depth, VTH, samplerate lists, channel modes, and channel enable by index.
- Updated `crates/dsview-sys/src/lib.rs` to expose Rust-safe `CaptureCapabilities` and `ChannelMode` snapshots plus apply helpers for samplerate, sample limit, and enabled channel state.
- Updated `crates/dsview-sys/build.rs` so the bridge shim now compiles and links with the GLib flags required for the new active-device config path.
- Connected `dsview-core` to the new sys-layer capability and apply APIs through `Discovery::dslogic_plus_capabilities()` and `Discovery::apply_capture_config(...)`.

## Files Created/Modified
- `crates/dsview-sys/bridge_runtime.c` - Adds capability reads and config apply helpers over DSView active-device APIs.
- `crates/dsview-sys/wrapper.h` - Declares the extended config bridge ABI and data structs.
- `crates/dsview-sys/src/lib.rs` - Adds Rust-safe capability snapshots and config apply methods.
- `crates/dsview-sys/build.rs` - Supplies GLib cflags/libs for the expanded bridge shim build.
- `crates/dsview-core/src/lib.rs` - Maps native capability snapshots into the Phase 3 core validation/apply flow.

## Decisions Made
- Kept samplerate and channel-mode list decoding inside `dsview-sys` so `dsview-core` never sees raw `GVariant` or pointer-valued native lists.
- Preserved a config-only ordered flow: load capabilities -> validate request -> apply settings, with no acquisition entry points added.
- Used native total/valid channel counts and hardware depth where available instead of keeping all constraints hard-coded in core.

## Deviations from Plan
- Channel-mode samplerate support is still modeled conservatively from the DSLogic Plus phase assumptions; future refinement can align it more tightly with upstream per-mode native lists if needed.

## Issues Encountered
- The bridge shim needed direct GLib compilation/link flags because the new capability path consumes `GVariant` data and list decoding.
- DSView returns some config-list values as pointer-backed native arrays, which required keeping shape normalization entirely within the sys boundary.

## User Setup Required
- None beyond the existing Phase 2 runtime/resource prerequisites for local builds and hardware-backed manual validation.

## Next Phase Readiness
- Wave 3 can now add broader tests around the config apply path and capability snapshots.
- Later acquisition work can assume a validated configuration has a real native apply path in place.

---
*Phase: 03-capture-configuration-surface*
*Completed: 2026-04-03*
