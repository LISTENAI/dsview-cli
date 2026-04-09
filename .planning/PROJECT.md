# DSView CLI

## What This Is

DSView CLI is a shipped Rust-based command-line tool for using `DSLogic Plus` devices without the DSView GUI. The v1.0 milestone proved a stable non-interactive workflow that reuses the existing `DSView/` device communication stack and exports machine-readable waveform artifacts for downstream automation and AI-agent analysis.

## Core Value

Users can reliably capture logic-analyzer data from `DSLogic Plus` via CLI and produce waveform output files that are easy for automation and AI agents to analyze.

## Requirements

### Validated

- [x] User can connect to a `DSLogic Plus` device from the CLI using the existing DSView/libsigrok4DSL stack. Shipped in v1.0.
- [x] User can configure core capture parameters from the CLI, including the options needed for a basic acquisition workflow. Shipped in v1.0.
- [x] User can start a capture from the CLI and export machine-readable waveform output for downstream analysis. Shipped in v1.0.
- [x] User can run the full capture-and-export workflow non-interactively from a single CLI command. Shipped in v1.0.
- [x] User can choose artifact output locations and receive clear artifact path reporting after a successful run. Shipped in v1.0.

### Active

- [ ] Protocol decode on captured logic data.
- [ ] Machine-readable export of decoded protocol results.
- [ ] Support for additional DSLogic-family devices.
- [ ] Additional export formats such as CSV.
- [ ] Reusable named capture presets.

### Out of Scope

- Built-in AI-agent invocation or analysis orchestration - the CLI should keep stopping at analyzable output artifacts.
- Full DSView GUI feature parity - future milestones should extend the workflow intentionally instead of mirroring the whole desktop app.
- Modifying the upstream `DSView/` codebase or libraries - the integration strategy still depends on consuming that stack as a read-only dependency.
- Terminal waveform rendering or a TUI viewer - the current product direction stays focused on export-first automation.

## Context

The workspace contains the upstream `DSView/` project, which remains the read-only native dependency for hardware communication through its modified `libsigrok4DSL` stack. The shipped v1.0 milestone now proves that a separate Rust CLI can sit on top of that stack, drive a bounded `DSLogic Plus` acquisition, and emit a reusable `VCD` plus JSON metadata sidecar without requiring the DSView GUI.

The verified milestone journey is now closed loop: discover the device, open it safely, validate and apply capture parameters, run a bounded capture, and publish analysis-ready artifacts. The phase backfill work also left a durable requirement-level verification chain across foundation, bring-up, configuration, acquisition, export, and CLI workflow behavior, with the milestone archive preserving the exact shipped scope.

## Current State

- `v1.0 MVP` shipped on 2026-04-09 and is archived in `.planning/milestones/`.
- The active workspace planning files are reset for the next milestone definition rather than another v1 closeout pass.
- Remaining known debt is procedural rather than product-blocking: validation/Nyquist coverage is still partial outside Phase 05, and source-runtime confidence still depends on the local native prerequisite toolchain and USB permissions used during validation.

## Next Milestone Goals

- Choose whether the next milestone centers on decode workflows, broader hardware support, export expansion, or preset reuse.
- Preserve the shipped `DSLogic Plus` capture/export path as the stable baseline while broadening scope.
- Keep future native work isolated behind the existing Rust boundary and continue treating `DSView/` as upstream dependency code.

## Constraints

- **Tech stack**: Implement the CLI in Rust - this remains an explicit project requirement.
- **Dependency boundary**: Reuse `DSView/` and its modified `libsigrok4DSL` stack without modifying that repository.
- **Scope discipline**: Build outward from the shipped `DSLogic Plus` workflow instead of broadening device or feature support opportunistically.
- **Workflow**: Optimize for scriptable CLI usage, not GUI or TUI interactions.
- **Output**: Preserve machine-readable waveform artifacts as the primary contract for downstream analysis.

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Build the new tool in Rust | User explicitly wants Rust for the CLI implementation | Adopted - implemented across `dsview-cli`, `dsview-core`, and `dsview-sys` |
| Keep `DSView/` unchanged and integrate with its existing libraries | Reuse proven device communication behavior while avoiding upstream modifications | Adopted - native integration stays behind the Rust boundary and `DSView/` remains read-only |
| Scope v1 to `DSLogic Plus` only | Narrower device scope reduces risk for the first usable release | Adopted - milestone v1.0 validates the DSLogic Plus capture/export workflow |
| Prioritize capture-and-export over full DSView parity | The immediate goal is a reliable automation workflow, not full feature coverage | Adopted - milestone v1.0 closes on non-interactive capture plus VCD/JSON artifacts |
| Do not include AI invocation in v1 | The first version only needs to generate output files for later analysis | Adopted - CLI stops at producing analyzable artifacts |

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
2. Core Value check - still the right priority?
3. Audit Out of Scope - reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-04-09 after shipping and archiving milestone v1.0*
