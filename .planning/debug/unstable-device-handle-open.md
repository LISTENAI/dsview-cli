# unstable-device-handle-open

## Context

- Issue: `devices list` returned a `handle` that could not be reused by a later `devices open --handle ...` in a separate CLI invocation.
- Expected behavior: the selector emitted by `devices list` should remain usable across process boundaries for the same supported device.
- Observed behavior: `devices open` failed with `unsupported_selection` after the DSView runtime rescanned hardware and assigned a different native handle on the second process launch.

## Investigation

### Reproduced code path

- CLI list path in `crates/dsview-cli/src/main.rs` serialized the raw native handle as the public `handle` field.
- CLI open path in `crates/dsview-cli/src/main.rs` parsed `--handle` back into a native `DeviceHandle` and passed it into `Discovery::open_device(...)`.
- Core open path in `crates/dsview-core/src/lib.rs` called `list_supported_devices()` again, then required an exact equality match on the native handle before calling the runtime open function.

### Root cause

- The DSView-native `DeviceHandle` is effectively process-local and can change every time hardware is rescanned.
- The CLI exposed that raw native handle as if it were a stable cross-process selector.
- A handle copied from `devices list` in process A therefore failed to match a supported device in process B.

## Fix implemented

### Design

- Keep the native runtime handle internal to the current process.
- Expose a deterministic CLI-facing selection handle instead.
- Resolve `devices open --handle ...` against the stable selection handle, then use the current scan's native handle internally when calling the runtime.

### Code changes

#### `crates/dsview-core/src/lib.rs`

- Added `SelectionHandle`, a non-zero CLI-visible selector.
- Updated `SupportedDevice` to store both:
  - `selection_handle`: deterministic selector used by CLI
  - `native_handle`: current process-local DSView handle kept internal
- Changed `filter_supported_devices(...)` to assign deterministic selection handles based on supported-device order (`1`, `2`, ...).
- Changed `Discovery::open_device(...)` to accept `SelectionHandle`, re-scan supported devices, resolve the stable selector, and only then open the current scan's `native_handle`.
- Updated `BringUpError::UnsupportedSelection` to report the stable selection handle instead of the raw native handle.
- Added regression coverage showing the selection handle stays the same even when the underlying native handle changes across scans.

#### `crates/dsview-cli/src/main.rs`

- `devices list` now emits `selection_handle.raw()` in the public `handle` field.
- `devices open --handle ...` now parses the CLI argument as `SelectionHandle` rather than a native `DeviceHandle`.
- Error mapping for `unsupported_selection` now refers to the stable selector path.
- Text output path continues to print the public selector.

## Validation

### Static validation

- Confirmed the old failure mode directly in code.
- Added unit coverage in `crates/dsview-core/src/lib.rs`:
  - `selection_handles_are_stable_across_native_handle_changes`

### Follow-up validation

Run after syncing and rebuilding the main workspace:

1. `cargo build`
2. `cargo test -p dsview-core -p dsview-cli`
3. Re-run hardware repro:
   - `./target/debug/dsview-cli devices list --resource-dir ./DSView/DSView/res --use-source-runtime`
   - copy returned `handle`
   - `./target/debug/dsview-cli devices open --handle <copied> --resource-dir ./DSView/DSView/res --use-source-runtime`
4. Confirm that a list handle from one invocation opens successfully in a later invocation even if DSView assigns a different internal native handle during the second scan.

## Status

- Root cause: confirmed
- Fix: applied to main workspace
- Full runtime validation: pending local build/test/hardware smoke
