---
status: complete
phase: 06-cli-productization
source:
  - .planning/phases/06-cli-productization/06-01-SUMMARY.md
  - .planning/phases/06-cli-productization/06-02-SUMMARY.md
  - .planning/phases/06-cli-productization/06-03-SUMMARY.md
started: 2026-04-08T09:25:08Z
updated: 2026-04-08T10:02:10Z
---

## Current Test

number: 4
name: Immediate Re-run Reuse
expected: |
  After one successful real-hardware run, launching the same capture workflow again should still work without restart-only recovery, proving the device remains reusable.
awaiting: none

## Tests

### 1. Final Capture Shell Workflow
expected: Run the final `capture` command once against the connected DSLogic Plus using the polished Phase 6 CLI surface. The command should stay non-interactive, exit successfully, and create both the VCD artifact and metadata JSON artifact at the requested or derived paths.
result: pass
notes: 2026-04-08 real-hardware run with `cargo run -p dsview-cli -- capture --use-source-runtime --resource-dir /home/seasonyuu/projects/dsview-cli/DSView/DSView/res --handle 1 --sample-rate-hz 1000000 --sample-limit 1024 --channels 0 --wait-timeout-ms 10000 --poll-interval-ms 50 --format text --output /home/seasonyuu/projects/dsview-cli/.tmp/manual-uat-phase6/text-run.vcd` returned `clean_success` and wrote both artifacts to the requested/derived paths.

### 2. Text Mode Artifact Summary
expected: On a successful `--format text` capture run, stdout should show a concise human-readable summary with the final completion state, the final VCD path, and the final metadata path.
result: pass
notes: stdout reported `capture clean_success`, the final VCD path, and the final metadata path exactly as the operator-facing summary.

### 3. JSON Automation Output
expected: On a successful JSON-mode capture run, stdout should remain machine-readable and include the final completion facts plus both artifact paths so scripts and agents can trust the result without parsing text.
result: pass
notes: 2026-04-08 real-hardware JSON run returned machine-readable fields for `completion`, `saw_logic_packet`, `saw_end_packet`, `saw_terminal_normal_end`, `cleanup_succeeded`, and `artifacts.{vcd_path,metadata_path}`.

### 4. Immediate Re-run Reuse
expected: After one successful real-hardware run, launching the same capture workflow again should still work without restart-only recovery, proving the device remains reusable.
result: pass
notes: immediate second JSON-mode hardware capture succeeded with `clean_success` and wrote a fresh `rerun.vcd` plus `rerun.json` artifact pair.

## Summary

total: 4
passed: 4
issues: 0
pending: 0
skipped: 0
blocked: 0

## Gaps

[none]
