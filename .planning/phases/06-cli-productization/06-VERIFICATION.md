# Phase 06 Verification

**Date:** 2026-04-08
**Phase:** 06 - CLI Productization
**Goal:** Deliver a polished non-interactive capture-and-export command that works well in shell and agent workflows.
**Requirements:** CLI-01, CLI-02, CLI-03

## Verdict

**Status: Achieved / passed.**

Phase 06 now meets its goal on the main workspace. The previously open manual hardware gate has been closed by successful DSLogic Plus shell-workflow UAT on current hardware, and the automated command-surface, artifact-path, and process-boundary checks remain green.

## What was verified

- The phase goal in `.planning/ROADMAP.md` remains to deliver one polished non-interactive `capture` command suitable for shell and agent workflows.
- The requirement targets in `.planning/REQUIREMENTS.md` remain `CLI-01`, `CLI-02`, and `CLI-03`.
- The implemented product surface still matches the locked Phase 6 research and plan intent:
  - `crates/dsview-cli/src/main.rs` exposes the final `capture` surface with explicit runtime selection, output-format control, VCD output path, optional metadata override path, and distinct text vs JSON success rendering.
  - `crates/dsview-core/src/lib.rs` preserves complete-artifact semantics while shaping final VCD and metadata destinations and rejecting invalid/conflicting combinations early.
  - `crates/dsview-cli/tests/capture_cli.rs` and `crates/dsview-core/tests/export_artifacts.rs` keep the command boundary, help/discoverability, and artifact path contract locked with automated coverage.
- Phase 06 plan inventory is now fully satisfied: `06-01`, `06-02`, and `06-03` each have a matching `SUMMARY.md` and no incomplete plans remain.

## Automated evidence

Commands re-run on current HEAD:

- `cargo test -p dsview-core --test export_artifacts`
- `cargo test -p dsview-cli --test capture_cli`
- `cargo test --workspace`

Observed result:

- all suites passed
- artifact-path derivation and validation remained green
- spawned-process CLI help, stdout/stderr, and failure-shape checks remained green
- no regression surfaced across `dsview-cli`, `dsview-core`, or `dsview-sys`

## Hardware UAT evidence incorporated

Manual DSLogic Plus validation was executed on 2026-04-08 using the source runtime and resource dir `/home/seasonyuu/projects/dsview-cli/DSView/DSView/res`.

### 1. Device discovery and open path remained healthy

Commands:

- `cargo run -p dsview-cli -- devices list --use-source-runtime --resource-dir /home/seasonyuu/projects/dsview-cli/DSView/DSView/res --format json`
- `cargo run -p dsview-cli -- devices open --use-source-runtime --resource-dir /home/seasonyuu/projects/dsview-cli/DSView/DSView/res --handle 1 --format json`

Observed result:

- one supported `DSLogic Plus` device was listed with handle `1`
- the device opened and released cleanly

Why it matters:

- confirms the real-hardware environment used for final product validation was ready
- confirms the final shell workflow still starts from the intended explicit selection path

### 2. Text-mode capture succeeded non-interactively and reported both artifacts clearly

Command:

- `cargo run -p dsview-cli -- capture --use-source-runtime --resource-dir /home/seasonyuu/projects/dsview-cli/DSView/DSView/res --handle 1 --sample-rate-hz 1000000 --sample-limit 1024 --channels 0 --wait-timeout-ms 10000 --poll-interval-ms 50 --format text --output /home/seasonyuu/projects/dsview-cli/.tmp/manual-uat-phase6/text-run.vcd`

Observed result:

- command completed successfully with `capture clean_success`
- stdout reported:
  - final completion state
  - final VCD path `/home/seasonyuu/projects/dsview-cli/.tmp/manual-uat-phase6/text-run.vcd`
  - final metadata path `/home/seasonyuu/projects/dsview-cli/.tmp/manual-uat-phase6/text-run.json`
- both artifact files were created on disk

Why it matters:

- closes the human-facing shell summary gate for `CLI-03`
- proves the final command stays one-shot and non-interactive for direct shell use (`CLI-01`)

