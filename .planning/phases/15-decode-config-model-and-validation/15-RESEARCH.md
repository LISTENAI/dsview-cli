# Phase 15: Decode Config Model and Validation - Research

**Researched:** 2026-04-21
**Domain:** JSON decode configuration shape, typed option modeling, stack compatibility validation, and CLI-facing diagnostics for pre-execution decode config checks.
**Confidence:** HIGH

## Summary

Phase 15 should introduce a JSON-only config model plus a strict validation pipeline that runs before any decode execution begins. The strongest approach is:

- one root decoder plus an ordered linear `stack`
- channel bindings as `{ canonical_channel_id: numeric_index }`
- option ids preserved exactly as upstream ids
- option values typed in JSON, not string-only
- a standalone `decode validate --config <PATH>` CLI surface so users can validate configs before Phase 16 adds execution

This phase should not attempt branching graphs, runtime sample feeding, or capture integration.

## Recommended Config Shape

A machine-first JSON shape should be preferred over DSView's GUI-oriented persistence shape.

Recommended baseline:

```json
{
  "version": 1,
  "decoder": {
    "id": "0:i2c",
    "channels": {
      "scl": 0,
      "sda": 1
    },
    "options": {
      "address_format": "unshifted"
    }
  },
  "stack": [
    {
      "id": "some-decoder",
      "options": {}
    }
  ]
}
```

Why this over DSView's current JSON:

- DSView stores channel bindings as an array of one-key objects, which is awkward for machine generation.
- DSView stores stacked decoders separately from the root decoder, but the CLI can keep the same semantics while making ordering explicit.
- A top-level `version` field gives future migration room without inventing token aliases now.

## DSView Compatibility Notes

Useful upstream concepts to preserve:

- root decoder id at the primary object level
- `stacked decoders` semantics as an ordered chain
- channel bindings keyed by canonical upstream channel id
- options keyed by canonical upstream option id

Useful upstream shapes to avoid copying directly:

- array-of-single-key channel binding objects
- GUI-only fields like label, show state, and view index

## Typed Option Modeling

Phase 15 has one important metadata gap to address: current decode option metadata in Rust only exposes `default_value: Option<String>` and `values: Vec<String>`.

That is not strong enough to validate typed JSON values reliably.

Recommended Phase 15 change:

- extend sys/core decode option descriptors with an explicit value-kind field derived from upstream metadata
- keep allowed values available in a typed-friendly form or preserve enough metadata to validate input deterministically

Recommended supported config value kinds for this phase:

- string
- integer
- float

Boolean should only be added if upstream metadata actually exposes boolean-like options in a way the Rust bridge can type safely. Do not force booleans in just because JSON can express them.

## Validation Architecture

Validation should be layered in this order:

1. **File / parse validation**
   - file exists
   - JSON parses
   - top-level structure matches schema

2. **Schema validation**
   - required fields exist
   - fields have the right JSON types
   - root decoder exists
   - `stack` is an array
   - channel binding values are integers
   - option values match declared JSON type expectations

3. **Metadata validation**
   - decoder id exists in registry
   - required channels are all bound
   - channel ids are valid for the chosen decoder
   - option ids are valid for the chosen decoder
   - option values are legal for the decoder metadata
   - stacked decoders are individually valid

4. **Stack compatibility validation**
   - each decoder in `stack` is compatible with the previous decoder's outputs
   - compatibility should use the same upstream `inputs` / `outputs` strings already exposed by Phase 14
   - Phase 15 should be stricter than upstream `srd_inst_stack()` warnings and fail on incompatible stacks

5. **Runtime prerequisite validation**
   - decode runtime exists
   - decoder directory exists and is readable
   - registry can be loaded before metadata checks begin

This layered approach lets CLI errors stay precise and actionable.

## Error Model Recommendation

Follow the repo's stable `code` + `message` + optional `detail` pattern.

Recommended error families:

- `decode_config_file_missing`
- `decode_config_parse_failed`
- `decode_config_schema_invalid`
- `decode_decoder_not_found`
- `decode_missing_required_channel`
- `decode_unknown_channel_binding`
- `decode_unknown_option`
- `decode_option_value_invalid`
- `decode_stack_incompatible`
- `decode_runtime_missing`
- `decode_decoder_dir_missing`
- `decode_registry_load_failed`

Strict failure-first means:

- no warning-only mode in Phase 15
- non-zero exit if config is not executable in principle
- report the first error with enough detail to fix quickly, or aggregate schema issues if the parser layer already supports that cleanly

## CLI Recommendation

Phase 15 should add a dedicated validation command:

```bash
dsview-cli decode validate --config path/to/decode.json
```

Why:

- it satisfies DEC-04 before Phase 16 adds execution
- it keeps validation separate from runtime decode execution
- it gives a clean place to lock diagnostics and JSON/text outputs with CLI tests

Recommended behaviors:

- `--format json|text` like the existing commands
- success: explicit valid result plus normalized high-level summary
- failure: stable error code and detail

## Reusable Code Patterns

Reuse from the current repo:

- `DecodeDiscoveryPaths` from `dsview-core` for runtime and decoder-dir discovery
- `DecoderDescriptor` from `dsview-core` as the source of truth for validation metadata
- CLI error rendering in `crates/dsview-cli/src/main.rs`
- JSON/text renderer split in `crates/dsview-cli/src/lib.rs`

## Key Risks

### 1. Typing gap in decoder option metadata

If Phase 15 does not add option kind metadata, validation will either become guessy or degrade all config values to strings.

Mitigation:

- make typed option metadata part of `15-01`

### 2. Over-copying DSView GUI JSON shape

If the schema mirrors DSView's persistence format too literally, the CLI config becomes harder for automation to generate and validate.

Mitigation:

- preserve semantics, not GUI-shaped serialization

### 3. Under-validating stack compatibility

If stack compatibility is left until execution, Phase 16 will inherit config errors that should have been caught earlier.

Mitigation:

- make stack compatibility a hard validation step in `15-02`

### 4. Hiding runtime preconditions behind generic config errors

If runtime load failures are reported as generic config failures, users cannot tell whether their JSON is wrong or the environment is broken.

Mitigation:

- keep runtime prerequisite errors distinct from schema/metadata errors

## Recommended Sequencing

### 15-01

Define the typed Rust config schema and fill the current metadata gap for option value kinds.

### 15-02

Implement metadata-driven validation and the stable error taxonomy, including strict stack compatibility checks.

### 15-03

Add `decode validate --config` plus CLI contract tests for valid configs and major invalid-config classes.

## Validation Architecture

The validation companion file for this phase should require:

- schema-level unit tests
- metadata-driven validation tests
- stack compatibility tests
- CLI validation command tests
- at least one end-to-end validation run against a real decoder metadata fixture from Phase 14 discovery

## RESEARCH COMPLETE

Wrote Phase 15 research guidance covering the JSON-only config shape, the linear stack model, typed option handling, layered validation architecture, stable error taxonomy, and the recommended sequencing for Plans 15-01 through 15-03.
