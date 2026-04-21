---
phase: 12-cli-device-option-surface
plan: 02
subsystem: cli
tags: [rust, clap, validation, dslogic-plus]
requires:
  - phase: 12-cli-device-option-surface
    provides: Capture-facing token contract and `devices options` inspection output from Plan 12-01
  - phase: 11-device-option-validation-model
    provides: Selected-device validation requests, stable error codes, and validation entrypoints
provides:
  - Optional `capture` flags for DSLogic Plus device-option selection
  - Resolver logic that merges current device state with CLI overrides and carries `--channels` into Phase 11 validation
  - Pre-acquisition selected-device validation for capture requests without claiming Phase 13 apply-time behavior
affects: [12-03, capture, validation]
tech-stack:
  added: []
  patterns: [capture-option-merge, selected-device-preflight-validation]
key-files:
  created: [.planning/phases/12-cli-device-option-surface/12-02-SUMMARY.md]
  modified: [crates/dsview-cli/src/main.rs, crates/dsview-cli/src/capture_device_options.rs]
key-decisions:
  - Keep the clap-facing `CaptureDeviceOptionArgs` in `main.rs` and adapt it into the shared resolver with a lightweight trait instead of duplicating token parsing logic.
  - Preserve omitted sibling option values from the selected-device snapshot, but let explicit channel-mode or stop-option tokens infer the parent operation mode only when the inference is unique.
  - Route `--channels` through the same selected-device validation preflight as the new flags so channel-count limits stay aligned with the resolved channel mode before capture begins.
patterns-established:
  - "Capture resolver pattern: friendly CLI tokens resolve through the Phase 12 token contract, merge with current selected-device values, and emit a full Phase 11 validation request."
  - "Capture validation branch: selected-device inspection plus Phase 11 validation runs before the existing capture-config and runtime path, but Phase 12 does not apply hardware options yet."
requirements-completed: [OPT-02, OPT-03, OPT-04, OPT-05, OPT-06, OPT-07]
duration: 6m
completed: 2026-04-13
---

# Phase 12 Plan 02: CLI device option surface Summary

**The existing `capture` command now accepts DSLogic Plus device-option flags, resolves them against current selected-device state, and validates the full request before acquisition without implying Phase 13 apply-time behavior.**

## Performance

- **Duration:** 6 min
- **Started:** 2026-04-13T08:28:11Z
- **Completed:** 2026-04-13T08:33:54Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- Added `CaptureDeviceOptionArgs` to `crates/dsview-cli/src/main.rs` so `capture` exposes grouped optional flags for operation mode, stop option, channel mode, threshold voltage, and filter selection, each pointing back to `devices options --handle <HANDLE>`.
- Implemented `resolve_capture_device_option_request(...)` in `crates/dsview-cli/src/capture_device_options.rs` so friendly tokens reuse the Plan 12-01 contract, omitted values fall back to the current device snapshot, unique child tokens can infer their parent operation mode, and `--channels` flows into `enabled_channels`.
- Updated `run_capture` in `crates/dsview-cli/src/main.rs` to inspect selected-device options, build the merged validation request, and call `validate_device_option_request(...)` before the existing capture-config, acquisition, and export path.
- Added binary unit coverage in `crates/dsview-cli/src/main.rs` for current-value preservation, channel-mode inference, explicit operation-mode precedence, channel carry-through, channels-only validation routing, and stable parse diagnostics.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add optional capture device-option flags and resolve them against current device state**
   - `e192a96` (`test`) RED: failing binary tests for resolver fallback, inference, and channel carry-through
   - `98f0c40` (`feat`) GREEN: capture flag surface plus resolver implementation
2. **Task 2: Validate resolved device options before the existing capture path and keep help/diagnostics truthful**
   - `b658e25` (`test`) RED: failing binary tests for the validation branch guard and parse diagnostics
   - `1a61f8c` (`feat`) GREEN: selected-device validation branch and stable parse-error reporting

## Files Created/Modified

- `crates/dsview-cli/src/main.rs` - adds the new capture flag group, help text, selected-device validation branch, parse-error classification, and direct binary unit coverage.
- `crates/dsview-cli/src/capture_device_options.rs` - adds the capture-option input trait, token-level parse error enum, request resolver, and unique parent-inference helpers.
- `.planning/phases/12-cli-device-option-surface/12-02-SUMMARY.md` - records the executed work, decisions, and verification results for this plan.

## Decisions Made

- Kept the clap-specific argument struct in the binary and exposed only a small trait to the shared resolver so the library module stays reusable without depending on binary-only types.
- Let explicit `--operation-mode` remain authoritative even when a child token would otherwise infer a different mode, so incompatible combinations still surface through the Phase 11 validator instead of being silently rewritten.
- Treated `--channels` as part of the device-option-aware preflight so channel-count limits are checked against the resolved channel mode before the existing capture path proceeds.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan `12-03` can focus on spawned CLI regression coverage for help text, parser behavior, and end-to-end diagnostics because the resolver and preflight validation branch are now in place.
- The new parse-error classifier and grouped help strings provide stable surfaces for the remaining integration assertions without adding Phase 13 apply-time behavior early.

## Self-Check: PASSED

- Found summary file: `.planning/phases/12-cli-device-option-surface/12-02-SUMMARY.md`
- Found commit: `e192a96`
- Found commit: `98f0c40`
- Found commit: `b658e25`
- Found commit: `1a61f8c`
