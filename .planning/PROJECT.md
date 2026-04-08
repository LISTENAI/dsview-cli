# DSView CLI

## What This Is

DSView CLI is a Rust-based command-line tool for using DSLogic devices without the DSView GUI, starting with `DSLogic Plus`. The first milestone focuses on a scriptable capture workflow that reuses the existing device communication stack from the `DSView/` repository and exports machine-readable waveform data for downstream AI-agent analysis.

## Core Value

Users can reliably capture logic-analyzer data from `DSLogic Plus` via CLI and produce waveform output files that are easy for automation and AI agents to analyze.

## Requirements

### Validated

- [x] User can connect to a `DSLogic Plus` device from the CLI using the existing DSView/libsigrok4DSL stack. Validated in Phases 02-06.
- [x] User can configure core capture parameters from the CLI, including the options needed for a basic acquisition workflow. Validated in Phases 03-06.
- [x] User can start a capture from the CLI and export machine-readable waveform output for downstream analysis. Validated in Phases 04-06.
- [x] User can run the full capture-and-export workflow non-interactively from a single CLI command. Validated in Phase 06: CLI Productization.
- [x] User can choose artifact output locations and receive clear artifact path reporting after a successful run. Validated in Phase 06: CLI Productization.

### Active

- (None — milestone v1.0 scope is fully validated)

### Out of Scope

- Built-in AI-agent invocation or analysis orchestration — v1 only needs to generate analyzable output files.
- Full DSView feature parity across all devices — v1 is intentionally scoped to the minimum useful workflow on `DSLogic Plus`.
- Modifying the upstream `DSView/` codebase or libraries — this project must integrate with the existing stack without changing it.
- Terminal waveform rendering or a TUI viewer — the current goal is exported waveform data, not in-terminal visualization.

## Context

The workspace already contains an open-source `DSView/` project, which is a GUI application for DSLogic devices and communicates with hardware through its modified `libsigrok4DSL` stack. The CLI milestone now proves that a separate Rust command-line workflow can reuse that existing stack without modifying `DSView/`, while still giving shell users and automation a stable non-interactive capture/export path.

The validated v1 journey on `DSLogic Plus` is now closed loop: discover the device, open it safely, configure capture parameters, run a bounded capture, and export both `VCD` waveform data plus a machine-readable JSON sidecar. Phase 6 additionally verified the final product surface on real hardware with clear text-mode and JSON-mode artifact reporting, explicit artifact destination control, and immediate rerun reuse. Phase 8 then backfilled the remaining durable requirement-level verification chain so both acquisition (`RUN-*`) and export (`EXP-*`) requirements now close through explicit verification artifacts ahead of milestone re-audit.

## Constraints

- **Tech stack**: Implement the CLI in Rust — this is an explicit user requirement.
- **Dependency boundary**: Reuse `DSView/` and its modified `libsigrok4DSL` stack without modifying that repository — preserve upstream code and reduce divergence.
- **Scope**: Support `DSLogic Plus` only in v1 — keep the first milestone narrow and testable.
- **Workflow**: Optimize for scriptable CLI usage, not GUI or TUI interactions — the tool should fit automation and agent workflows.
- **Output**: Export machine-readable waveform files — downstream AI analysis depends on structured capture output.

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Build the new tool in Rust | User explicitly wants Rust for the CLI implementation | Adopted — implemented across `dsview-cli`, `dsview-core`, and `dsview-sys` |
| Keep `DSView/` unchanged and integrate with its existing libraries | Reuse proven device communication behavior while avoiding upstream modifications | Adopted — native integration stays behind the Rust boundary and `DSView/` remains read-only |
| Scope v1 to `DSLogic Plus` only | Narrower device scope reduces risk for the first usable release | Adopted — milestone v1.0 validates the DSLogic Plus capture/export workflow |
| Prioritize capture-and-export over full DSView parity | The immediate goal is a reliable automation workflow, not full feature coverage | Adopted — milestone v1.0 closes on non-interactive capture plus VCD/JSON artifacts |
| Do not include AI invocation in v1 | The first version only needs to generate output files for later analysis | Adopted — CLI stops at producing analyzable artifacts |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd:transition`):
1. Requirements invalidated? -> Move to Out of Scope with reason
2. Requirements validated? -> Move to Validated with phase reference
3. New requirements emerged? -> Add to Active
4. Decisions to log? -> Add to Key Decisions
5. "What This Is" still accurate? -> Update if drifted

**After each milestone** (via `/gsd:complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-04-08 after Phase 8 verification backfill completion and milestone re-audit preparation*
