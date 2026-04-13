---
phase: 11
slug: device-option-validation-model
status: approved
nyquist_compliant: true
wave_0_complete: false
created: 2026-04-13
---

# Phase 11 - Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness via `cargo test` |
| **Config file** | none - standard Cargo test layout |
| **Quick run command** | `cargo test -p dsview-core --test device_option_validation -- --nocapture` once `11-01` creates the target; before that, use `cargo test -p dsview-core --lib -- --nocapture` |
| **Full suite command** | `cargo test --workspace` |
| **Estimated runtime** | ~60 seconds |

---

## Sampling Rate

- **After every task commit:** Run the smallest relevant crate-local validation target first; before `11-01` creates the dedicated test target, use `cargo test -p dsview-core --lib -- --nocapture`, then switch to `cargo test -p dsview-core --test device_option_validation -- --nocapture` and `cargo test -p dsview-cli --lib stable_validation_error_codes -- --nocapture` when those assertions are touched.
- **After every plan wave:** Run `cargo test -p dsview-sys --test device_options && cargo test -p dsview-core && cargo test -p dsview-cli`
- **Before `/gsd-verify-work`:** Full workspace suite must be green, plus any manual validation called out by the phase verification report.
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 11-01-01 | 01 | 1 | VAL-01, VAL-02 | T-11-01 | Core request/capability/error contracts exist before validator implementation, and the dedicated `device_option_validation` test target plus initial CLI stable-code assertions are scaffolded in the predecessor plan | unit | `cargo test -p dsview-core --test device_option_validation -- --nocapture && cargo test -p dsview-cli --lib stable_validation_error_codes -- --nocapture` | created in `11-01` | ⬜ pending |
| 11-01-02 | 01 | 1 | VAL-01 | T-11-01 | Validation capability loading keeps mode-aware samplerate and channel-limit facts inside the sys/core boundary without changing the public discovery schema | integration | `cargo test -p dsview-sys --test device_options -- --nocapture && cargo test -p dsview-core --test device_option_validation -- --nocapture` | created in `11-01` | ⬜ pending |
| 11-02-01 | 02 | 2 | VAL-01 | T-11-02 | Validator implementation grows against an already-existing test target, starting with RED requirement cases before runtime work begins | unit/integration | `cargo test -p dsview-core --test device_option_validation -- --nocapture` | ✅ predecessor target from `11-01` | ⬜ pending |
| 11-02-02 | 02 | 2 | VAL-02 | T-11-02 | CLI mapping expands existing stable validation-code assertions while wiring known validation failures away from generic `runtime_error` | unit/integration | `cargo test -p dsview-core --test device_option_validation -- --nocapture && cargo test -p dsview-cli --lib stable_validation_error_codes -- --nocapture` | ✅ predecessor assertions from `11-01` | ⬜ pending |
| 11-03-01 | 03 | 3 | VAL-01, VAL-02 | T-11-03 | Existing validation target expands to the full DSView-backed matrix across stream/buffer modes, channel ceilings, samplerate ceilings, threshold range checks, and filter/stop-option compatibility | integration | `cargo test -p dsview-core --test device_option_validation -- --nocapture` | ✅ existing | ⬜ pending |
| 11-03-02 | 03 | 3 | VAL-02 | T-11-03 | CLI preserves stable `ErrorResponse.code` mappings for validation failures and broader CLI regressions stay green after Phase 11 changes | CLI integration | `cargo test -p dsview-cli --lib stable_validation_error_codes -- --nocapture && cargo test -p dsview-cli --test capture_cli -- --nocapture && cargo test -p dsview-cli --test device_options_cli -- --nocapture` | ✅ existing | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 / predecessor requirements

- [ ] `crates/dsview-core/src/device_option_validation.rs` - unified request, capability, validated-request, and typed-error contracts for `VAL-01` and `VAL-02`
- [ ] `crates/dsview-core/tests/device_option_validation.rs` - dedicated test target scaffolded in `11-01` so `11-02` can add RED requirement cases before validator implementation
- [ ] `crates/dsview-cli/src/main.rs` initial stable validation-code assertions scaffolded in `11-01` so `11-02` can expand them while wiring `classify_validation_error()`
- [ ] `crates/dsview-sys` coverage for the mode-probing capability snapshot path created in `11-01`

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| None expected for Phase 11 if the validation model is fully exercised through deterministic capability fixtures and CLI error-code tests | VAL-01, VAL-02 | This phase should remain pre-acquisition and can be proven with repo-local automated coverage unless verification later finds a mode-probing gap that only appears on hardware | If execution introduces a live-device-only capability path, add a follow-up verification step that runs the validation entrypoint against a connected `DSLogic Plus` without starting a capture |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 60s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-04-13
