# Phase 5 Research: Export Artifacts

**Date:** 2026-04-07
**Phase:** 5 - Export Artifacts
**Goal:** Turn successful captures into reliable analysis artifacts by exporting VCD plus a machine-readable metadata sidecar.

## What Phase 5 Must Achieve

Phase 5 is where the already-validated Phase 4 acquisition lifecycle becomes durable output for downstream automation and AI-assisted analysis.

This phase must satisfy:

- `EXP-01`: user can export captured waveform data as a `VCD` file
- `EXP-02`: exported `VCD` preserves channel names and timing information needed for downstream waveform analysis
- `EXP-03`: CLI writes a machine-readable metadata file describing the capture session
- `EXP-04`: metadata includes device model, enabled channels, sample rate, sample limit or actual sample count, capture timestamp, and tool version

Planning implications:

- Phase 5 is not only a file-writing phase.
- It is the phase that locks the artifact contract for automation.
- It must decide what capture data is retained, when export happens relative to cleanup, and how artifact correctness is validated.

## Current Baseline After Phase 4

### Acquisition lifecycle is proven, but exportable payload retention does not exist yet

The current bridge and orchestration already prove:

- runtime init and exit
- device open and release
- capture configuration apply
- callback registration
- start/wait/stop/release lifecycle
- terminal event classification
- logic-packet presence and end-marker presence

The current bridge only retains summary state, not exportable waveform payloads.

It currently records:

- terminal events
- whether logic packets were seen
- whether end packets were seen
- packet end status
- whether collection is still active
- native last-error state

It does **not** retain:

- `SR_DF_META` payloads needed for export context
- `SR_DF_LOGIC` payload buffers
- packet ordering required for replay
- enough data to drive `sr_output_send()` after capture completes

This is the main technical gap Phase 5 must close.

### Phase 4 clean-success semantics are the correct export gate

Phase 4 already established `clean_success` to mean:

- capture start succeeds
- normal terminal completion occurs
- no competing detach/error terminal event occurs
- at least one logic packet is observed
- an end marker is observed
- cleanup succeeds

Planning decision:

- Phase 5 should only emit final artifacts for `clean_success` captures.
- v1 should not write a VCD for failed or partial captures.
- If partial-artifact support is ever needed, it should be a later explicit contract rather than an accidental side effect.

## Confirmed Upstream Export Facts

### Upstream already exposes a reusable output-module API

`DSView/libsigrok4DSL/output/output.c` provides a public output pipeline with:

- `sr_output_find("vcd")`
- `sr_output_new(...)`
- `sr_output_send(...)`
- `sr_output_free(...)`

Planning implication:

- Phase 5 should reuse the upstream output-module path for VCD rather than introducing a new serializer unless integration proves impossible.

### Upstream VCD logic already encodes the semantics required by `EXP-02`

`DSView/libsigrok4DSL/output/vcd.c` indicates that canonical VCD behavior is derived from the original stream and session context:

- channel names come from enabled `sr_channel` names
- timescale derives from samplerate
- timestamps are computed from streamed sample count and samplerate
- initial values are emitted before later deltas
- subsequent writes are delta-based rather than full-state rewrites
- stream completion is represented with a closing timestamp on `SR_DF_END`

Planning implication:

- Reusing upstream VCD logic is the strongest path for preserving naming and timing semantics.
- A Rust-side custom writer should remain a fallback only.

### The exporter consumes the ordered packet stream, not Phase 4 summaries

The VCD output module reacts to:

- `SR_DF_META`
- `SR_DF_LOGIC`
- `SR_DF_END`

Planning implication:

- The architectural question for Phase 5 is how to retain and replay the stream in capture order.
- Summary booleans are insufficient for export.

### Packet status remains part of validity

Upstream packet status includes:

- `SR_PKT_OK`
- `SR_PKT_SOURCE_ERROR`
- `SR_PKT_DATA_ERROR`

Planning implication:

- Export validation must preserve the rule that non-OK end status is not a valid clean-success artifact path.
- A VCD produced after a data/source error should not be treated as v1-valid output.

## Recommended Technical Direction

Plan Phase 5 around reusing upstream DSView/libsigrok VCD export logic.

Recommended direction:

- keep `DSView/` read-only
- isolate packet retention and replay in `crates/dsview-sys`
- keep orchestration, metadata, and artifact contract in `crates/dsview-core`
- keep CLI path handling and user-visible contract in `crates/dsview-cli`

Why this is the best fit:

