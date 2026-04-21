---
phase: 15-decode-config-model-and-validation
plan: 03
subsystem: cli
tags: [rust, clap, serde_json, assert_cmd, decode-validation]
requires:
  - phase: 15-02
    provides: strict decode config parsing and metadata-driven validation
provides:
  - `decode validate --config <PATH>` command wiring
  - stable JSON/text validation success rendering
  - spawned CLI contract coverage for valid and invalid decode configs
affects: [phase-16-decode-execution, decode-cli, validation-contracts]
tech-stack:
  added: []
  patterns:
    - response-builder plus text-renderer split for decode validation output
    - debug-only decode fixture modes for spawned CLI contract tests
key-files:
  created:
    - .planning/phases/15-decode-config-model-and-validation/15-03-SUMMARY.md
    - crates/dsview-cli/tests/decode_cli.rs
  modified:
    - crates/dsview-cli/src/lib.rs
    - crates/dsview-cli/src/main.rs
    - crates/dsview-core/src/lib.rs
    - crates/dsview-core/tests/decode_config.rs
key-decisions:
  - "Keep decode config file loading in `dsview-core` so file, parse, and validation ordering stays shared between CLI flows."
  - "Use a debug-only `validation-registry` fixture mode so spawned CLI tests exercise real validator behavior without packaged runtime dependencies."
patterns-established:
  - "Decode validation success follows the existing JSON-first CLI contract with text as a renderer only."
  - "Spawned decode CLI tests should prefer fixture-backed metadata seams over duplicating validation logic in the harness."
requirements-completed: [DEC-03, DEC-04]
duration: 8min
completed: 2026-04-21
---

# Phase 15 Plan 03: Decode Config Validation CLI Summary

**Validation-only decode config CLI wiring with stable JSON/text success summaries and spawned failure-contract coverage**

## Performance

- **Duration:** 8 min
- **Started:** 2026-04-21T09:03:46Z
- **Completed:** 2026-04-21T09:11:24Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments
- Added `decode validate --config <PATH>` so decode configs can be preflighted without introducing any decode execution surface.
- Added `DecodeValidateResponse` and text rendering that report config version, root decoder id, stack depth, and bound channel ids.
- Locked valid/invalid CLI behavior with spawned tests and core file-loading tests so schema, metadata, and stack failures keep stable machine-readable codes.

## Task Commits

Each task was committed atomically through TDD:

1. **Task 1: Add the decode validation CLI command and response models**
   - `5ebfd31` `test(15-03): add failing test for decode validation response`
   - `14ac447` `feat(15-03): add decode config validation command`
2. **Task 2: Lock CLI config validation success and failure behavior with tests**
   - `67cec44` `test(15-03): add failing decode validate cli coverage`
   - `6ced406` `feat(15-03): lock decode validate cli contracts`

## Files Created/Modified
- `crates/dsview-cli/src/lib.rs` - added decode validation response modeling and text rendering helpers.
- `crates/dsview-cli/src/main.rs` - added the `decode validate` subcommand, error classification, and debug-only validation fixture seam.
- `crates/dsview-core/src/lib.rs` - added shared decode config file loading and validation helper with distinct file/read/parse/validation errors.
- `crates/dsview-cli/tests/decode_cli.rs` - added spawned CLI contract tests for valid, missing-channel, incompatible-stack, and schema-invalid configs.
- `crates/dsview-core/tests/decode_config.rs` - added coverage proving config file missing/schema failures surface before runtime discovery.

## Decisions Made
- Kept decode config file reading in `dsview-core` so CLI callers reuse one authoritative load/parse/validate order.
- Added a debug-only `validation-registry` fixture mode instead of broad test-only flags so release behavior stays unchanged while spawned tests remain deterministic.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 16 can build decode execution on top of a shipped `decode validate` surface and its stable success/error contracts.
- The validation helper and fixture-backed tests give future decode execution work a safe regression baseline without expanding `capture`.

## Self-Check: PASSED

- Verified `.planning/phases/15-decode-config-model-and-validation/15-03-SUMMARY.md` exists on disk.
- Verified task commits `5ebfd31`, `14ac447`, `67cec44`, and `6ced406` exist in git history.

---
*Phase: 15-decode-config-model-and-validation*
*Completed: 2026-04-21*
