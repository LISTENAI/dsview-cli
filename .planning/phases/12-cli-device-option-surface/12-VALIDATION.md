---
phase: 12
slug: cli-device-option-surface
status: approved
nyquist_compliant: true
wave_0_complete: false
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
| 12-01-01 | 01 | 1 | OPT-02, OPT-03, OPT-04, OPT-06, OPT-07 | T-12-01 / — | CLI flag shape stays scriptable and resolves friendly tokens without inventing a second validation engine | unit/integration | `cargo test -p dsview-cli --bin dsview-cli -- --nocapture && cargo test -p dsview-cli --test device_options_cli -- --nocapture` | ✅ existing harness | ⬜ pending |
| 12-02-01 | 02 | 2 | OPT-02, OPT-03, OPT-04, OPT-05, OPT-06, OPT-07 | T-12-02 / — | Parsed CLI values merge with current device state, infer parent mode only when allowed, and validate before capture | integration | `cargo test -p dsview-cli --bin dsview-cli -- --nocapture && cargo test -p dsview-cli --test capture_cli -- --nocapture` | ✅ existing harness | ⬜ pending |
| 12-03-01 | 03 | 3 | OPT-02, OPT-03, OPT-04, OPT-05, OPT-06, OPT-07 | T-12-03 / — | Final CLI diagnostics and inspection output remain stable while the shipped baseline suites still pass | integration | `cargo test -p dsview-cli --test capture_cli -- --nocapture && cargo test -p dsview-cli --test device_options_cli -- --nocapture && cargo test -p dsview-cli --test devices_cli -- --nocapture` | ✅ existing harness | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/dsview-cli/tests/capture_cli.rs` — extend with happy-path and failure-path cases for `--operation-mode`, `--stop-option`, `--channel-mode`, `--threshold-volts`, and `--filter`
- [ ] `crates/dsview-cli/tests/device_options_cli.rs` — assert capture-oriented `devices options` JSON/text output
- [ ] `crates/dsview-cli/src/main.rs` tests — add resolver-focused unit coverage for token mapping, parent inference, and current-value preservation
- [ ] `crates/dsview-core/tests/device_option_validation.rs` — remain the semantic rule regression suite; no duplicate rule engine in the CLI layer

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
