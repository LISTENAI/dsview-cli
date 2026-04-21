---
gsd_state_version: 1.0
milestone: v1.2
milestone_name: DSView protocol decode CLI foundation
current_phase: 14
current_phase_name: Decode Runtime Boundary and Decoder Registry
current_plan: 1
status: executing
stopped_at: Milestone initialized; Phase 14 ready for discuss/plan
last_updated: "2026-04-21T04:04:58.471Z"
last_activity: 2026-04-21 -- Phase 14 execution started
progress:
  total_phases: 4
  completed_phases: 0
  total_plans: 3
  completed_plans: 0
  percent: 0
---

# Session State

## Project Reference

See: `.planning/PROJECT.md` (updated 2026-04-14)

**Core value:** Users can reliably capture logic-analyzer data from `DSLogic Plus` via CLI and produce waveform output files that are easy for automation and AI agents to analyze.
**Current focus:** Phase 14 — Decode Runtime Boundary and Decoder Registry

## Current Position

Phase: 14 (Decode Runtime Boundary and Decoder Registry) — EXECUTING
Plan: 1 of 3
Current Phase: 14
Current Phase Name: Decode Runtime Boundary and Decoder Registry
Current Plan: 1
Total Plans in Milestone: 3
Milestone: `v1.2 DSView protocol decode CLI foundation`
Status: Executing Phase 14
Last activity: 2026-04-21 -- Phase 14 execution started

## Accumulated Context

- `v1.0 MVP` shipped on 2026-04-09 and remains the baseline for the non-interactive capture/export workflow.
- `v1.1` shipped on 2026-04-13 and added DSView-backed device-option discovery, validation, selection, apply-time reporting, and schema-v2 requested/effective metadata for `DSLogic Plus`.
- The Rust layer now owns stable IDs, friendly capture tokens, deterministic option apply order, partial-apply diagnostics, and output reporting while `DSView/` stays read-only.
- Presets, repeat/loop collect behavior, advanced trigger configuration, broader device support, and full capture+decode orchestration remain candidate future work rather than shipped scope.
- `v1.2` is intentionally scoped around a config-driven decode workflow so protocol-analysis support does not bloat the existing `capture` command surface.

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
- [Phase 12]: Keep the clap-facing CaptureDeviceOptionArgs in main.rs and adapt it into the shared resolver with a lightweight trait instead of duplicating token parsing logic.
- [Phase 12]: Preserve omitted sibling option values from the selected-device snapshot, but let explicit channel-mode or stop-option tokens infer the parent operation mode only when the inference is unique.
- [Phase 12]: Route --channels through the same selected-device validation preflight as the new flags so channel-count limits stay aligned with the resolved channel mode before capture begins.
- [Phase 12]: Keep the selected-device integration-test seam debug-only and env-gated so release behavior stays unchanged while spawned CLI tests remain deterministic.
- [Phase 12]: Pin devices options token assertions to the same acceptance tokens used by capture_cli so discovery and execution cannot drift independently.
- [Phase 13]: Keep the D-05 setter sequence in Rust so core owns apply order, fail-fast behavior, and typed reporting instead of hiding sequencing in C.
- [Phase 13]: Reuse the Phase 11 validated request directly during capture execution and derive export validation config from it, rather than re-validating against the current active mode.
- [Phase 13]: Treat effective enabled channels as the successfully applied validated request after channel-enable setters succeed, while reading the other effective values back from runtime getters.
- [Phase 13]: Core now builds requested/effective device-option facts once and reuses that block across metadata and CLI JSON.
- [Phase 13]: Baseline captures mirror inherited current option state into both requested and effective reporting blocks so automation always sees explicit facts.
- [Phase 13]: Reused DSVIEW_CLI_TEST_DEVICE_OPTIONS_FIXTURE for full spawned capture success and failure coverage instead of adding a second fixture flag.
- [Phase 13]: Phase 13 hardware verification completed successfully on a real DSLogic Plus and closed the final live-runtime gap for `v1.1`.
- [Milestone v1.2]: Protocol decode planning will treat `libsigrokdecode4DSL` as the engine and avoid reusing DSView Qt decode UI classes as runtime abstractions.
- [Milestone v1.2]: Decode configuration should stay file-driven and separate from the `capture` command surface.

## Session Continuity

Last session: 2026-04-14T07:58:43Z

Stopped At: Milestone initialized; Phase 14 ready for discuss/plan

Resume File: None

## Immediate Next Steps

- Run `/gsd-execute-phase 14` to begin executing the planned decode runtime boundary work.
- Review `.planning/phases/14-decode-runtime-boundary-and-decoder-registry/*-PLAN.md` before execution if you want to inspect the wave layout.
- Preserve the shipped `v1.0` and `v1.1` workflows while introducing decode support as a separate layer.
