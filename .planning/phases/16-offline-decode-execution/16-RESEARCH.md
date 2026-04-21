# Phase 16: Offline Decode Execution - Research

**Researched:** 2026-04-21
**Domain:** Raw logic artifact execution, chunked sample feeding, stacked decoder runtime semantics, and failure handling for offline protocol decode.
**Confidence:** HIGH

## Summary

Phase 16 should execute decode runs against a raw logic artifact contract, not VCD text. The strongest execution model is:

- consume `ValidatedDecodeConfig` from Phase 15 directly
- define a raw offline decode input contract carrying samplerate, sample format, sample bytes, and optional packet boundaries
- feed the decode runtime incrementally using absolute sample numbers via `srd_session_send()`
- execute the decoder stack strictly linearly, relying on `OUTPUT_PYTHON` forwarding for stacked decoders
- treat any critical runtime/send/end failure as a failed decode run from the user-facing workflow perspective

Phase 16 should deliver a runnable offline `decode run` path, but Phase 17 should still own the final output/reporting contract and any public degraded-result semantics.

## Recommended Input Contract

The canonical execution input should be a raw logic artifact, not VCD.

Recommended minimum fields:

```json
{
  "samplerate_hz": 1000000,
  "format": "split_logic",
  "sample_bytes": "...",
  "unitsize": 1,
  "logic_packet_lengths": [1024, 512, 1024]
}
```

Where:

- `samplerate_hz` is mandatory
- `format` distinguishes how the raw bytes should be interpreted
  - `split_logic`
  - `cross_logic`
- `sample_bytes` is the canonical data payload
- `unitsize` is required when sample interpretation depends on byte width
- `logic_packet_lengths` is optional but should be preserved when available

Rationale:

- the decode runtime consumes sample streams, not VCD text
- the repo already has reusable raw sample / logic packet surfaces in `dsview-sys`
- VCD is better treated as an export/debug format than the source of truth for execution

## Reusable Execution Patterns In The Repo

### 1. Raw sample / packet helpers already exist

The current sys bridge already models three useful raw execution-adjacent shapes:

- sample bytes with explicit `unitsize`
- split logic packet lengths over a single raw byte stream
- cross-logic packet lengths over channel-expanded data

These helpers are currently used for VCD generation, but their shape is a strong starting point for the Phase 16 offline decode input contract.

### 2. Phase 15 already gives a validated config object

`ValidatedDecodeConfig` already exists and should be the direct execution input on the config side.

That means Phase 16 should not re-interpret raw JSON or redo schema checks. It should consume:

- validated root decoder
- validated linear stack entries
- validated numeric channel bindings
- validated typed option values

### 3. Upstream expects chunked sends with absolute sample numbering

`libsigrokdecode4DSL/session.c` is explicit:

- sample chunks must be sequential
- start/end sample numbers are absolute, not chunk-relative
- gaps or reusing zero-based numbering per chunk are invalid

That makes chunked execution with an absolute sample cursor a hard requirement, not an optimization.

## Stacked Decoder Runtime Semantics

The runtime behavior is already encoded upstream:

- lower decoder emits `OUTPUT_PYTHON`
- `type_decoder.c` forwards that Python object to the next decoder's `decode(startsample, endsample, data)`
- `srd_inst_stack()` already wires lower and upper instances into a stack chain

Implications for Phase 16:

- only the root decoder should bind logic channels
- stacked decoders should be instantiated with options but without direct logic-channel binding
- stack order should exactly follow the validated config order
- execution should fail if runtime stack construction or start/end callbacks fail

## Chunking Strategy Recommendation

Use this rule:

1. if the offline artifact includes packet boundaries, feed by packet
2. otherwise, feed fixed-size chunks
3. always maintain an absolute sample cursor across sends

Why:

- packet-aware sending preserves the nearest thing to the original capture segmentation
- fixed chunk fallback avoids forcing every input artifact to manufacture packet metadata
- both approaches can share one absolute-sample execution loop

The chunk size itself can remain an implementation detail in Phase 16.

## Failure Semantics Recommendation

User-facing semantics should remain binary in Phase 16:

- `success`
- `failure`

Recommended failure conditions:

- decode runtime init fails
- config cannot be projected into runtime instances
- session start fails
- any `srd_session_send()` call fails
- any stacked decoder runtime failure propagates out
- `srd_session_end()` or decoder `end()` fails

Important nuance:

- partial annotations may still be accumulated internally for debugging and future reporting
- but Phase 16 should not expose a public `partial_success` status yet

That keeps execution semantics simple while leaving Phase 17 room to define reporting behavior.

## Core / Sys / CLI Split Recommendation

### sys

Add raw/offline decode session support around:

- session creation / destruction
- metadata injection (`samplerate`)
- root + stacked instance creation from validated config
- chunked send loop support
- annotation callback capture into owned event buffers

### core

Own:

- offline decode input contract
- execution orchestration
- sample cursor management
- chunking policy selection
- final internal execution result type

### cli

Own:

- `decode run`
- input file loading and top-level argument validation
- exit behavior and user-facing rendering only

## Major Risks

### 1. Wrong sample layout assumptions

This is the biggest Phase 16 execution risk.

If the artifact format and runtime feed shape do not agree, decoders will silently produce wrong results or fail in non-obvious ways.

Mitigation:

- make the offline input contract explicit
- validate sample-bytes length against format expectations before execution
- add tests for split logic and cross logic separately

### 2. Treating stacked decoders like root decoders

If stacked decoders are given their own logic-channel binding path, execution semantics drift away from upstream behavior.

Mitigation:

- only the root decoder binds logic channels
- stacked layers only receive validated ids/options and are linked via `srd_inst_stack()`

### 3. Relative sample numbering bugs

If chunk numbering resets per packet/chunk, decode correctness breaks.

Mitigation:

- one absolute sample cursor in core
- tests specifically checking chunk boundary continuity

### 4. Premature public partial-result semantics

If Phase 16 exposes partial success now, Phase 17 loses the freedom to define output/reporting cleanly.

Mitigation:

- retain partial state internally only
- keep user-facing result binary for this phase

## Recommended Sequencing

### 16-01

Define the offline decode input contract and add the raw session/send/end bridge needed for execution.

### 16-02

Implement the core offline decode executor, including absolute-sample chunking and strict linear stack construction.

### 16-03

Add the CLI execution command and regression coverage for success/failure across representative artifact shapes.

## Validation Architecture

Phase 16 validation should prove:

- raw input contract checks for split/cross logic data
- chunked send loop preserves absolute sample numbering
- linear stack execution works end-to-end
- runtime send/end failures fail the decode run cleanly
- CLI `decode run` success/failure behavior works on representative fixtures

## Source Notes

Primary sources consulted:

- `DSView/libsigrokdecode4DSL/session.c` for absolute sample numbering and chunked send semantics
- `DSView/libsigrokdecode4DSL/instance.c` for worker-threaded decode behavior per chunk
- `DSView/libsigrokdecode4DSL/type_decoder.c` for `OUTPUT_PYTHON` forwarding to stacked decoders
- `DSView/DSView/pv/data/decoderstack.cpp` for DSView's own execution-loop semantics reference
- `crates/dsview-sys/src/lib.rs` and `crates/dsview-sys/bridge_runtime.c` for existing raw sample helpers
- sigrok protocol decoder API docs for stacked decoder behavior and output-python semantics

## RESEARCH COMPLETE

Wrote Phase 16 research guidance covering the raw logic input contract, absolute-sample chunked feeding, strict linear stacked execution semantics, failure handling, reusable sample/packet helpers, and the recommended sequencing for Plans 16-01 through 16-03.
