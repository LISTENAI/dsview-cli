---
phase: 03-capture-configuration-surface
plan: 01
subsystem: core
tags: [core, capture-config, validation, samplerate, channels, depth]
requires: [02-03]
provides:
  - Rust domain types for requested, validated, and capability-driven capture configuration
  - Pre-run validation for DSLogic Plus sample rate, sample limit, and enabled channel selection
  - Reusable effective configuration model for later native-apply and acquisition phases
affects: [core, validation, planning]
tech-stack:
  added: [capture configuration domain module]
  patterns: [capability-driven validation, effective config normalization, pre-run config contract]
key-files:
  created:
    - crates/dsview-core/src/capture_config.rs
  modified:
    - crates/dsview-core/src/lib.rs
key-decisions:
  - "Model capture settings as Rust domain types instead of passing raw CLI/native values through later phases."
  - "Reject zero enabled channels, unsupported sample rates, and over-capacity limits before acquisition begins."
  - "Represent normalized sample-limit alignment explicitly in the validated configuration."
patterns-established:
  - "Validation Pattern: requested config is converted into a validated/effective config before any native apply path runs."
  - "Capability Pattern: validation depends on a device capability snapshot rather than field-by-field checks in isolation."
requirements-completed: [CAP-01, CAP-02, CAP-03, CAP-04]
duration: 35 min
completed: 2026-04-03
---

# Phase 03 Plan 01: Define Rust domain types and validation rules for capture configuration Summary

**Built the Rust-side capture configuration model so DSLogic Plus requests can be validated and normalized before any acquisition work begins**

## Performance

- **Duration:** 35 min
- **Completed:** 2026-04-03
- **Primary verification:** `cargo test -p dsview-core capture_config`

## Accomplishments
- Added `crates/dsview-core/src/capture_config.rs` with `CaptureConfigRequest`, `CaptureCapabilities`, `ChannelModeCapability`, `ValidatedCaptureConfig`, and `CaptureConfigError`.
- Implemented capability-driven validation for sample rate, sample limit, enabled channel selections, mode-specific channel ceilings, and sample-limit alignment.
- Added focused tests covering valid configurations, unsupported sample rates, zero or out-of-range channels, excessive enabled channels, and enabled-channel-dependent depth failures.
- Exposed the Phase 3 validation contract through `dsview-core` so later phases can reuse validated/effective configuration objects instead of recalculating request rules ad hoc.

## Files Created/Modified
- `crates/dsview-core/src/capture_config.rs` - Capture request/capability/effective-config types and validation rules.
- `crates/dsview-core/src/lib.rs` - Re-exports Phase 3 config types and integrates the validation contract into the core surface.

## Decisions Made
- Kept the first validation pass in Rust so unsupported combinations fail early and predictably before touching native apply behavior.
- Used an explicit effective-configuration type because upstream sample-limit alignment should be visible rather than hidden.
- Rejected empty enabled-channel sets up front instead of relying on implicit native fallback behavior.

## Deviations from Plan
- The capability snapshot is still partly shaped from DSLogic Plus assumptions while Wave 2 completes the full native capability bridge.

## Issues Encountered
- DSLogic Plus validation depends on coupled constraints between channel count, sample limit, and active channel mode, so generic field-level validation was insufficient.

## User Setup Required
- None for automated validation; all Wave 1 checks run without connected hardware.

## Next Phase Readiness
- Wave 2 can now consume a stable validated/effective configuration type for native capability reads and config application.
- Wave 3 testing can expand from the current table-driven validation baseline.

---
*Phase: 03-capture-configuration-surface*
*Completed: 2026-04-03*
