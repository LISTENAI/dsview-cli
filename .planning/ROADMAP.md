# Roadmap: DSView CLI

## Overview

This roadmap takes DSView CLI from project setup to a first usable Rust-based capture tool for `DSLogic Plus`. The sequence deliberately reduces brownfield risk first by validating how the Rust project can safely reuse the `DSView/` submodule stack, then builds upward through device access, capture configuration, acquisition execution, and finally VCD export plus CLI workflow polish.

## Phases

- [x] **Phase 1: Native Integration Foundation** - Prove and stabilize the Rust-to-DSView native boundary.
- [x] **Phase 2: Device Discovery and Session Bring-Up** - Enumerate `DSLogic Plus` devices and open sessions safely.
- [x] **Phase 3: Capture Configuration Surface** - Expose and validate the minimum useful capture parameters.
- [x] **Phase 4: Acquisition Execution** - Run reliable logic captures and handle session lifecycle cleanly.
- [x] **Phase 5: Export Artifacts** - Produce VCD waveform files and machine-readable capture metadata.
- [x] **Phase 6: CLI Productization** - Deliver a scriptable end-to-end CLI command with usable diagnostics and output UX.
- [x] **Phase 7: Verification Backfill for Bring-Up and Configuration** - Reconstruct verification evidence for the device-discovery and capture-configuration requirements reopened by the milestone audit.
- [ ] **Phase 8: Verification Backfill for Acquisition and Export** - Reconstruct verification evidence for the capture execution and export requirements reopened by the milestone audit.
- [ ] **Phase 9: Audit Closeout Reconciliation** - Close the remaining audit-process gaps so the milestone can pass re-audit cleanly.

## Phase Details

### Phase 1: Native Integration Foundation
**Goal**: Establish a stable Rust project structure and verify the lowest-risk way to reuse the `DSView/` submodule's capture stack without modifying it.
**Depends on**: Nothing (first phase)
**Requirements**: DEV-01
**Success Criteria** (what must be TRUE):
  1. A Rust workspace exists for the CLI and clearly separates CLI code from native integration code.
  2. The project can build against the chosen DSView/libsigrok native boundary without requiring DSView GUI integration.
  3. The submodule boundary and supported native dependency path are documented for future phases.
**Plans**: 3 plans

Plans:
- [x] 01-01: Create Rust workspace and crate boundaries for CLI, core, and native integration.
- [x] 01-02: Validate native build/link strategy against the DSView submodule and document constraints.
- [x] 01-03: Add minimal smoke coverage for the chosen native integration path.

### Phase 2: Device Discovery and Session Bring-Up
**Goal**: Let the CLI discover supported hardware and safely open a `DSLogic Plus` session through the proven native stack.
**Depends on**: Phase 1
**Requirements**: DEV-01, DEV-02, DEV-03
**Success Criteria** (what must be TRUE):
  1. User can list supported devices from the CLI and identify `DSLogic Plus` when connected.
  2. User can target a `DSLogic Plus` device for a capture session.
  3. The CLI reports clear, actionable errors when device discovery or open fails.
**Plans**: 3 plans

Plans:
- [x] 02-01: Implement device enumeration and filtering for supported devices.
- [x] 02-02: Implement session open/close flow for `DSLogic Plus`.
- [x] 02-03: Normalize native device/session errors into stable CLI diagnostics.

### Phase 3: Capture Configuration Surface
**Goal**: Expose the minimum useful capture controls for `DSLogic Plus` and validate them before acquisition starts.
**Depends on**: Phase 2
**Requirements**: CAP-01, CAP-02, CAP-03, CAP-04
**Success Criteria** (what must be TRUE):
  1. User can set sample rate, sample limit/depth, and enabled channels from the CLI.
  2. Invalid or unsupported capture settings are rejected before a run begins.
  3. Effective capture settings are represented in a reusable Rust-side domain model.
**Plans**: 3 plans

