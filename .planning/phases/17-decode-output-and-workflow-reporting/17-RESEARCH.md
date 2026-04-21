# Phase 17: Decode Output and Workflow Reporting - Research

**Researched:** 2026-04-21
**Domain:** Decode result schema, failure reporting, partial diagnostics exposure, and artifact/stdout behavior for offline decode runs.
**Confidence:** HIGH

## Summary

Phase 17 should finalize a JSON-first decode reporting contract without changing the already-shipped Phase 16 execution semantics.

Recommended direction:

- canonical result shape = `run + flat events`
- `events` stay the primary machine-readable contract
- optional additive summaries may exist, but must not replace the flat event list
- failures remain `failure`, even when partial diagnostics/events are exposed
- stdout remains the default success path
- `--output` is an optional artifact-writing path, not the primary mode

## Recommended Result Schema

Recommended top-level success shape:

```json
{
  "run": {
    "status": "success",
    "root_decoder_id": "0:i2c",
    "stack_depth": 1,
    "sample_count": 4096,
    "event_count": 42
  },
  "events": [
    {
      "decoder_id": "0:i2c",
      "start_sample": 120,
      "end_sample": 144,
      "annotation_class": 3,
      "annotation_type": 111,
      "texts": ["Address write", "0x50"]
    }
  ]
}
```

Recommended failure shape:

```json
{
  "run": {
    "status": "failure"
  },
  "error": {
    "code": "decode_session_end_failed",
    "message": "decode session end failed"
  },
  "partial_events": [...],
  "diagnostics": {
    "partial_event_count": 42,
    "partial_events_available": true
  }
}
```

Key point:

- partial diagnostics can exist on failure
- but they must not upgrade the run into `partial_success`

## Why Flat Events Should Be Canonical

The current system already captures per-annotation events with:

- `decoder_id`
- `start_sample`
- `end_sample`
- `annotation_class`
- `annotation_type`
- `texts`

This is already the most reusable raw reporting unit.

Advantages of keeping flat events canonical:

- simple for automation, diffing, and AI consumption
- easy to re-group by decoder, row, or time later
- avoids hard-coding GUI/table assumptions into the public contract

Grouped views, row summaries, or per-decoder summaries can still be added later as secondary convenience fields.

## Reusable Reporting Pieces Already In The Repo

### 1. Stable CLI error envelope already exists

`crates/dsview-cli/src/main.rs` already has the right `ErrorResponse` pattern:

- stable `code`
- readable `message`
- optional `detail`

Phase 17 should reuse this shape for decode output/reporting failures instead of inventing a new failure envelope.

### 2. Phase 16 already exposes run summary fields

`DecodeRunResponse` already contains a coarse execution summary:

- `root_decoder_id`
- `stack_depth`
- `sample_count`
- `annotation_count`
- `annotation_decoder_ids`

That means Phase 17 can evolve from an existing success summary rather than creating output from scratch.

### 3. Sys layer already exposes raw captured annotation events

`dsview-sys` already exposes captured annotation records with the exact event-level data Phase 17 needs.

This strongly suggests Phase 17 should format/report those events rather than re-synthesize a higher-level structure from scratch.

## Partial Diagnostics Recommendation

Expose them, but only as supplementary failure information.

Recommended rules:

- on success:
  - `events` contains the full final event list
- on failure:
  - `run.status = failure`
  - `error` explains the failure class
  - optional `partial_events` may be included if execution collected any
  - optional `diagnostics` may include counts / notes

This preserves:

- strict workflow semantics from Phase 16
- practical debugging value for users and automation
- room for Phase 17 to define reporting without reopening execution-state semantics

## Artifact Behavior Recommendation

Recommended default:

- stdout is the default success output
- `--output` optionally writes the full JSON result to a file

Recommended failure behavior:

- if `--output` is provided and the run fails but partial diagnostics are allowed to be written, Phase 17 must define whether failure artifacts are still written
- the simplest consistent rule is:
  - always print the primary result/error to stdout/stderr as usual
  - if `--output` is provided, write the same top-level result document to disk

This keeps the command pipe-friendly and script-friendly.

## Error Taxonomy Recommendation

Recommended stable failure codes at the reporting layer:

- `decode_config_invalid`
- `decode_input_invalid`
- `decode_runtime_missing`
- `decode_decoder_dir_missing`
- `decode_session_start_failed`
- `decode_session_send_failed`
- `decode_session_end_failed`
- `decode_runtime_failed`
- `decode_output_write_failed`

Rule:

- the main error code explains why the run failed
- supplementary fields explain whether partial diagnostics exist

Do not create special codes like:

- `decode_partial_success`
- `decode_failed_with_partial_output`

That information belongs in diagnostics fields, not in the primary error taxonomy.

## Text Output Recommendation

Text output should stay compact and execution-focused.

Recommended success text should include:

- root decoder
- stack depth
- sample count
- event count
- optional output path if written

Recommended failure text should include:

- error code
- short failure message
- whether partial events were retained
- output path if a failure artifact was written

Avoid trying to print all events in text mode by default.

## Major Risks

### 1. Over-complicating the public schema

If Phase 17 makes grouped or nested decoder views the primary output, the schema becomes harder to consume and harder to evolve.

Mitigation:

- keep flat events canonical
- make grouped summaries additive only

### 2. Blurring failure semantics

If partial diagnostics are treated like a special success state, the clean execution model from Phase 16 is lost.

Mitigation:

- keep `status` binary
- expose partial data only as supplementary failure context

### 3. Artifact/output divergence

If stdout and `--output` diverge semantically, users and scripts will have to treat them as different products.

Mitigation:

- make stdout and file output representations consistent
- `--output` should persist the same canonical document shape

## Recommended Sequencing

### 17-01

Define the final decode result schema and the CLI/core reporting result types.

### 17-02

Implement failure reporting, partial diagnostics exposure, and optional artifact writing.

### 17-03

Lock stdout/file/JSON/text contracts and the final error semantics with CLI regressions.

## Validation Architecture

Phase 17 validation should prove:

- success result schema is stable and machine-readable
- failure result schema is stable and machine-readable
- partial diagnostics on failure do not change run status
- stdout default behavior remains intact
- `--output` writes the expected artifact shape

## RESEARCH COMPLETE

Wrote Phase 17 research guidance covering the canonical `run + flat events` schema, failure reporting model, partial diagnostics exposure, stdout-versus-artifact behavior, and the recommended sequencing for Plans 17-01 through 17-03.