- it preserves canonical VCD semantics from upstream
- it keeps unsafe/native concerns behind a small boundary
- it avoids semantic drift in timing, channel ordering, and end-of-stream behavior
- it aligns with project constraints already established in `CLAUDE.md`

## Architecture Split By Crate

### `crates/dsview-sys`

This crate should own packet retention and replay into the upstream VCD exporter.

Recommended responsibilities:

- retain exportable packets in capture order during acquisition
- own packet memory rather than borrowing upstream packet pointers
- preserve enough information to reconstruct replay packets:
  - packet type
  - packet status
  - logic `unitsize`
  - logic `length`
  - logic payload bytes
  - samplerate-bearing metadata needed by VCD export
- dynamically load or invoke the public output-module functions needed for VCD
- expose a narrow Rust-callable operation that exports retained packets to a VCD file
- return export statistics needed by higher layers, such as actual sample count and byte counts if available

Important constraint:

- `dsview-sys` should not leak raw libsigrok packet lifetimes into Rust.
- Retention should be in repo-owned memory using a normalized packet representation that is sufficient to reconstruct `sr_datafeed_packet` / `sr_datafeed_logic` for replay.

### `crates/dsview-core`

This crate should own successful-run artifact orchestration and metadata generation.

Recommended responsibilities:

- treat export as part of successful capture completion, not an unrelated post-process
- call the sys export bridge only after a clean-success acquisition result is established
- generate the JSON sidecar from validated config, observed acquisition facts, and export results
- define the stable metadata schema and versioning strategy
- enforce output ordering and failure semantics across artifact writes
- classify export-stage failures separately from acquisition failures

Recommended ordering:

1. capture completes cleanly
2. VCD export succeeds
3. metadata file is written last

Reasoning:

- metadata should not claim success if the VCD was never produced
- writing metadata last lets it describe the final artifact set accurately

### `crates/dsview-cli`

This crate should stay thin and own the public artifact contract.

Recommended responsibilities:

- accept output path arguments compatible with later Phase 6 productization
- surface generated artifact locations on success
- return stable diagnostics for capture failure vs VCD export failure vs metadata failure
- avoid embedding export logic or metadata rules directly in the CLI layer

Planning implication:

- Phase 5 should lock the artifact contract now so Phase 6 refines UX rather than redesigning artifact behavior.

## Lifecycle And Packet-Retention Decisions

### Export-before-release is the implemented v1 lifecycle rule

Phase 5 now treats VCD export as part of successful capture completion rather than a detached post-process. The clean-success gate from Phase 4 still decides whether export is allowed, but the export step runs before the final user-visible artifact path is considered complete.

Implemented rule:

- only `CaptureCompletion::CleanSuccess` is export-eligible
- non-clean-success outcomes (`StartFailure`, `RunFailure`, `Detached`, `Incomplete`, `CleanupFailure`, `Timeout`) never attempt VCD export
- the sys bridge retains normalized packets during acquisition so replay can happen without borrowing upstream packet lifetimes into Rust
- the final VCD path is written through a temporary sibling path and only promoted after the upstream replay/export step finishes successfully
- promotion failure removes the temp file so failed export does not leave a misleading final-path artifact

Lifecycle consequence:

- Phase 4 cleanup and device reuse rules stay intact at the acquisition boundary
- Phase 5 extends the success path with a cleanup-safe export write contract instead of redefining acquisition completion semantics
- higher layers only see stable export facts (`sample_count`, `packet_count`, `output_bytes`) and the final VCD path, not retained packet internals

### Packet retention must preserve stream order exactly

Planning requirements:

- logic packets and meta packets must remain in callback order
- `SR_DF_END` must replay last
- replay must preserve the same timing/count semantics that upstream export expects

### Logic payloads must be copied into owned memory

Implemented retained packet shape in `dsview-sys`:

- `SR_DF_META` retains packet status plus samplerate-bearing metadata needed for VCD header generation
- `SR_DF_LOGIC` retains packet status, `length`, `unitsize`, `data_error`, `error_pattern`, and copied payload bytes in repo-owned memory
- `SR_DF_END` retains packet status and is recorded as the terminal replay marker

Retention guarantees:

- callback order is preserved exactly in the retained packet array
- `SR_DF_END` replays last for a valid stream
- actual sample count is derived from retained logic packets, not just the requested sample limit
- Phase 4 summary booleans remain separate from the replay buffer so lifecycle classification and export fidelity do not depend on the same representation

### Packet-retention capacity and overflow contract

Finite capture does not automatically mean small memory usage, so packet retention cannot remain an implied best-effort behavior.

Recommended v1 contract:

