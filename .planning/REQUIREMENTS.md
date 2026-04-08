# Requirements: DSView CLI

**Defined:** 2026-04-03
**Core Value:** Users can reliably capture logic-analyzer data from `DSLogic Plus` via CLI and produce waveform output files that are easy for automation and AI agents to analyze.

## v1 Requirements

### Device Access

- [ ] **DEV-01**: User can list connected supported devices from the CLI.
- [ ] **DEV-02**: User can select a `DSLogic Plus` device explicitly for a capture run.
- [ ] **DEV-03**: CLI reports clear errors when no supported device is available or the target device cannot be opened.

### Capture Configuration

- [ ] **CAP-01**: User can set the sample rate for a capture run from the CLI.
- [ ] **CAP-02**: User can set the sample limit or capture depth for a capture run from the CLI.
- [ ] **CAP-03**: User can choose which logic channels are enabled for a capture run.
- [ ] **CAP-04**: CLI validates requested capture settings before starting acquisition.

### Capture Execution

- [ ] **RUN-01**: User can start a logic capture for `DSLogic Plus` from the CLI.
- [ ] **RUN-02**: CLI completes capture and closes the device session cleanly on success.
- [ ] **RUN-03**: CLI exits with a non-zero status and actionable diagnostics when capture fails.

### Export and Analysis Readiness

- [ ] **EXP-01**: User can export captured waveform data as a `VCD` file.
- [ ] **EXP-02**: Exported `VCD` preserves channel names and timing information needed for downstream waveform analysis.
- [ ] **EXP-03**: CLI writes a machine-readable metadata file describing the capture session.
- [ ] **EXP-04**: Metadata includes device model, enabled channels, sample rate, sample limit or actual sample count, capture timestamp, and tool version.

### CLI Workflow

- [x] **CLI-01**: User can run the full capture-and-export workflow non-interactively from a single CLI command.
- [x] **CLI-02**: User can choose the output path for generated artifacts.
- [x] **CLI-03**: CLI prints the locations of generated artifacts after a successful run.

## v2 Requirements

### Decode and Richer Analysis

- **DEC-01**: User can run protocol decoders on captured logic data.
- **DEC-02**: User can export decoded results in a machine-readable format.

### Broader Device and Export Support

- **SUP-01**: User can use the CLI with additional DSLogic-family devices.
- **SUP-02**: User can export capture data in additional formats such as CSV.
- **SUP-03**: User can reuse named capture presets.

## Out of Scope

| Feature | Reason |
|---------|--------|
| Built-in AI-agent invocation | v1 only needs to generate analyzable output files |
| Terminal waveform viewer | Current milestone focuses on capture/export, not visualization |
| Full DSView feature parity | Too broad for the first usable CLI release |
| All DSLogic or sigrok-compatible devices | v1 is intentionally limited to `DSLogic Plus` |
| Modifying `DSView/` submodule code | Project must consume the upstream stack without changing it |
| Protocol decode in v1 | Raw capture/export reliability comes first |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| DEV-01 | Phase 7 | Closed via `.planning/phases/02-device-discovery-and-session-bring-up/02-VERIFICATION.md` and `.planning/phases/02-device-discovery-and-session-bring-up/02-VALIDATION.md`; rerun `/gsd:audit-milestone` |
| DEV-02 | Phase 7 | Closed via `.planning/phases/02-device-discovery-and-session-bring-up/02-VERIFICATION.md` and `.planning/phases/02-device-discovery-and-session-bring-up/02-VALIDATION.md`; rerun `/gsd:audit-milestone` |
| DEV-03 | Phase 7 | Closed via `.planning/phases/02-device-discovery-and-session-bring-up/02-VERIFICATION.md` and `.planning/phases/02-device-discovery-and-session-bring-up/02-VALIDATION.md`; rerun `/gsd:audit-milestone` |
| CAP-01 | Phase 7 | Closed via `.planning/phases/03-capture-configuration-surface/03-VERIFICATION.md` and `.planning/phases/03-capture-configuration-surface/03-VALIDATION.md`; rerun `/gsd:audit-milestone` |
| CAP-02 | Phase 7 | Closed via `.planning/phases/03-capture-configuration-surface/03-VERIFICATION.md` and `.planning/phases/03-capture-configuration-surface/03-VALIDATION.md`; rerun `/gsd:audit-milestone` |
| CAP-03 | Phase 7 | Closed via `.planning/phases/03-capture-configuration-surface/03-VERIFICATION.md` and `.planning/phases/03-capture-configuration-surface/03-VALIDATION.md`; rerun `/gsd:audit-milestone` |
| CAP-04 | Phase 7 | Closed via `.planning/phases/03-capture-configuration-surface/03-VERIFICATION.md` and `.planning/phases/03-capture-configuration-surface/03-VALIDATION.md`; rerun `/gsd:audit-milestone` |
| RUN-01 | Phase 8 | Closed via `.planning/phases/04-acquisition-execution/VERIFICATION.md`; rerun `/gsd:audit-milestone` |
| RUN-02 | Phase 8 | Closed via `.planning/phases/04-acquisition-execution/VERIFICATION.md`; rerun `/gsd:audit-milestone` |
| RUN-03 | Phase 8 | Closed via `.planning/phases/04-acquisition-execution/VERIFICATION.md`; rerun `/gsd:audit-milestone` |
| EXP-01 | Phase 8 | Closed via `.planning/phases/05-export-artifacts/05-VERIFICATION.md`; rerun `/gsd:audit-milestone` |
| EXP-02 | Phase 8 | Closed via `.planning/phases/05-export-artifacts/05-VERIFICATION.md`; rerun `/gsd:audit-milestone` |
| EXP-03 | Phase 8 | Closed via `.planning/phases/05-export-artifacts/05-VERIFICATION.md`; rerun `/gsd:audit-milestone` |
| EXP-04 | Phase 8 | Closed via `.planning/phases/05-export-artifacts/05-VERIFICATION.md`; rerun `/gsd:audit-milestone` |
| CLI-01 | Phase 9 | Pending |
| CLI-02 | Phase 9 | Pending |
| CLI-03 | Phase 9 | Pending |

**Coverage:**
- v1 requirements: 17 total
- Mapped to phases: 17
- Unmapped: 0 ✓

---
*Requirements defined: 2026-04-03*
*Last updated: 2026-04-08 after aligning completed Phase 1-5 requirements with roadmap and export sign-off state*
