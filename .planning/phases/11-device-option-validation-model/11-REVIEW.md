---
phase: 11-device-option-validation-model
reviewed: 2026-04-13T05:55:31Z
depth: standard
files_reviewed: 9
files_reviewed_list:
  - crates/dsview-core/src/device_option_validation.rs
  - crates/dsview-core/src/capture_config.rs
  - crates/dsview-core/src/lib.rs
  - crates/dsview-core/tests/device_option_validation.rs
  - crates/dsview-cli/src/main.rs
  - crates/dsview-sys/wrapper.h
  - crates/dsview-sys/bridge_runtime.c
  - crates/dsview-sys/src/lib.rs
  - crates/dsview-sys/tests/device_options.rs
findings:
  critical: 0
  warning: 2
  info: 0
  total: 2
status: issues_found
---

# Phase 11: Code Review Report

**Reviewed:** 2026-04-13T05:55:31Z
**Depth:** standard
**Files Reviewed:** 9
**Status:** issues_found

## Summary

Reviewed the Phase 11 device-option validation model across `dsview-core`, `dsview-cli`, and the `dsview-sys` bridge/runtime boundary. The new validation structs and mode-scoped samplerate modeling are generally coherent, and the targeted Phase 11 tests in `dsview-core` and `dsview-sys` pass. Two correctness issues remain: capture validation still ignores the user-selected device handle, and the native mode-discovery helpers can leave the hardware in a mutated mode when the runtime cannot report the current mode values.

Post-review note: both warning findings were addressed before phase completion in commits `d21d864` and `d8caf09`, and the full workspace suite passed afterward.

## Warnings

### WR-01: Capture validation still uses the first supported device instead of the selected handle

**File:** `crates/dsview-cli/src/main.rs:297`, `crates/dsview-core/src/lib.rs:521`, `crates/dsview-core/src/lib.rs:615`, `crates/dsview-core/src/lib.rs:745`
**Issue:** `run_capture()` validates the request before starting capture, but `Discovery::validate_capture_config()` does not accept a `SelectionHandle`. It calls `dslogic_plus_capabilities()`, which opens the first supported DSLogic Plus device, not the device named by `--handle`. With two supported devices attached, validation can reject a configuration that is valid for the selected unit or accept one that only matches another unit's active mode/capabilities, and the later `run_capture()` path repeats the same wrong-device validation.
**Fix:**
```rust
pub fn validate_capture_config(
    &self,
    selection_handle: SelectionHandle,
    request: &CaptureConfigRequest,
) -> Result<ValidatedCaptureConfig, CaptureConfigError> {
    let opened = self
        .open_device(selection_handle)
        .map_err(CaptureConfigError::from_runtime_error)?;
    let capabilities = self.dslogic_plus_capabilities_for_opened(&opened)?;
    capabilities.validate_request(request)
}
```

Then update both callers to pass the selected handle:
- `crates/dsview-cli/src/main.rs` should call `validate_capture_config(handle, &run_request.config)`
- `crates/dsview-core/src/lib.rs` should use `request.selection_handle` inside `run_capture()`

### WR-02: Read-only mode discovery can silently change device state when current modes are unavailable

**File:** `crates/dsview-sys/bridge_runtime.c:1353`, `crates/dsview-sys/bridge_runtime.c:1387`, `crates/dsview-sys/bridge_runtime.c:1507`, `crates/dsview-sys/bridge_runtime.c:1557`
**Issue:** Both `dsview_bridge_ds_get_device_options()` and `dsview_bridge_ds_get_validation_capabilities()` switch `SR_CONF_OPERATION_MODE` and `SR_CONF_CHANNEL_MODE` while enumerating mode-scoped channel modes. They only restore the original configuration when `has_current_operation_mode || has_current_channel_mode` is true. If the runtime reports neither current mode, these functions still mutate the device during enumeration and then return success without restoring anything, so a supposedly read-only inspection/validation query can leave the hardware in the last enumerated mode.
**Fix:**
```c
if (!out_snapshot->has_current_operation_mode || !out_snapshot->has_current_channel_mode) {
    return SR_ERR_NA;
}
```

Or, if the bridge must tolerate partially missing metadata, resolve both current modes through dedicated getters before any mutation and fail closed when the original state cannot be restored. The key requirement is: do not start enumerating alternate modes unless the bridge already has enough information to restore the exact original mode pair.

---

_Reviewed: 2026-04-13T05:55:31Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
