---
phase: 15-decode-config-model-and-validation
plan: 01
subsystem: api
tags: [rust, serde, json, ffi, libsigrokdecode, decode-config]
requires:
  - phase: 14-decode-runtime-boundary-and-decoder-registry
    provides: decoder discovery metadata, canonical decoder ids, and decode inspect/list runtime plumbing
provides:
  - typed decode option value-kind metadata at the dsview-sys boundary
  - a JSON-first decode config schema with root decoder, numeric channel bindings, and a linear stack
  - parsing coverage for typed option values and normalized config defaults
affects: [15-02-validation, 15-03-cli-validate, 16-offline-decode-execution]
tech-stack:
  added: []
  patterns: [typed FFI option metadata, serde-driven JSON config normalization, canonical-id keyed BTreeMap bindings]
key-files:
  created: [crates/dsview-core/tests/decode_config.rs]
  modified:
    [
      crates/dsview-sys/wrapper.h,
      crates/dsview-sys/bridge_runtime.c,
      crates/dsview-sys/src/lib.rs,
      crates/dsview-sys/tests/boundary.rs,
      crates/dsview-core/src/lib.rs,
      crates/dsview-core/tests/device_options.rs,
    ]
key-decisions:
  - "Default the machine-facing decode config version to 1 so JSON authors can omit it without changing the normalized internal shape."
  - "Model channel bindings and typed options as canonical-id keyed BTreeMaps so later validation/execution phases can consume a stable, deterministic structure."
  - "Keep decode option defaults and allowed values as strings at the boundary while adding a separate value-kind enum, preserving upstream ids without inventing aliases."
patterns-established:
  - "Decode option metadata crossing the FFI boundary should expose explicit typing alongside the existing printable default/value strings."
  - "Decode config JSON should deserialize directly into normalized Rust structs with strict field checking and typed option enums."
requirements-completed: [DEC-03]
duration: 11 min
completed: 2026-04-21
---

# Phase 15 Plan 01: Decode Config Model and Validation Summary

**Typed decode option metadata plus a JSON-first root-decoder-and-stack config model ready for later validation and offline execution**

## Performance

- **Duration:** 11 min
- **Started:** 2026-04-21T08:32:38Z
- **Completed:** 2026-04-21T08:44:01Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments
- Added `DecodeOptionValueKind` across the C/Rust decode metadata boundary without changing canonical upstream option ids.
- Added boundary coverage proving string, integer, and float decoder options retain the expected typed classification.
- Defined a strict JSON decode config contract with `decoder`, `channels`, `options`, and ordered `stack` fields backed by typed Rust enums/maps.
- Added parsing tests that lock version defaulting, numeric channel bindings, and typed JSON option preservation.

## Task Commits

Each task was committed atomically:

1. **Task 1: Extend decode option metadata with explicit value kinds** - `8aeab7f` (feat)
2. **Task 2: Define and parse the JSON decode config model in core** - `a8a54fe` (feat)

**Plan metadata:** handled by a separate summary-only docs commit after self-check

## Files Created/Modified
- `crates/dsview-sys/wrapper.h` - Adds the raw decode option value-kind enum and FFI field.
- `crates/dsview-sys/bridge_runtime.c` - Derives string/integer/float kinds from upstream `GVariant` defaults.
- `crates/dsview-sys/src/lib.rs` - Exposes `DecodeOptionValueKind` and decodes it into the safe Rust option model.
- `crates/dsview-sys/tests/boundary.rs` - Verifies typed option-kind mapping against real decoder metadata fixtures.
- `crates/dsview-core/src/lib.rs` - Defines the serde-backed decode config schema and parse helpers.
- `crates/dsview-core/tests/decode_config.rs` - Covers linear stack parsing, numeric channel bindings, and typed option values.
- `crates/dsview-core/tests/device_options.rs` - Keeps the existing decoder fixture aligned with the expanded sys option shape.

## Decisions Made
- Defaulted `version` to `1` to keep the normalized config shape stable while letting JSON omit the field.
- Used `BTreeMap<String, u32>` for channel bindings and `BTreeMap<String, DecodeOptionValue>` for options so later validation/execution can iterate deterministically.
- Rejected permissive shape drift by using serde `deny_unknown_fields` on decode config structs.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Restored deterministic loader-failure behavior for decode runtime tests**
- **Found during:** Task 1 (Extend decode option metadata with explicit value kinds)
- **Issue:** The full `dsview-sys` boundary suite exposed that `DecodeRuntimeBridge::load` could report success for a missing path once the global decode bridge had already been loaded.
- **Fix:** Added a fail-fast file existence check before calling the native decode runtime loader.
- **Files modified:** `crates/dsview-sys/src/lib.rs`
- **Verification:** `cargo test -p dsview-sys --test boundary -- --nocapture`
- **Committed in:** `8aeab7f` (part of Task 1 commit)

**2. [Rule 3 - Blocking] Updated downstream core fixture for the expanded sys decode option shape**
- **Found during:** Task 2 (Define and parse the JSON decode config model in core)
- **Issue:** The existing `DecodeOption` fixture in `dsview-core` no longer compiled after Task 1 added `value_kind`.
- **Fix:** Added `DecodeOptionValueKind::String` to the existing fixture so downstream tests stayed in sync with the new boundary contract.
- **Files modified:** `crates/dsview-core/tests/device_options.rs`
- **Verification:** `cargo test -p dsview-core --test device_options -- --nocapture`
- **Committed in:** `a8a54fe` (part of Task 2 commit)

---

**Total deviations:** 2 auto-fixed (2 blocking)
**Impact on plan:** Both fixes were required to keep the planned work verifiable and downstream-compatible. No scope creep.

## Issues Encountered
- Sandbox restrictions blocked direct `git commit` writes to `.git/index.lock`; task commits were retried with escalation and succeeded.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Phase 15-02 can now validate JSON option values against typed decoder metadata instead of guessing from printed strings.
- Phase 16 can consume `DecodeConfig` directly for offline decode execution without first translating a GUI-shaped persistence format.
- No functional blockers remain for the planned validation work.

## Self-Check: PASSED

- Verified `.planning/phases/15-decode-config-model-and-validation/15-01-SUMMARY.md` exists.
- Verified task commits `8aeab7f` and `a8a54fe` exist in git history.

---
*Phase: 15-decode-config-model-and-validation*
*Completed: 2026-04-21*
