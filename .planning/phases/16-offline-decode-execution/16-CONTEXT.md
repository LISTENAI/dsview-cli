# Phase 16: Offline Decode Execution - Context

**Gathered:** 2026-04-21
**Status:** Ready for planning

<domain>
## Phase Boundary

This phase executes DSView protocol decoders offline against saved logic artifacts. It must consume the validated decode config from Phase 15, feed sample data through the decode runtime in a deterministic way, run the configured linear decoder stack, and produce an internal execution result that later phases can render/report. It does not add decode flags to `capture`, does not add GUI behavior, and does not define final user-facing partial-success output semantics.

</domain>

<decisions>
## Implementation Decisions

### Input artifact contract
- **D-01:** Phase 16 should treat raw logic artifacts as the canonical decode input, not VCD text.
- **D-02:** The offline decode input contract should carry explicit sample-layout metadata such as samplerate, data format, and raw sample bytes.
- **D-03:** If packet boundary information is available, it should be preserved and used rather than flattened away.

### Sample feeding model
- **D-04:** Sample data should be fed into the decode runtime incrementally in chunks/packets, not as a single monolithic send.
- **D-05:** The execution path must maintain absolute sample numbering across all sends.
- **D-06:** If packet boundaries exist, execution should prefer those boundaries; otherwise it may fall back to fixed chunking.

### Stack execution semantics
- **D-07:** Runtime execution is strictly linear: root decoder consumes logic samples, then each stacked decoder consumes the previous decoder's output.
- **D-08:** Stacked decoders should not bind their own logic channels at execution time.
- **D-09:** Phase 16 should not introduce branching graphs, multi-parent routing, or automatic rewiring of incompatible stacks.

### Failure semantics
- **D-10:** Phase 16 should treat decode execution as success-or-failure from the user-facing workflow perspective; it should not introduce a public `partial_success` state.
- **D-11:** Any critical execution failure — including runtime/session start failure, `srd_session_send` failure, stacked decoder failure, or end/finalization failure — should fail the whole decode run.
- **D-12:** Internal partial state or partial annotations may still be retained in memory for debugging and future reporting design, but Phase 16 should not promise them as successful output.

### the agent's Discretion
- Exact internal artifact/type names for the offline decode input contract, as long as they clearly encode raw sample bytes, samplerate, and any packet-boundary metadata.
- Chunk sizing strategy when the input does not come with explicit packet boundaries.
- Whether to add a thin adapter layer over existing VCD-export sample packet helpers, as long as raw logic artifacts remain the canonical execution path.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Milestone and phase scope
- `.planning/ROADMAP.md` — Defines the Phase 16 goal, success criteria, and relationship to Phases 15 and 17.
- `.planning/REQUIREMENTS.md` — Defines `DEC-05` and `DEC-07`, which Phase 16 must satisfy.
- `.planning/PROJECT.md` — Carries the milestone-level decision that decode remains separate from `capture`.
- `.planning/STATE.md` — Current milestone/phase state and sequencing context.

### Prior phase decisions that constrain Phase 16
- `.planning/phases/15-decode-config-model-and-validation/15-CONTEXT.md` — Locks JSON config, linear stack model, canonical ids, typed option values, and strict validation semantics.
- `.planning/phases/15-decode-config-model-and-validation/15-RESEARCH.md` — Documents the JSON config shape, strict validation layering, and stack compatibility rules that Phase 16 must consume rather than redesign.
- `.planning/phases/15-decode-config-model-and-validation/15-VERIFICATION.md` — Confirms `decode validate --config` and strict validated config loading are already available.
- `.planning/phases/14-decode-runtime-boundary-and-decoder-registry/14-CONTEXT.md` — Locks the separate decode runtime and the canonical decoder metadata model.
- `.planning/phases/14-decode-runtime-boundary-and-decoder-registry/14-VERIFICATION.md` — Confirms registry/inspect/runtime discovery already work.

### Upstream decode runtime behavior
- `DSView/libsigrokdecode4DSL/libsigrokdecode.h` — Public API for sessions, metadata, stacking, `srd_session_send`, and `srd_session_end`.
- `DSView/libsigrokdecode4DSL/session.c` — Documents absolute sample numbering and chunked send semantics.
- `DSView/libsigrokdecode4DSL/instance.c` — Shows how decoder instances consume chunked input and how stacked decoders are wired.
- `DSView/libsigrokdecode4DSL/type_decoder.c` — Shows how `OUTPUT_PYTHON` is forwarded from one decoder layer to the next.

### Existing Rust/native execution-adjacent code
- `crates/dsview-core/src/lib.rs` — Current validated decode config loading plus existing capture/export orchestration patterns.
- `crates/dsview-sys/src/lib.rs` — Existing raw sample / logic-packet helper surfaces and decode runtime bridge.
- `crates/dsview-sys/bridge_runtime.c` — Existing raw sample and logic packet handling patterns used today for VCD export helpers.
- `crates/dsview-sys/tests/boundary.rs` — Existing sample-bytes and logic-packet tests that can inform offline decode execution fixtures.
- `DSView/DSView/pv/data/decoderstack.cpp` — Reference for DSView's chunked send loop and end-of-session behavior; useful as semantics reference, not as a runtime abstraction to reuse directly.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/dsview-core/src/lib.rs`: Already exposes `ValidatedDecodeConfig`, decode discovery helpers, and runtime/discovery path policy.
- `crates/dsview-sys/src/lib.rs`: Already has `DecodeRuntimeBridge`, plus raw sample / logic-packet helper APIs used for VCD generation.
- `crates/dsview-sys/bridge_runtime.c`: Already models raw `sample_bytes`, packet lengths, and logic data formats in a way that can inform the offline decode input contract.
- `DSView/libsigrokdecode4DSL/session.c`: Already defines the required contract for chunked sample feeding with absolute sample numbers.
- `DSView/libsigrokdecode4DSL/type_decoder.c`: Already implements linear stacked decoder forwarding via `OUTPUT_PYTHON`.

### Established Patterns
- Decode remains a separate workflow from `capture`; Phase 16 must not add decode execution through `capture`.
- JSON/config validation is already complete and strict before execution begins.
- Canonical upstream ids remain preserved end to end.
- The project prefers explicit, machine-readable failure categories over permissive execution.

### Integration Points
- Phase 16 should consume `ValidatedDecodeConfig` directly from the completed Phase 15 validation path.
- The offline decode executor should sit in core over the existing `DecodeRuntimeBridge`, with any new raw sample/session APIs added in sys.
- Existing raw sample / logic-packet representations from the VCD export helpers are likely the closest reusable starting point for an offline decode input contract.
- Phase 17 should be able to take Phase 16's execution result and define final user-facing annotation/report contracts without changing execution semantics.

</code_context>

<specifics>
## Specific Ideas

- Treat packet-aware raw logic input as the “real” decode input and regard VCD as a downstream export format, not the source of truth for execution.
- The execution layer should mirror upstream expectations: chunked sends, absolute sample numbers, and explicit `end()`/session finalization.
- If partial annotations are retained internally for diagnostics, they should remain an implementation detail until Phase 17 decides what a user-facing artifact looks like.

</specifics>

<deferred>
## Deferred Ideas

- Accepting VCD as a first-class execution input can be deferred or treated as a compatibility adapter later.
- Public `partial_success` / degraded-result semantics are deferred to Phase 17.
- Any capture+decode orchestration remains out of scope for this phase.
- Branching decoder graphs or more advanced routing models remain out of scope.

</deferred>

---

*Phase: 16-offline-decode-execution*
*Context gathered: 2026-04-21*
