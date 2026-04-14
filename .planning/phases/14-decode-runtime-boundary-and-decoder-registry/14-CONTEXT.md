# Phase 14: Decode Runtime Boundary and Decoder Registry - Context

**Gathered:** 2026-04-14
**Status:** Ready for planning

<domain>
## Phase Boundary

This phase delivers the native/runtime boundary for `libsigrokdecode4DSL` plus the CLI-visible decoder registry and inspection surface. It covers runtime bring-up, decoder discovery, metadata inspection, and the shape of the CLI discovery commands. It does not cover full decode execution, capture-time integration, or flattening decoder options into the existing `capture` command.

</domain>

<decisions>
## Implementation Decisions

### Runtime boundary
- **D-01:** Use a dedicated decode runtime instead of merging decode support into the existing capture-focused `dsview_runtime`.
- **D-02:** Keep Python and decoder-script dependencies isolated to the decode runtime path so the shipped `capture` workflow does not inherit decode-specific runtime prerequisites.

### Decoder loading and environment discovery
- **D-03:** Decoder/runtime discovery uses explicit path overrides first, then bundled fallback paths.
- **D-04:** Runtime setup must surface distinct failure categories for missing runtime library, missing decoder scripts, Python runtime issues, and decoder load failures.

### CLI discovery surface
- **D-05:** Phase 14 CLI scope is limited to `decode list` and `decode inspect <decoder-id>`.
- **D-06:** `decode inspect` should expose the metadata needed for later config and stack planning, including channels, options, annotations/rows, and stack-relevant inputs/outputs.

### Metadata strategy
- **D-07:** Preserve upstream decoder, channel, and option ids as the canonical identifiers in Rust and CLI outputs.
- **D-08:** Rust should provide a structured stable schema around upstream metadata, but should not invent a second canonical decoder-id/token system in Phase 14.

### the agent's Discretion
- Exact JSON field names for list/inspect responses, as long as upstream canonical ids remain preserved.
- Text rendering layout for `decode list` and `decode inspect`, as long as JSON remains the authoritative contract.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Milestone and phase scope
- `.planning/ROADMAP.md` — Defines milestone `v1.2`, the Phase 14 boundary, and the success criteria for decoder registry + inspect work.
- `.planning/REQUIREMENTS.md` — Defines `DEC-01` and `DEC-02`, which this phase must cover.
- `.planning/PROJECT.md` — Captures the locked milestone-level decision to keep decode separate from `capture` and preserve the existing Rust/native boundary style.

### Upstream decode runtime API
- `DSView/libsigrokdecode4DSL/libsigrokdecode.h` — Public decode API surface for runtime init, decoder enumeration, session creation, sample streaming, and output callbacks.
- `DSView/libsigrokdecode4DSL/srd.c` — Runtime initialization path, Python setup, and decoder search-path behavior.
- `DSView/libsigrokdecode4DSL/decoder.c` — Decoder loading, metadata extraction, and registry behavior.
- `DSView/libsigrokdecode4DSL/session.c` — Session send/end flow and callback registration behavior.
- `DSView/libsigrokdecode4DSL/type_decoder.c` — Annotation/output conversion behavior and stacked decoder output routing.
- `DSView/libsigrokdecode4DSL/instance.c` — Input sample expectations, decode-thread behavior, and bit-packed channel access rules.

### DSView frontend reference behavior
- `DSView/DSView/pv/appcontrol.cpp` — Shows how DSView initializes `libsigrokdecode4DSL` and loads decoder scripts.
- `DSView/DSView/pv/storesession.cpp` — Shows DSView's decoder JSON concepts for ids, channels, options, and stacked decoders.
- `DSView/DSView/pv/data/decoderstack.cpp` — Reference for how DSView builds decode sessions and consumes annotation callbacks, but not a runtime abstraction to reuse directly.

### Existing Rust/native integration context
- `crates/dsview-sys/native/CMakeLists.txt` — Current source runtime boundary for capture/export and a reference for how a sibling decode runtime might be packaged.
- `crates/dsview-sys/src/lib.rs` — Existing sys-layer pattern for stable Rust-owned FFI wrappers, JSON-first artifacts, and runtime error shaping.
- `crates/dsview-cli/src/main.rs` — Existing CLI command structure and JSON/text output conventions that the new `decode` commands should follow.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/dsview-sys/src/lib.rs`: Existing pattern for wrapping native runtime calls into owned Rust structs and stable error types.
- `crates/dsview-cli/src/main.rs`: Existing `json` vs `text` output switch and clap-based command organization.
- `crates/dsview-sys/native/CMakeLists.txt`: Existing source-runtime build pattern that can inform a sibling decode runtime target.

### Established Patterns
- Native integration stays behind Rust-owned boundaries (`dsview-sys`), with `dsview-core` handling orchestration/validation and `dsview-cli` handling user-facing contracts.
- JSON is the authoritative automation contract; text output is a secondary human-readable rendering.
- The project prefers preserving upstream read-only dependencies rather than forking or rewriting DSView internals.

### Integration Points
- New decode runtime loading should mirror the current source/bundled runtime discovery style without coupling itself to capture-only initialization.
- Decoder metadata discovered in the sys layer should feed a Rust-owned registry model that `dsview-cli` can expose through `decode list` and `decode inspect`.
- Phase 14 outputs should be designed so Phase 15 can directly consume the same metadata for config validation without remapping canonical ids.

</code_context>

<specifics>
## Specific Ideas

- Keep upstream ids canonical now; if friendlier aliases are ever needed later, they should be additive rather than replacing canonical decoder/channel/option ids.
- `decode inspect` should be rich enough that later config design can be driven from it instead of requiring users to look into DSView source.

</specifics>

<deferred>
## Deferred Ideas

- Full offline decode execution against artifacts belongs to Phase 16.
- Decode config-file design and validation belongs to Phase 15.
- Capture+decode pipeline orchestration belongs to a later phase/milestone and should not pull decoder flags into `capture` during Phase 14.
- Live decode during acquisition remains out of scope for this phase.

</deferred>

---

*Phase: 14-decode-runtime-boundary-and-decoder-registry*
*Context gathered: 2026-04-14*
