---
phase: 5
slug: export-artifacts
status: draft
nyquist_compliant: false
wave_0_complete: false
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
| **Quick run command** | `cargo test -p dsview-sys --test boundary` |
| **Full suite command** | `cargo test --workspace` |
| **Estimated runtime** | ~45 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p dsview-sys --test boundary`
- **After every plan wave:** Run `cargo test --workspace`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 05-01-01 | 01 | 1 | EXP-01, EXP-02 | unit/integration | `cargo test -p dsview-sys --test boundary export` | ❌ W0 | ⬜ pending |
| 05-01-02 | 01 | 1 | EXP-01, EXP-02 | integration | `cargo test -p dsview-core export_capture` | ❌ W0 | ⬜ pending |
| 05-02-01 | 02 | 1 | EXP-03, EXP-04 | unit | `cargo test -p dsview-core metadata_sidecar` | ❌ W0 | ⬜ pending |
| 05-02-02 | 02 | 1 | EXP-03, EXP-04 | integration | `cargo test -p dsview-cli capture_cli -- --exact capture_success_reports_artifacts_json` | ❌ W0 | ⬜ pending |
| 05-03-01 | 03 | 2 | EXP-01, EXP-02, EXP-03, EXP-04 | golden/integration | `cargo test -p dsview-sys --test boundary synthetic_vcd_goldens` | ❌ W0 | ⬜ pending |
| 05-03-02 | 03 | 2 | EXP-01, EXP-02, EXP-03, EXP-04 | manual | `cargo test --workspace` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/dsview-sys/tests/boundary.rs` — export retention/replay and synthetic VCD golden scaffolding for EXP-01/EXP-02
- [ ] `crates/dsview-core/tests/export_artifacts.rs` or in-file equivalents — metadata sidecar and orchestration checks for EXP-03/EXP-04
- [ ] `crates/dsview-cli/tests/capture_cli.rs` additions — artifact path and error contract assertions

*If none: "Existing infrastructure covers all phase requirements."*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Real DSLogic Plus finite capture emits VCD + JSON sidecar | EXP-01, EXP-03 | Requires hardware and DSView runtime | Run a known-good bounded capture against connected hardware using `./DSView/DSView/res`; confirm both artifacts are created and command exits 0 |
| VCD channel naming and timing semantics look plausible in external tooling | EXP-02 | Needs human inspection against trusted waveform viewer and known signal pattern | Open exported VCD in a trusted viewer, verify channel names/order match capture config, and verify transition spacing is consistent with samplerate and known input pattern |
| Metadata reports observed capture facts rather than only requested settings | EXP-04 | Needs comparison between requested config, observed sample count, and produced artifacts | Inspect metadata JSON after a successful hardware run and confirm device model, enabled channels, sample rate, sample limit or actual sample count, timestamp, and tool version are present and plausible |
| Nyquist-safe timing validation with repeatable source pattern | EXP-02 | Alias risk makes pure automation insufficient for first hardware sign-off | Capture a repeatable square wave at or below `sample_rate / 4`; confirm VCD transition spacing is within one-sample quantization expectations and avoid treating near-Nyquist signals as primary proof |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
