---
phase: 10
slug: device-option-bridge-and-discovery
status: approved
nyquist_compliant: true
wave_0_complete: false
created: 2026-04-10
---

# Phase 10 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness via `cargo test` |
| **Config file** | none - standard Cargo test layout |
| **Quick run command** | `cargo test -p dsview-sys --test device_options device_options_snapshot_reads_current_and_supported_values -- --nocapture` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~45 seconds |

---

## Sampling Rate

- **After every task commit:** Run the task-specific smoke command from the verification map for the task that just changed; keep single-task feedback under one crate-local test target whenever possible.
- **After every plan wave:** Run `cargo test -p dsview-sys && cargo test -p dsview-core && cargo test -p dsview-cli`
- **Before `/gsd-verify-work`:** Full relevant workspace tests plus at least one manual option-discovery run against a real `DSLogic Plus`
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 10-01-01 | 01 | 1 | OPT-01 | T-10-01 | Bridge copies DSView option lists into owned structs without leaking raw pointers or `GVariant` lifetimes | integration | `cargo test -p dsview-sys --test device_options -- --nocapture` | ❌ W0 | ⬜ pending |
| 10-01-02 | 01 | 1 | OPT-01 | T-10-01 | Discovery restores original operation and channel mode on every exit path and treats unavailable options as optional facts, not hard failures | integration | `cargo test -p dsview-sys --test device_options -- --nocapture && cargo test -p dsview-sys --test boundary -- --nocapture` | ❌ W0 | ⬜ pending |
| 10-02-01 | 02 | 2 | OPT-01 | T-10-02 | Core normalizes raw option snapshots into stable IDs while preserving raw numeric codes and per-operation-mode channel groups | unit/integration | `cargo test -p dsview-core --test device_options -- --nocapture` | ❌ W0 | ⬜ pending |
| 10-02-02 | 02 | 2 | OPT-01 | T-10-02 | Core keeps threshold truthful as a voltage-range capability and preserves existing capture/export APIs | unit/integration | `cargo test -p dsview-core --test device_options -- --nocapture && cargo test -p dsview-core --lib -- --nocapture` | ❌ W0 | ⬜ pending |
| 10-03-01 | 03 | 3 | OPT-01 | T-10-03 | CLI renderers produce deterministic text and JSON option discovery output with stable field order | CLI integration | `cargo test -p dsview-cli --test device_options_cli -- --nocapture` | ❌ W0 | ⬜ pending |
| 10-03-02 | 03 | 3 | OPT-01 | T-10-03 | CLI wiring keeps `devices list`, `devices open`, and `capture` behavior green while adding `devices options` | CLI integration | `cargo test -p dsview-cli --test device_options_cli -- --nocapture && cargo test -p dsview-cli --test devices_cli -- --nocapture && cargo test -p dsview-cli --test capture_cli -- --nocapture` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/dsview-sys/tests/device_options.rs` - bridge coverage for enum-list copying, threshold metadata/current values, and restore-on-exit channel-mode enumeration
- [ ] `crates/dsview-core/tests/device_options.rs` - normalization coverage for stable IDs, option ordering, and per-operation-mode channel-mode grouping
- [ ] `crates/dsview-cli/tests/device_options_cli.rs` - text/JSON assertions for deterministic option discovery output
- [ ] A sys-level helper or fixture strategy to exercise option discovery without requiring live hardware for every automated test

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| CLI option discovery reflects the real connected `DSLogic Plus` values for operation mode, stop option, channel mode, threshold capability, and filter options | OPT-01 | Automated tests can validate bridge behavior and output shape, but a real device run is still needed to confirm the live DSView-backed option surface | Connect a `DSLogic Plus`, run the new discovery command in `text` and `json` modes, confirm the reported option lists and current values match DSView/device reality, then rerun the existing `v1.0` capture flow to confirm no regression |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 60s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-04-10