Plans:
- [x] 03-01: Define Rust domain types and validation rules for capture configuration.
- [x] 03-02: Wire validated capture settings into the native session layer.
- [x] 03-03: Add tests for valid, invalid, and device-specific capture configuration cases.

### Phase 4: Acquisition Execution
**Goal**: Execute logic captures reliably from the CLI while managing device/session lifecycle correctly.
**Depends on**: Phase 3
**Requirements**: RUN-01, RUN-02, RUN-03
**Success Criteria** (what must be TRUE):
  1. User can start a real capture from the CLI on `DSLogic Plus`.
  2. Successful runs close sessions cleanly and leave the device reusable.
  3. Failed runs return non-zero exit codes and actionable diagnostics.
**Plans**: 3 plans

Plans:
- [x] 04-01: Implement capture start/run/finish orchestration in the Rust service layer.
- [x] 04-02: Handle stop/cleanup/error paths so failed acquisitions do not leave broken state.
- [x] 04-03: Add smoke and integration validation for the acquisition lifecycle.

### Phase 5: Export Artifacts
**Goal**: Turn captures into reliable analysis artifacts by exporting VCD and a machine-readable metadata sidecar.
**Depends on**: Phase 4
**Requirements**: EXP-01, EXP-02, EXP-03, EXP-04
**Success Criteria** (what must be TRUE):
  1. User receives a valid VCD file from a successful capture run.
  2. Exported VCD retains channel naming and timing semantics needed for analysis.
  3. The CLI also writes a machine-readable metadata file with capture context.
  4. VCD and metadata writes follow an atomic-or-cleanup-safe contract so failed export does not leave misleading final-path artifacts.
  5. Phase completion includes explicit DSLogic Plus manual sign-off for real-hardware artifact plausibility and post-run device reusability.
**Plans**: 3 plans

Plans:
- [x] 05-01: Integrate or wrap the DSView-side VCD export path for CLI usage.
- [x] 05-02: Generate and validate JSON metadata sidecar output.
- [x] 05-03: Add artifact validation and golden-file checks for export correctness.

_Status note: automated 05-03 validation is complete, and manual DSLogic Plus export UAT has now passed on current hardware after the replay-ordering fix. The export path produces sane finite VCD timestamps, plausible metadata, and reusable post-run device state; the remaining milestone work is bookkeeping and Phase 6 sequencing rather than a Phase 5 export defect blocker._

### Phase 6: CLI Productization
**Goal**: Deliver a polished non-interactive capture-and-export command that works well in shell and agent workflows.
**Depends on**: Phase 5
**Requirements**: CLI-01, CLI-02, CLI-03
**Success Criteria** (what must be TRUE):
  1. User can run the full capture-and-export workflow from a single CLI command.
  2. User can choose artifact output locations explicitly.
  3. CLI reports generated artifact paths and final status clearly for automation.
**Plans**: 3 plans

Plans:
- [x] 06-01: Design and implement the end-to-end CLI command surface.
- [x] 06-02: Improve help text, logging, and output-path handling for scripts.
- [x] 06-03: Add end-to-end validation for the full capture-to-export workflow.

_Status note: Phase 6 is complete. The final `capture` command now supports non-interactive real-hardware capture/export from one CLI command, explicit artifact destination control, clear text-mode and JSON-mode artifact reporting, and successful immediate rerun reuse validated on the connected DSLogic Plus._

### Phase 7: Verification Backfill for Bring-Up and Configuration
**Goal**: Recreate durable verification evidence for the device-discovery and capture-configuration requirements so the milestone audit can prove those shipped behaviors instead of inferring them only from summaries.
**Depends on**: Phase 6
**Requirements**: DEV-01, DEV-02, DEV-03, CAP-01, CAP-02, CAP-03, CAP-04
**Gap Closure**: Closes milestone audit requirement gaps caused by missing verification artifacts for Phases 2 and 3.
**Success Criteria** (what must be TRUE):
  1. Phase 2 has a `02-VERIFICATION.md` artifact that explicitly verifies device listing, explicit device selection, and actionable device/open failure diagnostics.
  2. Phase 3 has a `03-VERIFICATION.md` artifact that explicitly verifies sample-rate, sample-limit, channel-selection, and pre-run validation behaviors.
  3. The evidence in those verification artifacts traces back to existing summaries, tests, and current code paths closely enough for milestone re-audit to mark the reopened requirements satisfied.
