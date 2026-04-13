---
gsd_state_version: 1.0
milestone: v1.1
milestone_name: milestone
current_phase: 12
current_phase_name: cli-device-option-surface
current_plan: 2
status: executing
stopped_at: Completed 12-cli-device-option-surface-01-PLAN.md
last_updated: "2026-04-13T08:18:07.069Z"
last_activity: 2026-04-13 -- Phase 12 planning complete
progress:
  total_phases: 4
  completed_phases: 2
  total_plans: 9
  completed_plans: 7
  percent: 78
---

# Session State

## Project Reference

See: `.planning/PROJECT.md` (updated 2026-04-10)

**Core value:** Users can reliably capture logic-analyzer data from `DSLogic Plus` via CLI and produce waveform output files that are easy for automation and AI agents to analyze.
**Current focus:** Phase 12 — cli-device-option-surface

## Current Position

Current Phase: 12
Current Phase Name: cli-device-option-surface
Current Plan: 2
Total Plans in Phase: 3
Phase: 12 (cli-device-option-surface) — READY
Plan: 2 of 3
Status: Ready to execute
Last activity: 2026-04-13 -- Phase 12 planning complete

## Accumulated Context

- `v1.0 MVP` shipped on 2026-04-09 and remains the baseline that new work must preserve.
- DSView source inspection confirms the relevant `DSLogic Plus` logic-mode options include operation mode, stop options, channel mode, threshold voltage, filter selection, and mode-dependent channel limits.
- `dsview-sys` now exposes an owned device-option snapshot with grouped channel modes, truthful VTH threshold facts, and restore-on-exit discovery semantics.
- Presets, repeat/loop collect behavior, advanced trigger configuration, protocol decode, and broader device support are explicitly deferred out of this milestone.

## Decisions

- Phase 10 option discovery stays behind a fixed-size owned C snapshot so safe Rust never touches live `GVariant`-backed pointers.
- Channel modes are enumerated per operation mode and use DSView-derived valid-channel metadata instead of label parsing.
- Threshold discovery is anchored on `SR_CONF_VTH` as a voltage-range fact, with legacy `SR_CONF_THRESHOLD` data treated as optional metadata only.
- [Phase 10]: Normalize automation IDs from raw native codes with fixed prefixes instead of relying on labels.
- [Phase 10]: Keep threshold as a fixed voltage-range capability rooted at threshold:vth-range and carry legacy threshold data only as raw metadata.
- [Phase 10]: Expose a dedicated discovery snapshot in dsview-core rather than extending Phase 9 capture capability types.
- [Phase 10]: Build a dedicated CLI response type from the normalized core snapshot so JSON stays authoritative while text formatting stays testable.
- [Phase 10]: Validate devices options --handle before runtime setup so invalid_selector remains available without hardware or resource files.
- [Phase 10]: Keep discovery rendering isolated to the new command so shipped devices and capture flows remain unchanged.
- [Phase 11]: Keep Phase 11 validation capabilities additive and internal instead of extending the shipped Phase 10 discovery schema.
- [Phase 11]: Probe validation facts entirely inside dsview-sys and restore original operation/channel modes on every exit path.
- [Phase 11]: Advertise stop-option compatibility only for Buffer Mode in the internal validation model.
- [Phase 11]: Use Phase 10 stable IDs as the allowlist source and carry native codes in the validated request for later apply-time phases.
- [Phase 11]: Reuse the shipped capture sample-limit alignment helpers instead of duplicating the arithmetic in the Phase 11 validator.
- [Phase 11]: Map current capture-config validation failures to the Phase 11 taxonomy while preserving their human-readable messages.
- [Phase 12-cli-device-option-surface]: Keep friendly capture-token generation in dsview-cli so Phase 10/11 stable IDs remain unchanged.
- [Phase 12-cli-device-option-surface]: Expose both token and stable_id in devices options JSON/text so automation and shell usage share one contract.
- [Phase 12-cli-device-option-surface]: Lead devices options text with capture flag examples plus --channels hints derived from channel-mode limits.

## Performance Metrics

| Phase | Plan | Duration | Tasks | Files |
|-------|------|----------|-------|-------|
| 10 | 01 | 8 min | 2 | 4 |
| 10 | 02 | 6 min | 2 | 3 |
| 10 | 03 | 3 min | 2 | 4 |
| Phase 11 P01 | 45 min | 2 tasks | 8 files |
| Phase 11 P02 | 13 min | 2 tasks | 5 files |
| Phase 11 P03 | 7 min | 2 tasks | 2 files |
| Phase 12-cli-device-option-surface P01 | 12m | 2 tasks | 4 files |

## Session Continuity

Last session: 2026-04-13T07:44:46.576Z

Stopped At: Completed 12-cli-device-option-surface-01-PLAN.md

Resume File: None

## Immediate Next Steps

- Execute Plan 12-02 to add `capture` device-option flags and resolve the new CLI tokens back to stable IDs.
- Reuse the locked `devices options` JSON/text contract while wiring Phase 11 validation into the `capture` command path.
- Keep Phase 13 apply-time mutation and reporting out of scope while finishing the remaining Phase 12 CLI parsing work.
