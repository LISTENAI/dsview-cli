# Phase 3 Research: Capture Configuration Surface

**Date:** 2026-04-03
**Phase:** 3 - Capture Configuration Surface
**Goal:** Expose the minimum useful capture controls for `DSLogic Plus` and validate them before acquisition starts.

## Goal Fit

Phase 3 starts after Phase 2 proved the runtime bridge, supported-device filtering, and safe open/release flow. This phase must stay strictly on configuration and pre-run validation for:

1. sample rate selection
2. sample limit/depth selection
3. enabled logic channel selection
4. rejection of invalid or unsupported settings before acquisition starts

This phase does **not** start or stop acquisition, stream samples, or export artifacts.

## Confirmed Upstream Facts

### 1. Active-device configuration is already exposed through the DSView-side facade

The established configuration path for the currently opened device is:

- `ds_get_actived_device_config`
- `ds_set_actived_device_config`
- `ds_get_actived_device_config_list`
- `ds_enable_device_channel_index`

This is the correct Phase 3 seam because it extends the same active-device/session boundary already proven in Phase 2 and avoids reaching into broader DSView internals from Rust.

### 2. Phase 3 settings map cleanly to the v1 requirements

Known keys needed for the minimum useful capture surface are:

- `SR_CONF_SAMPLERATE` for CAP-01
- `SR_CONF_LIMIT_SAMPLES` for CAP-02
- `SR_CONF_PROBE_EN` plus `ds_enable_device_channel_index` for CAP-03
- `SR_CONF_CHANNEL_MODE`, `SR_CONF_VLD_CH_NUM`, `SR_CONF_HW_DEPTH`, `SR_CONF_STREAM`, `SR_CONF_RLE_SUPPORT`, `SR_CONF_THRESHOLD`, and `SR_CONF_VTH` as validation helpers for CAP-04

These helpers matter even if they are not all CLI-visible in v1, because they constrain whether a requested configuration is actually valid on `DSLogic Plus`.

### 3. Samplerate discovery has a stable, automation-friendly shape

The samplerate list is returned as `a{sv}` with:

- `"samplerates"` => array of `uint64`

This is the most straightforward config-list path in the native boundary and is a good candidate for normalization into a Rust domain model without leaking GLib-native details upward.

### 4. Some config lists remain native-only and must stay behind the process boundary

Channel mode and threshold options come back as native pointer-valued lists (`sr_list_item[]`) inside the same process boundary.

Implication:

- Rust should not treat these as broad user-facing dynamic enums in Phase 3.
- The sys/native boundary should normalize only the minimum information needed for validation.
- If upstream requires channel mode or threshold state to compute valid sample rates or limits, that logic should stay inside the narrow native boundary or be converted into explicit Rust-safe data before leaving it.

### 5. Upstream validation is device-specific and includes coupled constraints

Existing findings show the configuration is not independently selectable per field. Important constraints include:

- channel-mode to sample-rate coupling
- enabled-channel ceilings
- depth limits that depend on the number of enabled channels
- alignment behavior for sample counts/depth

This means CAP-04 cannot be implemented as simple type checking. The Rust-side validation model must represent the requested configuration plus the device capability snapshot used to validate it.

## Planning Implications

### Recommended layering

1. `dsview-sys`
   - expose raw/native helpers to read capability lists and apply already validated config
   - keep pointer-valued list handling and GLib/native translation isolated here

2. `dsview-core`
   - define Rust domain types for requested config, device capability snapshot, and validated/effective config
   - own pre-run validation and normalization rules
   - reject unsupported combinations before any acquisition entry point is called

3. `dsview-cli`
   - parse CLI inputs into the core request model
   - render validation failures as stable diagnostics
   - show effective settings only after validation succeeds

### Recommended scope boundary

Phase 3 should include:

- capture config domain types
- capability loading for the active device
- validation and normalization before acquisition
- applying validated settings to the active device/session layer
- automated tests for valid and invalid combinations

Phase 3 should not include:

- capture start/stop
- sample retrieval/stream handling
- export generation
- threshold/channel-mode user-facing UX unless required as an internal validation input

## Recommended Direction

Plan Phase 3 around a two-step contract:

1. load a `DSLogic Plus` capability snapshot from the opened active device
2. validate a requested capture configuration into an explicit effective configuration before applying it

The effective configuration should at minimum contain:

- selected sample rate
- selected sample limit/depth after alignment/normalization
- enabled channel set
- any internal mode/capability-derived values required to make the request valid

This creates a reusable Rust-side model for later acquisition and export phases while keeping native complexity narrow.

## Risks To Address In The Plans

- Native capability reads may depend on the active device already being opened in Phase 2 style session flow.
- Pointer-valued list results must not leak ownership or lifetime ambiguity into Rust.
- Validation must distinguish:
  - malformed user input
  - unsupported-but-well-formed requests
  - device-state/native read failures
- Effective sample limits may differ from requested limits because of alignment or hardware depth ceilings; this must be explicit in the resulting model and diagnostics.

## Recommended Phase Split

- **03-01:** Define Rust domain types and validation rules for capture configuration.
- **03-02:** Wire validated capture settings into the native session layer.
- **03-03:** Add tests for valid, invalid, and device-specific capture configuration cases.

## Revalidation Triggers

Revisit the Phase 3 plan if any of these change:

- the `ds_*` active-device config API signatures or ownership rules
- `SR_CONF_*` key usage for `DSLogic Plus`
- upstream channel-mode or depth coupling behavior
- Phase 2 session ownership assumptions around active-device lifetime

## Recommendation Summary

Use Phase 3 to establish a capability-driven Rust capture-config model for `DSLogic Plus`, validate requested settings before acquisition begins, and apply only validated/effective settings through the narrow active-device config seam. Keep `DSView/` read-only, keep native list handling in `dsview-sys`, and keep this phase strictly limited to configuration plus pre-run validation.
