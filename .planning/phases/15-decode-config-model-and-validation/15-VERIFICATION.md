---
phase: 15-decode-config-model-and-validation
verified: 2026-04-21T09:19:51Z
status: passed
score: 8/8 must-haves verified
overrides_applied: 0
---

# Phase 15: Decode Config Model and Validation Verification Report

**Phase Goal:** Create a config-driven decoder stack model that stays aligned with DSView concepts and validates before runtime execution.
**Verified:** 2026-04-21T09:19:51Z
**Status:** passed
**Re-verification:** No - initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Users can express root decoder, stacked decoders, channel bindings, and typed options in a decode config file. | ✓ VERIFIED | Typed config structs live in `crates/dsview-core/src/lib.rs:278`, `crates/dsview-core/src/lib.rs:288`, `crates/dsview-core/src/lib.rs:298`, and `crates/dsview-core/src/lib.rs:306`; parsing is covered by `crates/dsview-core/tests/decode_config.rs:13`, `crates/dsview-core/tests/decode_config.rs:45`, and `crates/dsview-core/tests/decode_config.rs:69`. |
| 2 | Decode option metadata exposes explicit value kinds without inventing alias ids. | ✓ VERIFIED | Raw/native value kinds are defined in `crates/dsview-sys/wrapper.h:178`, derived from upstream `GVariant` defaults in `crates/dsview-sys/bridge_runtime.c:564`, exposed safely in `crates/dsview-sys/src/lib.rs:523`, and exercised against real decoder metadata in `crates/dsview-sys/tests/boundary.rs:272`; `rg -n "stable_id|token" crates/dsview-sys/src/lib.rs crates/dsview-sys/wrapper.h` returned no matches. |
| 3 | The Rust config model is shaped for later offline execution, not as a throwaway parse shape. | ✓ VERIFIED | `ValidatedDecodeConfig` and its validated decoder/stack entries are defined in `crates/dsview-core/src/lib.rs:351`, `crates/dsview-core/src/lib.rs:358`, and `crates/dsview-core/src/lib.rs:365`; shared file loading returns that validated shape from `crates/dsview-core/src/lib.rs:1274`. |
| 4 | Validation rejects missing required channels, unknown options, invalid option values, and invalid stack composition before decode starts, with no warning-only continuation. | ✓ VERIFIED | Strict validation happens in `crates/dsview-core/src/lib.rs:435`, `crates/dsview-core/src/lib.rs:502`, `crates/dsview-core/src/lib.rs:534`, and `crates/dsview-core/src/lib.rs:576`; regression coverage exists at `crates/dsview-core/tests/decode_config.rs:120`, `crates/dsview-core/tests/decode_config.rs:148`, `crates/dsview-core/tests/decode_config.rs:177`, and `crates/dsview-core/tests/decode_config.rs:208`; `rg -n "warning" crates/dsview-core/src/lib.rs` returned no matches. |
| 5 | Linear stack compatibility is enforced from canonical decoder inputs/outputs metadata. | ✓ VERIFIED | Stack linking compares upstream outputs to downstream inputs in `crates/dsview-core/src/lib.rs:576`, using canonical descriptor metadata normalized from discovery in `crates/dsview-core/src/lib.rs:183`; passing and failing stack cases are covered by `crates/dsview-core/tests/decode_config.rs:208` and `crates/dsview-core/tests/decode_config.rs:245`. |
| 6 | Validation distinguishes schema/config failures, metadata/compatibility failures, and runtime prerequisite failures. | ✓ VERIFIED | Load/parse/validation families are separated in `crates/dsview-core/src/lib.rs:422` and `crates/dsview-core/src/lib.rs:1274`; CLI mapping keeps runtime discovery errors on the existing decode path in `crates/dsview-cli/src/main.rs:1210`, parse/schema codes in `crates/dsview-cli/src/main.rs:1354`, validation codes in `crates/dsview-cli/src/main.rs:1381`, and final decode-validate dispatch in `crates/dsview-cli/src/main.rs:1464`. |
| 7 | Users can run `decode validate --config <PATH>` and receive the project's JSON-first success/failure contract. | ✓ VERIFIED | The subcommand is defined in `crates/dsview-cli/src/main.rs:86` and `crates/dsview-cli/src/main.rs:130`, delegates to core validation in `crates/dsview-cli/src/main.rs:451`, builds a JSON response in `crates/dsview-cli/src/lib.rs:47` and `crates/dsview-cli/src/lib.rs:202`, renders text in `crates/dsview-cli/src/lib.rs:336`, and is exercised end-to-end in `crates/dsview-cli/tests/decode_cli.rs:45`, `crates/dsview-cli/tests/decode_cli.rs:104`, `crates/dsview-cli/tests/decode_cli.rs:133`, and `crates/dsview-cli/tests/decode_cli.rs:169`. |
| 8 | Decoder-specific configuration stays out of `capture`; Phase 15 adds validation only, not decode execution. | ✓ VERIFIED | `decode validate` is isolated under `DecodeCommand::Validate` in `crates/dsview-cli/src/main.rs:86`, while `CaptureArgs` remains limited to runtime/capture/device-option parameters in `crates/dsview-cli/src/main.rs:189`; `run_decode_validate` only validates and renders a response in `crates/dsview-cli/src/main.rs:451` and never starts decode execution. |

