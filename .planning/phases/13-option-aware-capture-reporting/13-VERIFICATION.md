---
phase: 13-option-aware-capture-reporting
verified: 2026-04-13T11:55:11Z
status: human_needed
score: 7/7 must-haves verified
overrides_applied: 0
human_verification:
  - test: "Run an option-aware capture on a real DSLogic Plus and compare text, JSON, and metadata outputs."
    expected: "The run succeeds; JSON and metadata both include `device_options.requested` and `device_options.effective`; text shows only `effective options:` before artifact paths."
    why_human: "The local automated coverage uses mocks and the env-gated fixture seam; this machine still needs live libusb/device access for final runtime proof."
  - test: "Trigger a safe pre-acquisition device-option setter failure on real hardware."
    expected: "The command fails before acquisition starts and the JSON error still includes both `applied_steps` and `failed_step`."
    why_human: "The shipped failure shape is fixture-backed in automation, but only hardware can confirm the runtime reports a real rejected setter the same way."
  - test: "Inspect the metadata sidecar from a successful hardware run."
    expected: "`schema_version` is `2`, `device_options` contains requested/effective snapshots, and the paths match the emitted `.vcd`/`.json` artifacts."
    why_human: "The sidecar schema is covered by tests, but live capture/export still needs confirmation on a machine where `ds_lib_init` succeeds."
---

# Phase 13: Option-Aware Capture Reporting Verification Report

**Phase Goal:** Apply the selected options during capture and preserve the effective option facts in the final capture artifacts.
**Verified:** 2026-04-13T11:55:11Z
**Status:** human_needed
**Re-verification:** No - initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Capture applies the selected DSView-compatible device options in a deterministic order before acquisition starts. | ✓ VERIFIED | `apply_validated_device_options` applies `operation_mode -> stop_option -> channel_mode -> threshold -> filter -> enabled_channels -> sample_limit -> sample_rate` in Rust at `crates/dsview-core/src/lib.rs:648`; option-aware runs execute that path before `start_collect()` via `crates/dsview-core/src/lib.rs:950` and `crates/dsview-core/src/lib.rs:998`; ordering is locked by `crates/dsview-core/tests/acquisition.rs:430` and `crates/dsview-sys/tests/device_options.rs:285`. |
| 2 | The runtime stops on the first rejected setter and reports what already applied plus what failed. | ✓ VERIFIED | `apply_device_option_step` snapshots `applied_steps` and returns `DeviceOptionApplyFailure` at `crates/dsview-core/src/lib.rs:631`; CLI serialization exposes `applied_steps` and `failed_step` at `crates/dsview-cli/src/main.rs:1461`; fail-fast behavior is exercised by `crates/dsview-core/tests/acquisition.rs:469`, `crates/dsview-sys/tests/device_options.rs:354`, and `crates/dsview-cli/tests/capture_cli.rs:572`. |
| 3 | Successful runs report the effective device-option values in text output, JSON output, and metadata. | ✓ VERIFIED | `CaptureResponse` carries `device_options` at `crates/dsview-cli/src/main.rs:265`; text rendering prints effective values only at `crates/dsview-cli/src/main.rs:1627`; metadata includes `device_options` at `crates/dsview-core/src/lib.rs:374`; covered by `crates/dsview-cli/tests/capture_cli.rs:447`, `crates/dsview-cli/tests/capture_cli.rs:518`, and `crates/dsview-core/tests/export_artifacts.rs:389`. |
| 4 | Machine-readable outputs preserve both requested and effective facts from one shared core model. | ✓ VERIFIED | Shared `CaptureDeviceOptionFacts` / `CaptureDeviceOptionSnapshot` live in `crates/dsview-core/src/lib.rs:270`; facts are built once in `crates/dsview-core/src/lib.rs:1484` and reused by CLI success output through `export.metadata.device_options` at `crates/dsview-cli/src/main.rs:472`; schema-v2 metadata coverage is at `crates/dsview-core/tests/export_artifacts.rs:407`. |
| 5 | Text output stays concise and shows only the effective values actually used for the run. | ✓ VERIFIED | `render_effective_capture_options_text` only reads `facts.effective` at `crates/dsview-cli/src/main.rs:1627`, and `capture_success_text` appends that section directly before artifact paths at `crates/dsview-cli/src/main.rs:1662`; unit coverage is at `crates/dsview-cli/src/main.rs:2213` and spawned coverage is at `crates/dsview-cli/tests/capture_cli.rs:518`. |
| 6 | Regression coverage confirms the default `v1.0` capture/export path still works while option-aware success and failure flows also behave correctly. | ✓ VERIFIED | The baseline path bypasses option-aware apply when `validated_device_options` is absent at `crates/dsview-core/src/lib.rs:982` and `crates/dsview-core/tests/acquisition.rs:505`; spawned CLI coverage exercises option-aware success, effective-only text, apply failure, and inherited baseline runs at `crates/dsview-cli/tests/capture_cli.rs:447`, `crates/dsview-cli/tests/capture_cli.rs:518`, `crates/dsview-cli/tests/capture_cli.rs:572`, and `crates/dsview-cli/tests/capture_cli.rs:617`; `cargo test --workspace -- --nocapture` passed. |
| 7 | The validation artifact tells the next verifier exactly how to prove the remaining real-hardware behavior. | ✓ VERIFIED | `.planning/phases/13-option-aware-capture-reporting/13-VALIDATION.md:63` defines the manual-only section, and `.planning/phases/13-option-aware-capture-reporting/13-VALIDATION.md:67`, `.planning/phases/13-option-aware-capture-reporting/13-VALIDATION.md:68`, and `.planning/phases/13-option-aware-capture-reporting/13-VALIDATION.md:69` give concrete DSLogic Plus checks for success reporting, partial-apply honesty, and schema-v2 sidecar validation. |

