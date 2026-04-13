# Phase 13: Option-Aware Capture Reporting - Context

**Gathered:** 2026-04-13
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 13 applies the already validated DSLogic Plus device-option selections to the runtime before acquisition starts and records the effective option facts in CLI output and metadata. This phase covers deterministic apply order, failure reporting, and effective/requested reporting shape. It does not introduce new option-selection flags, presets, trigger features, or additional device support.

</domain>

<decisions>
## Implementation Decisions

### Apply strategy
- **D-01:** Apply device options in a fixed deterministic order before acquisition begins.
- **D-02:** If any apply step fails, stop immediately instead of attempting a full option rollback.
- **D-03:** On apply failure, the CLI must explicitly report which option changes already succeeded and which option failed, because the device may be left in a partially updated state.

### Apply ordering
- **D-04:** Apply mode-defining options first, then dependent device options, then channel/capture settings.
- **D-05:** Preferred apply sequence is:
  1. operation mode
  2. stop option
  3. channel mode
  4. threshold
  5. filter
  6. enabled channels
  7. sample limit
  8. sample rate

### Reporting shape
- **D-06:** CLI text output should stay concise and focus on the effective option values actually used for the run.
- **D-07:** JSON output and metadata should include both requested and effective device-option values.
- **D-08:** Effective values should at minimum include operation mode, stop option, channel mode, enabled channels, threshold volts, filter, sample rate, and sample limit.

### Apply semantics
- **D-09:** Phase 13 should explicitly apply the full validated request, including values inherited from current device state, rather than only applying fields the user explicitly changed.
- **D-10:** Deterministic full apply is preferred over partial “changed-only” apply, even when some values match the device’s prior state.

### the agent's Discretion
- Exact struct naming and serialization layout for requested/effective option facts, as long as text stays concise and JSON/metadata remain explicit.
- Exact error code and detail-field layout for partial apply failures, as long as successful steps and the failing step are both surfaced clearly.
- Whether the CLI text output uses one compact summary block or multiple lines for effective option reporting.

</decisions>

<specifics>
## Specific Ideas

- Failure reporting should feel operationally honest: tell the user what was already changed and what failed, not just “apply failed”.
- The run summary should remain easy to scan in text mode, while JSON and metadata should preserve enough detail for automation and post-run analysis.
- Phase 13 should preserve the already validated Phase 12 token/help/discovery surface and build on top of it rather than reshaping it again.

</specifics>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase boundary and milestone requirements
- `.planning/ROADMAP.md` — Phase 13 goal, success criteria, and dependency on the completed Phase 12 surface.
- `.planning/REQUIREMENTS.md` — `RUN-04` and `RUN-05` define the delivery target for this phase.
- `.planning/PROJECT.md` — baseline stability and DSLogic Plus-only scope constraints.

### Upstream implementation context
- `.planning/phases/11-device-option-validation-model/11-02-SUMMARY.md` — validated request model and selected-device validation entrypoint now available to drive apply-time work.
- `.planning/phases/11-device-option-validation-model/11-03-SUMMARY.md` — stable validation taxonomy and DSView-rule regression coverage that Phase 13 must preserve.
- `.planning/phases/12-cli-device-option-surface/12-02-SUMMARY.md` — capture flag resolution and selected-device validation branch already established.
- `.planning/phases/12-cli-device-option-surface/12-03-SUMMARY.md` — token/help/inspection contracts already locked and must not drift.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/dsview-core/src/lib.rs` already owns the capture session lifecycle, current `apply_capture_config(...)`, export flow, and metadata building path.
- `crates/dsview-core/src/device_option_validation.rs` already provides the validated request shape Phase 13 should consume.
- `crates/dsview-cli/src/main.rs` already has capture success JSON/text rendering and stable error classification patterns to extend.
- `crates/dsview-sys/src/lib.rs` and `crates/dsview-sys/bridge_runtime.c` already expose mode/option discovery and primitive runtime setters, but not yet the full apply-time device-option sequence.

### Established Patterns
- Selected-device validation is already performed before capture when Phase 12 flags are used.
- CLI contracts are locked by spawned integration tests and should remain stable while Phase 13 adds runtime application/reporting.
- Metadata is already emitted as a JSON sidecar and is the natural place for richer requested/effective facts.

### Integration Points
- Runtime apply sequencing will likely extend `dsview-core` capture-session preparation before `start_collect()`.
- Effective option reporting will likely extend `CaptureResponse` in `crates/dsview-cli/src/main.rs` and `CaptureMetadata` in `crates/dsview-core/src/lib.rs`.
- Partial-apply failure details will likely need to flow from the runtime/core layer into CLI error rendering without disturbing the existing baseline capture failure classes.

</code_context>

<deferred>
## Deferred Ideas

None — discussion stayed within the Phase 13 boundary.

</deferred>

---

*Phase: 13-option-aware-capture-reporting*
*Context gathered: 2026-04-13*
