---
phase: 12-cli-device-option-surface
verified: 2026-04-13T09:02:54Z
status: passed
score: 9/9 must-haves verified
overrides_applied: 0
---

# Phase 12: CLI Device Option Surface Verification Report

**Phase Goal:** Let users choose the relevant DSView-compatible `DSLogic Plus` device options directly from the CLI without relying on GUI profiles.
**Verified:** 2026-04-13T09:02:54Z
**Status:** passed
**Re-verification:** No - initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Users can select buffer/stream mode, stop option, channel mode, enabled channels, threshold voltage, and filter selection from the CLI. | ✓ VERIFIED | `capture` exposes `--operation-mode`, `--stop-option`, `--channel-mode`, `--threshold-volts`, `--filter`, and keeps `--channels` on the same command surface in `crates/dsview-cli/src/main.rs:166`; resolver maps them into a validation request in `crates/dsview-cli/src/capture_device_options.rs:206`; spawned acceptance/error coverage exists in `crates/dsview-cli/tests/capture_cli.rs:62`. |
| 2 | CLI help and diagnostics make valid values and mode-dependent constraints understandable without opening DSView. | ✓ VERIFIED | Help text for every new flag points back to `devices options --handle <HANDLE>` in `crates/dsview-cli/src/main.rs:167`; parse diagnostics provide stable codes plus guidance in `crates/dsview-cli/src/main.rs:894`; inspection text lists copy-paste flags, threshold range/step, and channel-mode limits in `crates/dsview-cli/src/device_options.rs:139`. |
| 3 | The option-selection surface stays non-interactive and script-friendly for shell and agent workflows. | ✓ VERIFIED | The feature is entirely flag-driven (`crates/dsview-cli/src/main.rs:166`), `devices options` stays JSON/text renderable via `DeviceOptionsResponse` in `crates/dsview-cli/src/device_options.rs:61`, and spawned CLI tests assert deterministic stdout/stderr contracts in `crates/dsview-cli/tests/capture_cli.rs:81` and `crates/dsview-cli/tests/device_options_cli.rs:158`. |
| 4 | `devices options --handle <HANDLE>` remains the authoritative inspection entrypoint and shows copy-pasteable capture tokens plus channel-count guidance. | ✓ VERIFIED | `run_options` still uses `inspect_device_options(handle)` and builds the response in `crates/dsview-cli/src/main.rs:448`; text rendering leads with `capture_flags` and a `--channels IDX[,IDX...]` hint tied to the current channel-mode limit in `crates/dsview-cli/src/device_options.rs:139` and `crates/dsview-cli/src/device_options.rs:320`; locked by `crates/dsview-cli/tests/device_options_cli.rs:328`. |
| 5 | Inspection JSON remains machine-readable while adding friendly CLI tokens alongside stable IDs. | ✓ VERIFIED | `CurrentCaptureOptionValues`, `CliTokenOption`, `CliChannelModeOption`, and `DeviceOptionsResponse` all carry token + stable-ID fields in `crates/dsview-cli/src/device_options.rs:21` and `crates/dsview-cli/src/capture_device_options.rs:13`; JSON contract assertions lock both fields in `crates/dsview-cli/tests/device_options_cli.rs:158`. |
| 6 | Token generation is deterministic and human-readable. | ✓ VERIFIED | `slug_token(...)` normalizes labels to kebab-case and operation modes collapse to exact `buffer`/`stream` aliases in `crates/dsview-cli/src/capture_device_options.rs:74` and `crates/dsview-cli/src/capture_device_options.rs:160`; lookup maps centralize stable token reuse in `crates/dsview-cli/src/capture_device_options.rs:107`. |
| 7 | Omitted sibling flags preserve current device values, and child tokens infer their parent mode only when the inference is unique. | ✓ VERIFIED | Resolver fallbacks use `snapshot.current.*` when flags are omitted and infer the operation mode only from unique child-token matches in `crates/dsview-cli/src/capture_device_options.rs:221` and `crates/dsview-cli/src/capture_device_options.rs:282`; binary tests cover omission, inference, and explicit-mode precedence in `crates/dsview-cli/src/main.rs`. |
| 8 | Friendly tokens resolve back to Phase 10/11 stable IDs and are validated before the existing capture path continues. | ✓ VERIFIED | `resolve_capture_device_option_request(...)` emits a `DeviceOptionValidationRequest` with stable IDs and `enabled_channels` in `crates/dsview-cli/src/capture_device_options.rs:206`; `run_capture` performs this preflight before `validate_capture_config(...)` and `run_capture(...)` in `crates/dsview-cli/src/main.rs:364`. Inference from source: the code uses `load_device_option_validation_capabilities(...)` + `validate_request(...)`, which is semantically equivalent to the convenience wrapper `Discovery::validate_device_option_request(...)` shown in `crates/dsview-core/src/lib.rs:604`. |
| 9 | Phase 12 does not claim Phase 13 apply-time behavior. | ✓ VERIFIED | `run_capture` validates selection state, then continues through the pre-existing capture-config/run/export path in `crates/dsview-cli/src/main.rs:364` and `crates/dsview-cli/src/main.rs:397`; verification grep found no Phase-13-style apply/effective-option plumbing in the Phase 12 CLI files, and success output still reports generic capture completion/artifact facts only. |

