---
phase: 17-decode-output-and-workflow-reporting
verified: 2026-04-22T01:22:17Z
status: passed
score: 8/8 must-haves verified
overrides_applied: 0
re_verification:
  previous_status: gaps_found
  previous_score: 7/8
  gaps_closed:
    - "Phase 17 locks the final decode output/reporting contract with end-to-end regression coverage."
  gaps_remaining: []
  regressions: []
---

# Phase 17: Decode Output and Workflow Reporting Verification Report

**Phase Goal:** Finalize decode output contracts, artifact reporting, and stable failure taxonomy for a separate decode workflow.
**Verified:** 2026-04-22T01:22:17Z
**Status:** passed
**Re-verification:** Yes - after gap closure

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Successful decode runs emit machine-readable annotation output with sample ranges, decoder identity, and payload text/numeric fields. | ✓ VERIFIED | `DecodeEvent` still exposes `decoder_id`, `start_sample`, `end_sample`, and `texts` in `crates/dsview-core/src/lib.rs:533`, and the success contract test still asserts those fields in `crates/dsview-cli/tests/decode_cli.rs:288`. |
| 2 | Decode failures are reported with stable categories covering runtime prerequisites, config issues, input issues, decode execution failures, and artifact write failures. | ✓ VERIFIED | Failure classification still spans bring-up/runtime/config/input/output paths in `crates/dsview-cli/src/main.rs:1644`, `crates/dsview-cli/src/main.rs:1790`, `crates/dsview-cli/src/main.rs:1933`, `crates/dsview-cli/src/main.rs:1947`, and `crates/dsview-cli/src/main.rs:1975`. |
| 3 | The decode workflow is explicitly separate from `capture` while leaving a clean future handoff point for pipeline orchestration. | ✓ VERIFIED | The CLI still keeps `Decode` and `Capture` as separate top-level subcommands in `crates/dsview-cli/src/main.rs:69`, with `decode run` defined independently in `crates/dsview-cli/src/main.rs:153`. |
| 4 | Phase 17 defines a canonical JSON-first decode result schema of `run + flat events`, with the flat list remaining the primary machine-readable contract. | ✓ VERIFIED | `DecodeReport { run, events }` remains the success schema in `crates/dsview-core/src/lib.rs:543`, and the end-to-end success contract still checks a flat `events` list in `crates/dsview-cli/tests/decode_cli.rs:309`. |
| 5 | The result types build directly on Phase 16 execution data instead of redefining execution semantics. | ✓ VERIFIED | `OfflineDecodeResult::to_report()` still projects live annotations into `events` in `crates/dsview-core/src/lib.rs:590`, and `OfflineDecodeRunError::to_failure_report()` still projects retained runtime diagnostics into failure payloads in `crates/dsview-core/src/lib.rs:651`. |
| 6 | Partial diagnostics may be emitted on failure, but they do not change the run status from `failure`. | ✓ VERIFIED | `DecodeFailureReport` still keeps `run.status = Failure` while exposing optional `partial_events` and `diagnostics` in `crates/dsview-core/src/lib.rs:557`; regression coverage remains in `crates/dsview-core/tests/decode_execute.rs:334`, `crates/dsview-cli/tests/decode_cli.rs:336`, and the restored named test at `crates/dsview-cli/tests/decode_cli.rs:419`. |
| 7 | `--output` persists the same canonical result document shape that stdout uses by default. | ✓ VERIFIED | `run_decode_run()` still writes the same success or failure payloads through `write_decode_report()` before rendering stdout in `crates/dsview-cli/src/main.rs:578` and `crates/dsview-cli/src/main.rs:610`; parity is still asserted in `crates/dsview-cli/tests/decode_cli.rs:589` and `crates/dsview-cli/tests/decode_cli.rs:627`. |
| 8 | Phase 17 locks the final decode output/reporting contract with end-to-end regression coverage. | ✓ VERIFIED | The missing plan-declared regression now exists as `decode_run_failure_can_emit_partial_diagnostics` in `crates/dsview-cli/tests/decode_cli.rs:419`, and `gsd-tools verify artifacts` now passes for `17-02-PLAN.md`. |

**Score:** 8/8 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `crates/dsview-core/src/lib.rs` | Final decode result domain types for success and failure reporting | ✓ VERIFIED | `DecodeRunSummary`, `DecodeEvent`, `DecodeReport`, and `DecodeFailureReport` remain present in `crates/dsview-core/src/lib.rs:522`. |
| `crates/dsview-cli/src/lib.rs` | Canonical response builders for final decode reporting | ✓ VERIFIED | `build_decode_report_response()` and the related failure/report helpers remain present in `crates/dsview-cli/src/lib.rs:223`. |
| `crates/dsview-core/tests/decode_execute.rs` | Coverage that execution results can be projected into the final reporting shape | ✓ VERIFIED | `offline_decode_result_projects_to_flat_event_report` and the failure projection regressions remain present in `crates/dsview-core/tests/decode_execute.rs:334` and `crates/dsview-core/tests/decode_execute.rs:426`. |
| `crates/dsview-cli/src/main.rs` | Final decode error/reporting behavior including optional output-file handling | ✓ VERIFIED | `classify_decode_run_error()` and `run_decode_run()` still wire stable failure codes plus optional artifact writing in `crates/dsview-cli/src/main.rs:1947` and `crates/dsview-cli/src/main.rs:578`. |
| `crates/dsview-cli/src/lib.rs` | Artifact-writing aware report serialization helpers | ✓ VERIFIED | `write_decode_report()` still serializes and writes the canonical payload in `crates/dsview-cli/src/lib.rs:288`. |
| `crates/dsview-cli/tests/decode_cli.rs` | Coverage for failure reports, partial diagnostics, and optional output writing | ✓ VERIFIED | `decode_run_failure_can_emit_partial_diagnostics` now exists in `crates/dsview-cli/tests/decode_cli.rs:419`, and output-writing regressions remain in `crates/dsview-cli/tests/decode_cli.rs:589` and `crates/dsview-cli/tests/decode_cli.rs:627`. |
| `crates/dsview-cli/tests/decode_cli.rs` | Final CLI contract coverage for decode reporting | ✓ VERIFIED | Final success, failure, and text contract regressions remain present in `crates/dsview-cli/tests/decode_cli.rs:288`, `crates/dsview-cli/tests/decode_cli.rs:336`, and `crates/dsview-cli/tests/decode_cli.rs:501`. |
| `crates/dsview-core/tests/decode_execute.rs` | Core regression coverage for final report projection semantics | ✓ VERIFIED | `offline_decode_failure_report_keeps_binary_status_with_partial_diagnostics` remains present in `crates/dsview-core/tests/decode_execute.rs:334`. |
| `crates/dsview-cli/src/lib.rs` | Finalized text/JSON report rendering behavior | ✓ VERIFIED | JSON serialization remains canonical in `crates/dsview-cli/src/lib.rs:280`, and summary-focused text rendering remains in `crates/dsview-cli/src/lib.rs:430`. |

