---
phase: 04-acquisition-execution
plan: 02
subsystem: reliability
tags: [core, cli, sys, cleanup, timeout, diagnostics, verification]
requires:
  - phase: 04-01
    provides: capture start/run/finish orchestration and callback-backed acquisition summaries
provides:
  - Deterministic stop, callback-clear, and release cleanup across failed and timed-out acquisition paths
  - Stable CLI failure categories with cleanup detail for start failure, run failure, detach, incomplete, timeout, and cleanup failure
  - Worktree-safe DSView source discovery during `dsview-sys` builds plus verifier-ready failure-path coverage notes
affects: [phase-04, reliability, hardware-validation, cli-diagnostics]
tech-stack:
  added: []
  patterns: [centralized acquisition cleanup, structured failure normalization, ancestor-based DSView source discovery]
key-files:
  created:
    - .planning/phases/04-acquisition-execution/04-02-SUMMARY.md
  modified:
    - crates/dsview-core/src/lib.rs
    - crates/dsview-cli/src/main.rs
    - crates/dsview-sys/build.rs
    - .planning/phases/04-acquisition-execution/04-RESEARCH.md
key-decisions:
  - "Treat missing end-packet status as incomplete so Phase 4 success still requires explicit end-of-stream evidence."
  - "Keep cleanup status structured and surfaced to the CLI so post-failure reuse problems are visible without hiding the primary acquisition outcome."
  - "Allow dsview-sys builds from isolated worktrees by resolving the nearest ancestor that contains the populated DSView submodule."
patterns-established:
  - "Cleanup Pattern: capture execution always attempts stop-if-collecting, callback clear, and release through one core-owned cleanup path."
  - "Diagnostic Pattern: CLI capture failures expose stable machine-readable codes plus optional native error, terminal event, and cleanup fields."
  - "Build Pattern: native DSView inputs are resolved from the nearest populated ancestor repo instead of assuming each worktree contains a synced submodule."
requirements-completed: [RUN-02, RUN-03]
duration: 45 min
completed: 2026-04-07
---

# Phase 04 Plan 02: Handle stop, cleanup, and error paths so failed acquisitions do not leave broken state Summary

**Hardened Phase 4 acquisition cleanup and diagnostics so failed or timed-out runs attempt deterministic teardown, surface stable failure categories, and keep the worktree build path usable for verification**

## Performance

- **Duration:** 45 min
- **Started:** 2026-04-07T05:48:00Z
- **Completed:** 2026-04-07T06:33:00Z
- **Tasks:** 4
- **Files modified:** 5

## Accomplishments
- Centralized capture cleanup in `crates/dsview-core/src/lib.rs` so start failure, timeout, detach, run failure, and incomplete completion all drive the same stop/clear/release flow.
- Expanded `crates/dsview-cli/src/main.rs` to preserve stable machine-readable failure categories while also reporting native error, terminal event, and cleanup detail.
- Tightened the clean-success rule so missing end-packet status is treated as `incomplete` rather than silent success.
- Unblocked local verification in the isolated worktree by teaching `crates/dsview-sys/build.rs` to discover the populated `DSView/` tree from an ancestor repo.
- Updated `04-RESEARCH.md` so every promised 04-02 failure class has an explicit verifier-ready proof path.

## Task Commits

Each task was committed atomically:

1. **Task 1: Define terminal acquisition outcome categories and diagnostic mapping** - `TO_BE_FILLED` (feat)
2. **Task 2: Implement deterministic stop and release ordering for all acquisition paths** - `TO_BE_FILLED` (feat)
3. **Task 3: Make bounded wait and cancellation policy explicit for v1** - `TO_BE_FILLED` (fix)
4. **Task 4: Attach a verification method to every promised failure class** - `TO_BE_FILLED` (docs)

**Plan metadata:** `TO_BE_FILLED` (docs: complete plan)

## Files Created/Modified
- `crates/dsview-core/src/lib.rs` - Adds structured cleanup state, centralized teardown, stricter completion classification, and timeout/cleanup result handling.
- `crates/dsview-cli/src/main.rs` - Exposes stable capture failure codes with terminal-event and cleanup metadata in JSON responses.
- `crates/dsview-sys/build.rs` - Resolves DSView headers and source runtime inputs from the nearest populated ancestor repo so tests run in the isolated worktree.
- `.planning/phases/04-acquisition-execution/04-RESEARCH.md` - Documents verifier-ready proof methods for every promised 04-02 failure class.
- `.planning/phases/04-acquisition-execution/04-02-SUMMARY.md` - Records plan completion context and verification evidence.

## Decisions Made
- Missing end-of-stream evidence remains a hard failure signal, even after a normal terminal event, because Phase 4 success must prove a usable finite capture.
- Cleanup failure still overrides apparent run success because `RUN-02` depends on device reusability, not just event completion.
- Worktree-local DSView absence is handled in the build layer instead of by editing upstream or weakening native validation requirements.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Worktree build could not see the populated DSView submodule**
- **Found during:** Verification (`cargo test -p dsview-core`)
- **Issue:** `dsview-sys/build.rs` assumed `DSView/` lived directly under the isolated worktree, so tests failed before the 04-02 code could be validated.
- **Fix:** Added ancestor-based DSView root discovery in `crates/dsview-sys/build.rs` and reused that resolved root for header lookup and source-runtime builds.
- **Files modified:** `crates/dsview-sys/build.rs`
- **Verification:** `cargo test -p dsview-core`, `cargo test -p dsview-sys`, `cargo test`
- **Committed in:** `TO_BE_FILLED` (part of task commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** The auto-fix was required to verify 04-02 inside the mandated isolated worktree. No scope creep beyond enabling the planned validation path.

## Issues Encountered
- The first test rerun failed because the worktree did not contain a populated `DSView/libsigrok4DSL/libsigrok.h`; resolving the nearest populated ancestor repo fixed the build without touching upstream `DSView/` sources.
- `crates/dsview-cli/src/main.rs` needed additional imports and derives after the richer error payload shape was introduced; those compile fixes were applied before rerunning the full suite.

## User Setup Required
- None for automated verification.
- Manual hardware verification is still required for Phase 4 completion: run one representative failure path on a connected `DSLogic Plus`, confirm non-zero actionable diagnostics, then verify immediate reopen or rerun succeeds.

## Next Phase Readiness
- 04-03 can now focus on smoke and integration validation with stable failure categories and explicit cleanup semantics already in place.
- The main remaining risk is hardware-backed proof of post-error reuse and at least one representative failure-path validation on a real `DSLogic Plus`.

---
*Phase: 04-acquisition-execution*
*Completed: 2026-04-07*
