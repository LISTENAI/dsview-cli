---
mode: quick
description: Expose the DSLogic Plus hardware trigger (simple, stage, and serial) on the capture command and report trigger results in capture output.
created_at: 2026-11-10
---

# Quick Plan

Requirement `TRIG-01` (deferred in `v1.1-REQUIREMENTS.md`): the user can configure DSView trigger settings from the CLI.

## Upstream facts (DSView `e93e644f`, read-only)

- `libsigrok4DSL/trigger.c` keeps one global `struct ds_trigger`. `dsl_fpga_arm()` writes it to the FPGA when collection starts. The setters are exported `SR_API` symbols in the runtime library: `ds_trigger_reset`, `ds_trigger_set_en`, `ds_trigger_set_mode`, `ds_trigger_set_pos`, `ds_trigger_set_stage`, `ds_trigger_probe_set`, `ds_trigger_stage_set_value`, `ds_trigger_stage_set_logic`, `ds_trigger_stage_set_inv`, `ds_trigger_stage_set_count`.
- Simple trigger: per-channel condition `X/0/1/R/F/C` in row `TriggerStages`. All set channels must match on the same sample (`LogicSignal::commit_trig`).
- Stage trigger (`ADV_TRIGGER`): 1-16 stages. Each stage has two 16-channel patterns, each with its own `==`/`!=` inversion, combined with `or`/`and`, plus a counter and a contiguous flag (`TriggerDock::commit_trigger`).
- Serial trigger (`SERIAL_TRIGGER`), as stages:
  - stage 0: start and stop flags
  - stage 1: clock flag, count 1
  - stage 2: data channel marked `0`
  - stage 3: data value, count `bits - 1`
- Position is a percent of the sample limit (GUI range 1-90, default 1). Stream mode forbids advanced triggers and fixes the position at 1.
- The device reports the trigger point in an `SR_DF_TRIGGER` packet: `ds_trigger_pos.status & 1` is the hit flag and `real_pos` is the sample index.

## Tasks

1. `dsview-sys`:
   - Load the trigger setters.
   - Add `dsview_bridge_ds_apply_trigger` to program a fixed-size trigger request mechanically.
   - Add `dsview_bridge_ds_get_trigger_state` to read the native struct back.
   - Record `SR_DF_TRIGGER` packets as a trigger observation, reset together with the acquisition summary.
   - Add safe Rust wrappers and a boundary test that applies programs and reads back the native state.
2. `dsview-core`, new `trigger` module:
   - Condition tokens.
   - JSON trigger config (version 1; modes `simple`, `stage`, `serial`).
   - Validation against the validated device options: enabled channels, channel-mode width, stream-mode limits, position range.
   - Native program building.
   - Apply the trigger as a `trigger` device-option apply step.
   - Return the trigger outcome in `CaptureRunSummary`.
   - Add an optional `trigger` block in schema-v2 metadata (omitted when no trigger is requested).
3. `dsview-cli`:
   - Add `--trigger CH=COND[,...]`, `--trigger-config PATH` and `--trigger-position PERCENT` to `capture`.
   - Add trigger-specific error codes, including a trigger-not-fired classification for timeouts.
   - Add a `trigger` block in the capture JSON and text output.
   - Report trigger capabilities in `devices options`.
4. Verify:
   - `cargo build`, `cargo test --workspace`, `cargo fmt --check`, `cargo clippy`.
   - DSLogic Plus hardware runs on the `internal-test` counter: simple level and edge triggers, a stage sequence, the trigger position, and a never-firing trigger timeout.
   - Keep the raw logs.
5. Update the README (EN and zh-CN) and this quick task's summary.
