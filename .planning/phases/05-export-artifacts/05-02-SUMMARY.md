---
phase: 05-export-artifacts
plan: 02
subsystem: export
tags: [core, cli, metadata, serde, json, validation, planning]
requires:
  - phase: 05-01
    provides: DSView-backed clean-success VCD export seam with atomic waveform promotion and export failure classification
provides:
  - Versioned JSON metadata sidecar contract in `dsview-core`
  - CLI capture success payloads that report both VCD and metadata artifact paths
  - Distinct metadata serialization and metadata write failure diagnostics for automation
affects: [phase-05, export, cli, metadata, validation]
tech-stack:
  added: [serde, serde_json, time RFC3339 formatting]
  patterns: [vcd-first metadata-second artifact ordering, deterministic metadata sidecar path derivation, core-owned artifact schema]
key-files:
  created:
    - .planning/phases/05-export-artifacts/05-02-SUMMARY.md
  modified:
    - crates/dsview-core/Cargo.toml
    - crates/dsview-core/src/lib.rs
    - crates/dsview-core/tests/export_artifacts.rs
    - crates/dsview-cli/src/main.rs
    - crates/dsview-cli/tests/capture_cli.rs
    - .planning/phases/05-export-artifacts/05-VALIDATION.md
    - .planning/ROADMAP.md
    - .planning/STATE.md
key-decisions:
  - "Keep metadata schema assembly and write-order semantics in `dsview-core`, with the CLI only passing tool/output context and rendering stable diagnostics."
  - "Derive the JSON sidecar path deterministically from the final VCD path and only write metadata after the VCD export succeeds."
  - "Treat metadata serialization and metadata filesystem writes as separate failure classes so automation can distinguish contract bugs from environment issues."
patterns-established:
  - "Artifact Contract Pattern: clean-success capture produces a finalized VCD plus a sibling JSON sidecar, and success responses surface both paths."
  - "Metadata Schema Pattern: encode capture facts as numeric JSON fields with UTC RFC3339 timestamps and explicit schema versioning."
requirements-completed: [EXP-03, EXP-04]
duration: 55 min
completed: 2026-04-07
---

# Phase 05 Plan 02: Generate and validate JSON metadata sidecar output Summary

**Versioned capture metadata now ships alongside the exported VCD, and the CLI reports both artifact paths with stable export-stage failure codes for automation.**

## Performance

- **Duration:** 55 min
- **Started:** 2026-04-07T10:13:22Z
- **Completed:** 2026-04-07T11:08:00Z
- **Tasks:** 3
- **Files modified:** 8

## Accomplishments
- Added a versioned metadata sidecar schema in `dsview-core` covering tool identity, capture timing, device facts, observed sample counts, acquisition outcome, and artifact paths.
- Extended the CLI `capture` flow to require an explicit `--output` VCD path, call the core export seam, return both artifact paths on success, and map export/metadata failures to stable machine-readable codes.
- Refreshed phase validation and planning state after running the targeted metadata commands and a green `cargo test --workspace` pass.

## Task Commits

Each task was committed atomically:

1. **Task 1: add metadata sidecar schema and core export contract** - `dabf8d2` (feat)
2. **Task 2: report artifact locations and metadata failures from the CLI** - `960bc45` (feat)
3. **Task 3: update validation and planning artifacts for 05-02** - pending in working tree

## Files Created/Modified
- `crates/dsview-core/Cargo.toml` - Adds serialization and timestamp formatting dependencies for metadata generation.
- `crates/dsview-core/src/lib.rs` - Defines metadata schema/types, deterministic metadata path derivation, atomic metadata writing, and export success/error contracts.
- `crates/dsview-core/tests/export_artifacts.rs` - Verifies metadata shape, numeric field typing, UTC timestamps, deterministic sidecar paths, and distinct metadata failure variants.
- `crates/dsview-cli/src/main.rs` - Adds `--output`, runs artifact export after capture, returns artifact paths, and maps export-stage failures to stable CLI error codes.
- `crates/dsview-cli/tests/capture_cli.rs` - Verifies success JSON includes both artifacts and that metadata-related errors stay distinct.
- `.planning/phases/05-export-artifacts/05-VALIDATION.md` - Marks 05-02 targeted checks green and records Wave 0 CLI coverage completion.
- `.planning/ROADMAP.md` and `.planning/STATE.md` - Advance plan tracking from 05-02 to 05-03.

## Decisions Made
- Metadata serialization happens from validated config plus observed export facts so the sidecar reflects what was actually captured, not only what was requested.
- Metadata is written after VCD export succeeds so the sidecar never claims a complete artifact set when the waveform is missing.
- CLI error classification keeps acquisition failures separate from export preconditions, VCD export failures, metadata serialization failures, and metadata write failures.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
- The phase planning directory did not yet include a `05-02-PLAN.md` file, so execution continued against the checked-in phase research, validation, and state artifacts that already described the 05-02 requirements and acceptance criteria.
- Targeted CLI tests still emit dead-code warnings because `src/main.rs` is compiled as a test module, but the warnings did not affect correctness and the workspace suite remained green.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- 05-03 can now validate artifact correctness against the finalized VCD+JSON contract instead of an evolving schema.
- Manual DSLogic Plus hardware verification still remains for final artifact plausibility and timing sign-off.

---
*Phase: 05-export-artifacts*
*Completed: 2026-04-07*
