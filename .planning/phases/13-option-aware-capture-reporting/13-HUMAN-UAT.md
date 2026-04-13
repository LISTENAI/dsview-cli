---
status: resolved
phase: 13-option-aware-capture-reporting
source: [13-VERIFICATION.md]
started: 2026-04-13T12:04:52Z
updated: 2026-04-13T12:35:52Z
---

## Current Test

hardware verification completed

## Tests

### 1. Real successful capture reporting
expected: A real DSLogic Plus run succeeds, JSON and metadata both include `device_options.requested` and `device_options.effective`, and text shows only `effective options:` before artifact paths.
result: passed — real device run completed successfully with JSON requested/effective blocks and text-only `effective options:` output.

### 2. Real partial-apply failure honesty
expected: A safe pre-acquisition setter failure still reports both `applied_steps` and `failed_step` in the JSON error.
result: passed — real hardware apply failure returned `device_option_apply_failed` with both `applied_steps` and `failed_step`.

### 3. Metadata sidecar on hardware
expected: `schema_version` is `2`, `device_options` contains requested/effective snapshots, and the paths match the emitted `.vcd` and `.json` artifacts.
result: passed — hardware sidecar shows `schema_version: 2`, contains `device_options.requested` / `device_options.effective`, and artifact paths match emitted files.

## Summary

total: 3
passed: 3
issues: 0
pending: 0
skipped: 0
blocked: 0

## Gaps

None - hardware verification completed successfully.