**Score:** 7/7 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `crates/dsview-sys/src/lib.rs` | Granular runtime setters/getters for apply and readback | ✓ VERIFIED | Exposes ordered setters and readback helpers at `crates/dsview-sys/src/lib.rs:1019` and `crates/dsview-sys/src/lib.rs:1075`; wired from core apply logic at `crates/dsview-core/src/lib.rs:658`. |
| `crates/dsview-core/src/lib.rs` | Option-aware capture orchestration and shared reporting model | ✓ VERIFIED | Defines `DeviceOptionApplyStep`, `DeviceOptionApplyFailure`, `EffectiveDeviceOptionState`, `CaptureDeviceOptionFacts`, option-aware session prep, and schema-v2 metadata at `crates/dsview-core/src/lib.rs:224`, `crates/dsview-core/src/lib.rs:287`, `crates/dsview-core/src/lib.rs:648`, and `crates/dsview-core/src/lib.rs:1523`. |
| `crates/dsview-core/tests/acquisition.rs` | Regression coverage for deterministic apply, fail-fast, and baseline fallback | ✓ VERIFIED | Locks ordered apply, partial failure, and baseline behavior at `crates/dsview-core/tests/acquisition.rs:430`, `crates/dsview-core/tests/acquisition.rs:469`, and `crates/dsview-core/tests/acquisition.rs:505`. |
| `crates/dsview-core/tests/export_artifacts.rs` | Regression coverage for requested/effective metadata reporting | ✓ VERIFIED | Verifies requested/effective facts, schema version 2, differing sample limits, and inherited baseline reporting at `crates/dsview-core/tests/export_artifacts.rs:389`, `crates/dsview-core/tests/export_artifacts.rs:407`, `crates/dsview-core/tests/export_artifacts.rs:443`, and `crates/dsview-core/tests/export_artifacts.rs:452`. |
| `crates/dsview-cli/src/main.rs` | CLI plumbing plus JSON/text success and failure rendering | ✓ VERIFIED | Threads `validated_device_options` into core at `crates/dsview-cli/src/main.rs:429`, reuses `export.metadata.device_options` at `crates/dsview-cli/src/main.rs:479`, renders effective-only text at `crates/dsview-cli/src/main.rs:1627`, and classifies apply failures at `crates/dsview-cli/src/main.rs:1461`. |
| `crates/dsview-cli/tests/capture_cli.rs` | Spawned CLI coverage for success, failure, and baseline reporting | ✓ VERIFIED | Exercises compiled-binary success, text, failure, and baseline flows at `crates/dsview-cli/tests/capture_cli.rs:447`, `crates/dsview-cli/tests/capture_cli.rs:518`, `crates/dsview-cli/tests/capture_cli.rs:572`, and `crates/dsview-cli/tests/capture_cli.rs:617`. |
| `.planning/phases/13-option-aware-capture-reporting/13-VALIDATION.md` | Final automated and hardware verification contract | ✓ VERIFIED | Documents the shipped automated commands and hardware-only follow-up at `.planning/phases/13-option-aware-capture-reporting/13-VALIDATION.md:46`, `.planning/phases/13-option-aware-capture-reporting/13-VALIDATION.md:57`, and `.planning/phases/13-option-aware-capture-reporting/13-VALIDATION.md:63`. |

