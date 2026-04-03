# Architecture Research: DSView CLI

**Researched:** 2026-04-03
**Scope:** Brownfield Rust CLI reusing `DSView/` submodule

## Summary

The architecture should isolate three concerns cleanly: CLI UX in Rust, native capture integration at the DSView/libsigrok boundary, and export/output assembly. The project should not mix command parsing with native session management, and it should not let DSView-specific assumptions leak throughout the Rust codebase.

A layered design is the safest path: raw bindings at the bottom, a safe Rust session API in the middle, and user-facing command orchestration at the top. This makes it possible to support later devices or export formats without rewriting the command surface.

## Recommended Component Boundaries

### 1. CLI layer
Responsible for:
- parsing arguments and subcommands
- validating user input
- choosing output paths
- rendering logs/errors
- returning stable exit codes

### 2. Application/service layer
Responsible for:
- mapping CLI input into capture requests
- orchestrating device selection, config, run, and export
- producing metadata manifests
- translating native errors into product errors

### 3. Native integration layer
Responsible for:
- binding to `libsigrok4DSL` or a narrow adapter around it
- enforcing lifecycle ordering
- managing native pointers/resources safely
- exposing a minimal Rust-friendly interface for sessions/devices

### 4. Export layer
Responsible for:
- invoking VCD output path
- validating output file creation
- generating JSON metadata sidecar
- keeping output contracts stable for downstream consumers

## Suggested Rust Workspace Shape

```text
crates/
  dsview-sys/         # raw FFI bindings to native layer
  dsview-core/        # safe wrappers, session lifecycle, domain types
  dsview-export/      # VCD + metadata orchestration
  dsview-cli/         # clap-based executable
```

If the native surface is too awkward, insert a small `native/` adapter library owned by this repo and bind Rust to that stable shim instead of binding many internal headers directly.

## Data Flow

1. User runs CLI command with device/config/output options.
2. CLI parses args and builds a validated capture request.
3. Service layer resolves target device (`DSLogic Plus`) and configures session parameters.
4. Native integration layer opens the device and drives acquisition.
5. Native/export layer emits VCD output.
6. Rust export layer writes JSON metadata sidecar.
7. CLI reports artifact locations and exits with success/failure code.

## Suggested Build Order

### Phase A: integration viability
- Prove the smallest possible native path for device discovery and session creation.
- Confirm whether a direct `libsigrok4DSL` link is sufficient or whether a shim is required.

### Phase B: stable domain model
- Define Rust types for device selection, capture config, and artifact metadata.
- Centralize validation rules.

### Phase C: capture path
- Implement session lifecycle for `DSLogic Plus` only.
- Prove reliable acquisition with a fixed config.

### Phase D: export contract
- Add VCD export and metadata sidecar.
- Validate output semantics with golden files and sample runs.

### Phase E: CLI polish
- Finalize command UX, diagnostics, help text, and scripting behavior.

## Device-Specific Isolation

`DSLogic Plus` assumptions should live behind a device-profile concept, even if there is only one profile in v1. This keeps later expansion from forcing a rewrite.

Recommended isolation points:
- supported device IDs / names
- allowed or default sampling configurations
- channel count assumptions
- any DSLogic Plus-specific setup or quirks

## Testing Strategy

### Unit tests
- CLI argument parsing
- config validation
- metadata generation
- output path handling

### Integration tests without hardware
- binding smoke tests where possible
- golden-file checks for VCD and metadata outputs
- fixture-driven tests for known capture/export behavior

### Hardware-in-the-loop
- manual or lab-run smoke tests for real `DSLogic Plus`
- happy-path capture
- unsupported/absent device behavior
- invalid parameter combinations

## Brownfield Pitfalls to Design Around

- Native resource ownership crossing the Rust/C boundary
- Assumptions hidden in DSView GUI code rather than native libraries
- Coupling command semantics to the current DSView internal structure
- Export success being reported before artifacts are fully valid

## Recommendation

Architect the project so the CLI depends on a small, stable Rust service layer, which in turn depends on a deliberately narrow native boundary. The narrower this native contract is, the easier every later phase becomes.
