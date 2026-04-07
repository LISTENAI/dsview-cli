# Phase 4 Research: Acquisition Execution

**Date:** 2026-04-07
**Phase:** 4 - Acquisition Execution
**Goal:** Execute logic captures reliably from the CLI while managing device/session lifecycle correctly.

## Goal Fit

Phase 4 begins after:

- Phase 2 proved runtime init/exit, supported-device discovery, and safe open/release bring-up.
- Phase 3 proved capability loading, request validation, and config modeling for an opened active device.
- Real hardware `devices list` -> `devices open --handle 1` -> release has now succeeded on this machine after the local udev/USB permission fix.
- Phase 3 manual UAT remains partial only because the CLI does not yet expose standalone capability/config verification commands, not because open/release is still blocked.

This phase must add the first real acquisition lifecycle for `DSLogic Plus` and satisfy:

- `RUN-01`: start a real logic capture from the CLI
- `RUN-02`: complete the capture and close the device session cleanly on success
- `RUN-03`: fail with non-zero exit status and actionable diagnostics on acquisition failure

This phase should still stop short of full export work. It may need to collect enough run metadata or packet facts to know whether acquisition really succeeded, but VCD/metadata artifact generation remains Phase 5 scope.

## What Already Exists In The Repo

### 1. The current Rust seam already matches the needed run ordering

Current layering is already close to the correct execution pipeline:

1. `dsview-cli` parses command input and renders stable machine-readable errors.
2. `dsview-core::Discovery` owns runtime init/exit, supported-device filtering, open/release, capability reads, and validated config apply.
3. `dsview-sys` owns the dynamic bridge to DSView's `ds_*` frontend facade.

The existing core flow is:

- connect runtime
- list/open supported device
- validate requested config against capability snapshot
- apply validated config to the opened active device
- release device via `OpenedDevice` drop or explicit `release()`

Phase 4 should extend this same seam instead of inventing a second session model.

### 2. The current sys bridge does not expose acquisition yet

`crates/dsview-sys/bridge_runtime.c` already loads and wraps:

- `ds_lib_init` / `ds_lib_exit`
- `ds_get_device_list`
- `ds_active_device` / `ds_release_actived_device`
- `ds_get_last_error`
- active-device config reads and writes
- channel enable operations

It does **not** yet expose the capture-specific APIs needed for Phase 4:

- `ds_set_datafeed_callback`
- `ds_set_event_callback`
- `ds_start_collect`
- `ds_stop_collect`
- `ds_is_collecting`

That makes the bridge shim the natural implementation point for Phase 4 planning.

### 3. The CLI currently has no capture command surface

`crates/dsview-cli/src/main.rs` only exposes:

- `devices list`
- `devices open`

Phase 4 therefore needs at least one CLI entry point for a real run, but planning should keep it minimal and machine-oriented. The command can stay temporary or narrow as long as it proves the lifecycle cleanly.

## Confirmed Upstream Acquisition Facts

### 1. DSView already has a public acquisition facade suitable for the bridge

`DSView/libsigrok4DSL/libsigrok.h` exposes the exact public lifecycle APIs Phase 4 needs:

- `ds_set_event_callback(dslib_event_callback_t cb)`
- `ds_set_datafeed_callback(ds_datafeed_callback_t cb)`
- `ds_start_collect()`
- `ds_stop_collect()`
- `ds_is_collecting()`
- `ds_release_actived_device()`

This is important because it means Phase 4 can stay aligned with the earlier project rule:

- keep `DSView/` read-only
- extend the repo-owned runtime bridge
- avoid binding Rust directly to deeper private `sr_*` internals

### 2. Starting acquisition creates a fresh session and requires callbacks up front

In `DSView/libsigrok4DSL/lib_main.c`, `ds_start_collect()`:

- rejects starting if collection is already active
- rejects starting if no active device exists
- rejects starting while the device is still initializing
- rejects starting if no channels are enabled
- rejects starting if no datafeed callback has been registered
- creates a fresh `sr_session_new()`
- ensures the active device is opened
- spawns a collection thread

Planning implication:

- Phase 4 must register callbacks before starting capture.
- A capture attempt is not just "run on the currently open handle"; it creates a DSView session internally.
- The Rust side should treat capture as a one-shot run operation with explicit setup and teardown rather than as a reusable streaming iterator for this phase.

### 3. The actual acquisition work runs on a DSView thread and blocks in `sr_session_run()`

