# DSView CLI

## What This Is

DSView CLI is a Rust-based command-line tool for using `DSLogic Plus` devices without the DSView GUI. The shipped `v1.0` milestone proved a stable non-interactive capture-and-export workflow, and `v1.1` now focuses on bringing the core DSView logic-mode device options into that CLI surface without breaking the validated baseline.

## Core Value

Users can reliably capture logic-analyzer data from `DSLogic Plus` via CLI and produce waveform output files that are easy for automation and AI agents to analyze.

## Requirements

### Validated

- [x] User can connect to a `DSLogic Plus` device from the CLI using the existing DSView/libsigrok4DSL stack. Shipped in `v1.0`.
- [x] User can configure core capture parameters from the CLI, including the options needed for a basic acquisition workflow. Shipped in `v1.0`.
- [x] User can start a capture from the CLI and export machine-readable waveform output for downstream analysis. Shipped in `v1.0`.
- [x] User can run the full capture-and-export workflow non-interactively from a single CLI command. Shipped in `v1.0`.
- [x] User can choose artifact output locations and receive clear artifact path reporting after a successful run. Shipped in `v1.0`.
- [x] User can validate DSLogic Plus option combinations before capture so the shipped `v1.0` capture/export path remains stable. Validated in Phase 11.
- [x] User can expose and choose DSView-style `DSLogic Plus` device options from the CLI, including operation mode, stop option, channel mode, enabled channels, threshold voltage, and filter selection. Validated in Phase 12.
- [x] User can use a discoverable non-interactive CLI surface to inspect supported device-option values before a run. Validated across Phases 10 and 12.

### Active

- [ ] Apply the selected DSView-compatible device options before acquisition begins.
- [ ] Report the effective option values used for the run in CLI output and metadata.

### Out of Scope

- Built-in AI-agent invocation or analysis orchestration - the CLI should keep stopping at analyzable output artifacts.
- Full DSView GUI feature parity - future milestones should extend the workflow intentionally instead of mirroring the whole desktop app.
- Modifying the upstream `DSView/` codebase or libraries - the integration strategy still depends on consuming that stack as a read-only dependency.
- Terminal waveform rendering or a TUI viewer - the product direction stays focused on export-first automation.
- Preset persistence and reusable named capture profiles - defer until the raw option semantics are stable in the CLI.
- `CollectMode` repeat/loop behavior, advanced trigger programming, and protocol decode - not part of the `v1.1` device-option milestone.

## Context

The workspace contains the upstream `DSView/` project, which remains the read-only native dependency for hardware communication through its modified `libsigrok4DSL` stack. The shipped `v1.0` milestone proved that a separate Rust CLI can sit on top of that stack, drive a bounded `DSLogic Plus` acquisition, and emit a reusable `VCD` plus JSON metadata sidecar without requiring the DSView GUI.

Milestone `v1.1` is grounded in direct DSView source inspection rather than greenfield feature guessing. The DSView device-session surface for `DSLogic Plus` includes operation mode, stop options, channel mode, threshold voltage, filter selection, and related trigger-facing settings. The current Rust bridge only exposes sample rate, sample limit, and enabled-channel application, so this milestone needs to widen the sys/core/cli boundary while keeping the proven capture/export contract intact.

## Current State

- `v1.0 MVP` shipped on `2026-04-09` and is archived in `.planning/milestones/`.
- The validated DSLogic Plus capture/export path is the baseline that `v1.1` must preserve.
- Phase 11 validation modeling completed on `2026-04-13`, including selected-device capability loading, pure validation, and stable CLI validation codes.
- Phase 12 CLI device-option surface completed on `2026-04-13`, including friendly capture flags, tokenized inspection output, and spawned CLI contract regressions.
- The next milestone is now defined around device-option parity for the existing `DSLogic Plus` target rather than broader hardware support or decode work.

## Current Milestone: v1.1 DSLogic Plus device options

**Goal:** Bring the core DSView logic-mode device options for `DSLogic Plus` into the CLI while preserving the shipped non-interactive capture/export workflow.

**Target features:**
- Operation mode selection for `Buffer Mode` and `Stream Mode`
- Stop-option selection where the selected mode supports it
- Channel-mode selection plus explicit enabled-channel control
- Threshold-voltage and filter selection
- Mode-aware validation and effective-option reporting in CLI output and metadata

## Constraints

- **Device scope**: `DSLogic Plus` only - this milestone does not expand support to other hardware.
- **Dependency boundary**: Reuse `DSView/` and its modified `libsigrok4DSL` stack without modifying that repository.
- **Workflow**: Optimize for scriptable CLI usage, not GUI, TUI, or profile-driven interaction.
- **Baseline stability**: Preserve the shipped `v1.0` capture/export behavior while adding richer option control.
- **Scope discipline**: Focus on DSView device options first; presets, collect-mode automation, trigger programming, and decode remain deferred.

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Build the tool in Rust | User explicitly wants Rust for the CLI implementation | Adopted - implemented across `dsview-cli`, `dsview-core`, and `dsview-sys` |
| Keep `DSView/` unchanged and integrate with its existing libraries | Reuse proven device communication behavior while avoiding upstream modifications | Adopted - native integration stays behind the Rust boundary and `DSView/` remains read-only |
| Scope `v1.0` to `DSLogic Plus` only | Narrower device scope reduced risk for the first usable release | Adopted - `v1.0` validated the DSLogic Plus capture/export workflow |
| Start `v1.1` with DSView device options instead of presets or decode | The current blocker is missing parity for real device configuration, not artifact post-processing | Adopted for milestone definition |
| Defer presets until option semantics are clear in CLI | DSView persists these settings, but CLI should first prove the raw option model and validation surface | Adopted for `v1.1` scope |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? -> Move to Out of Scope with reason
2. Requirements validated? -> Move to Validated with phase reference
3. New requirements emerged? -> Add to Active
4. Decisions to log? -> Add to Key Decisions
5. "What This Is" still accurate? -> Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check - still the right priority?
3. Audit Out of Scope - reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-04-13 after completing Phase 12 CLI device option surface*