**Score:** 8/8 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `crates/dsview-sys/wrapper.h` | Raw decode option value-kind enum and field | ✓ VERIFIED | `enum dsview_decode_option_value_kind` and `value_kind` exist at `crates/dsview-sys/wrapper.h:178` and `crates/dsview-sys/wrapper.h:185`. |
| `crates/dsview-sys/bridge_runtime.c` | Upstream default-type mapping into raw option metadata | ✓ VERIFIED | `dsview_decode_option_value_kind_from_variant()` maps string/int/float defaults and stores the result at `crates/dsview-sys/bridge_runtime.c:564` and `crates/dsview-sys/bridge_runtime.c:625`. |
| `crates/dsview-sys/src/lib.rs` | Safe Rust decode option value-kind metadata | ✓ VERIFIED | `DecodeOption` and `DecodeOptionValueKind` are defined at `crates/dsview-sys/src/lib.rs:523`, with raw conversion at `crates/dsview-sys/src/lib.rs:2097`. |
| `crates/dsview-sys/tests/boundary.rs` | Boundary regression for upstream option typing | ✓ VERIFIED | `decode_option_value_kind_follows_upstream_default_type` verifies string/integer/float metadata against runtime discovery at `crates/dsview-sys/tests/boundary.rs:272`. |
| `crates/dsview-core/src/lib.rs` | Typed config schema, validated config model, validator, and file-loading entrypoint | ✓ VERIFIED | Schema/validated types live at `crates/dsview-core/src/lib.rs:278` and `crates/dsview-core/src/lib.rs:351`; strict validation lives at `crates/dsview-core/src/lib.rs:435`; file loading lives at `crates/dsview-core/src/lib.rs:1274`. |
| `crates/dsview-core/tests/decode_config.rs` | Coverage for config parsing and validator failures | ✓ VERIFIED | Parsing tests start at `crates/dsview-core/tests/decode_config.rs:13`; validation regressions start at `crates/dsview-core/tests/decode_config.rs:120`; file-ordering tests live at `crates/dsview-core/tests/decode_config.rs:277`. |
| `crates/dsview-cli/src/main.rs` | `decode validate` wiring and stable error classification | ✓ VERIFIED | Command wiring is in `crates/dsview-cli/src/main.rs:86`, `crates/dsview-cli/src/main.rs:130`, and `crates/dsview-cli/src/main.rs:451`; stable code mapping is in `crates/dsview-cli/src/main.rs:1354` and `crates/dsview-cli/src/main.rs:1381`. |
| `crates/dsview-cli/src/lib.rs` | Decode validation response model and text renderer | ✓ VERIFIED | `DecodeValidateResponse` is defined at `crates/dsview-cli/src/lib.rs:47`; builder and text renderer are at `crates/dsview-cli/src/lib.rs:202` and `crates/dsview-cli/src/lib.rs:336`. |
| `crates/dsview-cli/tests/decode_cli.rs` | Spawned CLI contract coverage for valid and invalid configs | ✓ VERIFIED | Valid, missing-channel, incompatible-stack, and schema-invalid cases are covered at `crates/dsview-cli/tests/decode_cli.rs:45`, `crates/dsview-cli/tests/decode_cli.rs:104`, `crates/dsview-cli/tests/decode_cli.rs:133`, and `crates/dsview-cli/tests/decode_cli.rs:169`. |

