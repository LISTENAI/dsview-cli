---
status: complete
phase: 05-export-artifacts
source: 05-01-SUMMARY.md, 05-02-SUMMARY.md, 05-03-SUMMARY.md
started: 2026-04-08T03:54:11.734Z
updated: 2026-04-08T04:11:30.000Z
---

## Current Test

number: 5
name: Export path preserves scriptable CLI contract
expected: |
  The capture command should still behave like a scriptable CLI: explicit output path, machine-readable JSON response, stable success shape, and no misleading partial-artifact success result.
awaiting: none

## Tests

### 1. Successful capture exports both VCD and JSON artifacts
expected: Running the CLI capture command on a connected DSLogic Plus should finish with `clean_success`, write both the VCD and JSON sidecar to the requested output paths, and leave the command with a successful result.
result: pass

### 2. Exported VCD has sane timing semantics on real hardware
expected: The exported VCD should contain finite monotonic timestamps (for example `#0`, `#64`), no `#-nan` or `#inf`, and channel declarations that match the requested capture channels.
result: pass

### 3. Metadata sidecar reflects observed capture facts
expected: The JSON sidecar should report the DSLogic Plus model, selected handle, sample rate, requested sample limit, actual sample count, enabled channels, clean-success acquisition result, and the final VCD/JSON artifact paths.
result: pass

### 4. Device remains reusable after export
expected: A second bounded capture should also succeed immediately after the first export, producing a fresh VCD + JSON pair without needing a restart-only recovery step.
result: pass

### 5. Export path preserves scriptable CLI contract
expected: The capture command should still behave like a scriptable CLI: explicit output path, machine-readable JSON response, stable success shape, and no misleading partial-artifact success result.
result: pass

## Summary

total: 5
passed: 5
issues: 0
pending: 0
skipped: 0
blocked: 0

## Gaps

[none]
