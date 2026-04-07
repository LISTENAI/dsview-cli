# Phase 04 Verification

**Date:** 2026-04-07
**Phase:** 04 - Acquisition Execution
**Goal:** Execute logic captures reliably from the CLI while managing device/session lifecycle correctly.
**Requirements:** RUN-01, RUN-02, RUN-03

## Verdict

**Status: Achieved / passed.**

Phase 04 now meets its goal on the main workspace after successful manual DSLogic Plus hardware UAT closed the remaining verifier gates defined in `.planning/phases/04-acquisition-execution/04-RESEARCH.md`.

The prior verifier decision was correct that implementation and automated coverage were already strong but blocked on real-hardware proof. That blocker is now resolved by successful source-runtime validation using `./DSView/DSView/res` and a connected `DSLogic Plus`.

## What was re-verified

- The phase goal in `.planning/ROADMAP.md` remains: execute real captures from the CLI, finish cleanly, and fail with actionable non-zero diagnostics.
- The requirement targets in `.planning/REQUIREMENTS.md` remain `RUN-01`, `RUN-02`, and `RUN-03`.
- The locked Phase 4 research gates in `.planning/phases/04-acquisition-execution/04-RESEARCH.md` still require:
  - at least one natural finite capture proving the strict `clean_success` rule on hardware
  - immediate reuse after success
  - one representative failure with actionable non-zero diagnostics
  - immediate reuse after that failure
- The implementation still matches those gates:
  - `crates/dsview-core/src/lib.rs` performs preflight -> open -> validate/apply -> callback register -> start -> bounded wait -> stop/release cleanup.
  - `crates/dsview-core/src/lib.rs` only treats a run as success when the summary reaches the strict finite-capture rule and cleanup succeeds.
  - `crates/dsview-cli/src/main.rs` maps failure classes to stable machine-readable codes including `capture_start_failed`, `capture_run_failed`, `capture_detached`, `capture_incomplete`, `capture_timeout`, and `capture_cleanup_failed`.
  - `crates/dsview-core/tests/acquisition.rs` preserves the expected Phase 4 classification and cleanup contract in automated coverage.

## Hardware UAT evidence incorporated

Manual hardware UAT was reported as successful in the main workspace with source runtime and resource dir `./DSView/DSView/res`.

### 1. Device discovery and targeted open succeeded

Command:

- `devices list --use-source-runtime --resource-dir ./DSView/DSView/res --format json`

Observed result:

- one supported device found
- handle `1`
- `stable_id=dslogic-plus`

Command:

- `devices open --use-source-runtime --resource-dir ./DSView/DSView/res --handle 1 --format json`

Observed result:

- open succeeded
- device released cleanly

Why it matters:

- Confirms the hardware/runtime environment used for acquisition validation was actually ready.
- Confirms the verifier exercised the supported-device selection path the acquisition command depends on.

### 2. Real finite capture reached strict clean success

Command:

- `capture --use-source-runtime --resource-dir ./DSView/DSView/res --handle 1 --sample-rate-hz 20000000 --sample-limit 100000 --channels 0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15 --wait-timeout-ms 10000 --poll-interval-ms 50 --format json`

Observed result:

- `completion=clean_success`
- `saw_logic_packet=true`
- `saw_end_packet=true`
- `saw_terminal_normal_end=true`
- `cleanup_succeeded=true`

Why it matters:

- Closes the manual gate for natural finite completion.
- Matches the locked Phase 4 success rule from `.planning/phases/04-acquisition-execution/04-RESEARCH.md`:
  - start succeeded
  - normal terminal end observed
  - logic packet observed
  - end marker observed
  - cleanup succeeded

### 3. Immediate rerun after success also succeeded

Observed result:

- the same finite capture command succeeded again immediately

Why it matters:

- Provides the required hardware proof that a successful run leaves the device reusable.
- Closes the success-side reusability portion of `RUN-02`.

### 4. Representative failure returned non-zero timeout diagnostics and cleaned up correctly

Command:

- `capture --use-source-runtime --resource-dir ./DSView/DSView/res --handle 1 --sample-rate-hz 20000000 --sample-limit 1048576 --channels 0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15 --wait-timeout-ms 1 --poll-interval-ms 1 --format json`

Observed result:

- command exited non-zero
- `code=capture_timeout`
- cleanup reported:
  - `stop_attempted=true`
  - `stop_succeeded=true`
  - `callbacks_cleared=true`
  - `release_succeeded=true`

Why it matters:

- Closes the manual representative failure gate for `RUN-03`.
- Confirms the bounded-wait policy is not just implemented in software but works on real hardware.
- Confirms timeout failure still drives deterministic stop/callback-clear/release cleanup.

### 5. Immediate reuse after failure succeeded

Command:

- `devices open --use-source-runtime --resource-dir ./DSView/DSView/res --handle 1 --format json`

Observed result:

- open succeeded
- device released cleanly

Why it matters:

- Closes the post-failure hardware reusability gate explicitly called out in `.planning/phases/04-acquisition-execution/04-RESEARCH.md`.
- Proves the representative non-success path does not leave the DSLogic Plus in broken session state.

## Requirement-by-requirement assessment

- `RUN-01`: **Passed**
  - A real CLI capture was started and reached natural hardware completion on `DSLogic Plus`.
  - The successful run included observed logic data and end-of-stream evidence, satisfying the stricter Phase 4 success definition.

- `RUN-02`: **Passed**
  - Success-path cleanup completed cleanly on hardware.
  - Immediate rerun after success succeeded.
  - Representative failure cleanup also completed cleanly, and immediate reopen after failure succeeded, proving session/device reusability in both directions that matter for this phase.

- `RUN-03`: **Passed**
  - A representative hardware failure returned a stable non-zero diagnostic code: `capture_timeout`.
  - Cleanup diagnostics were actionable and showed stop, callback clear, and release outcomes explicitly.

## Final decision

**Mark Phase 04 complete.**

The previous blocker was manual hardware proof, not missing implementation. The newly supplied UAT results close the exact verifier gates defined by the phase research artifact, so the phase goal is now achieved.

## Residual non-blocking risk

- Hardware UAT evidence is strong for the validated paths, but it is still a bounded sample rather than exhaustive proof of every DSView/native failure mode.
- Detach and terminal runtime-error behavior remain primarily covered by synthetic tests rather than this specific hardware session.
- Planning/status artifacts outside this verification file may still be stale, for example `.planning/ROADMAP.md` still shows Phase 4 as incomplete; that is now an artifact-update issue, not a Phase 04 acceptance blocker.