- `dsview-sys` owns a bounded in-memory retention buffer sized from the validated requested sample limit plus explicit per-packet bookkeeping overhead
- the retained representation must be capacity-planned in terms of both total payload bytes and packet-entry count so pathological packetization cannot bypass the limit
- the requested capture is admitted for export only when the configured retention capacity can hold the full clean-success stream needed for replay
- if retention would exceed the declared capacity at any point, collection transitions to a fail-fast export-ineligible outcome rather than silently dropping packets or truncating the replay stream
- overflow is reported as a stable export-precondition failure category that higher layers can diagnose separately from device I/O errors
- v1 does not permit lossy retention, partial replay, or automatic disk spooling fallback; those remain explicit future-phase options if hardware validation proves the in-memory bound unacceptable

Implemented notes:

- the bridge enforces both payload-byte capacity and packet-entry capacity before appending retained data
- overflow marks the retained stream as export-ineligible and later export returns a stable overflow error instead of attempting partial replay
- the Rust-side path writer only publishes the final artifact after a successful temp write plus promotion step, preserving the cleanup-safe artifact rule even when export succeeds but file promotion fails

Why this contract is preferable for v1:

- it preserves trust in `EXP-01` and `EXP-02` by avoiding silent VCD corruption or truncation
- it keeps unsafe/native behavior isolated while making memory pressure visible at the product contract level
- it gives validation a concrete pass/fail rule rather than a vague "large captures may fail" note

Validation implications:

- tests should cover at least one capacity-fit case and one overflow/fail-fast case
- manual sign-off should use bounded captures that stay comfortably inside the declared retention limit so artifact plausibility is evaluated without confounding memory pressure

## Metadata Sidecar Contract

At minimum the metadata must satisfy `EXP-03` and `EXP-04`, but the schema should carry enough context for automation to trust the exported artifacts.

Recommended v1 schema shape:

- `schema_version`
- `tool`:
  - `name`
  - `version`
- `capture`:
  - `timestamp_utc`
  - `device_model`
  - `device_stable_id`
  - `selected_handle`
  - `sample_rate_hz`
  - `requested_sample_limit`
  - `actual_sample_count`
  - `enabled_channels`
- `acquisition`:
  - `completion`
  - `terminal_event`
  - `saw_logic_packet`
  - `saw_end_packet`
  - `end_packet_status`
- `artifacts`:
  - `vcd_path`
  - `metadata_path`
- `timing`:
  - `timescale_hint` or the exported timescale if it can be surfaced reliably

Recommended planning decisions:

- use a versioned JSON object rather than ad hoc flat keys only
- use UTC timestamps, preferably RFC 3339
- use numeric fields for sample rate and counts
- include both requested and actual sample counts when available even though `EXP-04` only requires one or the other

## Actual Sample Count Guidance

This must be grounded in observed data, not only requested configuration.

Recommended planning decision:

- count actual samples from retained `SR_DF_LOGIC` packets in `dsview-sys`
- treat requested sample limit as configuration, not proof of delivered samples
- optionally cross-check the count during replay/export for internal consistency

Why this matters:

- it prevents metadata from overstating what was actually captured
- it gives downstream analysis a trustworthy sample-count fact tied to the exported data

## Output Contract Decisions To Lock During Planning

### VCD and metadata path behavior

Recommended decision:

- let the user provide a base output path or explicit VCD path
- derive metadata path deterministically from the VCD path unless a later phase adds an explicit override

### Artifact atomicity

Recommended decision:

- avoid leaving a final-path VCD that appears complete when export failed midway
- write the VCD to a sibling temporary path and promote it to the final path with atomic rename semantics when the platform and filesystem support it
- when atomic rename cannot be guaranteed, treat the VCD path as cleanup-safe: write to a temporary path first and remove it on failure so no misleading final-path VCD remains
- keep metadata on the same atomic-or-cleanup-safe contract and continue writing it only after VCD success
- define overall success as both final paths existing only after their respective write steps have completed cleanly

### Failure semantics

Recommended decision:

- if VCD export fails, metadata must not claim success
- if metadata writing fails after VCD succeeds, the overall command still fails because the v1 artifact set is incomplete
- diagnostics should distinguish:
  - capture failure
  - packet-retention capacity overflow or export-precondition failure
  - VCD export failure
  - metadata serialization failure
  - metadata file write failure

## Validation Architecture

Phase 5 needs a validation architecture strong enough to support both implementation planning and a dedicated `05-VALIDATION.md` artifact. Validation should be split by layer so failures point to the right seam.

### Sys-level validation (`crates/dsview-sys`)