**Score:** 9/9 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `crates/dsview-cli/src/lib.rs` | Export the Phase 12 token helper module for reuse by the CLI binary and tests. | ✓ VERIFIED | `pub mod capture_device_options;` and `pub use ...` export the new contract in `crates/dsview-cli/src/lib.rs:1`; `crates/dsview-cli/src/main.rs:8` imports from that surface. |
| `crates/dsview-cli/src/capture_device_options.rs` | Friendly-token contract, token lookup maps, stable-ID resolver, omission fallback, and parent-mode inference. | ✓ VERIFIED | Substantive module with response structs, parse errors, deterministic token builders, and validation-request resolver in `crates/dsview-cli/src/capture_device_options.rs:13` and `crates/dsview-cli/src/capture_device_options.rs:206`; wired from `crates/dsview-cli/src/device_options.rs:3` and `crates/dsview-cli/src/main.rs:8`. |
| `crates/dsview-cli/src/device_options.rs` | Capture-oriented inspection response plus deterministic text rendering for `devices options`. | ✓ VERIFIED | Response types carry token/stable-ID data and threshold/channel guidance in `crates/dsview-cli/src/device_options.rs:21` and `crates/dsview-cli/src/device_options.rs:61`; text renderer exposes copy-paste flags in `crates/dsview-cli/src/device_options.rs:139`; wired by `crates/dsview-cli/src/main.rs:455`. |
| `crates/dsview-cli/src/main.rs` | Existing `capture` command extended with optional device-option flags, preflight validation branch, and stable diagnostics. | ✓ VERIFIED | Flag group lives in `crates/dsview-cli/src/main.rs:166`; validation preflight runs in `crates/dsview-cli/src/main.rs:364`; diagnostics map to stable codes in `crates/dsview-cli/src/main.rs:894`. |
| `crates/dsview-cli/tests/device_options_cli.rs` | Lock JSON/text inspection contract, flag order, shared tokens, and channel-limit guidance. | ✓ VERIFIED | Integration tests assert token+stable-ID JSON, channel limits, copy-paste text, flag order, and token alignment in `crates/dsview-cli/tests/device_options_cli.rs:158`, `crates/dsview-cli/tests/device_options_cli.rs:269`, `crates/dsview-cli/tests/device_options_cli.rs:328`, and `crates/dsview-cli/tests/device_options_cli.rs:401`. |
| `crates/dsview-cli/tests/capture_cli.rs` | Spawned coverage for help discoverability, valid-token acceptance, malformed primitive input, and stable validation failures. | ✓ VERIFIED | Public CLI contract is exercised in `crates/dsview-cli/tests/capture_cli.rs:62`, `crates/dsview-cli/tests/capture_cli.rs:81`, `crates/dsview-cli/tests/capture_cli.rs:118`, and `crates/dsview-cli/tests/capture_cli.rs:252`. |

