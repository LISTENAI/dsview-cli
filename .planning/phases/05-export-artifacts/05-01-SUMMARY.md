---
phase: 05-export-artifacts
plan: 01
subsystem: sys
tags: [sys, core, export, vcd, libsigrok, replay, lifecycle, dslogic-plus]
requires:
  - phase: 04-03
    provides: clean-success acquisition lifecycle coverage and cleanup semantics
provides:
  - Upstream-backed VCD export replay through the narrow `dsview-sys` boundary
  - Clean-success-gated core export orchestration with explicit export failure classification
  - Wave 0 sys/core coverage for export preconditions, facts, and cleanup-safe artifact behavior
affects: [phase-05, export-artifacts, sys-boundary, core-orchestration, verification]
tech-stack:
  added: []
  patterns: [recorded-packet replay via upstream `sr_output_*`, cleanup-safe temp-file promotion, clean-success-gated export orchestration]
key-files:
  created:
    - .planning/phases/05-export-artifacts/05-01-PLAN.md
    - .planning/phases/05-export-artifacts/05-RESEARCH.md
    - crates/dsview-core/tests/export_artifacts.rs
    - .planning/phases/05-export-artifacts/05-01-SUMMARY.md
  modified:
    - crates/dsview-sys/src/lib.rs
    - crates/dsview-sys/bridge_runtime.c
    - crates/dsview-sys/build.rs
    - crates/dsview-sys/wrapper.h
    - crates/dsview-sys/tests/boundary.rs
    - crates/dsview-core/src/lib.rs
    - .planning/STATE.md
    - .planning/ROADMAP.md
key-decisions:
  - "Reuse the upstream VCD output module through `sr_output_*` replay instead of introducing a Rust-side serializer."
  - "Export remains gated on Phase 4 clean-success and uses temp-file promotion so failed export leaves no misleading final-path VCD."
  - "Retained packet replay facts stay inside `dsview-sys`; `dsview-core` only surfaces stable export facts and failure categories."
patterns-established:
  - "Sys Pattern: retain normalized capture packets and replay them through the DSView/libsigrok VCD exporter behind a narrow Rust API."
  - "Core Pattern: treat export as part of successful capture completion, with precondition vs runtime export failures kept distinct."
requirements-completed: [EXP-01, EXP-02, EXP-03, EXP-04]
duration: 1h 10m
completed: 2026-04-07
---

# Phase 05 Plan 01: Integrate the upstream DSView VCD export path through the narrow sys boundary Summary

**Upstream VCD replay now runs through `dsview-sys` before artifact publication, while `dsview-core` only exports clean-success captures and surfaces stable export facts for later metadata work.**

## Performance

- **Duration:** 1h 10m
- **Started:** 2026-04-07T13:06:00Z
- **Completed:** 2026-04-07T14:16:40Z
- **Tasks:** 5
- **Files modified:** 11

## Accomplishments
- Added packet-retaining VCD export support in `dsview-sys`, including upstream `sr_output_*` replay wiring, export fact reporting, and cleanup-safe temp-file promotion.
- Wired clean-success-only export orchestration through `dsview-core` so export failures remain explicit via `CaptureRunError::ExportFailed` and later plans can consume `VcdExportFacts` without sys internals.
- Added Wave 0 export validation in `crates/dsview-sys/tests/boundary.rs` and `crates/dsview-core/tests/export_artifacts.rs`, then verified `cargo test -p dsview-sys --test boundary`, `cargo test -p dsview-core export_capture`, and `cargo test --workspace`.
- Recorded the final lifecycle, retention, overflow, and atomic-or-cleanup-safe artifact decisions in `.planning/phases/05-export-artifacts/05-RESEARCH.md`.

## Task Commits

Each task was committed atomically:

1. **Task 1-3: sys export replay bridge, retention contract, and research artifacts** - `a5d7705` (feat)
2. **Task 4-5: clean-success core export wiring and Wave 0 export tests** - `a63ce13` (feat)

**Plan metadata:** `TO_BE_FILLED` (docs)

