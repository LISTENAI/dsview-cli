---
phase: 06-cli-productization
plan: 03
subsystem: test
tags: [tests, validation, cli, process, help, exit-codes, shell, dslogic-plus]
requires:
  - phase: 06-01
    provides: final text/json output contract and capture success rendering
  - phase: 06-02
    provides: final path controls and path-validation contract
provides:
  - Spawned-process CLI validation for help text, stdout/stderr behavior, exit codes, and representative failure paths
  - Updated research and validation artifacts that lock the final discoverability contract
  - Explicit record that only the manual DSLogic Plus shell-workflow UAT remains open for Phase 6 closeout
affects: [phase-06, cli, process-boundary, help, validation]
tech-stack:
  added: [assert_cmd, predicates]
  patterns: [spawned-process-cli-tests, help-contract-locking, stdout-stderr-separation]
key-files:
  created:
    - .planning/phases/06-cli-productization/06-03-SUMMARY.md
  modified:
    - crates/dsview-cli/Cargo.toml
    - Cargo.lock
    - crates/dsview-cli/tests/capture_cli.rs
    - crates/dsview-cli/src/main.rs
    - .planning/phases/06-cli-productization/06-RESEARCH.md
    - .planning/phases/06-cli-productization/06-VALIDATION.md
    - .planning/ROADMAP.md
    - .planning/STATE.md
key-decisions:
  - "Use spawned-process assertions for CLI proof instead of relying only on helper-level tests."
  - "Lock key help fragments so runtime selection, output-path controls, and JSON-vs-text behavior stay discoverable from `capture --help`."
  - "Treat automated Phase 6 validation as complete while keeping the final DSLogic Plus shell-workflow UAT as an explicit open gate."
patterns-established:
  - "CLI Boundary Proof Pattern: validate exit status, stdout/stderr routing, and help text through the compiled binary."
  - "Discoverability Contract Pattern: key usage/help fragments are test-locked so product surface drift is caught automatically."
requirements-completed: [CLI-01, CLI-02, CLI-03]
duration: TO_BE_FILLED
completed: 2026-04-08
verification:
  - cargo test -p dsview-cli --test capture_cli
  - cargo test -p dsview-cli
  - cargo test --workspace
  - cargo run -p dsview-cli -- capture --help
---

# Phase 06 Plan 03 Summary

**Phase 06-03 automated validation is complete: the compiled CLI now has spawned-process coverage for help text, output-path validation failures, stdout/stderr routing, and representative machine-readable error behavior. The only remaining Phase 6 gate is the manual DSLogic Plus shell-workflow UAT.**

## Accomplishments
- Added `assert_cmd` and `predicates` to `crates/dsview-cli/Cargo.toml` so CLI tests can exercise the compiled binary directly.
- Expanded `crates/dsview-cli/tests/capture_cli.rs` with spawned-process checks for `capture --help`, missing runtime selector failures, invalid VCD and metadata destinations in text mode, and conflicting artifact-path failures before runtime work begins.
- Improved `crates/dsview-cli/src/main.rs` help text so runtime selection, resource requirements, output-format behavior, and artifact-path controls are directly discoverable from the CLI.
- Updated `.planning/phases/06-cli-productization/06-RESEARCH.md`, `.planning/phases/06-cli-productization/06-VALIDATION.md`, `.planning/ROADMAP.md`, and `.planning/STATE.md` so the checked-in planning artifacts reflect that automated Phase 6 proof is green and only the real-hardware shell workflow remains open.

## Verification Performed
- `cargo test -p dsview-cli --test capture_cli`
- `cargo test -p dsview-cli`
- `cargo test --workspace`
- `cargo run -p dsview-cli -- capture --help`

## Files Created/Modified
- `.planning/phases/06-cli-productization/06-03-SUMMARY.md` - records automated 06-03 completion and the remaining manual gate
- `crates/dsview-cli/Cargo.toml` - adds spawned-process CLI test dependencies
- `Cargo.lock` - updates lockfile for new dev dependencies
- `crates/dsview-cli/tests/capture_cli.rs` - adds binary-boundary help and failure-path assertions
- `crates/dsview-cli/src/main.rs` - documents final command-surface discoverability in clap help text
- `.planning/phases/06-cli-productization/06-RESEARCH.md` - records the intentional help/discoverability caveats for v1
- `.planning/phases/06-cli-productization/06-VALIDATION.md` - marks automated Phase 6 checks green and keeps the manual UAT open
- `.planning/ROADMAP.md` and `.planning/STATE.md` - advance milestone bookkeeping to Phase 6 with 2/3 plans complete

## Remaining Gate
- Manual DSLogic Plus shell-workflow UAT is still required. Phase 6 should not be marked complete until a real-hardware run confirms the final `capture` command remains non-interactive, writes both artifacts to the requested or derived paths, reports them clearly, and supports immediate rerun reuse.

---
*Phase: 06-cli-productization*
*Completed: 2026-04-08*
