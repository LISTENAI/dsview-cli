# Roadmap: DSView CLI

## Overview

This roadmap now tracks shipped milestones and the next planning checkpoint for the Rust-based `DSLogic Plus` CLI.

## Milestones

- ✅ **v1.1 DSLogic Plus device options** - Phases 10-13 (shipped 2026-04-13, archive: `.planning/milestones/v1.1-ROADMAP.md`)
- ✅ **v1.0 MVP** - Phases 1-9 (shipped 2026-04-09, archive: `.planning/milestones/v1.0-ROADMAP.md`)

## Current Status

- No active milestone is currently defined.
- Latest shipped scope: DSView-backed option discovery, validation, selection, runtime apply, and requested/effective reporting for `DSLogic Plus`.
- Next recommended workflow: `/gsd-new-milestone`

## Shipped Milestones

<details>
<summary>✅ v1.1 DSLogic Plus device options (Phases 10-13) - SHIPPED 2026-04-13</summary>

- [x] Phase 10: Device Option Bridge and Discovery (3/3 plans) - completed 2026-04-10
- [x] Phase 11: Device Option Validation Model (3/3 plans) - completed 2026-04-13
- [x] Phase 12: CLI Device Option Surface (3/3 plans) - completed 2026-04-13
- [x] Phase 13: Option-Aware Capture Reporting (3/3 plans) - completed 2026-04-13
- Archive: `.planning/milestones/v1.1-ROADMAP.md`

</details>

<details>
<summary>✅ v1.0 MVP (Phases 1-9) - SHIPPED 2026-04-09</summary>

- [x] Phase 1: Native Integration Foundation (3/3 plans) - completed 2026-04-03
- [x] Phase 2: Device Discovery and Session Bring-Up (3/3 plans) - completed 2026-04-03
- [x] Phase 3: Capture Configuration Surface (3/3 plans) - completed 2026-04-03
- [x] Phase 4: Acquisition Execution (3/3 plans) - completed 2026-04-07
- [x] Phase 5: Export Artifacts (3/3 plans) - completed 2026-04-08
- [x] Phase 6: CLI Productization (3/3 plans) - completed 2026-04-08
- [x] Phase 7: Verification Backfill for Bring-Up and Configuration (2/2 plans) - completed 2026-04-08
- [x] Phase 8: Verification Backfill for Acquisition and Export (2/2 plans) - completed 2026-04-08
- [x] Phase 9: Audit Closeout Reconciliation (2/2 plans) - completed 2026-04-08
- Archive: `.planning/milestones/v1.0-ROADMAP.md`

</details>

## Archived Milestones

- Milestone index: `.planning/MILESTONES.md`
- Archived roadmap copies: `.planning/milestones/`
- Archived audit currently available for `v1.0` at `.planning/milestones/v1.0-MILESTONE-AUDIT.md`

## Planning Notes

- Preserve the shipped `v1.1` baseline for DSLogic Plus device-option discovery, validation, and reporting.
- Keep `DSView/` read-only and continue isolating native integration behind the Rust boundary.
- Candidate future directions remain presets, repeat/loop collect behavior, trigger programming, protocol decode, and broader hardware support.

## Status

- Latest milestone completed: `v1.1 DSLogic Plus device options`
- Requirements archive: `.planning/milestones/v1.1-REQUIREMENTS.md`
- Next action: define the next milestone before recreating `.planning/REQUIREMENTS.md`
