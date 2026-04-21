# Phase 17: Decode Output and Workflow Reporting - Context

**Gathered:** 2026-04-21
**Status:** Ready for planning

<domain>
## Phase Boundary

This phase defines the user-facing output and reporting contract for offline decode runs. It must take the already-working Phase 16 execution path and turn it into a stable machine-readable result shape, stable failure reporting, and optional artifact writing, without changing the underlying execution semantics. It does not add capture integration, GUI output, or new execution models.

</domain>

<decisions>
## Implementation Decisions

### Result shape
- **D-01:** The canonical output schema should be JSON-first.
- **D-02:** The primary result structure should be `run summary + flat event list`, not a GUI-like grouped model.
- **D-03:** If grouped or row-oriented views are useful later, they should be additive fields and must not replace the flat event list as the canonical machine-readable result.

### Partial diagnostics
- **D-04:** Phase 17 may expose partial diagnostics and partial events externally.
- **D-05:** Even when partial diagnostics are exposed, the overall user-facing status must remain `failure`; do not introduce a public `partial_success` status.
- **D-06:** Partial diagnostics should be surfaced as supplementary fields (for example `partial_events` or `diagnostics`), not encoded as a separate success category.

### Artifact and stdout behavior
- **D-07:** stdout should remain the default output path for decode results.
- **D-08:** `--output` should be supported as an optional artifact-writing path.
- **D-09:** Writing an artifact should not become the only default success path; the command should remain shell- and pipeline-friendly by default.

### Failure reporting
- **D-10:** Failure reporting should use stable, fine-grained error codes that distinguish config, input, runtime initialization, and execution-stage failures.
- **D-11:** Whether partial diagnostics exist should be modeled as supplementary facts on the failure result, not folded into the main error code taxonomy.
- **D-12:** Phase 17 should finalize the user-facing error/reporting contract without broadening Phase 16 execution semantics.

### the agent's Discretion
- Exact JSON field names for the top-level result object, as long as they preserve the `run + events` structure and remain easy for automation to consume.
- Whether additive summary helpers such as per-decoder counts or row summaries are worth including in the output, as long as the flat event list remains canonical.
- Whether text-mode success output should be compact or moderately verbose, as long as JSON remains the authoritative contract.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Milestone and phase scope
- `.planning/ROADMAP.md` — Defines the Phase 17 goal, success criteria, and relationship to the already-completed Phase 16 execution path.
- `.planning/REQUIREMENTS.md` — Defines `DEC-06` and `PIPE-01`, which Phase 17 must satisfy.
- `.planning/PROJECT.md` — Carries the milestone-level decision that decode remains a separate workflow from `capture`.
- `.planning/STATE.md` — Current milestone/phase state and sequencing context.

### Prior phase decisions that constrain Phase 17
- `.planning/phases/16-offline-decode-execution/16-CONTEXT.md` — Locks raw logic artifacts as canonical input, strict linear stack execution, and binary success/failure semantics.
- `.planning/phases/16-offline-decode-execution/16-RESEARCH.md` — Documents the execution model, chunking approach, and failure semantics that reporting must not contradict.
- `.planning/phases/16-offline-decode-execution/16-VERIFICATION.md` — Confirms offline `decode run` works, stacked execution works, and internal partial diagnostics already exist.
- `.planning/phases/15-decode-config-model-and-validation/15-CONTEXT.md` — Locks JSON config, canonical ids, and strict validation/error behavior.
- `.planning/phases/14-decode-runtime-boundary-and-decoder-registry/14-CONTEXT.md` — Locks canonical upstream metadata and the JSON-first CLI style for decode discovery.

### Existing output/reporting surfaces
- `crates/dsview-cli/src/main.rs` — Existing CLI error response shape and the current `decode run` / `decode validate` command handling.
- `crates/dsview-cli/src/lib.rs` — Existing JSON/text response builders for decode discovery, validation, and basic execution results.
- `crates/dsview-core/src/lib.rs` — Current offline execution result type and any retained internal annotations/diagnostics that Phase 17 may choose to surface.
- `crates/dsview-sys/src/lib.rs` — Captured annotation event shape and decode runtime error taxonomy already available from the sys layer.
- `DSView/libsigrokdecode4DSL/libsigrokdecode.h` — Source of truth for raw annotation event fields coming out of the decode runtime.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/dsview-cli/src/lib.rs`: Already has JSON/text response builders for `decode list`, `decode inspect`, `decode validate`, and a minimal `decode run` success shape.
- `crates/dsview-cli/src/main.rs`: Already has a stable CLI `ErrorResponse` contract with `code`, `message`, and optional `detail`.
- `crates/dsview-core/src/lib.rs`: Already retains internal execution diagnostics and annotation state from Phase 16.
- `crates/dsview-sys/src/lib.rs`: Already exposes captured decode annotation events with decoder id, sample range, annotation class/type, and texts.

### Established Patterns
- JSON is the authoritative contract; text is a renderer.
- Execution semantics are already binary: success or failure.
- Decode remains separate from `capture`.
- Canonical upstream ids remain preserved through sys, core, and CLI layers.

### Integration Points
- Phase 17 should build directly on the existing Phase 16 execution result and error taxonomy instead of redesigning execution.
- The final reporting contract should reuse the existing CLI error envelope style where possible.
- Artifact writing should layer on top of the existing CLI execution path rather than introducing a separate “artifact-only” workflow.

</code_context>

<specifics>
## Specific Ideas

- A flat machine-readable event list is the safest canonical contract because downstream tools, scripts, and AI systems can regroup it however they want.
- Returning partial diagnostics on failure is useful, but it should still be obvious to automation that the run failed.
- `decode run` should remain easy to pipe into files or downstream commands even when `--output` exists.

</specifics>

<deferred>
## Deferred Ideas

- A public `partial_success` state is explicitly deferred and should not be introduced in this phase.
- More elaborate grouped/tree output shapes can be added later if needed, but they should not replace the flat event list.
- Capture+decode pipeline orchestration remains out of scope.
- GUI/table/panel-style output models remain out of scope.

</deferred>

---

*Phase: 17-decode-output-and-workflow-reporting*
*Context gathered: 2026-04-21*