This level proves that packet retention and replay are faithful enough for upstream export.

Must validate:

- retained packets preserve callback order exactly
- retained logic packets preserve `length`, `unitsize`, payload bytes, and packet status
- retained metadata includes the samplerate/config context required for VCD header generation
- replay reconstructs packets accepted by `sr_output_send()`
- export is rejected when no clean-success stream is available
- non-OK end packet status is surfaced as invalid for artifact generation
- actual sample count computed from retained packets matches replay/export expectations
- retention capacity is enforced deterministically and overflow fails fast without partial replay output

Concrete expectations:

- synthetic packet fixtures with known contents produce deterministic replay inputs
- sys tests can prove sample counts and packet ordering without requiring hardware
- symbol lookup or output-module initialization failures map to stable sys errors rather than process aborts
- overflow tests prove no final-path VCD is left behind when retention contract fails before export completion

### Core-level validation (`crates/dsview-core`)

This level proves orchestration, schema, and artifact semantics.

Must validate:

- export only runs after clean-success acquisition
- metadata contains all `EXP-04` fields
- metadata distinguishes requested config from observed/exported facts
- actual sample count is taken from observed packet data, not blindly copied from the request
- metadata is only written after VCD export succeeds
- failure modes are classified correctly when VCD export, serialization, or file writes fail
- artifact path derivation and atomic write rules behave as planned

Concrete expectations:

- unit/integration tests assert required JSON field presence and numeric typing
- orchestration tests verify "VCD first, metadata last" ordering
- tests prove that missing either artifact means overall failure for v1

### CLI-level validation (`crates/dsview-cli`)

This level proves contract and user-visible behavior.

Must validate:

- CLI accepts the intended output-path arguments and passes them through correctly
- success output reports artifact locations clearly and deterministically
- exit status is non-zero when export or metadata generation fails
- diagnostics clearly distinguish acquisition failure from artifact-generation failure
- machine-readable output behavior remains stable enough for scripting once introduced

Concrete expectations:

- CLI tests assert printed artifact paths and error categories rather than low-level native details
- path-handling tests cover default metadata derivation from the VCD output path

### Manual hardware validation

This level proves the real device plus native stack produce trustworthy artifacts, not only internally consistent mocks.

Must validate:

- a known-good finite capture produces both VCD and metadata artifacts
- the command exits `0` on success and leaves the device reusable afterward
- the VCD parses or opens successfully in at least one trusted tool path
- enabled channel names in the VCD match the capture configuration
- timing/timescale are plausible for the selected samplerate
- metadata contains all required `EXP-04` fields and plausible observed values
- rerunning the same bounded capture yields stable artifact structure even if timestamps differ
- the manual run stays within the documented retention-capacity envelope so Phase 5 sign-off proves the intended v1 path rather than an overflow edge case

Concrete expectations:

- hardware UAT should use at least one known repeatable signal pattern instead of only floating/noisy lines
- capture size should stay bounded so retention cost is observable and repeatable
- DSLogic Plus manual UAT is a required completion gate for the phase, not an optional follow-up

### Nyquist and timing validation guidance

Phase 5 validation should explicitly cover timing semantics, not only file existence.

Nyquist-oriented expectations:

- validation should include a synthetic or real square-wave pattern whose frequency is comfortably below Nyquist, ideally at or below `sample_rate / 4`, so transition timing can be checked with margin
- validation should avoid claiming correctness from signals at or near `sample_rate / 2`, because aliasing can make a broken export appear plausible
- when a near-limit pattern is used, it should be treated as a cautionary/manual diagnostic case, not the primary golden correctness check
- verification should confirm that observed transition spacing in the VCD is consistent with the configured samplerate and known input frequency within one-sample quantization expectations

Why this matters:

- `EXP-02` depends on timing semantics, and Nyquist discipline is the practical way to avoid over-trusting visually plausible but aliased captures
- this guidance gives `05-VALIDATION.md` a concrete basis for choosing safe sample-rate-to-input-frequency ratios

## Golden-File And Deterministic Synthetic Strategy

Golden files should be the center of automated export verification because they lock observable artifact semantics without requiring hardware.

### VCD golden strategy

Use tiny deterministic synthetic packet streams that are small enough to reason about by inspection.

Recommended fixture characteristics:

- 1-2 enabled channels
- fixed samplerate
- short logic stream with a known initial state and a few deliberate transitions
- explicit end packet
- packet boundaries chosen deliberately so replay logic is tested, not hidden

Assert in golden checks:

- header presence
- `$timescale`
- `$var` declarations with correct channel names/order
- initial value emission
- delta-only transitions after initialization
- final timestamp on stream end

