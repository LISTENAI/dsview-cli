# Phase 2 Research: Device Discovery and Session Bring-Up

**Date:** 2026-04-03
**Phase:** 2 - Device Discovery and Session Bring-Up
**Goal:** Let the CLI discover supported hardware and safely open a `DSLogic Plus` session through the proven native stack.

## Goal Fit

Phase 2 must move beyond the Phase 1 version-string proof and establish a real runtime bridge for:
1. library initialization and teardown
2. device enumeration and `DSLogic Plus` filtering
3. safe open/close bring-up without starting acquisition
4. stable error reporting for missing devices, unsupported devices, and open failures

This phase still stops short of capture execution, sample configuration, and export.

## Findings

### 1. DSView already exposes a frontend-facing device lifecycle facade

The most concrete bring-up path in the repo is the `ds_*` facade in `DSView/libsigrok4DSL/lib_main.c`:
- `ds_lib_init()` initializes logging, calls private `sr_init()`, registers drivers through `sr_driver_list()` / `sr_driver_init()`, triggers an initial hardware scan, and starts hotplug handling.
- `ds_lib_exit()` tears down the active device, hotplug thread, trigger state, and finally calls private `sr_exit()`.
- `ds_set_firmware_resource_dir()` configures the firmware/bitstream search path used during device bring-up.
- `ds_get_device_list()` exposes a compact list of discovered device handles and names.
- `ds_active_device()` selects and opens a device handle through existing DSView-side open logic.
- `ds_release_actived_device()` and `ds_get_last_error()` provide teardown and last-error state.

This facade is a better Phase 2 seam than binding Rust directly to many `SR_PRIV` internals.

### 2. Device enumeration for DSLogic Plus is data-driven in the DSLogic driver

The DSLogic scan path lives in `DSView/libsigrok4DSL/hardware/DSL/dslogic.c`:
- `scan()` walks libusb devices, matches them against `supported_DSLogic[]`, and creates `sr_dev_inst` entries for matched hardware.
- `scan()` may upload firmware if the device is present but not yet configured.
- The profile table in `DSView/libsigrok4DSL/hardware/DSL/dsl.h` contains the device IDs and model strings Phase 2 should treat as supported.

For `DSLogic Plus`, current repo evidence shows two relevant profile entries:
- `0x2A0E:0x0030` in `DSView/libsigrok4DSL/hardware/DSL/dsl.h`
- legacy/high-speed `DS_VENDOR_ID:0x0020` in `DSView/libsigrok4DSL/hardware/DSL/dsl.h`

The model string is spelled `DSLogic PLus` upstream, so Phase 2 should normalize device identification by USB IDs and/or exact upstream model strings rather than assuming corrected capitalization in native output.

### 3. Open/close bring-up depends on DSView-specific runtime assumptions

`ds_active_device()` in `lib_main.c` eventually delegates into the DSLogic open path in `DSView/libsigrok4DSL/hardware/DSL/dslogic.c`, which then reaches `dsl_dev_open()` in `DSView/libsigrok4DSL/hardware/DSL/dsl.c`.

This path does more than a simple handle open:
- opens and claims the libusb interface
- checks FPGA / firmware state
- may require firmware or resource binaries under `DS_RES_PATH`
- performs hardware-specific validation before the device becomes usable

That means Phase 2 needs a real runtime integration path, not just compile-time header proof.

### 4. A tiny repo-owned shim is the lowest-risk Rust-facing boundary

Phase 1 intentionally avoided private APIs like `sr_init`, `sr_exit`, and session internals. Phase 2 still should not spread raw DSView internals across Rust.

The safest path is:
- keep `DSView/` read-only
- add a tiny repo-owned shim that wraps only the `ds_*` operations needed for init/list/open/close/error lookup
- bind Rust to that shim from `dsview-sys`
- expose safe orchestration from `dsview-core`

This keeps unsafe/native global-state handling contained while still reusing the exact DSView-side behavior.

### 5. Firmware/resource configuration is a first-class Phase 2 concern

The scan/open path can require firmware assets named in `supported_DSLogic[]`, and `ds_set_firmware_resource_dir()` exists specifically to point DSView at those resources.

Phase 2 must therefore decide:
- where the CLI gets the resource directory from
- what default path is attempted in development
- what error message is surfaced when resources are missing or invalid
- when that path is validated relative to init, list, and open operations

