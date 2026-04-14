# Research Summary: v1.2 DSView protocol decode CLI foundation

**Date:** 2026-04-14

## Stack additions

- Add a dedicated native decode runtime around `DSView/libsigrokdecode4DSL`
- Add Rust-owned decoder metadata, config, execution, and annotation event types
- Package or discover Python runtime compatibility plus decoder script search paths

## Recommended milestone focus

- `decode list` and `decode inspect`
- config-driven decoder stack definition
- offline decode execution on saved logic data
- machine-readable annotation output and stable failure reporting

## Architectural direction

- Keep `capture` and `decode` separate
- Treat future `pipeline` support as orchestration, not parameter flattening
- Reuse the existing native-boundary -> core -> CLI layering already proven by `v1.0` and `v1.1`

## Watch out for

- Do not reuse Qt/PulseView decode UI classes as the CLI engine
- Do not bloat `capture` with decoder-specific flags
- Validate input sample layout and absolute sample sequencing carefully
- Make Python/decorator search-path failures explicit and testable

## Milestone recommendation

Proceed with an incremental `v1.2` milestone centered on a decode foundation, not full DSView protocol-analysis parity.
