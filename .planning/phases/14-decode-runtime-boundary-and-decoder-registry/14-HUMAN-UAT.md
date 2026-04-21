---
status: resolved
phase: 14-decode-runtime-boundary-and-decoder-registry
source: [14-VERIFICATION.md]
started: 2026-04-21T05:28:10Z
updated: 2026-04-21T08:04:00Z
---

## Current Test

approved after manual verification

## Tests

### 1. Packaged Decode Bundle
expected: The packaged binary resolves the bundled decode runtime and decoder scripts without extra flags, and returns the same canonical JSON/text fields verified from the source-built path.
result: [passed] Simulated bundled layout under `/tmp/dsview-cli-bundle.*` resolved `decode-runtime/` and `decoders/` automatically; both `decode list` and `decode inspect 0:i2c` exited 0 without extra flags.

### 2. Success-Path Stderr Cleanliness
expected: Any stderr output is understood and acceptable for operator/automation use, or a follow-up is scheduled to suppress upstream decoder import noise.
result: [passed] `decode inspect 0:i2c` now runs with clean stderr after the decode-runtime symbol visibility fix. `decode list` still emits upstream `SyntaxWarning` messages from vendored decoders, and this was explicitly accepted during manual review.

## Summary

total: 2
passed: 2
issues: 0
pending: 0
skipped: 0
blocked: 0

## Gaps
None - approved for Phase 14 closeout.
