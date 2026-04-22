# DSView CLI

## What This Is

DSView CLI is a Rust-based command-line tool for using `DSLogic Plus` logic analyzers without the DSView GUI. Shipped milestones now cover device discovery, bounded capture/export, DSView-backed device-option discovery, pre-acquisition validation, option-aware runtime apply, requested/effective reporting, protocol decoder discovery, decode configuration/validation, offline decode execution, and final decode reporting for automation-friendly workflows.

## Core Value

Users can reliably capture logic-analyzer data from `DSLogic Plus` via CLI and produce waveform output files that are easy for automation and AI agents to analyze.

## Requirements

### Validated

- [x] User can connect to a `DSLogic Plus` device from the CLI using the existing DSView/libsigrok4DSL stack. - `v1.0`
- [x] User can configure core capture parameters from the CLI, including the options needed for a basic acquisition workflow. - `v1.0`
- [x] User can start a capture from the CLI and export machine-readable waveform output for downstream analysis. - `v1.0`
- [x] User can run the full capture-and-export workflow non-interactively from a single CLI command. - `v1.0`
- [x] User can choose artifact output locations and receive clear artifact path reporting after a successful run. - `v1.0`
- [x] User can inspect the supported `DSLogic Plus` device-option values for operation mode, stop option, channel mode, threshold voltage, and filter selection from the CLI. - `v1.1`
- [x] User can choose DSView-style `DSLogic Plus` device options from the CLI, including operation mode, stop option, channel mode, enabled channels, threshold voltage, and filter selection. - `v1.1`
- [x] User can validate DSLogic Plus option combinations before capture so unsupported requests fail before acquisition begins. - `v1.1`
- [x] User can apply the selected DSView-compatible device options before acquisition begins. - `v1.1`
- [x] User can report requested and effective option values in CLI output and metadata. - `v1.1`
- [x] User can inspect the DSView protocol decoder registry from the CLI, including decoder ids, channels, options, and stack metadata. - `v1.2`
- [x] User can define protocol decode stacks in a config-driven workflow that does not bloat the existing `capture` command surface. - `v1.2`
- [x] User can run DSView protocol decoders on captured logic data from the CLI and receive machine-readable annotation output. - `v1.2`
- [x] User can reuse saved capture artifacts as decode inputs while keeping future capture+decode pipeline support open. - `v1.2`

### Out of Scope

- Built-in AI-agent invocation or analysis orchestration - the CLI should keep stopping at analyzable output artifacts.
- Full DSView GUI feature parity - future milestones should extend the workflow intentionally instead of mirroring the whole desktop app.
- Modifying the upstream `DSView/` codebase or libraries - the integration strategy still depends on consuming that stack as a read-only dependency.
- Terminal waveform rendering or a TUI viewer - the product direction stays focused on export-first automation.
- Flattening decoder-specific flags into `capture` - protocol decode should remain config-driven and separable from acquisition.
- Live decode visualization or Qt decode panel parity - the milestone targets headless decode execution only.

## Context

The workspace keeps the upstream `DSView/` project as a read-only native dependency while the Rust workspace owns the CLI, orchestration, validation, and reporting layers. `v1.0` proved that this split could deliver a stable non-interactive capture/export workflow for `DSLogic Plus`, and `v1.1` extended that same baseline with truthful DSView-backed device-option discovery and execution rather than inventing a parallel configuration model.

## Current State

- `v1.2 DSView protocol decode CLI foundation` shipped on `2026-04-22` and is archived at `.planning/milestones/v1.2-ROADMAP.md`.
- The CLI now exposes `devices list`, `devices options`, `decode list`, `decode inspect`, `decode validate`, and `decode run` flows for the `DSLogic Plus` automation workflow.
- Milestone `v1.2` completed protocol decode discovery, config validation, offline execution, and final output/reporting contracts on top of the shipped `v1.0` and `v1.1` baseline.
- No next milestone is defined yet; live requirements will be recreated when `/gsd-new-milestone` starts the next planning cycle.

## Next Milestone Goals

- Choose the next smallest shipped increment on top of the completed decode foundation.
- Candidate directions now include pipeline orchestration, capture/decode presets, live decode, and broader hardware support.
- Preserve the shipped `v1.0`, `v1.1`, and `v1.2` baseline while extending intentionally rather than chasing full DSView parity.

## Constraints

- **Device scope**: `DSLogic Plus` remains the only shipped target; broader hardware support remains future work.
- **Dependency boundary**: Reuse `DSView/` and its modified libraries without modifying that repository.
- **Workflow**: Optimize for scriptable CLI usage, not GUI, TUI, or profile-driven interaction.
- **Baseline stability**: Preserve the shipped `v1.0` capture/export path and the shipped `v1.1` device-option workflow.
- **Scope discipline**: Future milestones should extend the validated workflow incrementally rather than chasing full DSView feature parity.
- **Decode UX discipline**: Protocol decode must not force a large decoder-specific flag surface onto the existing `capture` command.

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Build the tool in Rust | User explicitly wants Rust for the CLI implementation | Adopted - implemented across `dsview-cli`, `dsview-core`, and `dsview-sys` |
| Keep `DSView/` unchanged and integrate with its existing libraries | Reuse proven device communication behavior while avoiding upstream modifications | Adopted - native integration stays behind the Rust boundary and `DSView/` remains read-only |
| Scope initial releases to `DSLogic Plus` | Narrow device scope keeps hardware/runtime risk bounded while the CLI contract matures | Adopted across `v1.0` and `v1.1` |
| Layer friendly CLI tokens on top of stable core IDs | Automation needs stable identifiers while humans need copy-pasteable command tokens | Adopted in `v1.1` discovery and capture surfaces |
| Keep device-option apply order and failure reporting in Rust core instead of C | Ordered execution, partial-apply facts, and output reuse need one typed source of truth | Adopted in `v1.1` runtime apply/reporting |
| Report requested and effective device-option facts separately | Devices can align or adjust runtime values, so outputs must preserve both intent and outcome | Adopted in `v1.1` JSON, text, and metadata reporting |
| Keep protocol decode as a separate config-driven workflow instead of expanding `capture` flags | Decoder surfaces vary too much and would otherwise bloat the main acquisition command | Adopted for `v1.2` milestone planning |

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
*Last updated: 2026-04-22 after completing milestone `v1.2`*
