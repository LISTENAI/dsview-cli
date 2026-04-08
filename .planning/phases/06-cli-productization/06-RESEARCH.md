# Phase 6 Research: CLI Productization

**Date:** 2026-04-08
**Phase:** 6 - CLI Productization
**Goal:** Deliver a polished non-interactive capture-and-export command that works well in shell and agent workflows.

## What Phase 6 Must Achieve

Phase 6 turns the validated acquisition-plus-export pipeline from Phases 4 and 5 into a stable product surface for scripts, shells, and AI-agent workflows.

This phase must satisfy:

- `CLI-01`: user can run the full capture-and-export workflow non-interactively from a single CLI command
- `CLI-02`: user can choose the output path for generated artifacts
- `CLI-03`: CLI prints the locations of generated artifacts after a successful run

Planning implication:

- Phase 6 is not a new capture/export engine.
- It is a productization phase for the already-working `capture` command.
- The work should focus on command UX, output-path control, result presentation, and process-boundary validation rather than redesigning the core runtime boundary.

## Current Baseline After Phase 5

### The single-command workflow already exists

`crates/dsview-cli/src/main.rs` already wires one end-to-end `capture` flow that:

1. validates capture config
2. runs acquisition
3. exports VCD on clean success
4. writes metadata
5. returns both artifact paths in JSON success output

This means `CLI-01` is partially implemented already.

Planning implication:

- Phase 6 should refine and harden the existing `capture` command rather than introduce a second command model.
- The current flow should remain the canonical non-interactive workflow.

### Artifact path control is only partially productized

The current CLI accepts one explicit `--output` path for the VCD artifact.
Metadata path handling is deterministic but implicit because it is derived in core from the VCD path.

This is enough for a narrow scriptable contract, but it is not yet a polished answer to `CLI-02` because:

- users cannot clearly choose both final artifact locations independently
- path derivation policy is not surfaced as an intentional product contract
- invalid or conflicting path choices are not yet a first-class UX topic

Planning implication:

- Phase 6 must either bless and document the single-path-plus-derived-sidecar model as the v1 contract, or add explicit metadata/output-directory controls.
- The chosen model must preserve the cleanup-safe artifact semantics established in Phase 5.

### Text mode is too weak for human shell workflows

`--format json` already provides a machine-readable success payload with `vcd_path` and `metadata_path`.
By contrast, text-mode success currently prints only `ok` for capture runs.

This is the main gap against `CLI-03`.

Planning implication:

- JSON should remain the authoritative automation contract.
- Text mode should become intentionally human-usable by printing final artifact locations and completion context.
- Productization should not make text mode so verbose or unstable that scripts are tempted to parse it instead of JSON.

### Existing CLI tests do not yet prove the full product surface

Current CLI tests focus mostly on helper-level response shapes and error classification.
They do not adequately prove:

- real clap parsing behavior
- `--help` quality
- stdout vs stderr behavior at the process boundary
- text-mode success output
- end-to-end path reporting against real created files

Planning implication:

- Phase 6 validation must move outward to the real binary boundary.
- Process-spawned tests and a small manual shell workflow UAT are the right proof.

## Constraints Carried Forward

### Keep the crate layering intact

- `dsview-sys` remains the native/runtime boundary
- `dsview-core` remains the orchestration and artifact-contract layer
- `dsview-cli` remains the command UX and rendering layer

Planning implication:

- Phase 6 should not push export logic or metadata assembly into `main.rs`.
- CLI productization should consume stable core results rather than reimplement business rules.

### Preserve the Phase 5 artifact contract

Phase 5 established these truths:

- export only follows clean-success acquisition
- VCD and metadata form one complete artifact set for v1
- failed artifact generation must not present a misleading partial-success result
- final artifact paths should only be surfaced after their respective cleanup-safe write contracts complete

Planning implication:

- Phase 6 can improve path controls and output presentation, but must not weaken complete-artifact semantics.
- Any new path model must stay compatible with temp-file promotion and cleanup-safe failure behavior.

### Preserve machine-readable diagnostics

The CLI already maps failures into stable machine-readable codes for acquisition, export, and metadata errors.

Planning implication:

- JSON mode should stay stable and explicit.
- Text mode can be improved for readability, but error-class distinctions must remain visible at the binary boundary.

## Recommended Product Direction

Plan Phase 6 around polishing the existing `capture` command as the one-shot product surface.

Recommended direction:

- keep `capture` as the canonical single-command workflow
- preserve JSON as the authoritative automation contract
- make text output useful for direct shell use
- expand output-path UX without regressing cleanup-safe artifact semantics
- validate the final product shape at the spawned-process CLI boundary

Why this is the best fit:

- it satisfies the roadmap requirement without reworking the proven Phase 4/5 internals
- it keeps the UX improvements local to the CLI and small core path-shaping extensions
- it aligns with the project goal of a scriptable DSLogic Plus capture tool rather than a GUI-adjacent wrapper