### Key Link Verification

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| `crates/dsview-cli/src/main.rs` | `crates/dsview-core/src/lib.rs` | validated device-option request is passed into the capture execution path | ✓ WIRED | `run_capture` populates `CaptureRunRequest.validated_device_options` at `crates/dsview-cli/src/main.rs:429`, and core consumes it at `crates/dsview-core/src/lib.rs:601` and `crates/dsview-core/src/lib.rs:982`. |
| `crates/dsview-core/src/lib.rs` | `crates/dsview-sys/src/lib.rs` | deterministic apply helper calls the runtime setters in D-05 order | ✓ WIRED | The ordered setter chain lives at `crates/dsview-core/src/lib.rs:655` and targets the sys setters exposed at `crates/dsview-sys/src/lib.rs:1019`. |
| `crates/dsview-core/src/lib.rs` | `crates/dsview-cli/src/main.rs` | CLI response reuses the core requested/effective facts instead of duplicating serialization logic | ✓ WIRED | Core builds `CaptureDeviceOptionFacts` at `crates/dsview-core/src/lib.rs:1484`; CLI reuses `export.metadata.device_options` at `crates/dsview-cli/src/main.rs:479`. |
| `crates/dsview-core/src/lib.rs` | `crates/dsview-core/tests/export_artifacts.rs` | metadata schema assertions lock the exact requested/effective block | ✓ WIRED | Metadata-side assertions hit `build_capture_device_option_facts` at `crates/dsview-core/tests/export_artifacts.rs:390` and schema version 2 at `crates/dsview-core/tests/export_artifacts.rs:407`. |
| `crates/dsview-cli/tests/capture_cli.rs` | `crates/dsview-cli/src/main.rs` | spawned binary assertions over the env-gated option-aware fixture path | ✓ WIRED | The tests reuse `DSVIEW_CLI_TEST_DEVICE_OPTIONS_FIXTURE` at `crates/dsview-cli/tests/capture_cli.rs:15`; the seam is defined in `crates/dsview-cli/src/main.rs:36` and routed through `crates/dsview-cli/src/main.rs:497`. |
| `.planning/phases/13-option-aware-capture-reporting/13-VALIDATION.md` | `crates/dsview-cli/tests/capture_cli.rs` | final automated commands and manual checklist reflect the shipped CLI contract | ✓ WIRED | The validation artifact names the spawned CLI cases at `.planning/phases/13-option-aware-capture-reporting/13-VALIDATION.md:57`, including `capture_apply_failure_reports_applied_steps_and_failed_step`. |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| --- | --- | --- | --- | --- |
| `crates/dsview-core/src/lib.rs` | `EffectiveDeviceOptionState` | Runtime getters `current_operation_mode_code/current_stop_option_code/current_channel_mode_code/current_threshold_volts/current_filter_code/current_sample_limit/current_samplerate` in `crates/dsview-core/src/lib.rs:615` backed by sys getters in `crates/dsview-sys/src/lib.rs:1075` | Yes - populated from runtime readback after successful apply | ✓ FLOWING |
| `crates/dsview-core/src/lib.rs` | `CaptureMetadata.device_options` | `build_capture_device_option_facts` at `crates/dsview-core/src/lib.rs:1484` from either validated request + effective runtime state or inherited snapshot + validated config | Yes - produces requested/effective snapshots used by metadata | ✓ FLOWING |
| `crates/dsview-cli/src/main.rs` | `CaptureResponse.device_options` | `export.metadata.device_options` at `crates/dsview-cli/src/main.rs:479`, sourced from the core metadata builder | Yes - CLI JSON/text reuse the same core facts block | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| Sys bridge exposes ordered setters and readback | `cargo test -p dsview-sys --test device_options -- --nocapture` | Exit 0; 10 tests passed including ordered apply, fail-fast stop, and readback coverage | ✓ PASS |
| Core option-aware apply is deterministic and baseline fallback stays intact | `cargo test -p dsview-core --test acquisition -- --nocapture` | Exit 0; 16 tests passed including ordered apply, partial failure, and no-option baseline | ✓ PASS |
| Metadata carries requested/effective facts and schema v2 | `cargo test -p dsview-core --test export_artifacts -- --nocapture` | Exit 0; 15 tests passed including requested/effective facts and inherited baseline reporting | ✓ PASS |
| CLI unit output uses shared facts and surfaces apply failures cleanly | `cargo test -p dsview-cli --bin dsview-cli -- --nocapture` | Exit 0; 32 tests passed including JSON/text success rendering and apply-failure classification | ✓ PASS |
| Spawned CLI contract covers success, failure, and inherited baseline runs | `cargo test -p dsview-cli --test capture_cli -- --nocapture && cargo test -p dsview-cli --test devices_cli -- --nocapture` | Exit 0; 19 `capture_cli` tests and 6 `devices_cli` tests passed | ✓ PASS |
| Phase regressions do not break the broader workspace | `cargo test --workspace -- --nocapture` | Exit 0; workspace suites passed across `dsview-cli`, `dsview-core`, and `dsview-sys` | ✓ PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| `RUN-04` | `13-01-PLAN.md`, `13-03-PLAN.md` | Capture applies the selected DSView-compatible device options before acquisition begins. | ✓ SATISFIED | Ordered apply occurs before `start_collect()` in `crates/dsview-core/src/lib.rs:950`; fail-fast contract is serialized in `crates/dsview-cli/src/main.rs:1461`; exercised by `crates/dsview-core/tests/acquisition.rs:430`, `crates/dsview-sys/tests/device_options.rs:354`, and `crates/dsview-cli/tests/capture_cli.rs:572`. |
| `RUN-05` | `13-02-PLAN.md`, `13-03-PLAN.md` | CLI success output and machine-readable metadata record the effective device option values used for the run. | ✓ SATISFIED | Shared facts model is built in `crates/dsview-core/src/lib.rs:1484`, stored in schema-v2 metadata at `crates/dsview-core/src/lib.rs:1523`, reused by CLI success output at `crates/dsview-cli/src/main.rs:479`, and covered by `crates/dsview-core/tests/export_artifacts.rs:389` plus `crates/dsview-cli/tests/capture_cli.rs:447`. |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| --- | --- | --- | --- | --- |
| `crates/dsview-cli/tests/capture_cli.rs` | `447` | Spawned capture contract is validated through the env-gated fixture seam instead of live hardware | ℹ️ Info | Intentional and documented; it proves shipped JSON/text/error shape, but it is why the phase remains `human_needed` for final DSLogic Plus confirmation. |
| `crates/dsview-core/src/lib.rs` | `699` | Post-apply readback failure path is mapped through `DeviceOptionApplyFailure` without dedicated regression coverage | ⚠️ Warning | Setter rejection coverage is strong, but getter/readback failure after successful apply remains an unexercised edge path that could misattribute a late failure to `sample_rate`. |