`collect_run_proc()` in `lib_main.c` does this:

- emits `DS_EV_COLLECT_TASK_START`
- calls device-driver `dev_acquisition_start()`
- emits `DS_EV_DEVICE_RUNNING`
- calls `sr_session_run()`
- emits `DS_EV_DEVICE_STOPPED`
- emits one of:
  - `DS_EV_COLLECT_TASK_END`
  - `DS_EV_COLLECT_TASK_END_BY_DETACHED`
  - `DS_EV_COLLECT_TASK_END_BY_ERROR`

Planning implication:

- The Rust side should not assume `ds_start_collect()` means capture has completed; it only means the worker thread started successfully.
- Success/failure completion must be derived from event/callback state, not only from the immediate return code of `ds_start_collect()`.

### 4. Stop is asynchronous at the session layer and synchronous at the DSView facade

`sr_session_stop()` only marks `abort_session = TRUE` when a session is running.

`ds_stop_collect()` then:

- calls `sr_session_stop()`
- joins the collect thread if present
- returns after the worker actually ends

Planning implication:

- The public DSView facade already gives a synchronous stop primitive that is suitable for a CLI.
- Cleanup logic should prefer `ds_stop_collect()` over trying to model private session-stop behavior in Rust.
- On cancellation/error cleanup, the Rust side must expect stop to block until the collection thread finishes.

### 5. Releasing the active device also stops collection and destroys the session

`ds_release_actived_device()`:

- checks whether an active device exists
- stops collection if one is still running
- closes the device instance
- destroys the current session with `sr_session_destroy()`

Planning implication:

- Release is the hard cleanup boundary.
- Phase 4 can keep cleanup simple if it preserves the invariant: every opened device is either explicitly released or dropped through an RAII wrapper that reaches release.
- A failed capture should still end by attempting release so the device is reusable for the next run.

### 6. DSView itself treats event completion and packet completeness separately

In the GUI session layer (`DSView/DSView/pv/sigsession.cpp`), DSView reacts to:

- `DS_EV_DEVICE_STOPPED` by expecting end-of-data markers to have been seen
- `DS_EV_COLLECT_TASK_END*` by treating missing `last_ended()` data as an error signal

Planning implication:

- A successful Phase 4 run should not be defined only as "start succeeded and stop event arrived".
- The plan should include some notion of packet completeness or at least end-of-stream observation in the Rust-side callback aggregation.
- Otherwise the CLI may report success on truncated or broken captures.

## Locked Planning Decisions

### 1. Final Phase 4 clean finite-capture success rule

For planning and verification purposes, a Phase 4 finite capture counts as `clean_success` only when **all** of the following are true:

- `ds_start_collect()` returns success.
- The terminal event observed by the callback bridge is `DS_EV_COLLECT_TASK_END`.
- No terminal `DS_EV_COLLECT_TASK_END_BY_ERROR` or `DS_EV_COLLECT_TASK_END_BY_DETACHED` event is observed for that run.
- At least one logic data packet is observed during the run.
- An end-of-stream or equivalent data-end marker is observed before the run is considered complete.
- Cleanup succeeds: collection is no longer active, release succeeds, and the run does not end in an uncertain cleanup state.

This is the Phase 4 planning decision, not a suggestion. A run that reaches normal terminal event but has zero logic packets, missing end-of-stream, or failed cleanup is classified as `incomplete` or `cleanup_failure`, not success.

### 2. Logic-packet presence is mandatory for finite-capture success

For Phase 4, a finite capture that never yields a logic packet does **not** count as success even if DSView emits a normal end event.

Reasoning:

- Phase 4 exists to prove real acquisition, not only session start/stop choreography.
- A zero-packet run cannot demonstrate that the CLI captured usable logic data.
- Treating zero-packet completion as success would make `RUN-01` and `RUN-02` too weak to support Phase 5 export work.

If later upstream evidence proves a legitimate zero-payload success mode for `DSLogic Plus`, that can be revisited in a later research update. It is not the planning rule for Phase 4.

### 3. Natural finite completion is a bounded execution assumption with a validation gate

Phase 4 will plan around this bounded assumption:

- A `DSLogic Plus` capture started with a valid finite sample limit and healthy device/runtime state is expected to end naturally without an explicit stop request.

Phase 4 will **not** treat that assumption as already proven. Instead it becomes an explicit validation gate:

