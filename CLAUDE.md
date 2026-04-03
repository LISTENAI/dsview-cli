## Project

DSView CLI is a Rust-based command-line tool for using DSLogic devices without the DSView GUI, starting with `DSLogic Plus`. The first milestone focuses on a scriptable capture workflow that reuses the existing device communication stack from the `DSView/` submodule and exports machine-readable waveform data for downstream AI-agent analysis.

Core value: reliably capture logic-analyzer data from `DSLogic Plus` via CLI and produce analyzable waveform output files.

## Technology Stack

- Rust CLI workspace for product logic and command UX
- Narrow native integration layer against `libsigrok4DSL` or a tiny adapter around it
- `DSView/` kept as a read-only git submodule dependency
- VCD as the primary waveform export, with JSON metadata sidecar for automation

## Conventions

- Treat `DSView/` as upstream dependency code; do not modify it for normal project work
- Keep unsafe/native integration isolated behind a small boundary
- Favor scriptable CLI behavior, explicit exit codes, and machine-readable outputs
- Scope v1 to `DSLogic Plus` only until capture/export is proven stable

## Architecture

- `dsview-cli`: clap-based executable and command UX
- `dsview-core`: safe Rust orchestration for device/session/config flows
- `dsview-sys` or native shim: raw bindings to the DSView/libsigrok integration boundary
- export layer: VCD generation plus metadata artifact writing and validation

## GSD Workflow Enforcement

Before using Edit, Write, or other file-changing tools, start work through a GSD command so planning artifacts and execution context stay in sync.

Use these entry points:
- `/gsd:quick` for small fixes, doc updates, and ad-hoc tasks
- `/gsd:debug` for investigation and bug fixing
- `/gsd:execute-phase` for planned phase work

Do not make direct repo edits outside a GSD workflow unless the user explicitly asks to bypass it.

## Developer Profile

> Profile not yet configured. Run `/gsd:profile-user` to generate your developer profile.
> This section is managed by `generate-claude-profile` — do not edit manually.