### Deterministic synthetic-render strategy

A useful planning addition for `05-03` is a normalized synthetic-render helper used only in tests.

Recommended test-only concept:

- define small packet fixtures from intended sample sequences rather than raw long byte dumps
- render those fixtures through the same sys replay bridge used by production export
- compare produced VCD against checked-in golden files
- optionally parse the produced VCD into a normalized event list for assertions that are less brittle than full-file text matching

This gives two complementary checks:

- full-file golden comparison for canonical output shape
- normalized event comparison for semantics such as transition time and channel value changes

### Metadata golden/schema strategy

Recommended checks:

- one or more checked-in JSON goldens for field names and structure
- targeted assertions for required fields, numeric typing, and RFC 3339 timestamp shape
- normalization or exclusion of inherently variable fields where needed, such as wall-clock timestamps or absolute output paths

## Biggest Risks To Plan Around

### Export may require delayed cleanup

If upstream export depends on active session/channel state, the current release timing from Phase 4 must move later.

### Packet retention may create significant memory amplification

This is the main runtime risk of the preferred architecture and needs explicit bounded validation.

### Channel naming may drift if recreated outside the original session context

`EXP-02` depends on preserving names from the same configuration context used during capture.

### Timing semantics are easy to approximate incorrectly

Timescale selection, delta emission, sample counting, and final timestamp behavior all matter to downstream tools.

### Metadata can accidentally describe intent instead of reality

The sidecar must differentiate:

- requested configuration
- observed acquisition outcome
- exported artifact facts

## Recommended Plan Split

### `05-01`: Integrate or wrap the DSView-side VCD export path for CLI usage

This plan should focus on:

- packet retention or normalized stream retention in `dsview-sys`
- public output-module invocation for VCD export
- export-before-release lifecycle decisions
- export-specific error mapping and result reporting

This plan should end with:

- a successful clean capture can produce a VCD via upstream export logic
- export-stage failures are distinguishable from acquisition failures
- retention capacity and overflow behavior are explicit enough to verify and reason about

### `05-02`: Generate and validate JSON metadata sidecar output

This plan should focus on:

- metadata schema versioning and field definitions
- actual vs requested value handling
- artifact-path derivation and write ordering
- ensuring metadata only exists for complete successful artifact generation

This plan should end with:

- metadata includes all `EXP-04` fields
- schema is stable enough for automation and later CLI polish
- artifact writes follow atomic-or-cleanup-safe semantics for both VCD and metadata

### `05-03`: Add artifact validation and golden-file checks for export correctness

This plan should focus on:

- deterministic synthetic packet fixtures
- VCD golden-file checks
- metadata schema/golden checks
- Nyquist-aware timing validation guidance
- manual DSLogic Plus UAT for real exported artifacts

This plan should end with:

- a verifier-ready validation matrix covering sys/core/cli/manual levels
- deterministic checks strong enough to support a dedicated `05-VALIDATION.md`
- explicit manual DSLogic Plus sign-off as a phase completion gate

## Planning Questions To Resolve Before Execution Starts

1. Does upstream VCD export require a still-live `sdi` and configured channel list at export time?
2. What is the smallest normalized packet representation that still supports faithful replay into `sr_output_send()`?
3. Should actual sample count be tracked during retention, during replay, or both as a consistency check?
4. What output-path contract should Phase 5 lock now so Phase 6 can refine it rather than redesign it?
5. What capture-size bounds are acceptable for v1 in-memory packet retention?
6. Which timing checks should be fully automated in goldens versus reserved for manual hardware validation?
7. What known signal pattern can be used in manual validation to support Nyquist-safe timing checks?

## Recommendation Summary

Plan Phase 5 around reusing the upstream DSView/libsigrok VCD export path rather than building a new serializer. The major architectural change is extending `dsview-sys` from a summary-only acquisition bridge into a packet-retaining replay bridge that can feed `SR_DF_META`, `SR_DF_LOGIC`, and `SR_DF_END` into upstream VCD export before device/session teardown. Keep metadata schema and artifact orchestration in `dsview-core`, keep CLI changes thin in `dsview-cli`, and center validation on layered sys/core/cli/manual checks plus deterministic golden VCD fixtures. The most important planning decisions to lock early are export-before-release lifecycle ordering, normalized packet retention shape, explicit packet-retention capacity and overflow behavior, atomic-or-cleanup-safe write semantics for both artifacts, Nyquist-safe timing validation expectations, and manual DSLogic Plus sign-off as a required gate for phase completion.
