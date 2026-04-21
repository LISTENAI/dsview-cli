---
phase: 16-offline-decode-execution
plan: 02
subsystem: decode-runtime
tags: [offline-decode, sigrokdecode, chunking, diagnostics]
requires:
  - phase: 15-decode-config-model-and-validation
    provides: strict validated decoder-stack configs and typed option values
  - phase: 14-decode-runtime-boundary-and-decoder-registry
    provides: decode runtime/session bridge and registry metadata
provides:
  - chunked offline decode execution over validated configs and raw logic artifacts
  - root-only channel binding with strictly linear stack construction
  - hard-fail send/end handling with retained diagnostic annotations
  - session-level stacked-runtime callback coverage for OUTPUT_PYTHON traffic
affects: [16-03, phase-17-reporting, decode-cli]
tech-stack:
  added: []
  patterns:
    - generic runtime-session seam for offline decode execution tests
    - callback-backed decode session annotation capture in dsview-sys
    - packet-aware preferred chunking with fixed-size fallback
key-files:
  created:
    - .planning/phases/16-offline-decode-execution/16-02-SUMMARY.md
  modified:
    - crates/dsview-core/src/lib.rs
    - crates/dsview-core/tests/decode_execute.rs
    - crates/dsview-sys/bridge_runtime.c
    - crates/dsview-sys/src/lib.rs
    - crates/dsview-sys/tests/boundary.rs
    - crates/dsview-sys/wrapper.h
key-decisions:
  - Use a 4096-byte aligned fallback chunk size when packet boundaries are absent.
  - Drain decode callback captures after each send/end and keep only true annotation events in core results.
  - Anchor stacked-runtime verification on the upstream `1:i2c` decoder because it emits the OUTPUT_PYTHON traffic needed to observe ordered stacked execution.
requirements-completed: [DEC-05, DEC-07]
duration: 14 min
completed: 2026-04-21T11:19:24Z
---

# Phase 16 Plan 02: Offline Decode Execution Summary

Chunked offline decode execution with hard-fail runtime semantics, root-only stack binding, and callback-backed stacked runtime diagnostics.

## Outcome

- Implemented `run_offline_decode()` in `crates/dsview-core/src/lib.rs` with deterministic packet-first chunking, a fixed-size fallback path, and a single absolute sample cursor across sends.
- Added public runtime/session test seams so integration tests can drive decode execution without requiring live runtime sessions.
- Kept success semantics binary: send/end/runtime failures now abort the run while retaining diagnostic annotations internally on the error path.
- Extended `dsview-sys` decode sessions to drain captured callback output so boundary coverage can observe ordered runtime callback traffic on real stacked decoder chains.

## Task Results

### Task 1: Implement core offline decode orchestration with strict linear stack semantics

- RED commit `310f5de` added failing executor regressions for absolute cursor progression, packet-aware chunking, and root-only logic bindings.
- GREEN commit `164a69a` added the offline executor, root/stack projection helpers, fixed-size fallback chunking, and the runtime-session seam used by the new tests.
- Verification passed: `cargo test -p dsview-core --test decode_execute -- --nocapture`

### Task 2: Propagate stacked decoder output and fail hard on runtime send/end errors

- RED commit `b6893b8` added failing failure-path coverage in `crates/dsview-core/tests/decode_execute.rs` and stacked-runtime regression coverage in `crates/dsview-sys/tests/boundary.rs`.
- GREEN commit `4fbbc69` added callback-backed decode-session capture plumbing in `dsview-sys`, drained those diagnostics through the Rust wrapper, and wired core execution to fail hard while retaining internal annotations.
- Verification passed:
  - `cargo test -p dsview-sys --test boundary -- --nocapture`
  - `cargo test -p dsview-core --test decode_execute -- --nocapture`

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Added decode-session callback capture plumbing**
- **Found during:** Task 2
- **Issue:** The existing session bridge could send/end stacked decode runs, but it could not surface callback output needed to verify ordered stacked runtime behavior or retain partial diagnostic annotations.
- **Fix:** Added decode-session callback capture and drain support in `crates/dsview-sys/bridge_runtime.c`, `crates/dsview-sys/wrapper.h`, and `crates/dsview-sys/src/lib.rs`, then filtered those captured records into core-facing annotation diagnostics in `crates/dsview-core/src/lib.rs`.
- **Files modified:** `crates/dsview-core/src/lib.rs`, `crates/dsview-sys/bridge_runtime.c`, `crates/dsview-sys/src/lib.rs`, `crates/dsview-sys/wrapper.h`
- **Verification:** `cargo test -p dsview-sys --test boundary -- --nocapture`; `cargo test -p dsview-core --test decode_execute -- --nocapture`
- **Commit:** `4fbbc69`

**Total deviations:** 1 auto-fixed

## Verification

- `cargo test -p dsview-sys --test boundary -- --nocapture`
- `cargo test -p dsview-core --test decode_execute -- --nocapture`
- `rg -n "pub fn run_offline_decode|struct OfflineDecodeResult" crates/dsview-core/src/lib.rs`
- `rg -n "offline_decode_uses_absolute_sample_cursor_across_chunks|offline_decode_prefers_packet_boundaries_when_available|offline_decode_root_only_binds_logic_channels|offline_decode_fails_when_session_send_fails|offline_decode_fails_when_session_end_fails|offline_decode_retains_partial_annotations_for_diagnostics_only" crates/dsview-core/tests/decode_execute.rs`
- `rg -n "stacked_decoder_python_output_flows_linearly" crates/dsview-sys/tests/boundary.rs`

## Known Stubs

None.

## Threat Flags

None.

## Commits

- `310f5de` — `test(16-02): add failing tests for offline decode execution`
- `164a69a` — `feat(16-02): implement offline decode executor`
- `b6893b8` — `test(16-02): add failing coverage for decode failure handling`
- `4fbbc69` — `feat(16-02): fail hard on decode runtime errors`

## Self-Check: PASSED

- Verified `.planning/phases/16-offline-decode-execution/16-02-SUMMARY.md` exists on disk.
- Verified commits `310f5de`, `164a69a`, `b6893b8`, and `4fbbc69` are present in `git log --oneline --all`.
