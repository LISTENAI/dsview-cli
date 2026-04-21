# Phase 17 Validation

**Date:** 2026-04-21
**Phase:** 17 - Decode Output and Workflow Reporting
**Purpose:** Validation strategy for final decode output schema, failure reporting, partial diagnostics exposure, and optional artifact writing.

## Validation Architecture

Phase 17 validation should prove five things:

1. success output schema correctness
2. failure output schema correctness
3. partial diagnostics behavior on failure
4. stdout default behavior
5. optional artifact writing behavior

## Required Validation Coverage

### Success schema

- result includes a `run` summary block
- result includes a flat `events` list
- event items preserve the canonical annotation fields needed for machine consumption

### Failure schema

- failures use stable machine-readable error codes
- failures do not masquerade as success when partial events exist
- partial diagnostics appear only as supplementary failure fields

### Artifact behavior

- stdout remains the default output path
- `--output` writes the canonical result document to disk
- file output remains structurally aligned with stdout JSON

### Text behavior

- text success output is concise and not event-dump oriented
- text failure output is concise but still reports code/message/diagnostics availability

### Regression expectations

- CLI tests for success JSON/text
- CLI tests for failure JSON/text
- CLI tests for optional output-file writing

## Outcome Target

By the end of Phase 17, `decode run` should have a stable JSON-first reporting contract, stable failure semantics, and optional artifact writing while preserving the execution model locked in Phase 16.