## Files Created/Modified
- `.planning/phases/05-export-artifacts/05-01-PLAN.md` - Restores the checked-in 05-01 execution plan inside the isolated worktree.
- `.planning/phases/05-export-artifacts/05-RESEARCH.md` - Records the final export-before-release, retention, overflow, and cleanup-safe artifact decisions.
- `crates/dsview-sys/src/lib.rs` - Adds the Rust-side export request/facts API, temp-file publication logic, and unit coverage for export helpers.
- `crates/dsview-sys/bridge_runtime.c` - Retains normalized packets and replays them through upstream `sr_output_find`, `sr_output_new`, `sr_output_send`, and `sr_output_free`.
- `crates/dsview-sys/build.rs` - Keeps the DSView/common include path and source-runtime build wiring aligned with the replay bridge.
- `crates/dsview-sys/wrapper.h` - Extends the narrow native boundary for retained-stream VCD export.
- `crates/dsview-sys/tests/boundary.rs` - Adds export-focused boundary checks for request shape, overflow code stability, temp promotion semantics, and surfaced export facts.
- `crates/dsview-core/src/lib.rs` - Gates export on `CleanSuccess` and maps precondition/runtime export failures into stable core error types.
- `crates/dsview-core/tests/export_artifacts.rs` - Adds export orchestration coverage for clean-success eligibility, fact shaping, and failure classification.
- `.planning/STATE.md` - Advances Phase 5 position to plan 2 of 3 and records the new export-boundary decisions.
- `.planning/ROADMAP.md` - Marks 05-01 complete and Phase 5 as in progress.

## Decisions Made
- Reused the upstream VCD exporter through the public output-module API so channel naming and timing semantics stay aligned with DSView/libsigrok behavior.
- Preserved Phase 4 cleanup semantics by making export part of the clean-success path, but publishing the final VCD only after temp-file write and promotion succeed.
- Kept actual export facts limited to `sample_count`, `packet_count`, and `output_bytes` so metadata work can proceed without exposing retained packet internals.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Restored missing 05-01 planning and implementation files into the isolated worktree**
- **Found during:** Task 1 (Lock the export-before-release lifecycle rule and packet-retention contract)
- **Issue:** The isolated worktree was missing the checked-in 05-01 plan, research artifact, export tests, and replay implementation already present in the main checkout.
- **Fix:** Copied only the known 05-01 planning and implementation files into the worktree, excluding unrelated checkout drift.
- **Files modified:** `.planning/phases/05-export-artifacts/05-01-PLAN.md`, `.planning/phases/05-export-artifacts/05-RESEARCH.md`, `crates/dsview-core/src/lib.rs`, `crates/dsview-core/tests/export_artifacts.rs`, `crates/dsview-sys/src/lib.rs`, `crates/dsview-sys/tests/boundary.rs`, `crates/dsview-sys/bridge_runtime.c`, `crates/dsview-sys/build.rs`, `crates/dsview-sys/wrapper.h`
- **Verification:** `cargo test -p dsview-sys --test boundary` and `cargo test -p dsview-core export_capture`
- **Committed in:** `a5d7705`, `a63ce13`

**2. [Rule 1 - Bug] Fixed local sys unit-test scaffolding so the full workspace suite builds cleanly**
- **Found during:** Plan verification (`cargo test --workspace`)
- **Issue:** `RawVcdExportRequest` lacked `Debug` for `unwrap_err()`-based tests, and one export-buffer unit test referenced `glib::g_malloc` without a Rust `glib` crate dependency.
- **Fix:** Derived `Debug` for `RawVcdExportRequest` and switched the test helper to use a C-allocator-backed buffer compatible with the bridge free path.
- **Files modified:** `crates/dsview-sys/src/lib.rs`
- **Verification:** `cargo test --workspace`
- **Committed in:** `TO_BE_FILLED` (included with docs closeout if no code-only follow-up commit is created)

---

**Total deviations:** 2 auto-fixed (1 blocking, 1 bug)
**Impact on plan:** Both fixes were necessary to finish 05-01 cleanly in the isolated worktree and keep the full verification story green. No scope creep.

## Issues Encountered
- The isolated worktree did not include the already-landed 05-01 implementation and planning files from the main checkout, so they had to be restored selectively before closeout work could proceed.
- The workspace-level test run surfaced a small `dsview-sys` unit-test seam that was invisible to the narrower target commands; fixing it kept the full suite green without changing runtime behavior.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Phase 5 now has the upstream-backed VCD replay path needed for artifact generation, plus stable export facts for metadata-side work in 05-02.
- `05-02` can build directly on the clean-success export result and final VCD path without reopening the sys replay design.

---
*Phase: 05-export-artifacts*
*Completed: 2026-04-07*