### 3. JSON-mode capture remained machine-readable and artifact paths matched reality

Command:

- `cargo run -p dsview-cli -- capture --use-source-runtime --resource-dir /home/seasonyuu/projects/dsview-cli/DSView/DSView/res --handle 1 --sample-rate-hz 1000000 --sample-limit 1024 --channels 0 --wait-timeout-ms 10000 --poll-interval-ms 50 --format json --output /home/seasonyuu/projects/dsview-cli/.tmp/manual-uat-phase6/json-run.vcd`

Observed result:

- command completed successfully with machine-readable JSON
- JSON success payload contained:
  - `completion=clean_success`
  - `saw_logic_packet=true`
  - `saw_end_packet=true`
  - `saw_terminal_normal_end=true`
  - `cleanup_succeeded=true`
  - `artifacts.vcd_path=/home/seasonyuu/projects/dsview-cli/.tmp/manual-uat-phase6/json-run.vcd`
  - `artifacts.metadata_path=/home/seasonyuu/projects/dsview-cli/.tmp/manual-uat-phase6/json-run.json`
- those exact files existed on disk
- metadata sidecar contents matched the run, including `sample_rate_hz=1000000` and `actual_sample_count=64`

Why it matters:

- closes the automation-facing success-contract gate for `CLI-01` and `CLI-03`
- confirms explicit output-path handling remains trustworthy for scripts (`CLI-02`)

### 4. VCD artifacts had sane timing semantics on the final shell workflow

Observed artifact checks:

- `text-run.vcd`, `json-run.vcd`, and `rerun.vcd` all existed on disk
- all contained `$timescale`
- all contained `#0`
- none contained `#-nan`
- none contained `#inf`
- representative header lines from `text-run.vcd` included:
  - `Acquisition with 1/1 channels at 1 MHz`
  - `$timescale 1 us $end`
  - `#0 1!`
  - `#64`

Why it matters:

- confirms the polished Phase 6 workflow still sits on top of a sane real-hardware export path
- provides an extra end-to-end check that the reported artifact paths correspond to valid produced files

### 5. Immediate rerun after success still worked

Command:

- `cargo run -p dsview-cli -- capture --use-source-runtime --resource-dir /home/seasonyuu/projects/dsview-cli/DSView/DSView/res --handle 1 --sample-rate-hz 1000000 --sample-limit 1024 --channels 0 --wait-timeout-ms 10000 --poll-interval-ms 50 --format json --output /home/seasonyuu/projects/dsview-cli/.tmp/manual-uat-phase6/rerun.vcd`

Observed result:

- immediate second capture also completed with `clean_success`
- a fresh `rerun.vcd` and `rerun.json` pair were created
- rerun metadata again reported the expected sample rate and actual sample count

Why it matters:

- closes the final device-reuse gate for `CLI-01`
- proves the polished workflow does not leave the DSLogic Plus in a restart-only state after success

## Requirement-by-requirement assessment

- `CLI-01`: **Passed**
  - one `capture` command performs the end-to-end workflow non-interactively on real hardware
  - both the initial successful run and the immediate rerun completed cleanly

- `CLI-02`: **Passed**
  - the final workflow accepts explicit output destinations
  - successful runs produced artifacts at the requested VCD paths and the expected metadata paths
  - invalid/conflicting path combinations remain covered by green automated tests

- `CLI-03`: **Passed**
  - text mode clearly reported completion plus final VCD and metadata paths
  - JSON mode kept a machine-readable success payload that included both artifact paths

## Final decision

**Mark Phase 06 complete.**

The remaining blocker was the manual DSLogic Plus shell-workflow UAT. That blocker is now resolved, and the automated plus hardware evidence together show that the final `capture` command satisfies the Phase 6 goal.

## Residual non-blocking risk

- The hardware proof is strong for the validated bounded workflow, but it remains a sampled real-device session rather than exhaustive coverage of every native/runtime edge case.
- The current main worktree still contains uncommitted bookkeeping and implementation-layer changes; those should be reviewed and committed deliberately, but they do not block the Phase 06 acceptance decision itself.