## Locked v1 Result Output Contract

The existing `capture` command remains the canonical non-interactive workflow for DSLogic Plus capture and export in v1.

### JSON success contract

JSON remains the authoritative automation contract.
Successful `capture` runs must continue to return machine-readable fields that let shells and agents trust the result without parsing text output:

- `selected_handle`
- `completion`
- `saw_logic_packet`
- `saw_end_packet`
- `saw_terminal_normal_end`
- `cleanup_succeeded`
- `artifacts.vcd_path`
- `artifacts.metadata_path`

Contract implications:

- both final artifact paths must always be present on JSON success
- JSON field names should remain stable through Phase 6 polish unless a deliberate contract update is documented and tested
- automation should treat JSON as the only parse target; text output is not a compatibility contract

### Text success contract

Text mode exists for direct shell use, not structured parsing.
Successful `capture` runs should print a short, shell-friendly artifact summary rather than only `ok`.

Required text facts for a successful capture:

- the final completion state
- the final VCD artifact path
- the final metadata artifact path

Recommended v1 shape:

- `capture <completion>`
- `vcd <path>`
- `metadata <path>`

Contract implications:

- the text output should stay concise enough to scan in a terminal
- it should help a human operator immediately find artifacts without inspecting the filesystem
- it should not grow into a verbose diagnostic dump that scripts might be tempted to parse

### Failure output contract

Failure classes must remain distinct at the CLI boundary.
JSON failure remains the stable machine-readable contract with explicit error codes and relevant detail fields.
Text failure may stay concise, but it must still surface the stable CLI error code and a human-readable message.

### Help and discoverability caveats for v1

The final Phase 6 command surface should be directly discoverable from `--help` without requiring roadmap context or implementation knowledge.

v1 help text must make these facts obvious:

- users must choose exactly one runtime selector: `--library <PATH>` or `--use-source-runtime`
- `--resource-dir <PATH>` is required because the runtime needs DSLogic firmware and bitstream resources
- `--output <PATH>` is the final VCD destination and must end with `.vcd`
- `--metadata-output <PATH>` is optional and overrides the derived JSON sidecar path when present
- `--format json` is the stable automation contract, while `--format text` is the concise shell-facing summary mode

Intentional v1 caveat:

- text mode is intentionally human-oriented and concise; JSON remains the only compatibility contract for scripts and agents


### 06-01: Finalize stable human-readable and machine-readable capture result output

Focus:

- lock the success/error output contract for text and JSON modes
- make text success print artifact locations and completion context
- preserve stable JSON fields and non-zero failure behavior

Primary files:

- `crates/dsview-cli/src/main.rs`
- `crates/dsview-cli/tests/capture_cli.rs`
- `crates/dsview-core/src/lib.rs` only if result shaping needs a small supporting adjustment

### 06-02: Expand output path controls for VCD and metadata artifacts without regressing automation safety

Focus:

- decide and implement the v1 path model
- support explicit artifact destination control beyond the current single VCD path
- validate invalid/conflicting path combinations early

Primary files:

- `crates/dsview-cli/src/main.rs`
- `crates/dsview-core/src/lib.rs`
- `crates/dsview-core/tests/export_artifacts.rs`
- `crates/dsview-cli/tests/capture_cli.rs`

### 06-03: Add process-boundary CLI validation and release-style workflow checks for capture export

Focus:

- move tests outward to spawned-process CLI coverage
- verify help text, output behavior, exit codes, and artifact creation
- document a final manual shell workflow UAT using connected hardware

Primary files:

- `crates/dsview-cli/tests/capture_cli.rs`
- `crates/dsview-cli/src/main.rs`
- `.planning/phases/06-cli-productization/06-VALIDATION.md`

## Major Risks and Gaps

### Requirement ambiguity around output paths

The roadmap says users can choose artifact output locations, plural.
Current implementation exposes one path and derives the sidecar path.

Risk:

- If this ambiguity is not resolved in planning, implementation can drift between “document current behavior” and “add explicit metadata path control.”

Planning response:

- Lock one v1 path model in Phase 6 research before implementation starts.

### Text mode may remain too vague if treated as an afterthought

Risk:

- A capture command that succeeds but prints only `ok` is awkward for direct shell use and pushes users toward filesystem inspection.

Planning response:

- Treat text success output as a first-class acceptance criterion in 06-01.

### Test coverage currently stops short of the real CLI boundary

Risk:

- Helper-level tests can miss regressions in clap parsing, output formatting, stdout/stderr routing, and exit-code behavior.

Planning response:

- Make spawned-process CLI tests and a final manual shell workflow UAT explicit Phase 6 proof.

### Local native prerequisites remain a standing environment dependency

Risk:

- Productization can be blocked by environment readiness even when the implementation is correct.

Planning response:

- Keep manual validation preflight-aware and distinguish environment failures from product defects.
