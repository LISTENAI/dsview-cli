---
phase: 12-cli-device-option-surface
plan: 01
subsystem: cli
tags: [rust, serde, device-options, dslogic-plus]
requires:
  - phase: 10-device-option-bridge-and-discovery
    provides: Stable device-option snapshots with authoritative stable IDs and grouped channel modes
  - phase: 11-device-option-validation-model
    provides: Selected-device validation contracts and current-value snapshots that the CLI surface can mirror
provides:
  - CLI token helpers for DSLogic Plus device-option values in `dsview-cli`
  - Capture-oriented `devices options` JSON/text output with tokens plus stable IDs
  - Regression coverage for copy-paste capture guidance and channel-mode channel limits
affects: [12-02, 12-03, capture, devices-options]
tech-stack:
  added: []
  patterns: [cli-token-overlay, capture-oriented-inspection]
key-files:
  created: [crates/dsview-cli/src/capture_device_options.rs]
  modified: [crates/dsview-cli/src/device_options.rs, crates/dsview-cli/src/lib.rs, crates/dsview-cli/tests/device_options_cli.rs]
key-decisions:
  - Keep friendly token generation in `dsview-cli` so Phase 10/11 stable IDs remain unchanged.
  - Carry `token` plus `stable_id` through JSON/text so shell users and automation share one inspection contract.
  - Lead text output with capture flag examples and channel-count hints derived from the active channel mode.
patterns-established:
  - 'CLI token overlay: derive human-friendly kebab-case tokens from labels while preserving stable IDs in every response object.'
  - 'Capture-oriented inspection rendering: surface future `capture` flag names before the detailed option sections.'
requirements-completed: [OPT-02, OPT-03, OPT-04, OPT-05, OPT-06, OPT-07]
duration: 12m
completed: 2026-04-13
---

# Phase 12 Plan 01: CLI device option surface Summary

**Capture-facing device-option tokens now sit on top of the stable DSLogic Plus IDs, and `devices options` renders them as copy-pasteable `capture` flags plus deterministic JSON/text contracts.**

## Performance

- **Duration:** 12 min
- **Started:** 2026-04-13T07:30:25Z
- **Completed:** 2026-04-13T07:42:19Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Added `crates/dsview-cli/src/capture_device_options.rs` with deterministic token builders, lookup maps, and capture-flag guide metadata.
- Reshaped `devices options` responses in `crates/dsview-cli/src/device_options.rs` so current values, option lists, threshold guidance, and channel-mode groups all expose CLI tokens alongside stable IDs.
- Locked the integration contract in `crates/dsview-cli/tests/device_options_cli.rs` for JSON tokens/stable IDs, threshold range facts, and copy-paste text guidance including `--channels` limits.

## Task Commits

Each task was committed atomically:

1. **Task 1: Create the friendly-token contract and capture-oriented inspection response**
   - `6922685` (`test`) RED: failing library tests for the tokenized response surface
   - `d1999ec` (`feat`) GREEN: token helpers, tokenized response types, and capture-oriented text rendering
2. **Task 2: Lock `devices options` JSON and text output to the new capture-facing token surface**
   - `9103f4f` (`test`) RED: failing integration tests for the new JSON/text contract
   - `fc4a658` (`test`) GREEN: final contract assertions for JSON/text inspection output

## Files Created/Modified
- `crates/dsview-cli/src/capture_device_options.rs` - CLI-only token structs, slug generation, lookup maps, and flag-guide helpers.
- `crates/dsview-cli/src/device_options.rs` - tokenized response structs, response building, deterministic text rendering, and focused unit coverage.
- `crates/dsview-cli/src/lib.rs` - exports the new capture-device-option helper module for later capture parsing work.
- `crates/dsview-cli/tests/device_options_cli.rs` - regression tests for token/stable-id JSON output and copy-paste text rendering.

## Decisions Made
- Keep token generation CLI-local so discovery/validation stable IDs in `dsview-core` stay authoritative and unchanged.
- Preserve both `token` and `stable_id` in machine-readable output so downstream automation can adopt friendly tokens without losing the stable identifiers.
- Use a dedicated `capture_flags` text section to answer “what do I pass to `capture`?” directly, including the `--channels IDX[,IDX...]` hint tied to channel-mode limits.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Restored STATE.md fields required by gsd-tools**
- **Found during:** Final state updates
- **Issue:** `state advance-plan` and `state record-session` could not parse/update the repo's `STATE.md` because the current plan and session continuity fields the tool expects were missing.
- **Fix:** Added the missing phase/plan/session fields, reran the official `gsd-tools` state updates, and cleaned the duplicate fallback lines so `STATE.md` now points at Plan `12-02`.
- **Files modified:** `.planning/STATE.md`
- **Verification:** `gsd-tools state advance-plan` and `gsd-tools state record-session --stopped-at "Completed 12-cli-device-option-surface-01-PLAN.md"` both succeeded on retry.
- **Committed in:** docs metadata commit

---

**Total deviations:** 1 auto-fixed (Rule 3: 1)
**Impact on plan:** Planning metadata needed a small compatibility repair, but product scope and code changes stayed on-plan.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Plan `12-02` can reuse `token_lookup_maps(...)` and the locked `devices options` contract to resolve `capture` flag tokens back to stable IDs.
- The inspection output now mirrors the intended `capture` vocabulary, so follow-up parser/help work can build on a fixed user-facing token surface.


## Self-Check: PASSED
- Verified summary file exists.
- Verified task commits `6922685`, `d1999ec`, `9103f4f`, and `fc4a658` exist in git history.
