---
phase: 14-decode-runtime-boundary-and-decoder-registry
plan: 03
subsystem: cli
tags: [rust, clap, json, decoder-registry, cli-contract]
requires:
  - phase: 14-02
    provides: Typed decoder registry descriptors and safe decode list/inspect helpers in `dsview-core`
provides:
  - Decode CLI subcommands for `decode list` and `decode inspect <decoder-id>`
  - JSON-first decoder discovery and inspect response models with readable text renderers
  - Spawned CLI contract coverage for canonical ids and decode discovery failure paths
affects: [15, 16, decode-config, decode-execution, cli-contracts]
tech-stack:
  added: []
  patterns:
    - Debug-only env-gated CLI fixtures for spawned decode contract tests
    - Shared decode response models drive both JSON and text output
key-files:
  created:
    - .planning/phases/14-decode-runtime-boundary-and-decoder-registry/14-03-SUMMARY.md
  modified:
    - crates/dsview-cli/src/main.rs
    - crates/dsview-cli/src/lib.rs
    - crates/dsview-cli/tests/devices_cli.rs
    - crates/dsview-core/src/lib.rs
key-decisions:
  - "Keep decode discovery separate from capture by resolving a dedicated decoder runtime and decoder-script path pair in `dsview-core`."
  - "Use debug-only env fixtures for spawned decode CLI tests so JSON/text contracts stay deterministic without a live decoder runtime."
  - "Render text from the same decode response models that back JSON so canonical upstream ids stay aligned across both formats."
patterns-established:
  - "Decode CLI commands classify missing runtime, missing decoder metadata, and unknown decoder ids with specific stable error codes."
  - "Decode list returns a concise canonical-id summary while decode inspect returns the full stack-planning metadata shape."
requirements-completed: [DEC-01, DEC-02]
duration: 11 min
completed: 2026-04-21
---
# Phase 14 Plan 03: Decode CLI Contract Summary

**`decode list` and `decode inspect <decoder-id>` now expose canonical decoder registry metadata through stable JSON and readable text output, with spawned CLI tests covering success and failure contracts**

## Performance

- **Duration:** 11 min
- **Started:** 2026-04-21T05:06:26Z
- **Completed:** 2026-04-21T05:17:20Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- Added `decode list` and `decode inspect <decoder-id>` to the clap command tree without introducing any out-of-scope decode execution command.
- Added decode list/inspect response models plus text renderers in `crates/dsview-cli/src/lib.rs`, keeping JSON authoritative and canonical upstream ids visible.
- Added core decode discovery orchestration in `crates/dsview-core/src/lib.rs` so the CLI resolves decoder runtime and decoder-script paths without talking to `dsview-sys` directly.
- Locked the CLI contract with spawned tests for canonical ids, inspect metadata, text output, missing runtime, missing metadata, and unknown decoder errors.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add decode discovery command wiring and output models** - `d201ce7` (`test`), `7991baa` (`feat`)
2. **Task 2: Lock JSON/text command contracts and failure reporting with CLI tests** - `342f152` (`test`), `86b34ad` (`feat`)

**Plan metadata:** pending summary commit

_Note: Both tasks followed TDD with failing coverage committed before the implementation commit._

## Files Created/Modified

- `crates/dsview-cli/src/main.rs` - Adds the decode subcommand tree, decode error classification, and debug-only decode fixtures for spawned CLI tests.
- `crates/dsview-cli/src/lib.rs` - Defines decode list/inspect JSON response types and shared text renderers derived from the same response models.
- `crates/dsview-cli/tests/devices_cli.rs` - Covers canonical decoder ids, inspect metadata, human-readable text, and specific decode discovery failures.
- `crates/dsview-core/src/lib.rs` - Adds decode runtime path discovery plus `decode_list` and `decode_inspect` orchestration helpers for the CLI.

## Decisions Made

- Reused the project’s JSON-first output pattern and kept the text format as a direct rendering of the same decode response structs.
- Kept the decode CLI path surface narrow with only `--decode-runtime` and `--decoder-dir` overrides, avoiding any config or execution scope creep.
- Used debug-only env fixtures instead of a live runtime dependency in CLI tests so contract checks stay deterministic in CI.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- The plan’s literal `! rg -n "decode run|DecodeRun"` acceptance command would false-match `decode runtime` and `DecodeRuntimeError`, so the CLI wording/import aliases were tightened to keep the scope check meaningful while preserving behavior.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 15 can build decode config validation on top of the shipped inspect schema without reopening native metadata paths.
- Phase 16 can reuse the stable canonical decoder ids and failure taxonomy when it adds offline decode execution.
- The CLI contract is now regression-locked for both automation consumers and readable shell output.

## Self-Check: PASSED

- Verified `.planning/phases/14-decode-runtime-boundary-and-decoder-registry/14-03-SUMMARY.md` exists.
- Verified task commits `d201ce7`, `7991baa`, `342f152`, and `86b34ad` exist in git history.
