---
phase: 13-option-aware-capture-reporting
plan: 01
subsystem: capture
tags: [rust, dslogic-plus, runtime-bridge, acquisition, cli]
requires:
  - phase: 12-cli-device-option-surface
    provides: Capture-side validated device-option requests and stable CLI token/validation contracts
  - phase: 11-device-option-validation-model
    provides: Full validated DSLogic Plus option requests with native codes and inherited current values
provides:
  - Ordered runtime setters and readback helpers for DSLogic Plus device-option apply
  - Core option-aware capture plumbing with typed partial-apply failures
  - CLI capture execution that preserves validated requests instead of discarding them
affects: [13-02, 13-03, capture, metadata, cli-errors]
tech-stack:
  added: []
  patterns: [ordered-device-option-apply, typed-partial-apply-failure, validated-request-plumbing]
key-files:
  created: [.planning/phases/13-option-aware-capture-reporting/13-01-SUMMARY.md]
  modified: [crates/dsview-sys/src/lib.rs, crates/dsview-sys/bridge_runtime.c, crates/dsview-sys/tests/device_options.rs, crates/dsview-core/src/lib.rs, crates/dsview-core/tests/acquisition.rs, crates/dsview-cli/src/main.rs, crates/dsview-core/tests/export_artifacts.rs]
key-decisions:
  - Keep the D-05 setter sequence in Rust so core owns apply order, fail-fast behavior, and typed reporting instead of hiding sequencing in C.
  - Reuse the Phase 11 validated request directly during capture execution and derive export validation config from it, rather than re-validating against the current active mode.
  - Treat effective enabled channels as the successfully applied validated request after channel-enable setters succeed, while reading the other effective values back from runtime getters.
patterns-established:
  - "Option-aware capture pattern: validated requests flow from CLI into core, core applies them before acquisition start, and legacy config-only runs stay on the original path when the option request is absent."
  - "Partial-apply reporting pattern: setter failures capture applied_steps, failed_step, and the underlying runtime error for direct CLI serialization."
requirements-completed: [RUN-04]
duration: 16m
completed: 2026-04-13
---

# Phase 13 Plan 01: Option-aware capture reporting Summary

**Validated DSLogic Plus device-option requests now reach runtime execution in locked D-05 order, fail fast with truthful partial-apply facts, and preserve the shipped config-only capture baseline when no option-aware request is present.**

## Performance

- **Duration:** 16 min
- **Started:** 2026-04-13T10:49:22Z
- **Completed:** 2026-04-13T11:05:22Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments

- Extended `dsview-sys` with granular setters and current-value readback helpers for operation mode, stop option, channel mode, threshold volts, filter, sample limit, and sample rate.
- Upgraded the native mock bridge to log exact apply order, inject per-step failures, and prove the sequence stops on the first rejected setter without attempting later steps.
- Threaded `validated_device_options` from the CLI into core capture execution, added typed apply contracts, and stopped discarding the validated request before runtime execution.
- Added core and CLI regressions for ordered apply, partial-apply failure facts, and the preserved config-only baseline.

## Task Commits

Each task was committed atomically:

1. **Task 1: Extend the sys bridge with ordered setter/readback support and fail-fast mock coverage**
   - `10e9b9c` (`test`) RED: failing sys bridge tests for ordered setters, apply logging, and readback
   - `09e7e5b` (`feat`) GREEN: runtime setter/readback helpers plus mock apply logging and failure injection
2. **Task 2: Thread the validated request into core capture execution and surface partial-apply failure facts**
   - `d626019` (`test`) RED: failing acquisition and CLI coverage for option-aware apply plumbing and partial-failure reporting
   - `dcd1a9c` (`feat`) GREEN: final core/CLI option-aware capture path and typed apply failure contract

## Files Created/Modified

- `crates/dsview-sys/src/lib.rs` - exposes granular runtime setters and effective-value readback helpers for Phase 13 apply-time fields.
- `crates/dsview-sys/bridge_runtime.c` - records ordered setter calls, supports step-specific failure injection, and returns mock current values for apply/readback tests.
- `crates/dsview-sys/tests/device_options.rs` - locks runtime setter coverage, D-05 apply order, fail-fast behavior, and effective readback assertions.
- `crates/dsview-core/src/lib.rs` - adds typed apply contracts, validated-request plumbing, and the option-aware capture branch that applies device options before `start_collect()`.
- `crates/dsview-core/tests/acquisition.rs` - covers ordered apply, partial-apply failure facts, and the no-option baseline branch.
- `crates/dsview-cli/src/main.rs` - passes the validated request into core execution, derives the validated export config from it, and classifies device-option apply failures for CLI output.
- `crates/dsview-core/tests/export_artifacts.rs` - updates capture fixtures for the additive `effective_device_options` field on run summaries.

## Decisions Made

- Kept sequencing in core Rust instead of C so later CLI and metadata plans can share one typed source of truth for both success and failure reporting.
- Added `validated_device_options` directly to `CaptureRunRequest` so the already validated Phase 11/12 request reaches execution unchanged.
- Returned effective enabled channels from the successfully applied validated request rather than inventing a new runtime getter in Phase 13.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Updated export-artifact test fixtures for the additive capture summary field**
- **Found during:** Task 2 (Thread the validated request into core capture execution and surface partial-apply failure facts)
- **Issue:** Adding `effective_device_options` to `CaptureRunSummary` broke existing export-artifact test fixtures even though the artifact behavior itself was unchanged.
- **Fix:** Updated the affected fixture constructors to initialize the new optional field with `None`.
- **Files modified:** `crates/dsview-core/tests/export_artifacts.rs`, `crates/dsview-core/src/lib.rs`
- **Verification:** `cargo test -p dsview-core --lib -- --nocapture`
- **Committed in:** `d626019`

---

**Total deviations:** 1 auto-fixed (Rule 3: 1)
**Impact on plan:** The extra fixture update was required to keep the additive core API compiling and verified. Runtime behavior stayed within plan scope.

## Issues Encountered

- `DeviceOptionApplyFailure` originally derived traits that `dsview_sys::RuntimeError` does not implement, so the final GREEN step trimmed those derives to keep the typed failure contract truthful and compilable.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan `13-02` can now reuse the typed `EffectiveDeviceOptionState` and partial-apply failure contract to report requested vs effective device-option facts in CLI output and metadata.
- The option-aware capture path is locked to the Phase 11 validated request, so later reporting work does not need to rediscover apply order or parse human error text.

## Self-Check: PASSED

- Found summary file: `.planning/phases/13-option-aware-capture-reporting/13-01-SUMMARY.md`
- Found commit: `10e9b9c`
- Found commit: `09e7e5b`
- Found commit: `d626019`
- Found commit: `dcd1a9c`
