# DSView CLI

## What This Is

DSView CLI is a Rust-based command-line tool for using DSLogic devices without the DSView GUI, starting with `DSLogic Plus`. The first milestone focuses on a scriptable capture workflow that reuses the existing device communication stack from the `DSView/` repository and exports machine-readable waveform data for downstream AI-agent analysis.

## Core Value

Users can reliably capture logic-analyzer data from `DSLogic Plus` via CLI and produce waveform output files that are easy for automation and AI agents to analyze.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] User can connect to a `DSLogic Plus` device from the CLI using the existing DSView/libsigrok4DSL stack.
- [ ] User can configure core capture parameters from the CLI, including the options needed for a basic acquisition workflow.
- [ ] User can start a capture from the CLI and export machine-readable waveform output for downstream analysis.

### Out of Scope

- Built-in AI-agent invocation or analysis orchestration — v1 only needs to generate analyzable output files.
- Full DSView feature parity across all devices — v1 is intentionally scoped to the minimum useful workflow on `DSLogic Plus`.
- Modifying the upstream `DSView/` codebase or libraries — this project must integrate with the existing stack without changing it.
- Terminal waveform rendering or a TUI viewer — the current goal is exported waveform data, not in-terminal visualization.

## Context

The workspace already contains an open-source `DSView/` project, which is a GUI application for DSLogic devices and communicates with hardware through its modified `libsigrok4DSL` stack. The user wants a separate CLI-oriented project that can achieve the same core device workflow without relying on the GUI. The implementation should be in Rust, but it should depend on and reuse the existing DSView-side libraries and behavior rather than reimplementing the device stack from scratch.

The first target device is `DSLogic Plus`. The desired initial user journey is a minimum closed loop: connect to the device, configure sampling parameters, acquire logic data, and save waveform output in a machine-readable format suitable for later AI-agent analysis. Broader device coverage, richer decode flows, and other DSView features may be added incrementally after the capture/export path is proven.

## Constraints

- **Tech stack**: Implement the CLI in Rust — this is an explicit user requirement.
- **Dependency boundary**: Reuse `DSView/` and its modified `libsigrok4DSL` stack without modifying that repository — preserve upstream code and reduce divergence.
- **Scope**: Support `DSLogic Plus` only in v1 — keep the first milestone narrow and testable.
- **Workflow**: Optimize for scriptable CLI usage, not GUI or TUI interactions — the tool should fit automation and agent workflows.
- **Output**: Export machine-readable waveform files — downstream AI analysis depends on structured capture output.

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Build the new tool in Rust | User explicitly wants Rust for the CLI implementation | — Pending |
| Keep `DSView/` unchanged and integrate with its existing libraries | Reuse proven device communication behavior while avoiding upstream modifications | — Pending |
| Scope v1 to `DSLogic Plus` only | Narrower device scope reduces risk for the first usable release | — Pending |
| Prioritize capture-and-export over full DSView parity | The immediate goal is a reliable automation workflow, not full feature coverage | — Pending |
| Do not include AI invocation in v1 | The first version only needs to generate output files for later analysis | — Pending |

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
*Last updated: 2026-04-03 after initialization*
