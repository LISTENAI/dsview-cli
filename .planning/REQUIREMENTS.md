# Requirements: DSView CLI

**Defined:** 2026-04-14
**Status:** Defined for milestone `v1.2`
**Core Value:** Users can reliably capture logic-analyzer data from `DSLogic Plus` via CLI and produce waveform output files that are easy for automation and AI agents to analyze.

## v1.2 Requirements

### Decoder Discovery and Inspection

- [ ] **DEC-01**: User can list the available DSView protocol decoders from the CLI.
- [ ] **DEC-02**: User can inspect a decoder's channels, options, annotations, and stack-relevant metadata from the CLI.

### Decode Configuration

- [ ] **DEC-03**: User can define a decoder stack, channel bindings, and decoder options in a decode configuration file.
- [ ] **DEC-04**: CLI validates a decode configuration against the selected decoder metadata before execution starts.

### Offline Decode Execution

- [ ] **DEC-05**: User can run DSView protocol decoders against previously captured logic data from the CLI.
- [ ] **DEC-07**: User can execute stacked decoders where upstream decoder output feeds downstream decoder input.

### Output and Reporting

- [ ] **DEC-06**: User can receive machine-readable decode annotations that include sample ranges and decoder payload text or numeric values.
- [ ] **PIPE-01**: User can run decode as a separate workflow from capture and receive stable error reporting for runtime, config, input, execution, and artifact failures.

## Future Requirements

### Deferred Workflow Expansion

- **PIPE-02**: User can run a first-class capture-and-decode pipeline command that composes the separate capture and decode workflows.
- **DEC-08**: User can stream protocol decode during capture instead of only decoding saved artifacts.
- **DEC-09**: User can import and reuse DSView session-level decoder presets directly.
- **SUP-01**: User can use the same decode workflow with additional DSLogic-family devices.

## Out of Scope

| Feature | Reason |
|---------|--------|
| Flatten decoder-specific flags into `capture` | Would bloat the main acquisition command and make decoder UX brittle |
| Full DSView decode panel parity | This milestone is headless and automation-focused |
| Reusing Qt `DecoderStack` UI code as the engine | The milestone should integrate `libsigrokdecode4DSL` directly instead |
| Live decode during acquisition | Offline decode is the smallest useful and testable first increment |
| Broad device-family expansion | `v1.2` should stay focused on decode foundation on top of the shipped DSLogic Plus baseline |
| Modifying `DSView/` submodule code | The project continues to consume the upstream stack read-only |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| DEC-01 | Phase 14 | Planned |
| DEC-02 | Phase 14 | Planned |
| DEC-03 | Phase 15 | Planned |
| DEC-04 | Phase 15 | Planned |
| DEC-05 | Phase 16 | Planned |
| DEC-07 | Phase 16 | Planned |
| DEC-06 | Phase 17 | Planned |
| PIPE-01 | Phase 17 | Planned |

**Coverage:**
- v1.2 requirements: 8 total
- Mapped to phases: 8
- Unmapped: 0 ✓

---
*Requirements defined: 2026-04-14*
*Last updated: 2026-04-14 after milestone `v1.2` definition*