- Manual hardware UAT must demonstrate at least one known-valid finite capture that reaches `DS_EV_COLLECT_TASK_END` before the timeout path fires and without the CLI needing to call `ds_stop_collect()` as the primary completion path.
- Until that manual proof exists, natural-finish behavior remains an execution assumption guarded by timeout/cleanup handling, not a fact claimed by the planning set.

Planning consequence:

- 04-01 may implement the happy-path wait for natural finish.
- 04-02 must still add bounded wait/timeout cleanup so the CLI cannot hang indefinitely if natural finish never arrives.

### 4. Post-error device reusability is a bounded requirement with a concrete verifier gate

Phase 4 will plan around this bounded expectation:

- If an acquisition attempt fails but stop/release complete successfully, the device should be reusable for a subsequent open or rerun in the same machine environment.

Phase 4 will **not** claim that every DSView/native failure mode is automatically recoverable. Instead the plan set locks this verifier gate:

- At least one failure-path validation must show immediate post-failure reuse on the same machine.
- The preferred proof is a manual hardware sequence: induce one safe failure class, confirm non-zero diagnostics, then immediately reopen the device or rerun acquisition successfully.
- Synthetic tests may prove cleanup ordering and error mapping, but they do not replace the manual reuse check for the real hardware path.

If the verifier cannot demonstrate immediate reuse after a representative failure, Phase 4 cannot claim `RUN-02` and `RUN-03` are fully satisfied on hardware even if synthetic tests pass.

### 5. Single-active-run process model is the v1 execution contract

Because DSView callback registration is global-process state, Phase 4 assumes:

- one DSView runtime owner per process
- one active acquisition per process
- no concurrent capture runs in one CLI invocation

This is a final v1 planning assumption and keeps the callback bridge and cleanup model tractable.

## Planning Implications By Layer

### `dsview-sys`

Phase 4 likely needs the following additions in the narrow runtime bridge:

- dynamic loading for:
  - `ds_set_event_callback`
  - `ds_set_datafeed_callback`
  - `ds_start_collect`
  - `ds_stop_collect`
  - `ds_is_collecting`
- a repo-owned C-side callback adapter that can translate DSView callbacks into a Rust-safe polling/result model
- explicit capture result structs that avoid leaking raw `sr_datafeed_packet` lifetimes into `dsview-core`

The most important planning choice is the callback shape.

Recommended direction for Phase 4 planning:

- keep raw packet parsing and callback registration inside `dsview-sys`
- expose a minimal Rust-safe acquisition summary rather than a general live streaming API

For this phase, the summary needs enough information to answer:

- did start succeed?
- which terminal event occurred?
- did a normal end event occur without competing error/detach terminal events?
- was at least one logic packet received?
- was an end-of-stream packet observed?
- did timeout/forced stop occur?
- what native last-error code was left by DSView?
- did stop/release succeed cleanly?

That is enough to satisfy acquisition lifecycle planning without pulling full export concerns into this phase.

### `dsview-core`

`dsview-core` should own the orchestration contract, not the callback mechanics.

Recommended core responsibilities:

- define a run request type that combines:
  - selected device handle
  - validated capture config
  - any run-time options needed for this phase only
- expose a single high-level acquisition method that performs:
  - connect runtime
  - open device
  - load capabilities if needed for revalidation
  - apply validated config
  - start capture
  - wait for completion summary
  - stop if needed on failure/timeouts
  - release device in all paths
- normalize native/event failures into domain errors with stable categories

Recommended core result shape for planning:

- a success result for lifecycle completion only
- a failure result with:
  - stable error code/category
  - human-actionable message
  - native error code name if available
  - event-based reason if available
  - cleanup status

The core layer is also the right place to define cleanup order guarantees and the timeout fallback for non-terminating runs.

### `dsview-cli`

The CLI should remain thin.

Recommended Phase 4 CLI behavior:

- add one run-oriented command for a real acquisition proof
- preserve JSON and text output modes
- return exit code `0` only when the acquisition summary matches the locked `clean_success` rule
- return non-zero on:
  - preflight-blocked execution
  - start failure
  - callback/event error end
  - detach during run
  - incomplete data/end markers
  - timeout or forced-stop path
  - cleanup failure if cleanup leaves the session/device in uncertain state

CLI diagnostics should include the same stable machine-readable style already used in Phase 2.

## Error And Cleanup Concerns That Matter For Planning

### 1. Start can fail after config has already been applied

By Phase 4 the device may already be:

- opened
- configured
- callback-wired

