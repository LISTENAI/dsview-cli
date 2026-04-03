# Research Summary: DSView CLI

**Completed:** 2026-04-03

## Key Findings

**Stack:** Use Rust for the CLI surface, with a narrow native integration layer against `libsigrok4DSL` or a tiny repo-owned shim around it. Avoid using the DSView GUI layer as the integration boundary.

**Core v1 shape:** Scope the first release to one end-to-end flow for `DSLogic Plus`: detect/select device, configure capture, run acquisition, export `VCD`, and write a JSON metadata sidecar.

**Architecture:** Keep a layered design:
- Rust CLI for command UX
- Rust service/core for capture orchestration and validation
- raw FFI / adapter layer for native integration
- export layer for `VCD + metadata`

**Biggest risks:**
- binding to the wrong DSView layer
- accidental dependence on the full DSView build graph
- ABI drift from submodule updates
- export files that are technically produced but not reliable for downstream analysis

**Recommended v1 export:** `VCD` as the canonical waveform artifact, plus machine-readable JSON metadata for AI-agent workflows.

## Decision-Ready Guidance

### Build now
- Rust CLI workspace
- `DSLogic Plus` only
- scriptable subcommands and exit codes
- capture configuration for the minimum useful path
- VCD export
- JSON metadata sidecar

### Defer
- protocol decode
- terminal waveform rendering
- built-in AI invocation
- multi-device support
- full DSView parity

### Guardrails
- Treat `DSView/` as a read-only submodule dependency
- Do not modify upstream code for v1
- Pin the submodule revision as part of the product contract
- Add validation around capture artifacts, not just native return codes
