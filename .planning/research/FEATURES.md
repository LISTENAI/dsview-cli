# Feature Research: DSView CLI

**Researched:** 2026-04-03
**Scope:** CLI-first logic-analyzer workflow for `DSLogic Plus`

## Summary

For this project, table stakes are not broad DSView parity; they are the minimum capabilities required to produce trustworthy, machine-readable captures from a real device. The first release should feel predictable and scriptable: explicit device selection, explicit capture parameters, deterministic export, and enough metadata that downstream tooling can understand the capture without reverse-engineering CLI behavior.

The biggest scope trap is confusing "core DSView features" with "first useful CLI features." For v1, the product succeeds if it can reliably capture logic data from `DSLogic Plus` and export VCD plus enough metadata for AI or scripts to analyze the result.

## Table Stakes for v1

### Device access
- Detect available DSLogic devices
- Select `DSLogic Plus` explicitly when present
- Fail clearly when the device is absent, busy, or unsupported

### Capture configuration
- Set core sampling parameters needed for a basic logic capture
- Select enabled channels
- Configure sample rate and capture depth / sample limit
- Surface the effective capture settings back to the user

### Capture execution
- Start a capture from the CLI
- Complete capture successfully or fail with actionable diagnostics
- Produce deterministic output files in a user-specified location

### Export for analysis
- Export VCD as the primary waveform artifact
- Emit machine-readable metadata describing the capture session
- Preserve timing/channel information needed by downstream tooling

### Scriptability
- Stable non-interactive command shape
- Exit codes suitable for shell automation
- No GUI/TUI dependency in the happy path

## Good Candidates for v2+

- Protocol decode workflows (SPI / I2C / UART, etc.)
- Additional export targets such as CSV or JSON waveform projections
- Capture presets / named profiles
- Trigger configuration beyond the minimal useful subset
- Batch capture workflows
- More DSLogic-family devices
- Richer device inspection and diagnostics commands

## Anti-Features for Early Phases

- Full DSView parity as a roadmap promise
- Terminal waveform renderer for v1
- Automatic AI-agent invocation inside the CLI
- Support for all sigrok-compatible devices before `DSLogic Plus` is stable
- Deep protocol decoding before the raw capture/export path is trustworthy
- Trying to expose every device knob on day one

## Dependencies Between Features

- Device detection and selection must exist before reliable capture commands.
- Capture parameter validation must exist before export quality can be trusted.
- Export correctness depends on channel mapping, samplerate, and sample-count metadata being preserved.
- Machine-readable metadata is a dependency for robust downstream AI analysis even if the waveform itself is in VCD.
- Future decode support depends on capture files and timing semantics being correct first.

## Complexity / Risk Notes

| Feature area | Complexity | Notes |
|---|---|---|
| Device listing / selection | Medium | Depends on how cleanly DSView/libsigrok4DSL exposes discovery |
| Core capture configuration | Medium | Parameters are straightforward conceptually but may have device-specific constraints |
| Capture lifecycle | High | Real hardware state, timeouts, and failure modes matter |
| VCD export | Medium | Existing backend helps, but correctness must be verified |
| Metadata sidecar | Low | Easy to add and high value for automation |
| Protocol decode | High | Valuable later, but not needed to prove the first workflow |

## Expectations for AI-Oriented Output

To be useful for downstream AI workflows, v1 output should optimize for reproducibility and context, not just raw bytes.

Recommended capture bundle:
- `capture.vcd` — canonical waveform file
- `capture.meta.json` — device model, timestamp, enabled channels, samplerate, sample limit, actual sample count, CLI version, submodule revision if available

This keeps the waveform standard while giving agents enough structured context to reason about it.

## Recommendation

Define v1 around one killer flow:

1. discover/select `DSLogic Plus`
2. configure core acquisition parameters
3. capture logic data
4. export `VCD + JSON metadata`

Everything else should be treated as explicitly deferred until that loop is stable.