If `ds_start_collect()` fails, the plan must still release the device cleanly.

### 2. Completion-by-error and completion-by-detach need different diagnostics

DSView distinguishes:

- normal end
- end by error
- end by detached device

These should map to distinct core/CLI error categories because the remediation differs:

- general capture failure
- USB/device disconnect during run
- permission/transport/device-state issues

### 3. Native `last_error` is necessary but not sufficient

The existing bridge already exposes `ds_get_last_error()`. That should remain part of the diagnostics, but planning should not rely on it alone because:

- some failures are surfaced via event outcome rather than immediate call failure
- packet incompleteness may indicate failure even when `last_error` is not descriptive

### 4. Cleanup may itself fail or partially succeed

Potential cleanup stages:

- stop collection if still running
- release active device
- runtime exit on `Discovery` drop

Planning should define precedence for reporting:

- if capture fails and release also fails, report capture failure first but include cleanup failure context
- if capture appears successful but release fails, treat the command as failed because `RUN-02` requires the device to remain reusable

### 5. Local environment readiness is no longer blocked on open/release, but still matters for acquisition validation

Current machine reality:

- Stable selector `devices list` -> `devices open --handle 1` -> release has passed after the udev fix.
- That removes the earlier open/release blocker for this machine.
- Manual capability/config verification is still only partially evidenced because the CLI lacks standalone verification commands.
- Acquisition validation still depends on the local USB permission fix staying intact and on the source-runtime resource path remaining correct.

Planning implication:

- manual hardware validation for Phase 4 should start from a narrow preflight gate rather than from a generic USB-blocked assumption
- failures during acquisition must distinguish environment-not-ready from implementation failure

## Validation And Manual Hardware Checks That Matter

### Automated coverage that should be planned

Phase 4 can still keep most automated tests hardware-free if the core contract is shaped correctly.

Recommended automated coverage split:

- `dsview-core` unit tests for run-state/error classification
- tests for cleanup precedence and result mapping
- tests for incomplete-run vs clean-run interpretation using synthetic acquisition summaries
- tests for timeout handling and post-timeout cleanup mapping
- any narrow `dsview-sys` tests that validate bridge argument handling and callback registration state without real hardware

Do **not** over-promise full acquisition correctness from no-hardware tests alone.

### Manual hardware checks that should be planned

A real `DSLogic Plus` validation pass should cover at least:

1. confirm preflight still passes on this machine: source runtime resources resolve, the stable selector opens the device, and release works cleanly
2. apply one known-valid Phase 3 configuration through the Phase 4 run path
3. start a real finite acquisition from the CLI
4. confirm the command returns success only after natural completion and only when the locked `clean_success` rule is met
5. confirm the device can be opened again immediately after the successful run
6. induce at least one safe failure path and confirm non-zero exit with actionable diagnostics
7. verify cleanup after that failure still leaves the device reusable

Important manual cases to include:

- normal finite logic capture
- rerun immediately after success
- rerun or reopen immediately after one representative failure
- disconnect during capture only if it is safe and practical on this machine

### Verifier-ready coverage matrix

| Case | Planned proof method | Verifier gate |
|------|----------------------|---------------|
| `clean_success` | Manual hardware finite capture plus synthetic result-mapping tests | Must observe normal terminal event, logic packet, end marker, and successful release |
| `preflight_blocked` | Environment-gated local checks plus CLI/core tests | Must distinguish environment not ready from acquisition failure |
| `start_failure` | Synthetic bridge/core outcome, optionally environment-gated local refusal | Must return non-zero and still attempt release |
| `run_failure` | Synthetic terminal error-event mapping plus manual hardware case if reproducible | Must preserve actionable diagnostics |
| `detach` | Synthetic event-path test, manual unplug only if safe | Must map to distinct detach outcome |
| `incomplete` | Synthetic summaries missing logic packet or end marker | Must fail even if normal terminal event occurs |
| `timeout` | Synthetic non-terminal wait outcome plus cleanup-path tests | Must trigger stop/release and non-zero exit |
| `cleanup_failure` | Injected stop/release failure tests | Must override apparent run success into command failure |
| `post_error_reuse` | Manual hardware reopen/rerun after one safe induced failure | Must pass before Phase 4 hardware verification is considered complete |

This matrix is the verifier-ready contract for Phase 4: every failure class promised by `04-02` now has a named proof path, even when that proof is synthetic, environment-gated, or manual-hardware-only.

