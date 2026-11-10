---
mode: quick
description: Expose the DSLogic Plus hardware trigger (simple, stage, and serial) on the capture command and report trigger results in capture output.
completed_at: 2026-11-10
status: complete
---

# Quick Summary

This change implements `TRIG-01` (configure DSView trigger settings from the CLI) for DSLogic Plus captures.

## Delivered

- `dsview-sys`:
  - `bridge_runtime.c` resolves the ten exported `ds_trigger_*` setters and, optionally, the `trigger` data symbol.
  - `dsview_bridge_ds_apply_trigger` resets the native state and replays a fixed-size program through the setters. It rejects values that would trip DSView `assert`s and resets again if a setter fails part-way.
  - `dsview_bridge_ds_get_trigger_state` reads the native struct back.
  - `SR_DF_TRIGGER` packets are recorded as a trigger observation (`status`, `real_pos`, `remain_cnt`), reset together with the acquisition summary.
  - The Windows `.def` exports the trigger setters and `trigger DATA`.
  - Safe wrappers: `RuntimeBridge::{apply_trigger, disable_trigger, trigger_state, trigger_observation}`.
- `dsview-core`, new `trigger` module:
  - Condition tokens, and parsing for `--trigger` tokens and version-1 JSON configs (`simple`, `stage`, `serial`).
  - Validation against the validated device options:
    - trigger channels must be enabled and within the channel-mode width
    - position 1-90 (default 1)
    - stream mode is simple-only, with the position fixed at 1
    - stage rules: 1-16 stages, count 1..=i32::MAX, no always-true or never-true patterns
    - serial `start`, `stop` and `clock` flags each need an edge
  - Native program building that mirrors `TriggerDock::commit_trigger` and `LogicSignal::commit_trig`.
  - The trigger is applied as the `trigger` device-option apply step, after the other options.
  - `CaptureRunSummary.trigger`, plus an optional metadata `trigger` block (`requested` / `observed`) that is omitted when no trigger is armed. The schema version stays 2 because the change is additive.
  - Failed stream captures now delete their partial `*.streaming.tmp` file.
- `dsview-cli`:
  - `capture` gains `--trigger CH=COND[,…]`, `--trigger-config PATH` and `--trigger-position PERCENT` (Trigger help heading; the two sources are mutually exclusive, and the position requires one of them).
  - New error codes: `trigger_config_invalid`, `trigger_config_unreadable`, `trigger_channel_not_enabled`, `trigger_channel_unsupported`, `trigger_mode_unsupported`, `trigger_position_out_of_range`, and `capture_trigger_not_fired` for timeouts or incomplete stream runs that never saw a trigger hit.
  - A `trigger` block in the capture JSON, and a text line.
  - A `trigger` capability guide in `devices options` (JSON and text).
- Docs: README (EN and zh-CN) gains a Features bullet, a "Capture on a Hardware Trigger" section, and the new reference flags.

## Hardware findings (DSLogic Plus, internal-test counter and idle inputs)

- `observed.sample_index` (`ds_trigger_pos.real_pos`) is the exact trigger sample in the exported VCD. This was verified on 16-channel level, edge, stage-count, sequence, invert, OR, contiguous and serial triggers.
- The position is the pre-trigger share and is approximate. On real signals the trigger landed 0.03 to 0.30% of the capture after the requested share. If the event comes before the pre-trigger window is full, which is common with periodic signals, the capture starts early and `sample_index` is smaller.
- Stage `count` is the number of matches: count 3 fired on the 3rd D15 rise.
- Serial trigger:
  - it compares only the first `data_bits` bits after the start flag; there is no sliding window
  - the first-received bit is the MSB, the leftmost character of `data_value`
  - an empty or level-only start or stop flag never fires, which is why validation requires edges
- The 200 MHz ×8 and 400 MHz ×4 buffer modes fail at the `enabled_channels` apply step with `SR_ERR_ARG`, with or without a trigger. The released v1.2.4 fails the same way, because `set_enabled_channels` walks 16 channels while those modes expose 8 or 4. This is an existing bug and is out of scope here, so the halved and quartered trigger-mask paths are covered only by validation, not by hardware.

## Real-signal validation

The installed `v1.2.4-trigger-dev` bundle passed 23 of 23 cases on real signals. The source was an FPGA board driving two 3.3 V GPIO lines with a fixed 10 ms frame; captures used a 1.6 V threshold and buffer mode at 100 MHz unless noted. A script checked in each VCD that the requested condition holds at `trigger.observed.sample_index`.

- Simple triggers fired with the named edge at offset 0 and the named levels holding: `1=rising`, `0=rising,1=high`, `0=falling,1=low`, `1=falling`, `0=edge`, `0=high,1=high`.
- The two-stage trigger (channel 1 falling, then 8 channel-0 rises) fired on the 8th rise.
- The serial trigger with 8 one-bits fired on the 8th clock edge after the start flag.
- Conditions that never occur returned `capture_trigger_not_fired`, and the device stayed usable.
- Stream mode at 10 MHz fired at offset 0.
- Refusals worked: `trigger_channel_not_enabled` and `trigger_position_out_of_range`.
- Position 10/50/90% landed 0.03 to 0.30% of the capture after the requested share.

Raw hardware logs from both rounds are kept on the test host and are not committed.

## Verification

- `cargo test --workspace`: 315 passed, 0 failed across 24 test binaries. The new tests are `dsview-sys/tests/trigger.rs` (6), the `dsview-core` trigger unit tests (11) and `dsview-cli/tests/trigger_cli.rs` (13).
- The new files pass rustfmt. The repository was not rustfmt-clean before, so the existing files were not reformatted.
- Clippy reports no new lints in the trigger code, apart from the codebase-wide `result_large_err` pattern.
