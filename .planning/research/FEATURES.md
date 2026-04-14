# Feature Research: v1.2 DSView protocol decode CLI foundation

**Date:** 2026-04-14
**Scope:** User-facing capabilities for protocol decode in `DSView CLI`

## Table Stakes

### Decoder discovery

Users should be able to:

- list available protocol decoders
- inspect decoder ids, names, channels, options, and stackable relationships

### Config-driven decode setup

Users should be able to:

- define decoder stacks in a config file
- bind capture channels to decoder channels
- set decoder options without memorizing huge command lines

### Offline decode execution

Users should be able to:

- run protocol decode against previously captured logic data
- receive machine-readable annotation output
- use stacked decoders where upstream decoder output feeds downstream decoders

### Stable reporting

Users should be able to:

- tell whether a failure is caused by bad config, bad input data, missing runtime prerequisites, or decoder execution errors
- save decode output to explicit artifact paths

## Differentiators For This Project

Good differentiators for this milestone:

- Preserve a clean `capture` command by keeping decode config separate
- Reuse DSView-compatible decoder concepts instead of inventing a parallel decode model
- Make JSON/JSONL output first-class for automation and AI workflows

## Anti-Features For This Milestone

Do not prioritize now:

- Full DSView session import/export parity
- Live streaming decode during capture
- GUI-equivalent decode visualization, row painting, or table interaction
- One-flag-per-decoder-option command design

## Complexity Notes

- Decoder discovery and inspect surfaces are relatively low-risk once runtime loading works.
- Decode execution is medium/high risk because of callback bridging, Python embedding, and input-sample layout correctness.
- A full capture+decode pipeline is broader than the core foundation and should remain future-facing unless the milestone still has margin.

## Milestone Recommendation

This milestone should focus on:

1. decoder registry and inspection
2. decode config schema and validation
3. offline decode execution
4. machine-readable output and stable failure reporting

It should not promise:

- live decode
- capture flag integration
- full workflow presets