## 04-02 Verification Commitments

`04-02` owns the failure-path contract rather than the happy-path run itself, so each promised failure class must be paired with one of these concrete checks:

- `preflight_blocked`: environment-gated local run where preflight returns `EnvironmentNotReady`, plus CLI/core mapping checks that preserve the stable `capture_environment_not_ready` code.
- `start_failure`: synthetic start result with non-`SR_OK` `start_status`, verified to attempt cleanup and report `capture_start_failed`.
- `run_failure`: synthetic acquisition summary with `DS_EV_COLLECT_TASK_END_BY_ERROR`, verified through core classification and CLI normalization.
- `detach`: synthetic acquisition summary with `DS_EV_COLLECT_TASK_END_BY_DETACHED`, plus optional safe hardware unplug validation if practical.
- `incomplete`: synthetic summaries missing logic packets, end packets, or end-packet status, verified to fail even when a normal terminal event is present.
- `timeout`: synthetic non-terminal wait outcome or timeout-path unit coverage that confirms forced cleanup and `capture_timeout` reporting.
- `cleanup_failure`: injected stop/release callback-clear failure path, verified to override otherwise successful capture classification.

The remaining hardware verifier for Phase 4 stays intentionally narrow: prove one representative failure path returns non-zero actionable diagnostics and then confirm the device can be reopened or rerun immediately after that failure.

## 04-03 Validation Coverage Matrix

| Case | Proof method | Location | Default mode |
|------|--------------|----------|--------------|
| `clean_success` | Synthetic success-shape assertions plus manual finite hardware capture that checks logic packet, end marker, terminal normal end, and clean release | `crates/dsview-core/tests/acquisition.rs`, `crates/dsview-core/src/lib.rs` tests, manual CLI UAT below | Mixed: automated + manual hardware |
| `preflight_blocked` | CLI/core error normalization test for `EnvironmentNotReady`, plus local preflight run if USB/resource/runtime prerequisites are intentionally missing | `crates/dsview-cli/tests/capture_cli.rs`, `crates/dsview-cli/src/main.rs` tests, manual CLI UAT below | Automated + environment-gated |
| `start_failure` | CLI error normalization test for `StartFailed` with cleanup detail | `crates/dsview-cli/src/main.rs` tests, `crates/dsview-core/tests/acquisition.rs` | Automated |
| `run_failure` | Core classification test for terminal error event plus CLI stable code assertions | `crates/dsview-core/src/lib.rs` tests, `crates/dsview-cli/src/main.rs` tests, `crates/dsview-cli/tests/capture_cli.rs` | Automated |
| `detach` | Core classification test for detached terminal event; sys summary boundary smoke; optional safe unplug manual check only if practical | `crates/dsview-core/src/lib.rs` tests, `crates/dsview-sys/src/lib.rs`, `crates/dsview-core/tests/acquisition.rs` | Automated by default |
| `incomplete` | Core success-rule tests for missing logic packet, missing end marker, and missing normal terminal signal; CLI diagnostic-shape assertion | `crates/dsview-core/src/lib.rs` tests, `crates/dsview-cli/src/main.rs` tests, `crates/dsview-core/tests/acquisition.rs` | Automated |
| `timeout` | CLI timeout normalization test and bounded-wait cleanup expectations from 04-02 | `crates/dsview-cli/src/main.rs` tests, `crates/dsview-core/tests/acquisition.rs`, `crates/dsview-core/src/lib.rs` tests | Automated |
| `cleanup_failure` | Core cleanup predicate checks, sys summary-shape checks, and CLI cleanup-failure normalization | `crates/dsview-core/src/lib.rs` tests, `crates/dsview-core/tests/acquisition.rs`, `crates/dsview-sys/src/lib.rs`, `crates/dsview-sys/tests/boundary.rs`, `crates/dsview-cli/src/main.rs` tests, `crates/dsview-cli/tests/capture_cli.rs` | Automated |
| `post_error_reuse` | Manual reopen or rerun immediately after one representative non-zero capture failure | Manual CLI UAT below | Manual hardware |

## 04-03 Manual DSLogic Plus UAT

Run these steps only on the current source-runtime machine with a connected `DSLogic Plus` after confirming the udev/USB permission fix is still present. If `LIBUSB_ERROR_ACCESS` reappears at any preflight step, stop and classify the machine as `preflight_blocked` rather than claiming Phase 4 hardware success.

