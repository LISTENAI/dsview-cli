---
phase: 13
slug: option-aware-capture-reporting
status: approved
nyquist_compliant: true
wave_0_complete: true
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
| **Quick run command** | `cargo test -p dsview-cli --test capture_cli -- --nocapture` |
| **Full suite command** | `cargo test --workspace -- --nocapture` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run the narrowest touched harness (`device_options`, `acquisition`, `export_artifacts`, or `capture_cli`)
- **After every plan wave:** Run `cargo test -p dsview-cli --test capture_cli -- --nocapture && cargo test -p dsview-cli --test devices_cli -- --nocapture`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 13-01-01 | 01 | 1 | RUN-04 | T-13-01 / — | Full validated device-option request is applied in deterministic order before acquisition starts, with partial-apply failures surfaced clearly | unit/integration | `cargo test -p dsview-sys --test device_options -- --nocapture` | ✅ existing harness | ✅ green |
| 13-01-02 | 01 | 1 | RUN-04 | T-13-01 / — | Option-aware capture path replaces hidden preflight mutation and preserves the `v1.0` baseline when no Phase 12 options are in play | unit/integration | `cargo test -p dsview-core --test acquisition -- --nocapture && cargo test -p dsview-core --lib -- --nocapture && cargo test -p dsview-cli --bin dsview-cli -- --nocapture` | ✅ existing harness | ✅ green |
| 13-02-01 | 02 | 2 | RUN-05 | T-13-02 / — | CLI JSON and metadata carry both requested and effective option facts from one shared source model | unit/integration | `cargo test -p dsview-core --test export_artifacts -- --nocapture` | ✅ existing harness | ✅ green |
| 13-02-02 | 02 | 2 | RUN-05 | T-13-02 / — | Text output stays concise while still surfacing effective option values used for the run | unit/integration | `cargo test -p dsview-cli --bin dsview-cli -- --nocapture && cargo test -p dsview-core --test export_artifacts -- --nocapture` | ✅ existing harness | ✅ green |
| 13-03-01 | 03 | 3 | RUN-04, RUN-05 | T-13-03 / — | Regression coverage proves option-aware runs and the default `v1.0` path both remain correct | integration | `cargo test -p dsview-cli --test capture_cli -- --nocapture && cargo test -p dsview-cli --test devices_cli -- --nocapture` | ✅ existing harness | ✅ green |
| 13-03-02 | 03 | 3 | RUN-04, RUN-05 | T-13-03 / — | Validation artifact is refreshed to the shipped automated commands and truthful hardware checklist | docs + integration | `cargo test -p dsview-cli --test capture_cli -- --nocapture && cargo test -p dsview-cli --test devices_cli -- --nocapture && rg -n "capture_json_success_reports_requested_and_effective_device_options|capture_apply_failure_reports_applied_steps_and_failed_step|cargo test -p dsview-cli --test capture_cli -- --nocapture|cargo test --workspace -- --nocapture" .planning/phases/13-option-aware-capture-reporting/13-VALIDATION.md && rg -n "requested and effective|schema_version|On a machine with a working DSLogic Plus" .planning/phases/13-option-aware-capture-reporting/13-VALIDATION.md && ! rg -n "wave_0_complete: false" .planning/phases/13-option-aware-capture-reporting/13-VALIDATION.md` | ✅ existing harness + validation artifact | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Completed Automated Coverage

- `cargo test -p dsview-sys --test device_options -- --nocapture` proves deterministic setter order plus fail-fast stop/filter/threshold/sample-rate/sample-limit/channel application behavior from Plan 13-01.
- `cargo test -p dsview-core --test acquisition -- --nocapture` proves the option-aware capture path applies validated requests once, reports partial-apply failures with `applied_steps` and `failed_step`, and preserves the baseline no-override branch.
- `cargo test -p dsview-core --test export_artifacts -- --nocapture` proves metadata carries requested and effective device-option facts, keeps `schema_version` at `2`, and preserves inherited baseline reporting.
- `cargo test -p dsview-cli --test capture_cli -- --nocapture` now locks the shipped spawned-binary contract with `capture_json_success_reports_requested_and_effective_device_options`, `capture_text_success_reports_effective_device_options_concisely`, `capture_apply_failure_reports_applied_steps_and_failed_step`, and `capture_without_overrides_reports_inherited_effective_device_options`.
- `cargo test -p dsview-cli --test devices_cli -- --nocapture` re-runs the untouched discovery contract so the Phase 12 `devices` surface does not drift while capture reporting changes land.
- `cargo test --workspace -- --nocapture` is the final pre-verification bundle before `/gsd-verify-work`.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Requested and effective option reporting stays aligned across text, JSON, and metadata during a real successful capture | RUN-04, RUN-05 | Local live runtime is currently unreliable on this machine (`ds_lib_init` fails with `LIBUSB_ERROR_OTHER`), so the final proof must run where device/libusb access works | On a machine with a working DSLogic Plus, run `cargo run -q -p dsview-cli -- capture --format json --handle <HANDLE> --sample-rate-hz 100000000 --sample-limit 4097 --channels 0,1,2,3,4,5,6,7 --operation-mode buffer --stop-option stop-after-samples --channel-mode buffer-200x8 --threshold-volts 2.4 --filter off --output /tmp/phase13-success.vcd --metadata-output /tmp/phase13-success.json`, confirm the command succeeds, then verify the JSON response includes a `device_options` block with both requested and effective values while the text-mode rerun prints only `effective options:` lines before the artifact paths. |
| Partial-apply failure reporting stays operationally honest on real hardware | RUN-04 | The debug fixture proves the shape, but only live hardware can confirm a real pre-acquisition setter failure surfaces the same contract | On a machine with a working DSLogic Plus, reproduce any pre-acquisition device-option setter failure you can trigger safely; when the capture fails before acquisition starts, verify the JSON error still includes both `applied_steps` and `failed_step`, and record which hardware/setup condition produced the failure. |
| Metadata sidecar preserves `schema_version` and the `device_options` block for downstream automation | RUN-05 | The local machine cannot complete a trustworthy libusb-backed capture, so the final sidecar proof must happen on hardware | On a machine with a working DSLogic Plus, open `/tmp/phase13-success.json` from the successful run above and verify `schema_version` is `2`, the `device_options` block contains requested and effective snapshots, and the artifact paths still match the `.vcd`/`.json` files written by the run. |

Do not mark the hardware step complete on this machine. Leave it for `/gsd-verify-work` on hardware with working libusb/device access.

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 90s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-04-13
