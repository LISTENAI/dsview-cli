# Phase 03 Validation

**Date:** 2026-04-08
**Phase:** 03 - Capture Configuration Surface
**Purpose:** Minimal validation companion for the Phase 03 verification backfill.

## Scope

This artifact is intentionally minimal. Phase 07 plan `07-02` backfills durable evidence for already-shipped Phase 3 behavior; it does not broaden Phase 3 scope or invent a new configuration feature or hardware campaign.

## Validation approach

No new broad manual rerun was required to close the durable-evidence gap for `CAP-01`, `CAP-02`, `CAP-03`, or `CAP-04`.

The verification backfill in `.planning/phases/03-capture-configuration-surface/03-VERIFICATION.md` is sufficient because it already ties each requirement to:

- current implementation evidence in `crates/dsview-core/src/capture_config.rs`, `crates/dsview-core/src/lib.rs`, and `crates/dsview-sys/src/lib.rs`
- original Phase 3 summary evidence in `.planning/phases/03-capture-configuration-surface/03-01-SUMMARY.md`, `.planning/phases/03-capture-configuration-surface/03-02-SUMMARY.md`, and `.planning/phases/03-capture-configuration-surface/03-03-SUMMARY.md`
- existing command/test evidence already on record, including `cargo test -p dsview-core capture_config`, `cargo test -p dsview-core`, and `cargo test -p dsview-sys`
- the truthful but partial hardware context in `.planning/phases/03-capture-configuration-surface/03-UAT.md`

## Why minimal validation is sufficient

- The audit gap was missing persistent verification artifacts, not missing shipped Phase 3 behavior.
- `.planning/phases/03-capture-configuration-surface/03-UAT.md` already preserves the real-device context honestly:
  - opening a `DSLogic Plus` session from the source runtime passed
  - releasing the device cleanly after config checks passed
  - direct capability inspection, valid config apply, and invalid config rejection were skipped because the current CLI did not expose config-only commands for manual verification
- Because those skipped UAT items leave `CAP-03` and `CAP-04` under-proved if treated as hardware-only requirements, the narrow supplementary evidence path is the existing automated Phase 3 command set already recorded in the phase plans and summaries:
  - `cargo test -p dsview-core capture_config`
  - `cargo test -p dsview-core`
  - `cargo test -p dsview-sys`
- Those commands are the truthful audit-closure supplement because they directly exercise the shipped request-validation model, capability-driven limits, and native apply boundary shape without claiming hardware evidence that was never captured.

## Requirement supplement map

### CAP-01

No supplement beyond the existing automated evidence inventory was required. The current implementation and Phase 3 summaries already durably prove sample-rate request support.

### CAP-02

No supplement beyond the existing automated evidence inventory was required. The current implementation and Phase 3 summaries already durably prove sample-limit/depth request and normalization support.

### CAP-03

`03-UAT.md` was not sufficient alone because the valid config-apply item was skipped.

Truthful supplement path retained for audit closure:

- `cargo test -p dsview-core capture_config`
- `cargo test -p dsview-core`

Why these close the gap:

- they prove enabled-channel requests are modeled explicitly
- they prove empty, out-of-range, and excessive channel selections are rejected
- they preserve the validated channel set that the Phase 3 apply path consumes

### CAP-04

`03-UAT.md` was not sufficient alone because the invalid-config rejection item was skipped.

Truthful supplement path retained for audit closure:

- `cargo test -p dsview-core capture_config`
- `cargo test -p dsview-core`
- `cargo test -p dsview-sys`

Why these close the gap:

- they prove malformed and unsupported requests are rejected before acquisition
- they prove the ordered core contract remains `capabilities -> validate -> apply`
- they preserve the native capability/apply boundary shape without implying acquisition or export success

## Outcome

Minimal validation accepted.

The Phase 03 backfill is complete once `03-VERIFICATION.md` exists, this minimal validation rationale exists, and the milestone audit is later re-run against the new artifacts instead of the old orphaned-verification snapshot.