1. Preflight
   - Confirm the source runtime library path still resolves and the DSView resource directory exists.
   - Confirm the local user still has USB access to the device and explicitly watch for `LIBUSB_ERROR_ACCESS` during any list/open/capture command.
   - Run `devices list --use-source-runtime --resource-dir <DSView/DSView/res>` and confirm a supported `dslogic-plus` handle is present.
   - Run `devices open --use-source-runtime --resource-dir <DSView/DSView/res> --handle <handle>` and confirm open/release succeeds before capture validation.
   - Use one known-valid Phase 3 configuration only through the capture flow; do not assume separate capability/config inspection commands exist.
2. Clean finite capture
   - Run `capture --use-source-runtime --resource-dir <DSView/DSView/res> --handle <handle> --sample-rate-hz 20000000 --sample-limit <known-valid finite limit> --channels 0,1,...` with the validated 16-channel-safe rate.
   - Confirm exit code `0`.
   - Confirm the JSON response reports `completion=clean_success`, `saw_logic_packet=true`, `saw_end_packet=true`, `saw_terminal_normal_end=true`, and `cleanup_succeeded=true`.
3. Immediate rerun after success
   - Repeat the same capture command immediately.
   - Confirm the second run also succeeds without reopening the shell environment.
4. Representative failure path
   - Trigger one safe non-zero run, preferably by using a deliberately invalid local environment or another reversible operator-controlled condition rather than risky hardware manipulation.
   - Confirm the CLI returns one of the stable failure codes (`capture_environment_not_ready`, `capture_start_failed`, `capture_run_failed`, `capture_detached`, `capture_incomplete`, `capture_timeout`, or `capture_cleanup_failed`) with actionable JSON detail.
5. Immediate reuse after failure
   - Immediately rerun `devices open` or a known-good finite `capture` after the failure case.
   - Confirm the device is reusable without restarting the machine or manually resetting DSView internals.

This plan deliberately limits manual proof to acquisition lifecycle behavior and does not claim any Phase 5 export validation.

## Recommended Phase Split

### 04-01: Implement capture start/run/finish orchestration in the Rust service layer

This plan should focus on:

- extending `dsview-sys` with acquisition lifecycle bindings and callback/event support
- defining a Rust-safe acquisition summary/result contract that directly encodes the locked `clean_success` rule
- adding `dsview-core` orchestration for open -> apply -> start -> wait -> release
- adding the narrow preflight needed to distinguish environment readiness from capture failure

It should not yet chase every edge-case cleanup nuance if that would blur the seam design.

### 04-02: Handle stop/cleanup/error paths so failed acquisitions do not leave broken state

This plan should focus on:

- error classification for preflight-blocked vs start failure vs run failure vs detach
- timeout and forced-stop handling when natural finish does not arrive
- stop/release ordering and RAII cleanup behavior
- incomplete data/end-marker handling
- cleanup-failure reporting rules
- the explicit post-error reuse verifier gate

This is where `RUN-02` and `RUN-03` become explicit acceptance criteria, not just side effects of the happy path.

### 04-03: Add smoke and integration validation for the acquisition lifecycle

This plan should focus on:

- unit coverage for core run result interpretation
- any environment-gated bridge/runtime smoke checks that make sense locally
- documented manual hardware UAT for real capture success, rerunability, and failure recovery
- the verifier-ready coverage matrix above

## Risks To Address In The Plans

- DSView callback APIs are global-process state, so careless design can create hidden cross-run state leakage.
- A run can appear complete by event ordering while still having incomplete packet/end state.
- Natural finite completion is not yet proven on this machine and therefore needs a timeout-backed validation gate.
- Releasing after a failed run may still leave the device busy if stop/join assumptions are wrong.
- Local USB permission state can regress even though open/release now passes after the udev fix.
- Phase 3 capability assumptions may need refinement once real logic capture is exercised on hardware.
- If timeout and cleanup policy are not implemented, a broken run may hang the CLI indefinitely.

## Recommendation Summary

Plan Phase 4 around the existing bridge/core/cli seam, not around new direct bindings into DSView internals. Extend `dsview-sys` with the public `ds_*` acquisition lifecycle and callback hooks, let `dsview-core` own a single-shot run orchestration contract that guarantees release on all paths, and treat success as the locked combination of normal terminal event, observed logic data, observed end marker, and successful cleanup. Treat natural finite completion and post-error device reuse as bounded execution assumptions backed by explicit manual verifier gates rather than as already-proven facts.