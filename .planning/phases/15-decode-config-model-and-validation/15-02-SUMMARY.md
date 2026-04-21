---
phase: 15-decode-config-model-and-validation
plan: 02
subsystem: validation
tags: [rust, serde, protocol-decode, validation, cli-errors]
requires:
  - phase: 15-01
    provides: typed decode config schema and decoder option metadata
  - phase: 14-decode-runtime-boundary-and-decoder-registry
    provides: canonical decoder inputs, outputs, channels, and option ids
provides:
  - strict decode config validation before execution begins
  - metadata-driven linear stack compatibility checks
  - stable CLI-facing decode config parse and validation error codes
affects: [15-03, decode-validation-command, decode-execution]
tech-stack:
  added: []
  patterns: [metadata-driven validation, strict preflight failure taxonomy, TDD]
key-files:
  created: []
  modified:
    - crates/dsview-core/src/lib.rs
    - crates/dsview-core/tests/decode_config.rs
    - crates/dsview-cli/src/main.rs
    - crates/dsview-cli/src/lib.rs
key-decisions:
  - "Derive stack compatibility strictly from canonical decoder outputs and downstream inputs."
  - "Keep decode config parse/schema diagnostics separate from decode runtime bring-up errors."
  - "Propagate decoder option value kinds into the normalized core metadata so option typing stays metadata-driven."
patterns-established:
  - "Decode configs fail closed: malformed schema, unknown ids, bad option values, and incompatible stacks all stop execution before runtime."
  - "CLI error codes stay stable by classifying parse, schema, metadata, and stack failures through dedicated decode-config helpers."
requirements-completed: [DEC-04]
duration: 6 min
completed: 2026-04-21
---
# Phase 15 Plan 02: Strict Decode Config Validation Summary

**Metadata-driven decode config validation with typed option checks, linear stack compatibility enforcement, and stable CLI failure codes**

## Performance

- **Duration:** 6 min
- **Started:** 2026-04-21T16:50:57+08:00
- **Completed:** 2026-04-21T16:57:04+08:00
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Added `validate_decode_config` and `DecodeConfigValidationError` so decode configs now fail before execution on schema/version, metadata, channel, option, and stack problems.
- Enforced typed option validation and allowed-value checks from canonical decoder metadata instead of string-only guesswork.
- Added CLI-side classifiers for decode config parse and validation failures with stable machine-readable codes that stay separate from runtime bring-up diagnostics.

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement strict decode config validation in core**
   - `1d4e1b9` (`test`): failing validation regressions for channels, options, and linear stacks
   - `73b5ec7` (`feat`): strict metadata-driven validator and validation taxonomy
2. **Task 2: Map validation failures into stable CLI-oriented diagnostics**
   - `a1c4d04` (`test`): failing stable-code contract for decode config parse and validation errors
   - `a3d68ac` (`feat`): CLI classifiers for parse/schema/metadata/stack failures

## Files Created/Modified
- `crates/dsview-core/src/lib.rs` - Stores decoder option value kinds, splits parse errors, validates root and stacked decoders, and returns typed validation failures.
- `crates/dsview-core/tests/decode_config.rs` - Covers missing channels, unknown options, type mismatches, incompatible stacks, and valid linear stacks.
- `crates/dsview-cli/src/main.rs` - Maps decode config parse and validation failures to stable CLI error codes and messages.
- `crates/dsview-cli/src/lib.rs` - Updates the decoder fixture literal to match the normalized option metadata contract.

## Decisions Made
- Reused Phase 14 `inputs` and `outputs` ids as the only stack-compatibility source so validation cannot drift from discovery metadata.
- Kept runtime-precondition reporting on the existing decode bring-up path instead of collapsing it into schema or validation errors.
- Treated option type validation as a core responsibility by carrying `DecodeOptionValueKind` through normalization before any CLI mapping occurs.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Propagated decoder option value kinds into core metadata**
- **Found during:** Task 1
- **Issue:** `DecoderOptionDescriptor` still dropped upstream option value kinds, which made strict typed option validation impossible.
- **Fix:** Re-exported `DecodeOptionValueKind`, stored it on normalized decoder option descriptors, and validated config option JSON kinds against that metadata.
- **Files modified:** `crates/dsview-core/src/lib.rs`
- **Verification:** `cargo test -p dsview-core --test decode_config -- --nocapture`
- **Committed in:** `73b5ec7`

**2. [Rule 3 - Blocking] Updated CLI decoder fixtures for the new metadata shape**
- **Found during:** Task 2
- **Issue:** The CLI sample decoder literals were missing the new `value_kind` field and blocked the binary test target from compiling.
- **Fix:** Added `value_kind` to the decoder fixture literals in the CLI binary and library test helpers without changing the inspect response surface.
- **Files modified:** `crates/dsview-cli/src/main.rs`, `crates/dsview-cli/src/lib.rs`
- **Verification:** `cargo test -p dsview-cli --bin dsview-cli -- --nocapture`
- **Committed in:** `a3d68ac`

---

**Total deviations:** 2 auto-fixed (1 Rule 2, 1 Rule 3)
**Impact on plan:** All deviations were required for correctness and compilation; no feature scope expanded beyond strict validation and stable diagnostics.

## Issues Encountered
None.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- `ValidatedDecodeConfig` and `DecodeConfigValidationError` are ready for a dedicated validation command or decode-execution preflight step.
- CLI decode-config classifiers are ready to surface stable codes/messages once the validation command is wired.
- No blockers remain for Phase 15 Plan 03 beyond command wiring and output/reporting integration.

## Self-Check: PASSED

- Verified `.planning/phases/15-decode-config-model-and-validation/15-02-SUMMARY.md` exists.
- Verified task commits `1d4e1b9`, `73b5ec7`, `a1c4d04`, and `a3d68ac` are present in git history.

---
*Phase: 15-decode-config-model-and-validation*
*Completed: 2026-04-21*
