# Phase 12: CLI Device Option Surface - Context

**Gathered:** 2026-04-13
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 12 delivers the CLI-facing option selection surface for `DSLogic Plus`. This phase covers how users specify operation mode, stop option, channel mode, enabled channels, threshold voltage, and filter selection from the command line, plus how the CLI explains and inspects those values. It does not cover applying those options to capture hardware or reporting effective applied values in capture artifacts; those remain in Phase 13.

</domain>

<decisions>
## Implementation Decisions

### Command surface
- **D-01:** Extend the existing `capture` command instead of introducing a new top-level command or a nested `capture run` structure.
- **D-02:** Keep `devices options` as the inspection entrypoint rather than adding a separate capture-option inspection command in Phase 12.

### CLI value format
- **D-03:** Expose human-readable, stable CLI tokens for option values instead of requiring internal stable IDs such as `operation-mode:0`.
- **D-04:** Internally map those human-readable CLI tokens onto the existing Phase 10/11 stable IDs and native codes rather than changing the core validation model.

### Help and inspection behavior
- **D-05:** Make `devices options` output more directly aligned with the future `capture` flags so users can read valid values and copy them into `capture`.
- **D-06:** Keep `capture --help` concise and use it to point users toward `devices options` for the full supported-value surface and compatibility context.

### Defaults and option inference
- **D-07:** All new device-option flags should be optional; if a flag is omitted, the CLI should preserve the current device value rather than forcing the user to restate every option.
- **D-08:** More specific option flags may infer their parent mode automatically. Example: if a user passes a channel-mode token that belongs to buffer mode, the CLI may infer the matching operation mode rather than requiring an additional explicit parent-mode flag.

### the agent's Discretion
- Exact token spellings for each CLI-facing value, as long as they remain human-readable and stable.
- Whether help output includes inline examples, grouped sections, or “see `devices options`” references.
- Exact text/json layout changes in `devices options`, as long as it becomes more capture-oriented and remains machine-readable.

</decisions>

<specifics>
## Specific Ideas

- Keep the current one-command `capture` workflow intact; device-option support should feel like an extension of the existing `v1.0` path, not a redesign.
- The CLI should optimize for scriptability first, but values should still be readable enough that a human can type them from memory without looking at internal IDs.
- Inspection output should be shaped so that users can easily answer “what exact value do I pass to `capture`?” without reverse-mapping internal discovery IDs themselves.

</specifics>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase boundary and requirements
- `.planning/ROADMAP.md` — Phase 12 goal, dependencies, success criteria, and plan slots.
- `.planning/REQUIREMENTS.md` — OPT-02 through OPT-07 requirement targets for this phase.
- `.planning/PROJECT.md` — milestone constraints, baseline stability requirements, and DSLogic Plus-only scope.

### Phase 11 foundations
- `.planning/phases/11-device-option-validation-model/11-02-SUMMARY.md` — pure validation entrypoint and stable validation taxonomy now available to Phase 12.
- `.planning/phases/11-device-option-validation-model/11-03-SUMMARY.md` — DSView-rule regression coverage and CLI validation-code coverage already locked before Phase 12 wiring.
- `.planning/phases/11-device-option-validation-model/11-VERIFICATION.md` — verified truths about selected-device validation, stable codes, and baseline CLI suite coverage.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/dsview-cli/src/main.rs` — existing `capture` command and clap argument structure provide the natural insertion point for new device-option flags.
- `crates/dsview-cli/src/device_options.rs` — existing `devices options` response builder and text renderer can be reshaped to better mirror future `capture` flag values.
- `crates/dsview-core/src/lib.rs` — `Discovery::validate_device_option_request(...)` already exists as the selected-device validation entrypoint for Phase 12 to call.
- `crates/dsview-core/src/device_option_validation.rs` — stable Phase 11 request/capability/error contracts are already in place and should be reused rather than redesigned.

### Established Patterns
- The CLI currently uses clap-based flat flags on `capture` rather than nested subcommands for the main acquisition path.
- Error handling follows stable machine-readable `ErrorResponse.code` values with separate JSON and text rendering paths.
- `devices options` already exposes a machine-readable JSON form plus deterministic text output; Phase 12 should preserve that pattern while making it more capture-oriented.

### Integration Points
- New CLI flags should attach to `CaptureArgs` in `crates/dsview-cli/src/main.rs`.
- Parsed CLI tokens should map into a Phase 11 `DeviceOptionValidationRequest` before runtime apply work begins.
- Help text and diagnostics should connect the `capture` command and `devices options` command so they feel like a single workflow instead of separate features.

</code_context>

<deferred>
## Deferred Ideas

None — discussion stayed within the Phase 12 boundary.

</deferred>

---

*Phase: 12-cli-device-option-surface*
*Context gathered: 2026-04-13*
