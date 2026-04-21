# Phase 16 Validation

**Date:** 2026-04-21
**Phase:** 16 - Offline Decode Execution
**Purpose:** Validation strategy for offline decode execution, sample-feeding correctness, stack orchestration, and execution-failure behavior.

## Validation Architecture

Phase 16 validation should prove five things:

1. offline decode input contract correctness
2. chunked sample feeding with absolute sample numbering
3. strict linear stack execution behavior
4. failure handling for send/end/runtime problems
5. CLI `decode run` success/failure behavior on representative fixtures

## Required Validation Coverage

### Input contract

- reject empty sample data
- reject missing samplerate
- reject malformed split/cross logic layout
- reject packet lengths that do not sum to payload size

### Chunking and sample numbering

- prove chunked execution preserves an absolute sample cursor
- prove packet-based execution uses provided packet boundaries when available
- prove fixed chunk fallback still preserves absolute numbering

### Stack execution

- root decoder binds logic channels
- stacked decoders are linked linearly
- stacked decoder execution succeeds when metadata compatibility was validated in Phase 15

### Failure handling

- runtime/session start failure aborts the run
- `srd_session_send` failure aborts the run
- `srd_session_end` / finalization failure aborts the run
- no public `partial_success` state is introduced

### CLI behavior

- valid offline decode run succeeds
- invalid artifact shape fails with stable machine-readable errors
- runtime execution failures fail the command cleanly

## Minimum Evidence Expected From Plans

- sys/core tests for chunked send semantics
- tests for stacked decoder orchestration
- tests for execution failure propagation
- CLI tests for `decode run` success/failure

## Outcome Target

By the end of Phase 16, users should be able to run a decode session offline against saved raw logic artifacts through the CLI, with strict success/failure behavior and a sound execution model ready for Phase 17 output/reporting work.
