# Phase 15: Decode Config Model and Validation - Context

**Gathered:** 2026-04-21
**Status:** Ready for planning

<domain>
## Phase Boundary

This phase defines the config-driven decode model and validates it before runtime execution. It must let users describe a root decoder, a linear stack of downstream decoders, channel bindings, and typed option values, then fail early with clear CLI errors when the config is invalid. It does not execute decode sessions, feed sample data, or add decoder-specific flags to `capture`.

</domain>

<decisions>
## Implementation Decisions

### Config format
- **D-01:** Phase 15 should support JSON config files only.
- **D-02:** The config shape should be optimized for machine generation and machine consumption first; human readability matters, but not enough to justify YAML or TOML in this phase.

### Stack model
- **D-03:** A decode config describes one root decoder plus an ordered linear `stack` of downstream decoders.
- **D-04:** Phase 15 must not introduce branching graphs, DAGs, or multi-parent decoder routing.
- **D-05:** Each stacked decoder is assumed to consume the immediately previous decoder's output unless a later milestone explicitly broadens that model.

### Channels and option encoding
- **D-06:** Channel bindings use numeric capture-channel indexes keyed by canonical upstream channel ids.
- **D-07:** Decoder, channel, and option ids stay canonical and upstream-aligned; Phase 15 must not invent a second id/token namespace.
- **D-08:** Option values should be type-aware in the JSON schema (for example string, integer, float, or boolean as metadata allows), rather than forced to strings.

### Validation behavior
- **D-09:** Validation is strict and failure-first; invalid configs should not continue with warnings.
- **D-10:** Errors should stay CLI-friendly and machine-readable, following the project's stable `code` + `message` (+ optional `detail`) pattern.
- **D-11:** Validation must distinguish schema errors, metadata/compatibility errors, and runtime prerequisite errors.

### the agent's Discretion
- Exact JSON field names and nesting, as long as they honor the locked decisions above and remain easy to validate against decoder metadata.
- Whether to include an optional schema version field in the config if it simplifies future evolution without weakening the canonical id model.
- How much metadata from `decode inspect` should be referenced verbatim in validation errors versus summarized for readability.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Milestone and phase scope
- `.planning/ROADMAP.md` — Defines the Phase 15 goal, success criteria, and relationship to Phases 14 and 16.
- `.planning/REQUIREMENTS.md` — Defines `DEC-03` and `DEC-04`, which Phase 15 must satisfy.
- `.planning/PROJECT.md` — Carries the milestone-level decision that decode must stay config-driven and separate from `capture`.
- `.planning/STATE.md` — Current milestone/phase state and sequencing context.

### Prior phase decisions that constrain Phase 15
- `.planning/phases/14-decode-runtime-boundary-and-decoder-registry/14-CONTEXT.md` — Locks canonical upstream ids, separate decode runtime, and `decode list` / `decode inspect` as the metadata source of truth.
- `.planning/phases/14-decode-runtime-boundary-and-decoder-registry/14-RESEARCH.md` — Documents the separate decode runtime, packaging/discovery assumptions, and metadata boundary that Phase 15 must build on.
- `.planning/phases/14-decode-runtime-boundary-and-decoder-registry/14-VERIFICATION.md` — Confirms the discovery/inspect baseline is working and available for config validation.

### DSView reference behavior
- `DSView/DSView/pv/storesession.cpp` — Reference for DSView's decoder JSON concepts: root decoder id, channel map, options, and `stacked decoders` representation.
- `DSView/libsigrokdecode4DSL/libsigrokdecode.h` — Source of truth for decoder metadata structures, channel definitions, option metadata, and output/stack relationships.

### Existing Rust decode metadata surfaces
- `crates/dsview-core/src/lib.rs` — Current canonical Rust decoder registry model (`DecoderDescriptor`, channels, options, annotations, inputs/outputs).
- `crates/dsview-cli/src/lib.rs` — Current JSON/text output contract for `decode list` and `decode inspect`, which should inform config ergonomics and error wording.
- `crates/dsview-cli/src/main.rs` — Existing CLI error-shaping pattern for decode discovery and machine-readable failure reporting.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/dsview-core/src/lib.rs`: Already exposes a normalized `DecoderDescriptor` tree with canonical ids, required/optional channels, options, annotations, and stack-relevant inputs/outputs.
- `crates/dsview-cli/src/lib.rs`: Already serializes decoder metadata into JSON/text responses; this can guide field naming and error phrasing for config validation.
- `DSView/DSView/pv/storesession.cpp`: Already shows how DSView serializes root decoder + `stacked decoders` + channel bindings + options.

### Established Patterns
- Canonical upstream ids are preserved across sys, core, and CLI discovery layers.
- JSON is the authoritative interface contract; text is a rendering layer.
- Decode remains separate from `capture`; the config model must not reintroduce capture-flag sprawl.
- The project prefers strict preflight validation over permissive warning-based execution when machine-driven workflows are involved.

### Integration Points
- Phase 15 should validate configs directly against `DecoderDescriptor` metadata returned by the completed Phase 14 discovery path.
- The resulting validated config model should be shaped so Phase 16 can consume it directly for offline decode execution.
- Error contracts should align with existing CLI `ErrorResponse` patterns so later command surfaces do not invent a parallel failure language.

</code_context>

<specifics>
## Specific Ideas

- A good Phase 15 config should be derivable from `decode inspect` output without requiring users to read DSView source files.
- The DSView-style concept of `stacked decoders` is useful, but the CLI should make the linear ordering explicit rather than leaving relationships implicit.
- Validation errors should be specific enough that a user can fix a bad config from the CLI output alone.

</specifics>

<deferred>
## Deferred Ideas

- Supporting YAML or TOML config formats can be deferred until the JSON model stabilizes.
- Supporting branching decoder graphs, fan-out, or multi-parent routing is out of scope for Phase 15.
- Friendly aliases for decoder/channel/option ids are deferred; Phase 15 should keep upstream ids canonical.
- Permissive warning-mode validation is out of scope; this phase should use strict failure-first validation.
- Any `decode run` execution semantics belong to Phase 16, not this phase.

</deferred>

---

*Phase: 15-decode-config-model-and-validation*
*Context gathered: 2026-04-21*
