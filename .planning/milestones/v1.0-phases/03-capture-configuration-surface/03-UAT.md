---
status: partial
phase: 03-capture-configuration-surface
source:
  - 03-01-SUMMARY.md
  - 03-02-SUMMARY.md
  - 03-03-SUMMARY.md
started: 2026-04-07T00:00:00Z
updated: 2026-04-07T12:05:00Z
---

## Current Test

[testing paused — 3 items outstanding]

## Tests

### 1. Open DSLogic Plus session from the source runtime
expected: Run the DSView CLI against your connected DSLogic Plus with `--use-source-runtime` and the real resource directory. The command should reach the device/session path without crashing, and it should identify a supported DSLogic Plus device that can be opened for configuration work.
result: pass

### 2. Inspect capture capabilities from the opened device
expected: After opening the device, capability inspection should return a coherent DSLogic Plus capture snapshot, including sample-rate choices, channel counts, current mode, and depth-related values instead of a native read failure.
result: skipped
reason: current CLI does not expose a capability-inspection command for manual verification

### 3. Apply one valid capture configuration
expected: A valid DSLogic Plus configuration should apply successfully before acquisition starts. Sample rate, sample limit, and enabled channels should be accepted without starting capture or forcing a reconnect.
result: skipped
reason: current CLI does not expose a config-apply command for manual verification

### 4. Reject one invalid capture configuration before acquisition
expected: An invalid configuration such as an unsupported sample rate, too many enabled channels, or an over-capacity sample limit should be rejected before acquisition starts, with a clear validation failure instead of a partial native apply.
result: skipped
reason: current CLI does not expose a config-validation/apply command for manual verification

### 5. Release the device cleanly after config checks
expected: After the checks, the device should release cleanly so the session can end without leaving DSLogic Plus stuck busy for the next attempt.
result: pass

## Summary

total: 5
passed: 2
issues: 0
pending: 0
skipped: 3
blocked: 0

## Gaps

[none yet]
