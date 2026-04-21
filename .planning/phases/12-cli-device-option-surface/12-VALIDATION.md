---
phase: 12
slug: cli-device-option-surface
status: approved
nyquist_compliant: true
wave_0_complete: true
created: 2026-04-13
---

# Phase 12 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` with integration tests via `assert_cmd` + `predicates`, plus binary unit tests in `src/main.rs` |
| **Config file** | none — Cargo defaults |
| **Quick run command** | `cargo test -p dsview-cli --bin dsview-cli -- --nocapture` |
| **Full suite command** | `cargo test --workspace -- --nocapture` |
| **Estimated runtime** | ~60 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p dsview-cli --bin dsview-cli -- --nocapture`
- **After every plan wave:** Run `cargo test -p dsview-cli --test capture_cli --test device_options_cli --test devices_cli -- --nocapture`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 12-01-01 | 01 | 1 | OPT-02, OPT-03, OPT-04, OPT-06, OPT-07 | T-12-01 / — | CLI token contract and inspection surface are established before capture-path wiring depends on them | unit/integration | `cargo test -p dsview-cli --bin dsview-cli -- --nocapture && cargo test -p dsview-cli --test device_options_cli -- --nocapture` | ✅ existing harness | ⬜ pending |
| 12-01-02 | 01 | 1 | OPT-02, OPT-03, OPT-04, OPT-05, OPT-06, OPT-07 | T-12-01 / — | Inspection JSON/text stay locked to the capture-facing token surface and preserve channel-limit guidance for `--channels` | integration | `cargo test -p dsview-cli --test device_options_cli -- --nocapture` | ✅ existing harness | ⬜ pending |
| 12-02-01 | 02 | 2 | OPT-02, OPT-03, OPT-04, OPT-05, OPT-06, OPT-07 | T-12-02 / — | Parsed CLI values merge with current device state, infer parent mode only when allowed, and validate before capture | unit/integration | `cargo test -p dsview-cli --bin dsview-cli -- --nocapture` | ✅ existing harness | ⬜ pending |
| 12-02-02 | 02 | 2 | OPT-02, OPT-03, OPT-04, OPT-05, OPT-06, OPT-07 | T-12-02 / — | The selected-device validation branch covers device-option flags and `--channels`-only overrides without claiming Phase 13 apply behavior | unit/integration | `cargo test -p dsview-cli --bin dsview-cli -- --nocapture` | ✅ existing harness | ⬜ pending |
| 12-03-01 | 03 | 3 | OPT-02, OPT-03, OPT-04, OPT-05, OPT-06, OPT-07 | T-12-03 / — | Final CLI diagnostics and inspection output remain stable while the shipped baseline suites still pass | integration | `cargo test -p dsview-cli --test capture_cli -- --nocapture && cargo test -p dsview-cli --test device_options_cli -- --nocapture && cargo test -p dsview-cli --test devices_cli -- --nocapture` | ✅ existing harness | ⬜ pending |
| 12-03-02 | 03 | 3 | OPT-02, OPT-03, OPT-04, OPT-05, OPT-06, OPT-07 | T-12-03 / — | Spawned CLI regressions prove valid tokens reach semantic validation, device-dependent bad tokens fail through resolver/validation, and malformed primitive input still fails in clap | integration | `cargo test -p dsview-cli --test capture_cli -- --nocapture && cargo test -p dsview-cli --test device_options_cli -- --nocapture && cargo test -p dsview-cli --test devices_cli -- --nocapture` | ✅ existing harness | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure now covers the predecessor needs for this phase:

- [x] `crates/dsview-cli/tests/device_options_cli.rs` — capture-oriented inspection output groundwork landed in Plan `12-01`
- [x] `crates/dsview-cli/src/main.rs` tests — binary unit harness already in use and remains the quick verification target
- [x] `crates/dsview-core/tests/device_option_validation.rs` — semantic rule regression suite remains the authoritative validator coverage from Phase 11

Remaining Phase 12 test work happens in the normal execution waves:
- Plan `12-02` adds resolver and flag-wiring unit coverage
- Plan `12-03` adds spawned `capture_cli` and final `device_options_cli` / `devices_cli` regression coverage

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| None expected for this phase if CLI parsing, inspection rendering, and validation wiring remain fully covered by deterministic tests | OPT-02, OPT-03, OPT-04, OPT-05, OPT-06, OPT-07 | This phase does not yet apply options to hardware; Phase 13 handles runtime application and any live-device proof | If execution accidentally introduces apply-time behavior, add a follow-up manual verification step before sign-off |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 60s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-04-13