### Key Link Verification

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| `crates/dsview-sys/src/lib.rs` | `crates/dsview-core/src/lib.rs` | Typed decoder option metadata flows into config validation without remapping ids | WIRED | `DecodeOption.value_kind` is exposed at `crates/dsview-sys/src/lib.rs:523`, normalized into `DecoderOptionDescriptor.value_kind` at `crates/dsview-core/src/lib.rs:244`, and enforced in `crates/dsview-core/src/lib.rs:550`. |
| `crates/dsview-core/src/lib.rs` | `crates/dsview-cli/src/main.rs` | Core validation errors map cleanly into stable CLI codes | WIRED | `DecodeConfigValidationError` is defined at `crates/dsview-core/src/lib.rs:372`, and each variant is classified in `crates/dsview-cli/src/main.rs:1381`. |
| `crates/dsview-cli/src/main.rs` | `crates/dsview-core/src/lib.rs` | CLI validation command delegates to the strict core validator | WIRED | `run_decode_validate()` calls `core_validate_decode_config_file()` at `crates/dsview-cli/src/main.rs:451`, which loads, parses, discovers metadata, and validates in `crates/dsview-core/src/lib.rs:1274`. |
| `crates/dsview-cli/src/main.rs` | `crates/dsview-cli/src/lib.rs` | Validated config data reaches JSON/text output renderers | WIRED | `run_decode_validate()` builds `DecodeValidateResponse` at `crates/dsview-cli/src/main.rs:472`, and success rendering dispatches to `render_decode_validate_text()` at `crates/dsview-cli/src/main.rs:2338` and `crates/dsview-cli/src/lib.rs:336`. |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| --- | --- | --- | --- | --- |
| `crates/dsview-sys/src/lib.rs` | `DecodeOption.value_kind` | `RawDecodeOption.value_kind` from `crates/dsview-sys/bridge_runtime.c:625` after `GVariant` classification at `crates/dsview-sys/bridge_runtime.c:564` | Yes | ✓ FLOWING |
| `crates/dsview-core/src/lib.rs` | `ValidatedDecodeConfig` | `fs::read()` -> `parse_decode_config_slice()` -> `decode_list()` -> `validate_decode_config()` in `crates/dsview-core/src/lib.rs:1274` | Yes | ✓ FLOWING |
| `crates/dsview-cli/src/main.rs` | `bound_channel_ids`, `validated.stack.len()` | `ValidatedDecodeConfig` returned by `core_validate_decode_config_file()` in `crates/dsview-cli/src/main.rs:456` | Yes | ✓ FLOWING |
| `crates/dsview-cli/src/lib.rs` | `DecodeValidateResponse` fields | Builder inputs supplied by `run_decode_validate()` at `crates/dsview-cli/src/main.rs:472` and copied into the response at `crates/dsview-cli/src/lib.rs:202` | Yes | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| Upstream option metadata yields typed value kinds | `cargo test -p dsview-sys --test boundary decode_option_value_kind_follows_upstream_default_type -- --nocapture` | `1 passed; 0 failed` | ✓ PASS |
| Core parsing and strict validation cover config success/failure paths | `cargo test -p dsview-core --test decode_config -- --nocapture` | `10 passed; 0 failed` | ✓ PASS |
| CLI `decode validate` handles valid and invalid configs end-to-end | `cargo test -p dsview-cli --test decode_cli -- --nocapture` | `4 passed; 0 failed` | ✓ PASS |
| Stable decode-config error codes stay locked in the binary | `cargo test -p dsview-cli --bin dsview-cli validation_error_codes_remain_stable_for_decode_config_failures -- --nocapture` | `1 passed; 0 failed` | ✓ PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| `DEC-03` | `15-01-PLAN.md`, `15-03-PLAN.md` | User can define a decoder stack, channel bindings, and decoder options in a decode configuration file. | ✓ SATISFIED | Typed config schema is defined in `crates/dsview-core/src/lib.rs:278`; parsing/shape coverage lives in `crates/dsview-core/tests/decode_config.rs:13`; CLI file-based validation entrypoint is exposed in `crates/dsview-cli/src/main.rs:130` and exercised in `crates/dsview-cli/tests/decode_cli.rs:45`. |
| `DEC-04` | `15-02-PLAN.md`, `15-03-PLAN.md` | CLI validates a decode configuration against the selected decoder metadata before execution starts. | ✓ SATISFIED | Metadata-driven validation and error taxonomy live in `crates/dsview-core/src/lib.rs:372` and `crates/dsview-core/src/lib.rs:435`; CLI mapping lives in `crates/dsview-cli/src/main.rs:1381`; end-to-end failure cases are covered in `crates/dsview-cli/tests/decode_cli.rs:104`, `crates/dsview-cli/tests/decode_cli.rs:133`, and `crates/dsview-cli/tests/decode_cli.rs:169`. |

No orphaned Phase 15 requirements were found in `.planning/REQUIREMENTS.md`; the only Phase 15 requirement IDs are `DEC-03` and `DEC-04`, and both appear in the phase plans.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| --- | --- | --- | --- | --- |
| None | - | No TODO/FIXME/placeholders, empty handlers, or stub returns were found in the scanned phase files. | - | No blocker anti-patterns detected across the Phase 15 implementation files. |

### Gaps Summary

No actionable gaps found. Phase 15 achieves the roadmap goal and satisfies both `DEC-03` and `DEC-04`.

Residual test risk from the disconfirmation pass: `InvalidOptionValue` allow-list rejection is implemented in `crates/dsview-core/src/lib.rs:560` and mapped to a stable CLI code in `crates/dsview-cli/src/main.rs:1438`, but the current regression suite does not include a dedicated out-of-allowlist config fixture. This does not block the phase goal because the code path exists and the surrounding validation/CLI contract is otherwise exercised.

---

_Verified: 2026-04-21T09:19:51Z_
_Verifier: Claude (gsd-verifier)_
