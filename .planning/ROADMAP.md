# Roadmap: DSView CLI

## Overview

This roadmap defines milestone `v1.1 DSLogic Plus device options`. It builds directly on the shipped `v1.0 MVP` capture/export baseline and focuses on exposing the core DSView logic-mode device options through the existing Rust CLI without modifying the upstream `DSView/` codebase.

## Milestone

**Milestone:** `v1.1 DSLogic Plus device options`
**Goal:** Bring the core DSView logic-mode device options for `DSLogic Plus` into the CLI while preserving the shipped non-interactive capture/export workflow.
**Baseline:** `v1.0 MVP` shipped on 2026-04-09 and is archived at `.planning/milestones/v1.0-ROADMAP.md`.

## Phases

- [x] **Phase 10: Device Option Bridge and Discovery** - Extend the Rust/native boundary to expose DSView-backed option lists and current values for `DSLogic Plus`. (completed 2026-04-10)
- [x] **Phase 11: Device Option Validation Model** - Represent DSLogic Plus device-option combinations in Rust and validate them before acquisition. (completed 2026-04-13)
- [ ] **Phase 12: CLI Device Option Surface** - Add a discoverable non-interactive CLI surface for selecting DSView-compatible device options.
- [ ] **Phase 13: Option-Aware Capture Reporting** - Apply selected options during capture and publish the effective option facts in outputs and metadata.

## Phase Details

### Phase 10: Device Option Bridge and Discovery
**Goal**: Expose the DSView-backed `DSLogic Plus` option surface through the Rust boundary and make the supported values inspectable from the CLI.
**Depends on**: Phase 9 from milestone `v1.0`
**Requirements**: OPT-01
**Success Criteria** (what must be TRUE):
  1. The native bridge can enumerate the supported `DSLogic Plus` values for operation mode, stop option, channel mode, threshold voltage, and filter selection.
  2. Rust-side types normalize those option lists into stable IDs and labels suitable for automation-facing CLI output.
  3. The CLI can print the supported option surface for a selected `DSLogic Plus` device in stable text and JSON forms.
**Plans**: 3 plans

Plans:
- [x] `10-01-PLAN.md` - Map the DSView/libsigrok option IDs and current-value access needed for `DSLogic Plus` device options.
- [x] `10-02-PLAN.md` - Implement sys-bridge and core typed structures for DSLogic option discovery and current-value reporting.
- [x] `10-03-PLAN.md` - Add CLI discovery output plus automated coverage for stable option-list reporting.

### Phase 11: Device Option Validation Model
**Goal**: Define and enforce the DSLogic Plus device-option rules before any acquisition begins.
**Depends on**: Phase 10
**Requirements**: VAL-01, VAL-02
**Success Criteria** (what must be TRUE):
  1. The Rust domain model represents operation mode, stop option, channel mode, sample rate, sample limit, enabled channels, threshold voltage, and filter selection together.
  2. Validation enforces mode-aware constraints before capture, including channel-count limits, sample-rate ceilings, and option incompatibilities.
  3. Validation failures surface stable machine-readable error categories that the CLI can report clearly.
**Plans**: 3 plans

Plans:
- [x] `11-01-PLAN.md` - Define the internal validation request/capability model and selected-device capability loader without changing Phase 10 discovery output.
- [x] `11-02-PLAN.md` - Implement pure mode-aware validation plus stable machine-readable validation error codes.
- [x] `11-03-PLAN.md` - Lock DSView-rule coverage and CLI validation-code regressions.

### Phase 12: CLI Device Option Surface
**Goal**: Let users choose the relevant DSView-compatible `DSLogic Plus` device options directly from the CLI without relying on GUI profiles.
**Depends on**: Phase 11
**Requirements**: OPT-02, OPT-03, OPT-04, OPT-05, OPT-06, OPT-07
**Success Criteria** (what must be TRUE):
  1. Users can select buffer/stream mode, stop option, channel mode, enabled channels, threshold voltage, and filter selection from the CLI.
  2. CLI help and diagnostics make the valid values and mode-dependent constraints understandable without opening DSView.
  3. The option-selection surface stays non-interactive and script-friendly for shell and agent workflows.
**Plans**: 3 plans

Plans:
- [ ] `12-01-PLAN.md` - Define the friendly token contract and capture-oriented `devices options` inspection surface.
- [ ] `12-02-PLAN.md` - Add optional `capture` device-option flags and validate resolved selections against the Phase 11 model.
- [ ] `12-03-PLAN.md` - Lock help, parser behavior, and inspection output with spawned CLI regression tests.

### Phase 13: Option-Aware Capture Reporting
**Goal**: Apply the selected options during capture and preserve the effective option facts in the final capture artifacts.
**Depends on**: Phase 12
**Requirements**: RUN-04, RUN-05
**Success Criteria** (what must be TRUE):
  1. Capture applies the selected DSView-compatible device options in a deterministic order before acquisition starts and fails cleanly if the runtime rejects them.
  2. Successful runs report the effective device-option values in text/JSON output and in the metadata sidecar.
  3. Regression coverage confirms the default `v1.0` capture/export path still works while the new option-aware flows also behave correctly.
**Plans**: 3 plans

Plans:
- [ ] 13-01: Apply validated device options through the sys/core runtime layer before capture start.
- [ ] 13-02: Extend capture output and metadata with effective DSLogic Plus option facts.
- [ ] 13-03: Add regression coverage and manual validation for the default baseline plus option-aware runs.

## Progress

**Execution Order:**
Phases execute in numeric order: 10 -> 11 -> 12 -> 13

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 10. Device Option Bridge and Discovery | 3/3 | Complete    | 2026-04-10 |
| 11. Device Option Validation Model | 3/3 | Complete    | 2026-04-13 |
| 12. CLI Device Option Surface | 0/3 | Pending | - |
| 13. Option-Aware Capture Reporting | 0/3 | Pending | - |

## Archived Milestones

- `v1.0 MVP`: `.planning/milestones/v1.0-ROADMAP.md`
- Milestone index: `.planning/MILESTONES.md`
- Archived audit: `.planning/milestones/v1.0-MILESTONE-AUDIT.md`

## Planning Notes

- Preserve the validated `v1.0` capture/export path as the stable baseline for all `v1.1` work.
- Keep `DSView/` read-only and continue isolating native integration behind the Rust boundary.
- Defer presets, repeat/loop collect behavior, advanced trigger work, protocol decode, and broader hardware support until after DSLogic Plus device options are stable in the CLI.

## Status

- Active milestone: `v1.1 DSLogic Plus device options`
- Phase 10 completed and live-verified on: `2026-04-10`
- Next action: `/gsd-discuss-phase 12`
