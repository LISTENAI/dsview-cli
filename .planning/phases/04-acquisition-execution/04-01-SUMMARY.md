---
phase: 04-acquisition-execution
plan: 01
subsystem: core
tags: [core, sys, cli, acquisition, callbacks, dslogic-plus, preflight]
requires: [03-03]
provides:
  - DSView acquisition lifecycle bindings and callback aggregation through the sys bridge
  - Rust-side capture orchestration with preflight, config apply, wait, cleanup, and clean-success enforcement
  - Minimal `capture` CLI command with stable machine-readable success and failure diagnostics
affects: [core, native-integration, cli, validation]
tech-stack:
  added: [DSView acquisition callback bridge, finite-capture orchestration]
  patterns: [single-run acquisition contract, callback-summary bridge, preflight-gated hardware execution]
key-files:
  created: []
  modified:
    - crates/dsview-sys/wrapper.h
    - crates/dsview-sys/bridge_runtime.c
    - crates/dsview-sys/src/lib.rs
    - crates/dsview-sys/build.rs
    - crates/dsview-core/src/lib.rs
    - crates/dsview-core/src/capture_config.rs
    - crates/dsview-cli/src/main.rs
key-decisions:
  - "Define clean finite-capture success as normal terminal end plus logic packet plus end packet plus successful cleanup."
  - "Keep DSView callback registration and packet inspection inside dsview-sys and expose only Rust-safe acquisition summaries upward."
  - "Gate hardware-backed capture execution behind a narrow preflight that checks runtime/resource/open/config readiness before starting collection."
patterns-established:
  - "Acquisition Pattern: dsview-core owns open -> validate/apply -> callback register -> start -> wait -> cleanup orchestration for one finite run."
  - "CLI Pattern: capture results and failures are rendered as stable machine-readable completion and error codes instead of exposing raw DSView lifecycle details directly."
requirements-completed: [RUN-01, RUN-02, RUN-03]
duration: 59 min
completed: 2026-04-07
---

# Phase 04 Plan 01: Resolve finite-capture success signal and implement capture start/run/finish orchestration in the Rust service layer Summary

**Added the first real DSLogic Plus capture command by wiring DSView acquisition callbacks through the sys/core/cli seam and enforcing a finite-run clean-success contract**

## Performance

- **Duration:** 59 min
- **Started:** 2026-04-07T04:54:39Z
- **Completed:** 2026-04-07T05:53:54Z
- **Tasks:** 5
- **Files modified:** 7

## Accomplishments
- Extended `dsview-sys` with dynamic acquisition bindings for callback registration, start/stop, collection-state polling, and Rust-safe acquisition summaries.
- Added `dsview-core` capture orchestration with preflight checks, validated config apply, callback lifecycle handling, finite-run polling, cleanup, and clean-success classification.
- Added a minimal `capture` CLI command that accepts the stable selector plus Phase 3 config inputs and returns structured completion/error output.
- Verified one real hardware finite capture on the current machine with `clean_success`, observed logic data, observed end packet, and successful cleanup.

## Task Commits

Each task was committed atomically as completed during closeout:

1. **Task 1-5: Plan 04-01 implementation and validation closeout** - `d84d880` (feat)
2. **Plan metadata:** `[pending]` (docs)

## Files Created/Modified
- `crates/dsview-sys/wrapper.h` - Declares the acquisition bridge ABI and callback summary structs.
- `crates/dsview-sys/bridge_runtime.c` - Loads DSView acquisition symbols and aggregates callback events into a repo-owned summary.
- `crates/dsview-sys/src/lib.rs` - Exposes Rust-safe acquisition lifecycle types, summary polling, and start/stop helpers.
- `crates/dsview-sys/build.rs` - Keeps the source-backed runtime build aligned with the expanded acquisition bridge.
- `crates/dsview-core/src/lib.rs` - Implements acquisition preflight, session preparation, finite-run waiting, cleanup, and completion classification.
- `crates/dsview-core/src/capture_config.rs` - Tightens capability validation to respect the active channel mode during acquisition setup.
- `crates/dsview-cli/src/main.rs` - Adds the `capture` command and stable machine-readable capture diagnostics.

## Decisions Made
- Success for Phase 4 run execution is enforced in core from callback-derived summary state, not from `ds_start_collect()` alone.
- Hardware-backed capture validation uses the current active channel mode's supported sample rates; the earlier 100 MHz smoke command was invalid for the machine's 16-channel 20 MHz mode.
- Preflight remains intentionally narrow so acquisition failures can be distinguished from local runtime or permission readiness problems.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Corrected active-device capability sequencing during capture preparation**
- **Found during:** Task 2 (Add a local hardware and runtime preflight gate for Phase 4 execution)
- **Issue:** capture preflight originally failed with `ds_get_current_samplerate` call-status errors before the device was in the correct active state.
- **Fix:** Reworked capture preparation to open the selected device, read capability state through the active device session, and validate/apply config in that explicit ordering.
- **Files modified:** `crates/dsview-core/src/lib.rs`, `crates/dsview-core/src/capture_config.rs`
- **Verification:** `cargo test -p dsview-core`; real `capture` command advanced past the old sequencing failure and completed successfully with a valid rate.
- **Committed in:** `d84d880`

**2. [Rule 3 - Blocking] Adjusted hardware smoke validation to use the active mode's supported sample rate**
- **Found during:** Task 5 (Add single-run acquisition orchestration and a minimal CLI command)
- **Issue:** the initial 100 MHz smoke invocation was invalid for the current hardware mode `Use 16 Channels (Max 20MHz)`, so hardware validation could not prove the run path.
- **Fix:** Re-ran the manual finite-capture smoke check with `--sample-rate-hz 20000000`, which matches the current active mode and exercises the intended run path truthfully.
- **Files modified:** None - validation command only.
- **Verification:** real `cargo run -p dsview-cli -- capture ... --sample-rate-hz 20000000 ... --format json` returned `clean_success` with logic packet, end packet, normal terminal end, and cleanup success.
- **Committed in:** `d84d880`

---

**Total deviations:** 2 auto-fixed (1 bug, 1 blocking)
**Impact on plan:** Both fixes were necessary to make the happy-path acquisition proof truthful on real hardware. No scope creep beyond the planned Phase 4 execution seam.

## Issues Encountered
- The source-runtime capture path initially failed on capability reads until the active-device sequencing was made explicit in core.
- Real hardware validation exposed that the machine's current DSLogic Plus mode only supported 20 MHz, so the first 100 MHz smoke command was an invalid config test rather than an acquisition-lifecycle failure.

## User Setup Required
- None beyond the existing source-runtime prerequisites and USB permission setup already documented for this machine.

## Next Phase Readiness
- Phase 04-02 can now build on a working happy-path acquisition seam and focus on timeout, detach, cleanup precedence, and stable failure classification.
- Hardware-backed proof still needs the broader failure-path and post-error reuse coverage planned for 04-02 and 04-03.
- The metadata docs commit still needs to be created during plan closeout.

---
*Phase: 04-acquisition-execution*
*Completed: 2026-04-07*
