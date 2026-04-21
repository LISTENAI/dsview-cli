# Roadmap: DSView CLI

## Overview

This roadmap defines milestone `v1.2 DSView protocol decode CLI foundation`. It builds directly on the shipped `v1.0` and `v1.1` baseline and adds a headless, config-driven protocol-decode workflow without expanding the existing `capture` command into a decoder-option surface.

## Milestone

**Milestone:** `v1.2 DSView protocol decode CLI foundation`
**Goal:** Bring DSView protocol decoder discovery, config-driven offline decode execution, and machine-readable decode output into the CLI while keeping capture and decode as separate composable workflows.
**Baseline:** `v1.1 DSLogic Plus device options` shipped on 2026-04-13 and is archived at `.planning/milestones/v1.1-ROADMAP.md`.

## Phases

- [ ] **Phase 14: Decode Runtime Boundary and Decoder Registry** - Add the native/runtime seam for `libsigrokdecode4DSL` and expose decoder discovery/inspection metadata to Rust and CLI.
- [ ] **Phase 15: Decode Config Model and Validation** - Define the config-driven decoder stack model and validate channel bindings/options before decode starts.
- [ ] **Phase 16: Offline Decode Execution** - Execute offline protocol decode against saved logic artifacts, including stacked decoder runs.
- [ ] **Phase 17: Decode Output and Workflow Reporting** - Publish stable machine-readable decode results and error/artifact reporting for a separate decode workflow.

## Phase Details

### Phase 14: Decode Runtime Boundary and Decoder Registry
**Goal**: Expose `libsigrokdecode4DSL` through the Rust/native boundary and make decoder metadata inspectable from the CLI.
**Depends on**: Phase 13 from milestone `v1.1`
**Requirements**: DEC-01, DEC-02
**Success Criteria** (what must be TRUE):
  1. The native/runtime layer can initialize the DSView decode engine, load decoder scripts, and enumerate available decoders without touching Qt UI classes.
  2. Rust-side types expose decoder ids, channels, options, annotations, and stack-relevant metadata in owned structures.
  3. The CLI can list and inspect decoder metadata in stable JSON and text forms.
**Plans**: 3 plans

Plans:
- [ ] `14-01-PLAN.md` - Map the `libsigrokdecode4DSL` API and packaging requirements into a minimal native decode runtime boundary.
- [ ] `14-02-PLAN.md` - Implement Rust-owned decoder registry and inspect structures over the new runtime seam.
- [ ] `14-03-PLAN.md` - Add `decode list` / `decode inspect` CLI commands with automated contract coverage.

### Phase 15: Decode Config Model and Validation
**Goal**: Create a config-driven decoder stack model that stays aligned with DSView concepts and validates before runtime execution.
**Depends on**: Phase 14
**Requirements**: DEC-03, DEC-04
**Success Criteria** (what must be TRUE):
  1. Users can express root decoder, stacked decoders, channel bindings, and options in a typed decode config file.
  2. Validation catches missing required channels, unknown options, invalid option values, and invalid stack composition before decode starts.
  3. The design keeps decoder-specific configuration out of the main `capture` command surface.
**Plans**: 3 plans

Plans:
- [ ] `15-01-PLAN.md` - Define the decode config schema and DSView-compatible decoder-stack model in Rust.
- [ ] `15-02-PLAN.md` - Implement metadata-driven config validation and stable validation errors.
- [ ] `15-03-PLAN.md` - Add CLI config loading/diagnostics coverage for valid and invalid decode configs.

### Phase 16: Offline Decode Execution
**Goal**: Run DSView protocol decoders against saved logic artifacts from the CLI, including stacked decoder flows.
**Depends on**: Phase 15
**Requirements**: DEC-05, DEC-07
**Success Criteria** (what must be TRUE):
  1. CLI can execute a decode run against a saved logic-data artifact without requiring GUI components.
  2. Runtime execution correctly feeds sample data and absolute sample ranges into the decode engine.
  3. Decoder stacks work when upstream decoder output is required by downstream decoders.
**Plans**: 3 plans

Plans:
- [ ] `16-01-PLAN.md` - Define the saved-artifact input contract and native sample-feeding path for offline decode.
- [ ] `16-02-PLAN.md` - Implement decode-session execution and stacked decoder orchestration through Rust/core/native layers.
- [ ] `16-03-PLAN.md` - Add regression coverage for offline decode success and representative execution failures.

### Phase 17: Decode Output and Workflow Reporting
**Goal**: Finalize decode output contracts, artifact reporting, and stable failure taxonomy for a separate decode workflow.
**Depends on**: Phase 16
**Requirements**: DEC-06, PIPE-01
**Success Criteria** (what must be TRUE):
  1. Successful decode runs emit machine-readable annotation output with sample ranges, decoder identity, and payload text/numeric fields.
  2. Decode failures are reported with stable categories covering runtime prerequisites, config issues, input issues, decode execution failures, and artifact write failures.
  3. The decode workflow is explicitly separate from `capture` while leaving a clean future handoff point for pipeline orchestration.
**Plans**: 3 plans

Plans:
- [ ] `17-01-PLAN.md` - Define the stable decode output schema and annotation event model.
- [ ] `17-02-PLAN.md` - Implement output/artifact writing plus stable workflow/reporting errors.
- [ ] `17-03-PLAN.md` - Lock CLI output contracts and end-to-end decode reporting coverage.

## Progress

**Execution Order:**
Phases execute in numeric order: 14 -> 15 -> 16 -> 17

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 14. Decode Runtime Boundary and Decoder Registry | 1/3 | In Progress|  |
| 15. Decode Config Model and Validation | 0/3 | Not started | - |
| 16. Offline Decode Execution | 0/3 | Not started | - |
| 17. Decode Output and Workflow Reporting | 0/3 | Not started | - |

## Archived Milestones

- `v1.1 DSLogic Plus device options`: `.planning/milestones/v1.1-ROADMAP.md`
- `v1.0 MVP`: `.planning/milestones/v1.0-ROADMAP.md`
- Milestone index: `.planning/MILESTONES.md`
- Archived audit: `.planning/milestones/v1.0-MILESTONE-AUDIT.md`

## Planning Notes

- Preserve the validated `v1.0` capture/export path and the shipped `v1.1` option workflow as the stable baseline.
- Keep `DSView/` read-only and continue isolating upstream integration behind Rust-owned boundaries.
- Favor config-driven decode over command-line option flattening.
- Keep full capture+decode pipelines, live decode, and broader device support for later milestones unless later phases prove they fit naturally.

## Status

- Active milestone: `v1.2 DSView protocol decode CLI foundation`
- Requirements file: `.planning/REQUIREMENTS.md`
- Next action: `/gsd-discuss-phase 14` or `/gsd-plan-phase 14`