### Human Verification Required

### 1. Real Successful Capture Reporting

**Test:** On a machine with a working DSLogic Plus, run the option-aware capture command from `.planning/phases/13-option-aware-capture-reporting/13-VALIDATION.md:67`, then rerun in text mode with the same device-option arguments.
**Expected:** The JSON response and metadata sidecar both include `device_options.requested` and `device_options.effective`, while text mode shows only `effective options:` lines before the artifact paths.
**Why human:** The automated path proves the contract through mocks and the debug-only fixture seam, but this workstation still lacks trustworthy live libusb/device confirmation.

### 2. Real Partial-Apply Failure Honesty

**Test:** Safely trigger a pre-acquisition device-option setter failure on a real DSLogic Plus, following `.planning/phases/13-option-aware-capture-reporting/13-VALIDATION.md:68`.
**Expected:** The capture fails before acquisition starts and the JSON error still includes both `applied_steps` and `failed_step`.
**Why human:** The shipped failure shape is automated through fixtures, but only hardware can confirm a real runtime rejection reports the same facts.

### 3. Metadata Sidecar On Hardware

**Test:** Inspect the metadata sidecar from the successful hardware run described at `.planning/phases/13-option-aware-capture-reporting/13-VALIDATION.md:69`.
**Expected:** `schema_version` is `2`, `device_options` contains requested/effective snapshots, and the artifact paths match the emitted `.vcd` and `.json` files.
**Why human:** The schema is fully tested in-process, but real capture/export still needs a hardware-backed final pass.

### Gaps Summary

No code or wiring gaps were found in the Phase 13 implementation. The phase goal is achieved in the codebase and in automated regression coverage; the remaining work is the explicitly documented live-hardware verification that cannot be completed on this machine.

---

_Verified: 2026-04-13T11:55:11Z_
_Verifier: Claude (gsd-verifier)_
