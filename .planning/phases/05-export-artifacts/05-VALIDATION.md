---
phase: 5
slug: export-artifacts
status: verifier-ready
nyquist_compliant: true
wave_0_complete: true
created: 2026-04-07
---

# Phase 5 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` |
| **Config file** | `Cargo.toml` workspace manifests |
| **Quick run command** | `cargo test -p dsview-sys --test boundary synthetic_vcd_goldens` |
| **Full suite command** | `cargo test --workspace` |
| **Estimated runtime** | ~45 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p dsview-sys --test boundary synthetic_vcd_goldens`
- **After every plan wave:** Run `cargo test --workspace`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command / Verification | File Exists | Status |
|---------|------|------|-------------|-----------|----------------------------------|-------------|--------|
| 05-01-01 | 01 | 1 | EXP-01, EXP-02 | unit/integration | `cargo test -p dsview-sys --test boundary export` | ✅ | ✅ green |
| 05-01-02 | 01 | 1 | EXP-01, EXP-02 | integration | `cargo test -p dsview-core export_capture` | ✅ | ✅ green |
| 05-02-01 | 02 | 1 | EXP-03, EXP-04 | unit | `cargo test -p dsview-core metadata_sidecar` | ✅ | ✅ green |
| 05-02-02 | 02 | 1 | EXP-03, EXP-04 | integration | `cargo test -p dsview-cli capture_success_reports_artifacts_json -- --exact` | ✅ | ✅ green |
| 05-03-01 | 03 | 2 | EXP-01, EXP-02, EXP-03, EXP-04 | golden/integration | `cargo test -p dsview-sys --test boundary synthetic_vcd_goldens` | ✅ | ✅ green |
| 05-03-02 | 03 | 2 | EXP-01, EXP-02, EXP-03, EXP-04 | layered automated regression | `cargo test -p dsview-core`; `cargo test -p dsview-cli`; `cargo test --workspace` | ✅ | ✅ green |
| 05-03-03 | 03 | 2 | EXP-01, EXP-02, EXP-03, EXP-04 | manual DSLogic Plus UAT | Execute the manual checks in `Manual-Only Verifications` against connected hardware | ✅ | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [x] `crates/dsview-sys/tests/boundary.rs` — synthetic packet replay, VCD goldens, semantic timing assertions, cleanup-safe write checks, and overflow precondition coverage for EXP-01/EXP-02
- [x] `crates/dsview-core/tests/export_artifacts.rs` — metadata schema, observed-fact export request shaping, and export failure classification checks for EXP-03/EXP-04
- [x] `crates/dsview-cli/tests/capture_cli.rs` — artifact path reporting and stable export failure-code assertions for scriptable CLI behavior

*If none: "Existing infrastructure covers all phase requirements."*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Real DSLogic Plus finite capture emits VCD + JSON sidecar | EXP-01, EXP-03 | Requires hardware and DSView runtime | Run a known-good bounded capture against connected hardware using `./DSView/DSView/res`; confirm both artifacts are created and command exits `0` |
| VCD channel naming and timing semantics look plausible in external tooling | EXP-02 | Needs human inspection against trusted waveform viewer and known signal pattern | Open exported VCD in a trusted viewer, verify channel names/order match capture config, and verify transition spacing is consistent with samplerate and known input pattern below `sample_rate / 4` |
| Metadata reports observed capture facts rather than only requested settings | EXP-04 | Needs comparison between requested config, observed sample count, and produced artifacts | Inspect metadata JSON after a successful hardware run and confirm device model, enabled channels, sample rate, sample limit or actual sample count, timestamp, and tool version are present and plausible |
| Post-export device remains reusable after artifact generation | EXP-01, EXP-03 | Requires a second real-hardware interaction on the same environment | Re-run a bounded capture after the first successful export and confirm the device opens, captures, and emits artifacts again without restart-only recovery |
| Nyquist-safe timing validation with repeatable source pattern | EXP-02 | Alias risk makes pure automation insufficient for first hardware sign-off | Capture a repeatable square wave at or below `sample_rate / 4`; confirm VCD transition spacing is within one-sample quantization expectations and avoid treating near-Nyquist signals as primary proof |

**Manual gate status:** Complete. Manual DSLogic Plus UAT confirmed artifact creation, sane finite VCD timestamps, metadata plausibility, and immediate device reuse on current hardware after the replay-ordering fix.

---

## Validation Sign-Off

- [x] All automated tasks have verifier-runnable checks
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all automated MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 60s
- [x] `nyquist_compliant: true` is set truthfully for the documented automated guidance and manual timing gate
- [x] Manual DSLogic Plus export UAT rerun recorded green on current environment after the replay-ordering fix is validated

**Approval:** automated 05-03 validation is complete and verifier-ready, and manual DSLogic Plus UAT has now passed on current hardware after the replay-ordering fix. Phase 5 export validation is green.
