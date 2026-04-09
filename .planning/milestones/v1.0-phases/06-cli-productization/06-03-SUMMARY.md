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
  - Explicit historical record that 06-03 completed automated validation first and the later 2026-04-08 hardware closeout passed the remaining shell-workflow gate
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
  - "Record 06-03 as the automated closeout step, then point to the later passed manual DSLogic Plus shell-workflow UAT instead of leaving the gate described as still open."
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

**Phase 06-03 completed the automated closeout work for the final capture-to-export command surface, and Phase 6 is now complete because the later 2026-04-08 DSLogic Plus manual shell-workflow UAT also passed.**

## Accomplishments
- Added `assert_cmd` and `predicates` to `crates/dsview-cli/Cargo.toml` so CLI tests can exercise the compiled binary directly.
- Expanded `crates/dsview-cli/tests/capture_cli.rs` with spawned-process checks for `capture --help`, missing runtime selector failures, invalid VCD and metadata destinations in text mode, and conflicting artifact-path failures before runtime work begins.
- Improved `crates/dsview-cli/src/main.rs` help text so runtime selection, resource requirements, output-format behavior, and artifact-path controls are directly discoverable from the CLI.
- Updated `.planning/phases/06-cli-productization/06-RESEARCH.md`, `.planning/phases/06-cli-productization/06-VALIDATION.md`, `.planning/ROADMAP.md`, and `.planning/STATE.md` so the checked-in planning artifacts reflected automated Phase 6 proof at execution time; later closeout evidence in `.planning/phases/06-cli-productization/06-VALIDATION.md` and `.planning/phases/06-cli-productization/06-VERIFICATION.md` confirmed the manual DSLogic Plus shell-workflow UAT passed on 2026-04-08.

## Verification Performed
- `cargo test -p dsview-cli --test capture_cli`
- `cargo test -p dsview-cli`
- `cargo test --workspace`
- `cargo run -p dsview-cli -- capture --help`

## Files Created/Modified
- `.planning/phases/06-cli-productization/06-03-SUMMARY.md` - records automated 06-03 completion and the later passed manual closeout evidence
- `crates/dsview-cli/Cargo.toml` - adds spawned-process CLI test dependencies
- `Cargo.lock` - updates lockfile for new dev dependencies
- `crates/dsview-cli/tests/capture_cli.rs` - adds binary-boundary help and failure-path assertions
- `crates/dsview-cli/src/main.rs` - documents final command-surface discoverability in clap help text
- `.planning/phases/06-cli-productization/06-RESEARCH.md` - records the intentional help/discoverability caveats for v1
- `.planning/phases/06-cli-productization/06-VALIDATION.md` - now also serves as the passed manual UAT record for final closeout
- `.planning/ROADMAP.md` and `.planning/STATE.md` - captured execution-time bookkeeping while Phase 6 closeout was still in progress

## Closeout Note
- 06-03 completed the automated validation and discoverability work during execution time.
- The later 2026-04-08 real-hardware shell-workflow UAT closed the last gate, so Phase 6 is complete and the final `capture` command is verified as non-interactive, artifact-reporting, and immediately reusable on the connected DSLogic Plus.

---
*Phase: 06-cli-productization*
*Completed: 2026-04-08*