### Key Link Verification

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| `crates/dsview-cli/src/device_options.rs` | `crates/dsview-cli/src/capture_device_options.rs` | Response building that maps stable IDs to capture-friendly tokens | ✓ WIRED | `build_device_options_response(...)` uses `build_capture_token_guide(...)`, `token_lookup_maps(...)`, and token builders from the helper module in `crates/dsview-cli/src/device_options.rs:73`. |
| `crates/dsview-cli/src/main.rs` | `crates/dsview-cli/src/capture_device_options.rs` | `run_capture` building a full validation request from CLI flags plus current device state | ✓ WIRED | `run_capture` calls `resolve_capture_device_option_request(...)` before runtime capture work in `crates/dsview-cli/src/main.rs:379`. |
| `crates/dsview-cli/src/main.rs` | `crates/dsview-core/src/lib.rs` | Selected-device inspection plus Phase 11 validation | ✓ WIRED | `run_capture` loads the device snapshot and validation capabilities in `crates/dsview-cli/src/main.rs:370`; inference from source: it then calls `capabilities.validate_request(&request)` in `crates/dsview-cli/src/main.rs:393`, which is the same validation step wrapped by `Discovery::validate_device_option_request(...)` in `crates/dsview-core/src/lib.rs:604`. |
| `crates/dsview-cli/src/main.rs` | `crates/dsview-cli/src/device_options.rs` | `devices options` inspection path | ✓ WIRED | `run_options` calls `inspect_device_options(handle)`, `build_device_options_response(...)`, and `render_device_options_success(...)` in `crates/dsview-cli/src/main.rs:448`. |
| `crates/dsview-cli/tests/capture_cli.rs` | `crates/dsview-cli/src/main.rs` | Spawned binary assertions for help text and parser/validation behavior | ✓ WIRED | The compiled binary is invoked directly and checked for help output, parse failures, and validation errors in `crates/dsview-cli/tests/capture_cli.rs:62`. |
| `crates/dsview-cli/tests/device_options_cli.rs` | `crates/dsview-cli/src/device_options.rs` | JSON/text contract assertions against the tokenized inspection output | ✓ WIRED | Integration tests serialize and render the response builder output directly in `crates/dsview-cli/tests/device_options_cli.rs:158` and `crates/dsview-cli/tests/device_options_cli.rs:328`. |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| --- | --- | --- | --- | --- |
| `crates/dsview-cli/src/main.rs` | `snapshot` in `run_options` | `Discovery::inspect_device_options(handle)` in `crates/dsview-cli/src/main.rs:452` | Yes - comes from the core device-option snapshot path, not static placeholders | ✓ FLOWING |
| `crates/dsview-cli/src/device_options.rs` | `DeviceOptionsResponse.current/operation_modes/threshold/channel_modes_by_operation_mode` | `DeviceOptionsSnapshot` fields + token maps in `crates/dsview-cli/src/device_options.rs:73` | Yes - response fields are derived from the inspected snapshot and stable-ID/token lookups | ✓ FLOWING |
| `crates/dsview-cli/src/main.rs` | `request` in `run_capture` | CLI args + current device snapshot via `resolve_capture_device_option_request(...)` in `crates/dsview-cli/src/main.rs:379` | Yes - stable IDs, threshold volts, and enabled channels come from actual CLI input plus current device state | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| Binary unit coverage for resolver fallback, inference, stable diagnostics, and validation routing | `cargo test -p dsview-cli --bin dsview-cli -- --nocapture` | 29 passed; 0 failed | ✓ PASS |
| Public `capture` help/parser/validation contract | `cargo test -p dsview-cli --test capture_cli -- --nocapture` | 15 passed; 0 failed | ✓ PASS |
| Public `devices options` contract and CLI help alignment | `cargo test -p dsview-cli --test device_options_cli --test devices_cli -- --nocapture` | 14 passed; 0 failed | ✓ PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| OPT-02 | `12-01`, `12-02`, `12-03` | User can choose `Buffer Mode` or `Stream Mode` for a `DSLogic Plus` capture run from the CLI. | ✓ SATISFIED | `--operation-mode` is defined in `crates/dsview-cli/src/main.rs:170`; exact `buffer`/`stream` tokens are built in `crates/dsview-cli/src/capture_device_options.rs:160`; help/acceptance tests cover the public surface in `crates/dsview-cli/tests/capture_cli.rs:62`. |
| OPT-03 | `12-01`, `12-02`, `12-03` | User can choose the DSLogic stop option for operation modes that support it. | ✓ SATISFIED | `--stop-option` is defined in `crates/dsview-cli/src/main.rs:176`; token/stable-ID exposure is locked in `crates/dsview-cli/tests/device_options_cli.rs:158`; incompatible combinations produce stable validation errors in `crates/dsview-cli/tests/capture_cli.rs:252`. |
| OPT-04 | `12-01`, `12-02`, `12-03` | User can choose a DSLogic channel mode that determines valid channel count and maximum sample rate. | ✓ SATISFIED | `--channel-mode` is defined in `crates/dsview-cli/src/main.rs:182`; channel-mode tokens and max-enabled limits are exposed in `crates/dsview-cli/src/device_options.rs:185`; integration coverage locks them in `crates/dsview-cli/tests/device_options_cli.rs:269`. |
| OPT-05 | `12-01`, `12-02`, `12-03` | User can choose which logic channels are enabled for a run within the selected channel-mode limit. | ✓ SATISFIED | Existing `--channels` remains the CLI surface and is carried into `enabled_channels` in `crates/dsview-cli/src/capture_device_options.rs:258`; channel-limit diagnostics are exercised in `crates/dsview-cli/tests/capture_cli.rs:282`; inspection text explains the limit in `crates/dsview-cli/tests/device_options_cli.rs:328`. |
| OPT-06 | `12-01`, `12-02`, `12-03` | User can choose the `DSLogic Plus` threshold voltage from the CLI. | ✓ SATISFIED | `--threshold-volts` is defined in `crates/dsview-cli/src/main.rs:188`; inspection exposes current/min/max/step volts in `crates/dsview-cli/src/device_options.rs:101` and `crates/dsview-cli/src/device_options.rs:204`; malformed input coverage exists in `crates/dsview-cli/tests/capture_cli.rs:227`. |
| OPT-07 | `12-01`, `12-02`, `12-03` | User can choose the DSLogic filter option from the CLI. | ✓ SATISFIED | `--filter` is defined in `crates/dsview-cli/src/main.rs:194`; token/stable-ID exposure is locked in `crates/dsview-cli/tests/device_options_cli.rs:158`; unsupported filter tokens return stable diagnostics in `crates/dsview-cli/tests/capture_cli.rs:201`. |

No orphaned Phase 12 requirements found: all requirement IDs declared in plan frontmatter appear in `.planning/REQUIREMENTS.md`, and the traceability table maps each of `OPT-02` through `OPT-07` to Phase 12.

### Anti-Patterns Found

No blocker or warning anti-patterns found in the Phase 12 implementation files. Grep hits from the automated scan were limited to format strings such as `stable_id={}` and did not indicate stubs, placeholders, or empty implementations.

### Human Verification Required

None. The Phase 12 scope is CLI parsing, inspection rendering, and preflight validation wiring, and those behaviors are covered by deterministic unit/integration tests.

### Gaps Summary

No gaps found. Phase 12 delivers the CLI-facing device-option selection surface, keeps `devices options` as the authoritative discovery surface, preserves machine-readable output, and stops at validation rather than Phase 13 hardware-apply behavior.

---

_Verified: 2026-04-13T09:02:54Z_
_Verifier: Claude (gsd-verifier)_
