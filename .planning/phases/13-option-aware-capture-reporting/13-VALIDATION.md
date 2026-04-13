---
phase: 13
slug: option-aware-capture-reporting
status: approved
nyquist_compliant: true
wave_0_complete: false
created: 2026-04-13
---

# Phase 13 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` with unit tests, integration tests via `assert_cmd` + `predicates`, and existing `dsview-sys` C mock-backed tests |
| **Config file** | none — Cargo defaults |
| **Quick run command** | `cargo test -p dsview-core --test acquisition -- --nocapture` |
| **Full suite command** | `cargo test --workspace -- --nocapture` |
| **Estimated runtime** | ~90 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p dsview-core --test acquisition -- --nocapture`
- **After every plan wave:** Run `cargo test -p dsview-cli --test capture_cli -- --nocapture && cargo test -p dsview-sys --test device_options -- --nocapture`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 45 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 13-01-01 | 01 | 1 | RUN-04 | T-13-01 / — | Full validated device-option request is applied in deterministic order before acquisition starts, with partial-apply failures surfaced clearly | unit/integration | `cargo test -p dsview-sys --test device_options -- --nocapture && cargo test -p dsview-core --lib -- --nocapture` | ✅ existing harness | ⬜ pending |
| 13-01-02 | 01 | 1 | RUN-04 | T-13-01 / — | Option-aware capture path replaces hidden preflight mutation and preserves the `v1.0` baseline when no Phase 12 options are in play | unit/integration | `cargo test -p dsview-core --lib -- --nocapture` | ✅ existing harness | ⬜ pending |
| 13-02-01 | 02 | 2 | RUN-05 | T-13-02 / — | CLI JSON and metadata carry both requested and effective option facts from one shared source model | unit/integration | `cargo test -p dsview-core --test export_artifacts -- --nocapture` | ✅ existing harness | ⬜ pending |
| 13-02-02 | 02 | 2 | RUN-05 | T-13-02 / — | Text output stays concise while still surfacing effective option values used for the run | unit/integration | `cargo test -p dsview-cli --bin dsview-cli -- --nocapture && cargo test -p dsview-core --test export_artifacts -- --nocapture` | ✅ existing harness | ⬜ pending |
| 13-03-01 | 03 | 3 | RUN-04, RUN-05 | T-13-03 / — | Regression coverage proves option-aware runs and the default `v1.0` path both remain correct | integration | `cargo test -p dsview-cli --test capture_cli -- --nocapture && cargo test -p dsview-cli --test devices_cli -- --nocapture` | ✅ existing harness | ⬜ pending |
| 13-03-02 | 03 | 3 | RUN-04, RUN-05 | T-13-03 / — | Validation artifact is refreshed to the shipped automated commands and truthful hardware checklist | docs + integration | `cargo test -p dsview-cli --test capture_cli -- --nocapture && cargo test -p dsview-cli --test devices_cli -- --nocapture && rg -n "capture_json_success_reports_requested_and_effective_device_options|capture_apply_failure_reports_applied_steps_and_failed_step|cargo test -p dsview-cli --test capture_cli -- --nocapture|cargo test --workspace -- --nocapture" .planning/phases/13-option-aware-capture-reporting/13-VALIDATION.md && rg -n "requested and effective|schema_version|On a machine with a working DSLogic Plus" .planning/phases/13-option-aware-capture-reporting/13-VALIDATION.md && ! rg -n "wave_0_complete: false" .planning/phases/13-option-aware-capture-reporting/13-VALIDATION.md` | ✅ existing harness + validation artifact | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/dsview-sys/tests/device_options.rs` — extend the C mock so it records stop option, filter, threshold, sample limit, sample rate, and channel enable setter order/failures
- [ ] `crates/dsview-core/tests/acquisition.rs` — add option-aware partial-apply failure and no-hidden-preflight-mutation coverage
- [ ] `crates/dsview-core/tests/export_artifacts.rs` — add requested/effective option reporting assertions for metadata
- [ ] `crates/dsview-cli/tests/capture_cli.rs` — extend spawned CLI coverage for option-aware success and partial-apply failure reporting

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Option-aware capture on a real DSLogic Plus device applies the selected options in order and reports the effective values correctly | RUN-04, RUN-05 | Local live runtime is currently unreliable on this machine (`ds_lib_init` fails with `LIBUSB_ERROR_OTHER`), so final hardware proof must run where device/libusb access works | On a machine with a working `DSLogic Plus`, run a capture using non-default operation mode / channel mode / threshold / filter values, confirm the run succeeds, and inspect CLI text, JSON, and metadata for the expected effective values |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 90s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-04-13