The recommended contract for this phase is:
- `dsview-cli` accepts an explicit resource-directory option for Phase 2 bring-up commands
- `dsview-core` owns validation and passes the resolved path into the sys layer before operations that may require firmware assets
- the CLI emits a dedicated actionable error when the path is missing, unreadable, or does not contain the required firmware files for supported `DSLogic Plus` variants

This cannot be deferred entirely to later capture work because bring-up may fail before any capture begins.

## Options Compared

### Option A - Bind Rust directly to more of `libsigrok4DSL`

**Approach:** Extend `dsview-sys` with direct FFI declarations for `ds_lib_init()`, `ds_get_device_list()`, `ds_active_device()`, and related types.

**Pros**
- Minimal extra native code.
- Uses the frontend facade instead of private `sr_*` internals directly.

**Cons**
- Still leaks DSView-native structs and memory conventions into Rust.
- Makes future ABI drift handling harder if the facade changes.
- Encourages more raw bindings as the phase grows.

**Assessment:** Possible, but higher risk than a narrow shim.

### Option B - Add a tiny repo-owned C shim over the `ds_*` facade

**Approach:** Create a very small repo-owned native adapter that wraps init/list/open/close/error operations into a narrower ABI for Rust.

**Pros**
- Keeps Rust insulated from DSView memory ownership and global-state details.
- Easier to test and extend incrementally.
- Preserves the rule that unsafe/native complexity stays in one place.

**Cons**
- Adds one more maintained native layer.
- Still depends on system native prerequisites and DSView headers.

**Assessment:** Best Phase 2 direction.

### Option C - Reach into private `sr_*` lifecycle/session APIs directly

**Approach:** Bind Rust to `sr_init`, `sr_exit`, session functions, and driver internals.

**Pros**
- Maximum control.

**Cons**
- Violates the Phase 1 boundary guidance.
- Increases ABI and ownership risk sharply.
- Couples Rust to DSView-private internals before basic bring-up is proven.

**Assessment:** Reject unless the `ds_*` facade proves unusable.

## Recommended Direction

Plan Phase 2 around a tiny repo-owned shim that wraps the DSView `ds_*` facade for:
- library init / exit
- firmware resource configuration
- device list retrieval
- supported-device filtering helpers
- active-device open / release
- last-error lookup

Then layer the implementation like this:
1. `dsview-sys`
   - owns the shim build and raw ABI
   - exposes a minimal device/session bring-up boundary
2. `dsview-core`
   - owns safe library/session lifecycle and `DSLogic Plus` filtering
   - normalizes native failure modes into Rust domain errors
3. `dsview-cli`
   - exposes narrow commands for listing devices and proving open/close bring-up
   - returns stable exit codes and machine-readable output

## Planning Implications

Phase 2 should be split into:
- **02-01:** extend the native boundary to support init/list/open/close/error lookup through a tiny shim over the `ds_*` facade
- **02-02:** add safe Rust-side device/session bring-up orchestration in `dsview-core`
- **02-03:** expose CLI commands and stable diagnostics for supported-device discovery and open failures

The plans must not promise:
- sample-rate or channel configuration
- acquisition start/stop
- VCD export
- protocol decode

## Revalidation Triggers

Revisit the Phase 2 plan if any of these change:
- `DSView/libsigrok4DSL/lib_main.c` changes the `ds_*` facade signatures or lifecycle assumptions
- `DSView/libsigrok4DSL/hardware/DSL/dsl.h` changes `DSLogic Plus` profile IDs, model strings, or firmware asset names
- `DSView/libsigrok4DSL/hardware/DSL/dslogic.c` changes scan/open behavior around firmware upload or device matching
- the project finds a true standalone reusable `libsigrok4DSL` library path and no longer needs a shim-based bridge

## Open Uncertainties

- Which exact native/system dependencies are minimally required to build a repo-owned Phase 2 shim on this machine.
- Whether `ds_lib_init()` and its hotplug thread behavior are safe to wrap in a simple single-owner Rust API without additional synchronization.
- Whether Phase 2 should use exact USB IDs, native model strings, or both when reporting `DSLogic Plus` support.

## Recommendation Summary

Use Phase 2 to prove a real runtime bring-up path through a tiny repo-owned shim over the DSView `ds_*` facade. Keep `DSView/` read-only, keep unsafe in `dsview-sys`, normalize `DSLogic Plus` support explicitly, and stop at enumeration plus open/close bring-up with stable diagnostics.