**Plans**: 2 plans

Plans:
- [x] 07-01: Backfill verification evidence for Phase 2 device discovery and session bring-up.
- [x] 07-02: Backfill verification evidence for Phase 3 capture configuration surface.

### Phase 8: Verification Backfill for Acquisition and Export
**Goal**: Recreate durable verification evidence for the acquisition and export requirements so the milestone audit can prove end-to-end capture/export behavior from persistent verification artifacts.
**Depends on**: Phase 7
**Requirements**: RUN-01, RUN-02, RUN-03, EXP-01, EXP-02, EXP-03, EXP-04
**Gap Closure**: Closes milestone audit requirement gaps caused by missing verification artifacts for Phase 5 and stale Phase 4 acquisition traceability.
**Success Criteria** (what must be TRUE):
  1. Existing `.planning/phases/04-acquisition-execution/VERIFICATION.md` is accepted as the requirement-closing artifact for RUN-01, RUN-02, and RUN-03; 08-01 reconciles roadmap and requirements traceability rather than creating a duplicate verification file.
  2. Phase 5 has a `05-VERIFICATION.md` artifact that explicitly verifies VCD export, metadata sidecar semantics, and the real-hardware export evidence already captured in validation and UAT documents.
  3. The reopened RUN-* and EXP-* requirements can be traced from `REQUIREMENTS.md` through verification evidence without relying on milestone-level inference.
**Plans**: 2 plans

Plans:
- [ ] 08-01: Reconcile existing Phase 4 acquisition verification for audit closure.
- [ ] 08-02: Backfill verification evidence for Phase 5 export artifacts.

### Phase 9: Audit Closeout Reconciliation
**Goal**: Eliminate the remaining process and documentation drift that still prevents a clean milestone re-audit.
**Depends on**: Phase 8
**Requirements**: DEV-01, CLI-01, CLI-02, CLI-03
**Gap Closure**: Closes the missing Phase 1 verification artifact, Phase 6 closeout-document drift, and residual validation metadata inconsistencies noted by the milestone audit.
**Success Criteria** (what must be TRUE):
  1. Phase 1 has a `01-VERIFICATION.md` artifact that records how the native integration foundation was validated strongly enough for the milestone audit.
  2. Phase 6 planning artifacts consistently reflect that the manual shell-workflow UAT passed and no longer describe it as an open gate.
  3. Phase 6 validation metadata is reconciled well enough that a milestone re-audit no longer reports partial closeout-process drift for the final shipped workflow.
**Plans**: 2 plans

Plans:
- [ ] 09-01: Backfill verification evidence for Phase 1 native integration foundation.
- [ ] 09-02: Reconcile Phase 6 validation and closeout artifacts for re-audit.

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3 -> 4 -> 5 -> 6

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Native Integration Foundation | 3/3 | Complete | 2026-04-03 |
| 2. Device Discovery and Session Bring-Up | 3/3 | Complete | 2026-04-03 |
| 3. Capture Configuration Surface | 3/3 | Complete | 2026-04-03 |
| 4. Acquisition Execution | 3/3 | Complete | 2026-04-07 |
| 5. Export Artifacts | 3/3 | Complete | 2026-04-08 |
| 6. CLI Productization | 3/3 | Complete | 2026-04-08 |
| 7. Verification Backfill for Bring-Up and Configuration | 2/2 | Complete | 2026-04-08 |
| 8. Verification Backfill for Acquisition and Export | 1/2 | In Progress | - |
| 9. Audit Closeout Reconciliation | 0/2 | Not started | - |