### Key Link Verification

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| `crates/dsview-core/src/lib.rs` | `crates/dsview-cli/src/lib.rs` | execution results and captured annotation events are transformed into the final `run + flat events` schema | ✓ WIRED | `build_decode_report_response()` still delegates straight to `OfflineDecodeResult::to_report()` in `crates/dsview-cli/src/lib.rs:223` and `crates/dsview-core/src/lib.rs:590`. |
| `crates/dsview-cli/src/main.rs` | `crates/dsview-cli/src/lib.rs` | CLI execution routes stdout and optional artifact writing through the same canonical report serializer | ✓ WIRED | Both success and failure paths still call `write_decode_report()` and render the same payload objects in `crates/dsview-cli/src/main.rs:578` and `crates/dsview-cli/src/main.rs:610`. |
| `crates/dsview-cli/tests/decode_cli.rs` | `crates/dsview-cli/src/main.rs` | CLI contract tests lock the final decode workflow behavior end to end | ✓ WIRED | The contract suite still invokes `decode run` and checks success/failure/text envelopes in `crates/dsview-cli/tests/decode_cli.rs:288`, `crates/dsview-cli/tests/decode_cli.rs:336`, and `crates/dsview-cli/tests/decode_cli.rs:501`. |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| --- | --- | --- | --- | --- |
| `crates/dsview-core/src/lib.rs` | `self.annotations` / retained diagnostics | `run_offline_decode()` accumulates annotations and diagnostics before report projection | Yes - `to_report()` still maps real annotations into `events`, and `to_failure_report()` still maps retained annotations into `partial_events` in `crates/dsview-core/src/lib.rs:590` and `crates/dsview-core/src/lib.rs:651`. | ✓ FLOWING |
| `crates/dsview-cli/src/main.rs` | `response` / `failure_response` | Built from core result/error projections, then serialized to stdout and optional file output | Yes - both success and failure payloads still flow through `write_decode_report()` and renderer functions in `crates/dsview-cli/src/main.rs:578` and `crates/dsview-cli/src/main.rs:610`. | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| Core decode projection regressions pass | `cargo test -p dsview-core --test decode_execute -- --nocapture` | `11 passed; 0 failed` | ✓ PASS |
| CLI decode contract regressions pass | `cargo test -p dsview-cli --test decode_cli -- --nocapture` | `14 passed; 0 failed` | ✓ PASS |
| CLI report helpers pass unit coverage | `cargo test -p dsview-cli --lib -- --nocapture` | `15 passed; 0 failed` | ✓ PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| `DEC-06` | `17-01-PLAN.md`, `17-02-PLAN.md`, `17-03-PLAN.md` | User can receive machine-readable decode annotations that include sample ranges and decoder payload text or numeric values. | ✓ SATISFIED | `DecodeEvent` still carries sample ranges and payload text in `crates/dsview-core/src/lib.rs:533`, and the success contract still asserts those fields in `crates/dsview-cli/tests/decode_cli.rs:288`. |
| `PIPE-01` | `17-02-PLAN.md`, `17-03-PLAN.md` | User can run decode as a separate workflow from capture and receive stable error reporting for runtime, config, input, execution, and artifact failures. | ✓ SATISFIED | Separate `Decode` and `Capture` commands still exist in `crates/dsview-cli/src/main.rs:69`, and stable decode failure codes still cover config/input/runtime/output paths in `crates/dsview-cli/src/main.rs:1790`, `crates/dsview-cli/src/main.rs:1933`, `crates/dsview-cli/src/main.rs:1947`, and `crates/dsview-cli/src/main.rs:1975`. |

No orphaned Phase 17 requirements were found in `.planning/REQUIREMENTS.md`: both mapped IDs, `DEC-06` and `PIPE-01`, are declared in Phase 17 plan frontmatter and mapped to Phase 17 in `.planning/REQUIREMENTS.md:59`.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| --- | --- | --- | --- | --- |
| - | - | None found in inspected Phase 17 implementation files | ℹ️ Info | No TODO, placeholder, empty-implementation, or hardcoded-empty-output stub patterns were found in the checked Phase 17 production or test files. |

---

_Verified: 2026-04-22T01:22:17Z_
_Verifier: Claude (gsd-verifier)_
