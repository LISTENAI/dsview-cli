# Phase 15 Validation

**Date:** 2026-04-21
**Phase:** 15 - Decode Config Model and Validation
**Purpose:** Validation strategy for the decode config schema, metadata-driven checks, and CLI diagnostics added in Phase 15.

## Validation Architecture

Phase 15 validation should prove five layers:

1. JSON file and schema parsing
2. Typed config-model correctness
3. Decoder metadata-driven validation
4. Stack compatibility enforcement
5. CLI-facing diagnostics and exit behavior

## Required Validation Coverage

### Schema and parsing

- invalid JSON fails with a stable parse/config error
- missing required fields fail deterministically
- wrong JSON types fail deterministically
- config version handling is explicit if a version field is added

### Typed config model

- channel bindings remain numeric indexes
- option values retain their intended typed shape
- root decoder and stacked decoder objects normalize into a stable internal model

### Metadata validation

- unknown decoder ids fail cleanly
- missing required channels fail cleanly
- unknown channel ids fail cleanly
- unknown option ids fail cleanly
- option values outside the decoder metadata contract fail cleanly

### Stack compatibility

- incompatible linear stacks fail before execution
- valid linear stacks pass without requiring Phase 16 execution plumbing

### CLI diagnostics

- `decode validate --config <PATH>` succeeds for known-good configs
- invalid configs produce stable machine-readable errors
- text output remains actionable for humans

## Minimum Evidence Expected From Plans

- unit tests for config parsing and normalization
- unit tests for metadata-driven validation
- tests for stack compatibility decisions
- CLI tests for success and failure diagnostics

## Outcome Target

By the end of Phase 15, users should be able to author a JSON decode config and ask the CLI to validate it with strict, machine-readable results before any decode execution begins.
