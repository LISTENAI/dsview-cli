---
phase: 6
slug: cli-productization
status: draft
nyquist_compliant: n/a
wave_0_complete: false
created: 2026-04-08
---

# Phase 6 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` plus spawned-process CLI integration checks |
| **Config file** | `Cargo.toml` workspace manifests |
| **Quick run command** | `cargo test -p dsview-cli --test capture_cli` |
| **Full suite command** | `cargo test --workspace` |
| **Estimated runtime** | ~60 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p dsview-cli --test capture_cli`
- **After every plan wave:** Run `cargo test --workspace`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 90 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command / Verification | File Exists | Status |
|---------|------|------|-------------|-----------|----------------------------------|-------------|--------|
| 06-01-01 | 01 | 1 | CLI-01, CLI-03 | planning review | Confirm `06-RESEARCH.md` locks the text/json output contract for the final `capture` workflow | ✅ | ✅ green |
| 06-01-02 | 01 | 1 | CLI-03 | process-boundary CLI test | `cargo test -p dsview-cli --test capture_cli` with coverage for text-mode success output naming final artifact paths | ✅ | ✅ green |
| 06-01-03 | 01 | 1 | CLI-01, CLI-03 | process-boundary CLI test | `cargo test -p dsview-cli --test capture_cli` with coverage for stable JSON success/failure contract | ✅ | ✅ green |
| 06-02-01 | 02 | 1 | CLI-02 | core/CLI validation | `cargo test -p dsview-core --test export_artifacts`; `cargo test -p dsview-cli --test capture_cli` for path-model and derivation coverage | ✅ | ✅ green |
| 06-02-02 | 02 | 1 | CLI-02 | CLI validation | `cargo test -p dsview-cli --test capture_cli` with invalid/conflicting output-path combination checks | ✅ | ✅ green |
| 06-02-03 | 02 | 1 | CLI-02, CLI-03 | tempdir integration | `cargo test -p dsview-cli --test capture_cli` with reported-paths-match-created-files coverage | ✅ | ✅ green |
| 06-03-01 | 03 | 2 | CLI-01, CLI-02, CLI-03 | process-boundary regression | `cargo test -p dsview-cli`; verify spawned-process success/failure, stdout/stderr, and exit-code behavior | ✅ | ✅ green |
| 06-03-02 | 03 | 2 | CLI-01, CLI-02, CLI-03 | help/usage validation | Inspect `--help` output and/or automated checks to confirm runtime, format, and path controls are discoverable | ✅ | ✅ green |
| 06-03-03 | 03 | 2 | CLI-01, CLI-02, CLI-03 | manual DSLogic Plus UAT | Execute the final non-interactive shell workflow against connected hardware and confirm artifact reporting plus immediate rerun reuse | ⬜ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [x] `crates/dsview-cli/tests/capture_cli.rs` — process-boundary success/failure rendering, output-path reporting, and help/usage checks for the final command surface
- [x] `crates/dsview-core/tests/export_artifacts.rs` — path derivation and complete-artifact semantics where core path shaping is involved
- [x] `.planning/phases/06-cli-productization/06-RESEARCH.md` — locked text/json contract and v1 output-path model used by implementation

*If none: "Existing infrastructure covers all phase requirements."*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Final shell workflow remains one-shot and non-interactive | CLI-01 | Requires real runtime/resource path and current shell environment | Run the final `capture` command using the source runtime and confirm one command performs validate -> capture -> export without extra manual steps |
| Final artifact paths are clear in operator-facing output | CLI-03 | Human-usable output quality is best judged manually even with automated text checks | Inspect successful text and JSON output and confirm both final artifact paths are obvious and accurate |
| Final path controls behave plausibly on current hardware workflow | CLI-02 | Requires real path choices, runtime prerequisites, and current machine permissions | Run a bounded capture using the final Phase 6 path controls and confirm artifacts land at the requested or documented derived locations |
| Device remains reusable after the polished CLI workflow | CLI-01, CLI-03 | Requires a second real-hardware interaction on the same environment | Re-run the final capture workflow immediately after a successful run and confirm the device opens, captures, exports, and reports artifacts again without restart-only recovery |

**Manual gate status:** Passed on 2026-04-08. The final DSLogic Plus shell-workflow UAT executed successfully on current hardware in text mode, JSON mode, and an immediate rerun; artifacts landed at the requested paths with sane VCD timing markers and matching metadata sidecars.

---

## Validation Sign-Off

- [x] All automated tasks have verifier-runnable checks
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all automated MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 90s
- [x] Final manual DSLogic Plus shell-workflow UAT recorded green on current environment

**Approval:** Phase 6 validation is green. Automated command-surface, help/discoverability, output-path validation, and CLI-boundary checks are green, and the 2026-04-08 real-hardware shell workflow confirmed non-interactive capture success, clear text/json artifact reporting, and immediate rerun reuse.
