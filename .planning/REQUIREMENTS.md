# Requirements: DSView CLI

**Defined:** 2026-04-09
**Status:** Between milestones
**Core Value:** Users can reliably capture logic-analyzer data from `DSLogic Plus` via CLI and produce waveform output files that are easy for automation and AI agents to analyze.

## Current Status

- `v1.0 MVP` shipped on 2026-04-09 and is archived at `.planning/milestones/v1.0-REQUIREMENTS.md`.
- No new milestone requirements are active yet.
- The next planning step is to choose which candidate requirements become the next milestone scope.

## Active Requirements

- (None - define the next milestone first.)

## Candidate Requirements

### Decode and Richer Analysis

- **DEC-01**: User can run protocol decoders on captured logic data.
- **DEC-02**: User can export decoded results in a machine-readable format.

### Broader Device and Export Support

- **SUP-01**: User can use the CLI with additional DSLogic-family devices.
- **SUP-02**: User can export capture data in additional formats such as CSV.
- **SUP-03**: User can reuse named capture presets.

## Standing Constraints

| Constraint | Why it stays in force |
|-----------|------------------------|
| Treat `DSView/` as a read-only upstream dependency | The project still aims to reuse the proven native stack without forking it |
| Keep native/unsafe work isolated behind a small Rust boundary | This keeps future device and decode work from spreading FFI risk through the workspace |
| Favor scriptable CLI behavior and machine-readable outputs | The product still targets shell automation and agent workflows first |
| Preserve the shipped `DSLogic Plus` capture/export path while expanding scope carefully | Future milestones should build on the validated v1 baseline rather than destabilize it |

## Archived Milestone Reference

- `v1.0 MVP`: `.planning/milestones/v1.0-REQUIREMENTS.md`
- Milestone index: `.planning/MILESTONES.md`

## Next Step

Promote a subset of candidate requirements into the next milestone, then add requirement-to-phase traceability once the new roadmap is defined.

---
*Last updated: 2026-04-09 after shipping and archiving milestone v1.0*
