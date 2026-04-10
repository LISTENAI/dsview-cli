# Requirements: DSView CLI

**Defined:** 2026-04-10
**Status:** Defined for milestone `v1.1`
**Core Value:** Users can reliably capture logic-analyzer data from `DSLogic Plus` via CLI and produce waveform output files that are easy for automation and AI agents to analyze.

## v1.1 Requirements

### Device Option Discovery

- [ ] **OPT-01**: User can inspect the supported `DSLogic Plus` device-option values for operation mode, stop option, channel mode, threshold voltage, and filter selection from the CLI.

### Device Option Configuration

- [ ] **OPT-02**: User can choose `Buffer Mode` or `Stream Mode` for a `DSLogic Plus` capture run from the CLI.
- [ ] **OPT-03**: User can choose the DSLogic stop option for operation modes that support it.
- [ ] **OPT-04**: User can choose a DSLogic channel mode that determines valid channel count and maximum sample rate.
- [ ] **OPT-05**: User can choose which logic channels are enabled for a run within the selected channel-mode limit.
- [ ] **OPT-06**: User can choose the `DSLogic Plus` threshold voltage from the CLI.
- [ ] **OPT-07**: User can choose the DSLogic filter option from the CLI.

### Validation and Execution

- [ ] **VAL-01**: CLI rejects unsupported combinations of operation mode, channel mode, sample rate, sample limit, and enabled channels before acquisition starts.
- [ ] **VAL-02**: CLI rejects unsupported threshold, filter, or mode-incompatible stop-option values before acquisition starts.
- [ ] **RUN-04**: Capture applies the selected DSView-compatible device options before acquisition begins.
- [ ] **RUN-05**: CLI success output and machine-readable metadata record the effective device option values used for the run.

## v2 Requirements

### Deferred Workflow Expansion

- **CLI-04**: User can persist reusable named device-option presets.
- **CLI-05**: User can switch between `Single`, `Repeat`, and `Loop` collect behavior from the CLI.
- **TRIG-01**: User can configure advanced DSView trigger settings from the CLI.
- **DEC-01**: User can run protocol decoders on captured logic data.
- **SUP-01**: User can use the same device-option workflow with additional DSLogic-family devices.

## Out of Scope

| Feature | Reason |
|---------|--------|
| Preset persistence | Defer until raw option semantics are stable in the CLI |
| Repeat/loop collect behavior | This milestone focuses on device options, not higher-level acquisition scheduling |
| Advanced trigger programming | Too broad for the first device-option milestone |
| Protocol decode | Post-capture analysis remains a later milestone |
| Additional DSLogic-family devices | `v1.1` stays scoped to `DSLogic Plus` |
| Modifying `DSView/` submodule code | The project must continue consuming the upstream stack without changing it |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| OPT-01 | Phase 10 | Pending |
| OPT-02 | Phase 12 | Pending |
| OPT-03 | Phase 12 | Pending |
| OPT-04 | Phase 12 | Pending |
| OPT-05 | Phase 12 | Pending |
| OPT-06 | Phase 12 | Pending |
| OPT-07 | Phase 12 | Pending |
| VAL-01 | Phase 11 | Pending |
| VAL-02 | Phase 11 | Pending |
| RUN-04 | Phase 13 | Pending |
| RUN-05 | Phase 13 | Pending |

**Coverage:**
- v1.1 requirements: 11 total
- Mapped to phases: 11
- Unmapped: 0 ✓

---
*Requirements defined: 2026-04-10*
*Last updated: 2026-04-10 after milestone `v1.1` definition*
