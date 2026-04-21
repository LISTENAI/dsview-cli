# Phase 11: Device Option Validation Model - Research

**Researched:** 2026-04-13
**Domain:** DSLogic Plus pre-acquisition option validation in the Rust core/runtime boundary. [VERIFIED: `.planning/ROADMAP.md` + `crates/dsview-core/src/lib.rs` + `crates/dsview-sys/src/lib.rs`]
**Confidence:** MEDIUM

<user_constraints>
## User Constraints

- No Phase 11 `CONTEXT.md` exists, so the effective locked constraints come from `.planning/ROADMAP.md`, `.planning/REQUIREMENTS.md`, `.planning/STATE.md`, and `CLAUDE.md`. [VERIFIED: phase init output + phase directory listing + `.planning/ROADMAP.md` + `.planning/REQUIREMENTS.md` + `.planning/STATE.md` + `CLAUDE.md`]
- Scope is limited to representing and validating DSLogic Plus device-option combinations before acquisition; CLI option selection belongs to Phase 12 and option application/reporting belongs to Phase 13. [VERIFIED: `.planning/ROADMAP.md`]
- This phase must address `VAL-01` and `VAL-02`. [VERIFIED: `.planning/ROADMAP.md` + `.planning/REQUIREMENTS.md`]
- The shipped `v1.0 MVP` capture/export workflow remains the baseline and must stay working while validation grows richer. [VERIFIED: `.planning/STATE.md` + `.planning/ROADMAP.md`]
- Carry Phase 10's stable discovery contract forward into validation without changing the public option-discovery schema. [VERIFIED: `.planning/STATE.md`]
- Keep `DSView/` read-only and keep unsafe/native integration isolated behind the small `dsview-sys` boundary. [VERIFIED: `.planning/REQUIREMENTS.md` + `CLAUDE.md`]
- Milestone scope remains `DSLogic Plus` only; presets, repeat/loop behavior, advanced trigger work, protocol decode, and broader hardware support stay out of scope. [VERIFIED: `.planning/STATE.md` + `.planning/REQUIREMENTS.md` + `.planning/ROADMAP.md`]
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| VAL-01 | CLI rejects unsupported combinations of operation mode, channel mode, sample rate, sample limit, and enabled channels before acquisition starts. [VERIFIED: `.planning/REQUIREMENTS.md`] | Research identifies DSView's mode-scoped channel-mode table, mode-dependent samplerate clamping, enabled-channel ceilings, sample-limit capacity formula, and the current Rust gaps that must be replaced. [VERIFIED: `DSView/libsigrok4DSL/hardware/DSL/dsl.h` + `DSView/libsigrok4DSL/hardware/DSL/dsl.c` + `DSView/libsigrok4DSL/hardware/DSL/dslogic.c` + `crates/dsview-core/src/capture_config.rs` + `crates/dsview-core/src/lib.rs`] |
| VAL-02 | CLI rejects unsupported threshold, filter, or mode-incompatible stop-option values before acquisition starts. [VERIFIED: `.planning/REQUIREMENTS.md`] | Research confirms DSLogic Plus exposes filter and stop-option lists plus a VTH voltage range, and recommends a stable error taxonomy so these failures do not collapse into generic runtime strings. [VERIFIED: `DSView/libsigrok4DSL/hardware/DSL/dslogic.c` + `DSView/DSView/pv/prop/binding/deviceoptions.cpp` + `crates/dsview-core/src/device_options.rs` + `crates/dsview-cli/src/main.rs`] |
</phase_requirements>

## Project Constraints (from CLAUDE.md)

- Treat `DSView/` as an upstream dependency and do not modify it for this phase. [VERIFIED: `CLAUDE.md`]
- Keep unsafe/native integration isolated behind a small boundary. [VERIFIED: `CLAUDE.md`]
- Favor scriptable CLI behavior, explicit exit codes, and machine-readable outputs. [VERIFIED: `CLAUDE.md`]
- Keep the milestone scoped to `DSLogic Plus` only until the option workflow is proven stable. [VERIFIED: `CLAUDE.md`]
- Stay inside the GSD workflow; for this task that means producing the research artifact only. [VERIFIED: `CLAUDE.md`]

## Summary

Phase 10 already gives the project a stable automation-facing option snapshot: operation modes, stop options, filters, grouped channel modes, and threshold voltage facts are normalized into Rust with stable IDs and mode-grouped channel modes. [VERIFIED: `crates/dsview-core/src/device_options.rs` + `crates/dsview-core/tests/device_options.rs` + `.planning/phases/10-device-option-bridge-and-discovery/10-VERIFICATION.md`] The missing Phase 11 work is not discovery; it is a richer validation model that joins those option facts with sample rate, sample limit, enabled channels, and a stable validation error taxonomy before any capture begins. [VERIFIED: `.planning/ROADMAP.md` + `.planning/REQUIREMENTS.md` + `crates/dsview-core/src/capture_config.rs`]

The main implementation risk is that the current Rust validator is still the older capture-only path. `CaptureConfigRequest` only contains sample rate, sample limit, and enabled channels, while `Discovery::validate_capture_config()` loads capabilities from the first supported device and validates only against the active channel mode plus one global samplerate list. [VERIFIED: `crates/dsview-core/src/capture_config.rs` + `crates/dsview-core/src/lib.rs`] That shape cannot represent operation mode, stop option, threshold, or filter, and it cannot correctly validate non-default channel-mode combinations that Phase 12 will later expose. [VERIFIED: `.planning/ROADMAP.md` + `crates/dsview-core/src/device_options.rs` + `crates/dsview-core/src/lib.rs`]

DSView itself is mode-aware in exactly the places Phase 11 cares about: DSLogic Plus supports four stream channel modes and three buffer channel modes, each channel mode has its own `vld_num` enabled-channel ceiling and its own `max_samplerate`, and the driver clamps the advertised samplerate list through `dsl_adjust_samplerate()` based on the currently selected channel mode. [VERIFIED: `DSView/libsigrok4DSL/hardware/DSL/dsl.h` + `DSView/libsigrok4DSL/hardware/DSL/dsl.c` + `DSView/libsigrok4DSL/hardware/DSL/dslogic.c`] The planner should therefore treat Phase 11 as a selected-device, mode-aware validation-model phase, not as a minor extension of `capture_config.rs`. [VERIFIED: `.planning/ROADMAP.md` + `crates/dsview-core/src/capture_config.rs`] 

**Primary recommendation:** Build a new internal validation capability snapshot per selected device and per operation/channel mode, keep public discovery JSON unchanged, implement validation as a pure `request + capabilities -> validated request | typed error` function in `dsview-core`, and map those typed errors to stable CLI `code` values instead of stringifying them into `runtime_error`. [VERIFIED: `.planning/STATE.md` + `CLAUDE.md` + `crates/dsview-core/src/lib.rs` + `crates/dsview-cli/src/main.rs`] 

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `dsview-sys` | `0.1.0` [VERIFIED: `crates/dsview-sys/Cargo.toml`] | Own any mode-switching/native probing needed to build an internal validation-capability snapshot without leaking DSView stateful APIs into safe Rust. [VERIFIED: `crates/dsview-sys/src/lib.rs` + `crates/dsview-sys/bridge_runtime.c`] | This already owns restore-on-exit device-option discovery and is the project's required unsafe boundary. [VERIFIED: `CLAUDE.md` + `crates/dsview-sys/bridge_runtime.c`] |
| `dsview-core` | `0.1.0` [VERIFIED: `crates/dsview-core/Cargo.toml`] | Define the Phase 11 request/capability/error model and the pure validation function. [VERIFIED: `crates/dsview-core/src/lib.rs` + `crates/dsview-core/src/capture_config.rs`] | Core already owns typed orchestration, current validation, and normalized device-option models. [VERIFIED: `crates/dsview-core/src/lib.rs` + `crates/dsview-core/src/device_options.rs`] |
| `dsview-cli` | `0.1.0` [VERIFIED: `crates/dsview-cli/Cargo.toml`] | Map validation failures to stable machine-readable `ErrorResponse.code` values. [VERIFIED: `crates/dsview-cli/src/main.rs`] | The CLI already uses stable error codes for other failure classes; Phase 11 should extend that pattern instead of inventing a second reporting style. [VERIFIED: `crates/dsview-cli/src/main.rs`] |
| `DSView/libsigrok4DSL` local submodule | repo snapshot `74948be` [VERIFIED: `git rev-parse --short HEAD` + `DSView/libsigrok4DSL/hardware/DSL/dsl.h`] | Source of truth for DSLogic Plus operation modes, channel-mode limits, samplerate clamping, threshold/filter lists, and sample-depth rules. [VERIFIED: `DSView/libsigrok4DSL/hardware/DSL/dsl.h` + `DSView/libsigrok4DSL/hardware/DSL/dsl.c` + `DSView/libsigrok4DSL/hardware/DSL/dslogic.c`] | The milestone explicitly requires reusing upstream DSView behavior without modifying that code. [VERIFIED: `.planning/REQUIREMENTS.md` + `CLAUDE.md`] |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `thiserror` | `2.0.18` [VERIFIED: `Cargo.lock` + `crates/dsview-core/Cargo.toml` + `crates/dsview-sys/Cargo.toml`] | Define a typed validation error enum with clear human messages while keeping a stable machine code alongside it. [VERIFIED: current project usage in `crates/dsview-core/src/capture_config.rs` + `crates/dsview-sys/src/lib.rs`] | Use for the new Phase 11 validation taxonomy. [VERIFIED: existing project pattern] |
| `serde` | `1.0.228` [VERIFIED: `Cargo.lock` + `crates/dsview-core/Cargo.toml` + `crates/dsview-cli/Cargo.toml`] | Keep normalized/request/error structs serializable where the CLI or metadata needs them. [VERIFIED: `crates/dsview-core/src/device_options.rs` + `crates/dsview-cli/src/device_options.rs`] | Use for core/domain structures that the CLI may render or test as JSON later. [VERIFIED: current crate pattern] |
| `serde_json` | `1.0.149` [VERIFIED: `Cargo.lock` + `crates/dsview-core/Cargo.toml` + `crates/dsview-cli/Cargo.toml`] | Preserve the existing pretty-JSON output path once validation errors are surfaced through the CLI. [VERIFIED: `crates/dsview-cli/src/main.rs`] | Use only through the existing CLI render path. [VERIFIED: `crates/dsview-cli/src/main.rs`] |
| `clap` | `4.6.0` [VERIFIED: `Cargo.lock` + `crates/dsview-cli/Cargo.toml`] | Already owns the current CLI surface and error/help plumbing. [VERIFIED: `crates/dsview-cli/src/main.rs`] | Relevant because Phase 11 error taxonomy should be ready for Phase 12 CLI flags, but Phase 11 does not need new parsing primitives. [VERIFIED: `.planning/ROADMAP.md` + `crates/dsview-cli/src/main.rs`] |
| Rust built-in test harness | `cargo 1.94.1` / `rustc 1.94.1` [VERIFIED: local `cargo --version` + `rustc --version`] | Unit/integration coverage for core/sys/CLI validation behavior. [VERIFIED: workspace test layout + `cargo test -q --workspace -- --list`] | Use for Wave 0 and the Phase 11 regression matrix. [VERIFIED: `crates/dsview-core/tests` + `crates/dsview-sys/tests` + `crates/dsview-cli/tests`] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Reusing `CaptureConfigRequest` and `CaptureCapabilities` as the Phase 11 model | A new `device_option_validation` model layered beside `capture_config` | Reuse is lower-effort, but the current types cannot represent operation mode, stop option, filter, threshold, or selected-device mode groups. [VERIFIED: `crates/dsview-core/src/capture_config.rs` + `crates/dsview-core/src/lib.rs` + `.planning/ROADMAP.md`] |
| Building per-mode validation facts in `dsview-core` by adding more runtime setters there | Keep stateful mode switching/probing inside `dsview-sys`, then expose an owned snapshot to core | Core-side probing would spread mutable device-state orchestration outside the unsafe/native boundary that `CLAUDE.md` explicitly wants kept small. [VERIFIED: `CLAUDE.md` + `crates/dsview-sys/bridge_runtime.c`] |
| Extending the public Phase 10 discovery response with validation-only internals like per-mode samplerate arrays | Keep discovery JSON stable and add a separate internal validation-capability snapshot | Extending public discovery would couple planner/CLI consumers to validation internals and contradict the explicit Phase 10 carry-forward constraint. [VERIFIED: `.planning/STATE.md` + `crates/dsview-cli/src/device_options.rs`] |
| Trial-applying options to the device and seeing what fails | Pure validation against a preloaded capability snapshot | Trial application mutates device state, blurs validation vs execution, and can only fail after the user already reached the capture path. [VERIFIED: `.planning/ROADMAP.md` + `crates/dsview-core/src/lib.rs`] |

**Installation:**
```bash
# No new dependencies are required for Phase 11.
# Keep using the existing workspace crates and Cargo test harness.
```

**Version verification:** Workspace crate versions were verified from `Cargo.toml`, resolved dependency versions were verified from `Cargo.lock`, and local toolchain versions were verified from `cargo --version` / `rustc --version`. [VERIFIED: `Cargo.toml` + `crates/dsview-core/Cargo.toml` + `crates/dsview-cli/Cargo.toml` + `crates/dsview-sys/Cargo.toml` + `Cargo.lock` + local command output]

## Architecture Patterns

### Recommended Project Structure

```text
crates/
├── dsview-sys/
│   ├── wrapper.h                 # Private/native snapshot shape for validation capabilities
│   ├── bridge_runtime.c          # Mode-switch probing + restore-on-exit enumeration
│   └── src/lib.rs                # Safe Rust wrapper over the owned validation snapshot
├── dsview-core/
│   ├── src/device_option_validation.rs  # New Phase 11 request/capability/error model
│   ├── src/capture_config.rs            # Either reduced to shared helpers or left as compatibility code
│   └── tests/device_option_validation.rs
└── dsview-cli/
    ├── src/main.rs               # Validation-error -> `ErrorResponse.code` mapping
    └── tests/capture_cli.rs      # Machine-readable validation-code assertions
```

This layout preserves the existing crate layering and keeps validation-state probing separate from the public discovery renderer introduced in Phase 10. [VERIFIED: `CLAUDE.md` + workspace layout + `.planning/STATE.md`]

### Pattern 1: Separate the request model from the validation-capability model

**What:** Treat user intent as one typed request and DSView-backed valid combinations as a second typed capability snapshot. [VERIFIED: current project already separates `CaptureConfigRequest` from `CaptureCapabilities` in `crates/dsview-core/src/capture_config.rs`]

**When to use:** Use this for Phase 11 because the request must carry operation mode, stop option, channel mode, sample rate, sample limit, enabled channels, threshold voltage, and filter selection together, while the capability model must carry DSView-backed allowed values and compatibility rules. [VERIFIED: `.planning/ROADMAP.md` + `.planning/REQUIREMENTS.md`]

**Example:**
```rust
// Source pattern: crates/dsview-core/src/capture_config.rs
pub fn validate_request(
    &self,
    request: &CaptureConfigRequest,
) -> Result<ValidatedCaptureConfig, CaptureConfigError> {
    // Pure validation against a preloaded capability snapshot.
}
```
[VERIFIED: `crates/dsview-core/src/capture_config.rs`]

**Recommended Phase 11 request shape:** A unified request carrying stable Phase 10 IDs for `operation_mode_id`, `stop_option_id`, `channel_mode_id`, and `filter_id`, plus `sample_rate_hz`, `sample_limit`, `enabled_channels`, and `threshold_volts`, is the cleanest fit for the roadmap and Phase 10 discovery contract. [ASSUMED]

### Pattern 2: Build per-mode samplerate facts from the selected device, not from one global list

**What:** DSView's samplerate list is not static; `dsl_adjust_samplerate()` clamps the exposed samplerates to the currently selected `channel_mode`, and `config_list(SR_CONF_CHANNEL_MODE)` changes with `devc->stream`. [VERIFIED: `DSView/libsigrok4DSL/hardware/DSL/dsl.c` + `DSView/libsigrok4DSL/hardware/DSL/dslogic.c`]

**When to use:** Use this when loading validation capabilities for a selected device. For each operation mode and each channel mode in that group, temporarily switch modes, capture the allowed samplerates, and then restore the original device state. [VERIFIED: `crates/dsview-sys/bridge_runtime.c` already performs restore-on-exit enumeration for device options; samplerate clamping behavior is in `DSView/libsigrok4DSL/hardware/DSL/dsl.c`] 

**Example:**
```c
// Source: DSView/libsigrok4DSL/hardware/DSL/dsl.c
for (i = 0; devc->profile->dev_caps.samplerates[i]; i++) {
    if (devc->profile->dev_caps.samplerates[i] >
            channel_modes[devc->ch_mode].max_samplerate)
        break;
}
devc->samplerates_max_index = i - 1;
```
[VERIFIED: `DSView/libsigrok4DSL/hardware/DSL/dsl.c`]

### Pattern 3: Key validation by stable IDs, but carry native codes for application

**What:** Phase 10 normalized stable IDs like `operation-mode:<code>`, `stop-option:<code>`, `filter:<code>`, and `channel-mode:<code>` in `dsview-core`, while preserving raw native codes in the snapshot. [VERIFIED: `crates/dsview-core/src/device_options.rs` + `crates/dsview-core/tests/device_options.rs`]

**When to use:** Validate CLI-facing requests against stable IDs in core, then keep native codes in the validated result so Phase 13 can apply options deterministically without another lookup pass. [VERIFIED: Phase 10 stable-ID contract + Phase 13 will apply validated options per `.planning/ROADMAP.md`] 

**Example:**
```rust
// Source: crates/dsview-core/src/device_options.rs
fn normalize_channel_mode(mode: DeviceOptionChannelMode) -> ChannelModeOptionSnapshot {
    ChannelModeOptionSnapshot {
        id: channel_mode_id(mode.code),
        native_code: mode.code,
        label: mode.label,
        max_enabled_channels: mode.max_enabled_channels,
    }
}
```
[VERIFIED: `crates/dsview-core/src/device_options.rs`]

### Pattern 4: Return a typed validation error enum with stable machine codes

**What:** The CLI already has a stable `ErrorResponse.code` contract for invalid selectors and runtime failures, but the current capture validation path stringifies `CaptureConfigError` into `RuntimeError::InvalidArgument`, which the CLI then reports as generic `runtime_error`. [VERIFIED: `crates/dsview-cli/src/main.rs` + `crates/dsview-core/src/lib.rs` + `crates/dsview-core/src/capture_config.rs`]

**When to use:** Phase 11 should introduce a dedicated validation error type in core and a direct CLI mapping function instead of going through `to_string()`. [VERIFIED: current generic mapping behavior in `crates/dsview-cli/src/main.rs`] 

**Example:**
```rust
// Recommended shape derived from the existing `ErrorResponse.code` pattern.
impl DeviceOptionValidationError {
    pub const fn code(&self) -> &'static str {
        match self {
            Self::UnknownOperationMode { .. } => "invalid_operation_mode",
            Self::UnknownStopOption { .. } => "invalid_stop_option",
            Self::StopOptionIncompatibleWithMode { .. } => "stop_option_incompatible",
            Self::UnknownChannelMode { .. } => "invalid_channel_mode",
            Self::UnsupportedSampleRate { .. } => "sample_rate_unsupported",
            Self::TooManyEnabledChannels { .. } => "enabled_channels_exceed_mode_limit",
            Self::SampleLimitExceedsCapacity { .. } => "sample_limit_exceeds_capacity",
            Self::ThresholdOutOfRange { .. } => "threshold_out_of_range",
            Self::UnknownFilter { .. } => "invalid_filter",
        }
    }
}
```
[ASSUMED]

### Anti-Patterns to Avoid

- **Reusing `Discovery::validate_capture_config()` as the long-term Phase 11 path:** It validates against the first supported device and only the active channel mode, so it cannot serve Phase 11 once operation mode and channel mode become request inputs. [VERIFIED: `crates/dsview-core/src/lib.rs` + `crates/dsview-core/src/capture_config.rs`]
- **Treating one samplerate list as valid for all channel modes:** DSView clamps samplerates per `ch_mode`, so a global list will accept invalid combinations or reject valid ones. [VERIFIED: `DSView/libsigrok4DSL/hardware/DSL/dsl.c` + `DSView/libsigrok4DSL/hardware/DSL/dsl.h`]
- **Parsing human labels to infer mode ceilings or compatibility:** DSView already carries `vld_num` and `max_samplerate` in the native `channel_modes[]` table, and Phase 10 already normalized code-backed IDs. [VERIFIED: `DSView/libsigrok4DSL/hardware/DSL/dsl.h` + `crates/dsview-core/src/device_options.rs`]
- **Stringifying validation failures into runtime errors:** That throws away the machine-readable category required by Phase 11 success criteria. [VERIFIED: `.planning/ROADMAP.md` + `crates/dsview-cli/src/main.rs`] 

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Mode-aware samplerate compatibility | A label parser or hard-coded Rust table of DSLogic Plus mode limits | A sys-backed validation-capability loader that probes samplerates while each operation/channel mode is active, then restores original state | DSView already computes the right list via `dsl_adjust_samplerate()`; duplicating that logic in Rust risks drift. [VERIFIED: `DSView/libsigrok4DSL/hardware/DSL/dsl.c` + `crates/dsview-sys/bridge_runtime.c`] |
| Option membership checks | Ad hoc string comparisons against CLI input | Stable-ID lookups against the Phase 10 normalized snapshot, with raw native codes preserved for apply-time use | The project already has code-backed stable IDs and should not regress to label-coupled validation. [VERIFIED: `crates/dsview-core/src/device_options.rs` + `crates/dsview-core/tests/device_options.rs`] |
| CLI validation classification | Parsing error messages or mapping every validation failure to `runtime_error` | A typed core validation error enum plus a direct CLI `ErrorResponse` mapper | Stable error categories are a Phase 11 success criterion, and current stringification loses that information. [VERIFIED: `.planning/ROADMAP.md` + `crates/dsview-cli/src/main.rs`] |
| Threshold validation | Raw floating-point equality or an enum-only threshold model | Range/step validation against the VTH capability (`0.0..5.0`, step `0.1`) with legacy threshold data treated as metadata only | DSLogic Plus device options use `SR_CONF_VTH` as a voltage range; the legacy threshold list is supplementary. [VERIFIED: `DSView/libsigrok4DSL/hardware/DSL/dslogic.c` + `DSView/DSView/pv/prop/binding/deviceoptions.cpp` + `crates/dsview-core/src/device_options.rs`] |

**Key insight:** The public discovery snapshot and the internal validation-capability snapshot should not be identical. Keep Phase 10 discovery output stable for automation, and add the richer per-mode samplerate/compatibility data only to the internal Phase 11 validation path. [VERIFIED: `.planning/STATE.md` + `crates/dsview-cli/src/device_options.rs`] 

## Common Pitfalls

### Pitfall 1: The existing validator is not Phase 11's validator

**What goes wrong:** Planning assumes `capture_config.rs` already covers most of Phase 11 because it checks sample rate, sample limit, and enabled channels. [VERIFIED: `crates/dsview-core/src/capture_config.rs`]

**Why it happens:** The current code validates only `CaptureConfigRequest` and loads capabilities through `dslogic_plus_capabilities()`, which opens the first supported device and inspects only the current mode. [VERIFIED: `crates/dsview-core/src/capture_config.rs` + `crates/dsview-core/src/lib.rs`]

**How to avoid:** Treat `capture_config.rs` as a useful pure-validation pattern, not as the finished Phase 11 model. Build a new selected-device validation entrypoint that consumes a richer request and richer capabilities. [VERIFIED: current code shape + `.planning/ROADMAP.md`] 

**Warning signs:** Validation code still has no place for `operation_mode`, `stop_option`, `threshold_volts`, or `filter_id`. [VERIFIED: `crates/dsview-core/src/capture_config.rs`] 

### Pitfall 2: DSView samplerates are channel-mode scoped

**What goes wrong:** A request like "buffer 400x4 + 100 MHz" is validated against the same list used for "buffer 100x16 + 100 MHz" or "stream 20x16 + 20 MHz". [VERIFIED: example mode ceilings in `DSView/libsigrok4DSL/hardware/DSL/dsl.h`]

**Why it happens:** `dsl_adjust_samplerate()` trims the samplerate list using `channel_modes[devc->ch_mode].min_samplerate` and `max_samplerate`. [VERIFIED: `DSView/libsigrok4DSL/hardware/DSL/dsl.c`]

**How to avoid:** Cache supported samplerates per channel mode in the validation capability snapshot, not once per device. [VERIFIED: DSView behavior above] 

**Warning signs:** Validation stores a single `supported_sample_rates: Vec<u64>` at the device level instead of under each channel mode. [VERIFIED: current `CaptureCapabilities` shape in `crates/dsview-core/src/capture_config.rs`] 

### Pitfall 3: Sample-limit validation depends on enabled-channel count and 1024-sample alignment

**What goes wrong:** The validator accepts a sample limit that looks smaller than hardware depth, but the aligned effective limit or enabled-channel count still exceeds capacity. [VERIFIED: `crates/dsview-core/src/capture_config.rs` + `DSView/libsigrok4DSL/hardware/DSL/dsl.c` + `DSView/libsigrok4DSL/libsigrok.h`]

**Why it happens:** DSView computes effective channel depth as `hw_depth / enabled_channels` and aligns sample counts with `SAMPLES_ALIGN 1023ULL`, while current Rust already mirrors this with `sample_limit_alignment: 1024`. [VERIFIED: `DSView/libsigrok4DSL/hardware/DSL/dsl.c` + `DSView/libsigrok4DSL/libsigrok.h` + `crates/dsview-core/src/lib.rs` + `crates/dsview-core/src/capture_config.rs`]

**How to avoid:** Keep sample-limit math as a pure function and reuse it from the richer Phase 11 validator instead of rewriting it inside CLI code. [VERIFIED: existing pure helpers in `crates/dsview-core/src/capture_config.rs`] 

**Warning signs:** Validation checks raw `sample_limit` against hardware depth directly or ignores enabled-channel count after channel selection. [VERIFIED: required math in DSView and current Rust validator] 

### Pitfall 4: Validation failures currently lose their machine category

**What goes wrong:** Invalid capture config becomes a generic `runtime_error` in the CLI even when the underlying failure is precise. [VERIFIED: `crates/dsview-cli/src/main.rs` + `crates/dsview-core/src/lib.rs`]

**Why it happens:** `run_capture()` converts validation failures to `RuntimeError::InvalidArgument(error.to_string())`, and `classify_runtime_error()` maps that fallback branch to `runtime_error`. [VERIFIED: `crates/dsview-cli/src/main.rs`]

**How to avoid:** Add a dedicated `classify_validation_error()` path that consumes the typed Phase 11 enum directly. [VERIFIED: existing stable-code pattern in `invalid_handle_error()` and `classify_capture_error()` in `crates/dsview-cli/src/main.rs`] 

**Warning signs:** New tests only assert human message text and never assert `ErrorResponse.code`. [VERIFIED: current stable-code testing style in `crates/dsview-cli/src/main.rs` tests and `crates/dsview-cli/tests/capture_cli.rs`] 

## Code Examples

Verified patterns from current sources:

### Pure validation against a loaded capability snapshot
```rust
// Source: crates/dsview-core/src/capture_config.rs
if enabled_channels.len() > active_mode.max_enabled_channels as usize {
    return Err(CaptureConfigError::TooManyEnabledChannels {
        enabled_channel_count: enabled_channels.len(),
        max_enabled_channels: active_mode.max_enabled_channels,
    });
}

if !active_mode.supported_sample_rates.contains(&request.sample_rate_hz) {
    return Err(CaptureConfigError::UnsupportedSampleRate {
        sample_rate_hz: request.sample_rate_hz,
        mode_name: active_mode.name.clone(),
    });
}
```
[VERIFIED: `crates/dsview-core/src/capture_config.rs`]

### DSView samplerate clamping is keyed by current channel mode
```c
// Source: DSView/libsigrok4DSL/hardware/DSL/dsl.c
for (i = 0; devc->profile->dev_caps.samplerates[i]; i++) {
    if (devc->profile->dev_caps.samplerates[i] >
            channel_modes[devc->ch_mode].max_samplerate)
        break;
}
devc->samplerates_max_index = i - 1;
```
[VERIFIED: `DSView/libsigrok4DSL/hardware/DSL/dsl.c`]

### Restore-on-exit mode probing already exists in the sys boundary
```c
// Source: crates/dsview-sys/bridge_runtime.c
status = dsview_bridge_set_int16_config(SR_CONF_OPERATION_MODE, operation_mode_code);
if (status != SR_OK) {
    goto restore;
}

status = dsview_bridge_copy_channel_modes_for_current_operation(
    group->channel_modes,
    DSVIEW_CHANNEL_MODE_CAPACITY,
    &group->channel_mode_count);
```
[VERIFIED: `crates/dsview-sys/bridge_runtime.c`]

### CLI error-code assertions are already the house style
```rust
// Source: crates/dsview-cli/src/main.rs
#[test]
fn invalid_handle_maps_to_stable_error_code() {
    assert_eq!(invalid_handle_error().code, "invalid_selector");
}
```
[VERIFIED: `crates/dsview-cli/src/main.rs`]

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `CaptureConfigRequest` + `CaptureCapabilities` validate only sample rate, sample limit, and enabled channels for the active/default channel mode. [VERIFIED: `crates/dsview-core/src/capture_config.rs` + `crates/dsview-core/src/lib.rs`] | Phase 11 should validate a unified device-option request against a selected-device, mode-aware capability snapshot that includes operation mode, channel mode, stop option, filter, threshold, and per-mode samplerates. [VERIFIED: `.planning/ROADMAP.md` + `.planning/REQUIREMENTS.md`] [ASSUMED] | Phase 10 completed on 2026-04-10 and created the stable discovery foundation this phase must build on. [VERIFIED: `.planning/STATE.md` + `.planning/ROADMAP.md`] | Planning should treat this as a new domain model, not a small patch to the old capture validator. [VERIFIED: current code vs roadmap scope] |
| Validation errors are stringified into `runtime_error`. [VERIFIED: `crates/dsview-cli/src/main.rs`] | Validation errors should become a first-class enum with stable machine codes. [VERIFIED: Phase 11 success criteria in `.planning/ROADMAP.md`] [ASSUMED] | Needed now in Phase 11. [VERIFIED: `.planning/ROADMAP.md`] | Without this, the CLI cannot satisfy success criterion 3 even if the underlying validation logic is correct. [VERIFIED: `.planning/ROADMAP.md` + `crates/dsview-cli/src/main.rs`] |
| Public discovery and internal validation facts are currently conflated around whichever device/mode is active. [VERIFIED: `crates/dsview-core/src/lib.rs` + `crates/dsview-core/src/device_options.rs`] | Keep public discovery stable and add a richer internal validation-capability path. [VERIFIED: `.planning/STATE.md`] [ASSUMED] | Triggered by Phase 10 completion and Phase 11 scope. [VERIFIED: `.planning/STATE.md` + `.planning/ROADMAP.md`] | Avoids breaking automation that already consumes `devices options`. [VERIFIED: `.planning/STATE.md` + `crates/dsview-cli/src/device_options.rs`] |

**Deprecated/outdated:**

- Reusing `CaptureCapabilities` as the future source of truth for all DSLogic Plus option validation is outdated for this milestone because it has no place for stop option, filter, threshold, or per-operation-mode channel-mode groups. [VERIFIED: `crates/dsview-core/src/capture_config.rs` + `.planning/ROADMAP.md`]
- Letting validation errors surface only as free-form text is outdated because Phase 11 explicitly requires stable machine-readable categories. [VERIFIED: `.planning/ROADMAP.md` + `.planning/REQUIREMENTS.md`]

## Assumptions Log

> List all claims tagged `[ASSUMED]` in this research. The planner and discuss-phase use this
> section to identify decisions that need user confirmation before execution.

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | The cleanest Phase 11 request shape is a unified struct carrying stable Phase 10 IDs for option selections plus numeric capture fields. | Architecture Patterns | Low-Medium - planner may choose a different internal field layout, but validation logic still stands. |
| A2 | The core validation error enum should expose codes like `invalid_operation_mode`, `stop_option_incompatible`, and `sample_rate_unsupported`. | Architecture Patterns | Medium - planner may pick different exact code strings, but the stable taxonomy requirement remains. |
| A3 | The richer validation-capability snapshot should stay internal rather than extending the public Phase 10 discovery response. | Standard Stack / State of the Art | Low - exposing it publicly would still work technically, but it would increase contract churn. |
| A4 | Explicit stop-option selection should likely be treated as buffer-mode-only behavior unless live-device testing proves DSView meaningfully honors it in stream/internal-test modes. | Resolved Planning Decisions | Low - this is now a deliberate milestone rule rather than an unresolved blocker, and later phases can relax it if fresh evidence appears. |

## Resolved Planning Decisions

1. **Stop-option compatibility is a deliberate Phase 11 planning rule: explicit stop-option selection is buffer-mode-only for this milestone.**
   - Grounding: DSView exposes `SR_CONF_BUFFER_OPTIONS` from a static `bufoption_list`, the roadmap explicitly requires rejection of "mode-incompatible stop-option values", and `DSView/NEWS31` describes upload/stop-option behavior as a buffer-mode workflow. [VERIFIED: `DSView/libsigrok4DSL/hardware/DSL/dslogic.c` + `.planning/ROADMAP.md` + `DSView/NEWS31`]
   - Planning decision: Model stop-option compatibility in the Phase 11 capability snapshot so buffer mode advertises the stop-option IDs and non-buffer modes advertise an empty compatibility set. The validator should therefore return `stop_option_incompatible` when a request supplies a stop option outside buffer mode. [RESOLVED: 2026-04-13 planning revision]
   - Follow-on note: If later live-device verification proves DSView honors explicit stop-option changes in additional modes, Phase 12 or 13 can widen the compatibility set without changing the public Phase 10 discovery schema. [ASSUMED]

2. **Threshold step validation is resolved as epsilon-tolerant step checking.**
   - Grounding: DSView binds `SR_CONF_VTH` with a `0.0..5.0` range and `0.1` step, and Phase 10 already exports that same voltage-range contract. [VERIFIED: `DSView/DSView/pv/prop/binding/deviceoptions.cpp` + `crates/dsview-sys/bridge_runtime.c` + `crates/dsview-core/src/device_options.rs`]
   - Planning decision: Validate `threshold_volts` by checking the range first, then comparing the normalized step count `(value - min_volts) / step_volts` against the nearest integer with a small epsilon tolerance so values such as `1.799999999` are accepted while true off-step values still fail with `threshold_step_invalid`. [RESOLVED: 2026-04-13 planning revision]
   - Test expectation: Phase 11 tests should include both a normalized near-step value that passes and a genuinely off-step value that fails deterministically. [RESOLVED: 2026-04-13 planning revision]

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo` | Build and run Phase 11 tests | ✓ | `1.94.1` [VERIFIED: local `cargo --version`] | — |
| `rustc` | Compile new core/sys/CLI validation types | ✓ | `1.94.1` [VERIFIED: local `rustc --version`] | — |

**Missing dependencies with no fallback:**
- None identified for automated Phase 11 implementation work. [VERIFIED: local tool audit]

**Missing dependencies with fallback:**
- Live `DSLogic Plus` hardware availability was not audited in this session; automated sys/core tests already use mocks as the fallback until manual verification. [VERIFIED: `crates/dsview-sys/tests/device_options.rs` + `.planning/phases/10-device-option-bridge-and-discovery/10-VERIFICATION.md`]

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness via `cargo test` [VERIFIED: workspace layout + `cargo test -q --workspace -- --list`] |
| Config file | none - standard Cargo test layout [VERIFIED: workspace layout] |
| Quick run command | `cargo test -p dsview-core capture_config -- --nocapture` [VERIFIED: local command output] |
| Full suite command | `cargo test --workspace` [VERIFIED: existing workspace test layout + `cargo test -q --workspace -- --list`] |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| VAL-01 | Reject invalid operation-mode/channel-mode/sample-rate/sample-limit/enabled-channel combinations before capture. [VERIFIED: `.planning/REQUIREMENTS.md`] | unit/integration | `cargo test -p dsview-core --test device_option_validation -- --nocapture` [ASSUMED] | created in `11-01` as scaffold, expanded in `11-02` |
| VAL-02 | Reject unsupported threshold, filter, or mode-incompatible stop-option selections with stable machine-readable categories. [VERIFIED: `.planning/REQUIREMENTS.md`] | unit + CLI mapping | `cargo test -p dsview-core --test device_option_validation -- --nocapture && cargo test -p dsview-cli --lib stable_validation_error_codes -- --nocapture` [ASSUMED] | core target scaffolded in `11-01`; initial CLI stable-code assertions reserved in `11-01` and expanded in `11-02` |

### Sampling Rate

- **Per task commit:** Use `cargo test -p dsview-core capture_config -- --nocapture` only until `11-01` lands the new Phase 11 scaffold; after that, use `cargo test -p dsview-core --test device_option_validation -- --nocapture` plus the smallest relevant CLI lib test when validation-code assertions are touched. [VERIFIED: local command output] [ASSUMED]
- **Per wave merge:** `cargo test -p dsview-sys --test device_options && cargo test -p dsview-core && cargo test -p dsview-cli` [VERIFIED: existing crate test targets + Phase 10 validation pattern in `.planning/phases/10-device-option-bridge-and-discovery/10-VALIDATION.md`]
- **Phase gate:** Full workspace suite green before `/gsd-verify-work`. [VERIFIED: `.planning/config.json` sets `workflow.nyquist_validation` true + current workspace test harness]

### Wave 0 / predecessor-plan requirements

- [ ] `crates/dsview-core/src/device_option_validation.rs` — new unified request/capability/error model for `VAL-01` and `VAL-02`. [VERIFIED: file does not exist in current workspace]
- [ ] `crates/dsview-core/tests/device_option_validation.rs` — scaffold the dedicated Phase 11 test target in `11-01`, then add the first requirement-specific RED cases in `11-02` before validator logic is finalized. [VERIFIED: file does not exist in current workspace]
- [ ] `crates/dsview-cli/src/main.rs` validation-code tests — reserve initial stable validation-code assertions in `11-01`, then expand them in `11-02` before wiring the capture validation path. [VERIFIED: stable-code tests exist, but no validation-specific code assertions exist yet in `crates/dsview-cli/src/main.rs`]
- [ ] `crates/dsview-sys` capability-loading coverage if the sys boundary gains per-mode samplerate probing for validation. [VERIFIED: current sys tests cover option grouping and restore-on-exit, but not a Phase 11 validation snapshot path, in `crates/dsview-sys/tests/device_options.rs`] 

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no [VERIFIED: Phase 11 scope is local CLI validation only in `.planning/ROADMAP.md` + `.planning/REQUIREMENTS.md`] | None required for this phase. [VERIFIED: no auth features in scope] |
| V3 Session Management | no [VERIFIED: Phase 11 scope does not introduce sessions in `.planning/ROADMAP.md`] | None required for this phase. [VERIFIED: no session features in scope] |
| V4 Access Control | no [VERIFIED: Phase 11 scope does not introduce authorization boundaries in `.planning/ROADMAP.md`] | None required for this phase. [VERIFIED: no access-control features in scope] |
| V5 Input Validation | yes [VERIFIED: `VAL-01` and `VAL-02` are explicit pre-acquisition validation requirements in `.planning/REQUIREMENTS.md`] | Typed Rust request structs, allowlist validation against DSView-backed capabilities, numeric range checks, and stable error taxonomy in `dsview-core`. [VERIFIED: existing validation pattern in `crates/dsview-core/src/capture_config.rs`] [ASSUMED] |
| V6 Cryptography | no [VERIFIED: no crypto concerns appear in this phase scope or current codepaths] | None required for this phase. [VERIFIED: `.planning/ROADMAP.md` + current workspace layout] |

### Known Threat Patterns for this stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Unsupported option IDs or translated labels reaching the runtime | Tampering | Validate only against Phase 10 stable IDs and retain native codes internally for apply-time use. [VERIFIED: `crates/dsview-core/src/device_options.rs`] |
| Out-of-range channels or enabled-channel count exceeding the active mode limit | Tampering / DoS | Keep `BTreeSet<u16>` input, check against total channel count and per-mode `max_enabled_channels`, and reject before any acquisition setup. [VERIFIED: `crates/dsview-core/src/capture_config.rs` + `DSView/libsigrok4DSL/hardware/DSL/dsl.h`] |
| Oversized sample limits exhausting device depth | DoS | Align requested limits and compare against `hw_depth / enabled_channels`, mirroring DSView's depth math. [VERIFIED: `crates/dsview-core/src/capture_config.rs` + `DSView/libsigrok4DSL/hardware/DSL/dsl.c` + `DSView/libsigrok4DSL/libsigrok.h`] |
| Threshold or filter values outside the supported DSView contract | Tampering | Validate threshold against the VTH range/step and filter membership against the discovered list before runtime calls. [VERIFIED: `DSView/DSView/pv/prop/binding/deviceoptions.cpp` + `DSView/libsigrok4DSL/hardware/DSL/dslogic.c` + `crates/dsview-core/src/device_options.rs`] |

## Sources

### Primary (HIGH confidence)
- `.planning/ROADMAP.md` - Phase 11 scope, success criteria, sequencing, and milestone boundaries. [VERIFIED]
- `.planning/REQUIREMENTS.md` - `VAL-01` / `VAL-02` requirement text and out-of-scope boundaries. [VERIFIED]
- `.planning/STATE.md` - carry-forward Phase 10 decisions, especially preserving the stable discovery contract. [VERIFIED]
- `CLAUDE.md` - project constraints around DSView immutability, unsafe-boundary ownership, and machine-readable CLI behavior. [VERIFIED]
- `crates/dsview-core/src/capture_config.rs` - current validation pattern and its present scope limits. [VERIFIED]
- `crates/dsview-core/src/device_options.rs` - Phase 10 stable IDs, grouped channel modes, and threshold capability model. [VERIFIED]
- `crates/dsview-core/src/lib.rs` - current discovery/open/validate paths and the first-device validation gap. [VERIFIED]
- `crates/dsview-cli/src/main.rs` - current error-code mapping and the generic `runtime_error` fallback for validation failures. [VERIFIED]
- `crates/dsview-sys/bridge_runtime.c` - restore-on-exit option probing pattern already available in the sys boundary. [VERIFIED]
- `DSView/libsigrok4DSL/hardware/DSL/dsl.h` - DSLogic Plus channel-mode table, `vld_num`, `max_samplerate`, supported modes, and hardware depth. [VERIFIED]
- `DSView/libsigrok4DSL/hardware/DSL/dsl.c` - samplerate clamping and channel-depth math. [VERIFIED]
- `DSView/libsigrok4DSL/hardware/DSL/dslogic.c` - static option lists, current-value getters, config setters, and device-option session surface. [VERIFIED]
- `DSView/DSView/pv/prop/binding/deviceoptions.cpp` - VTH GUI range contract (`0.0..5.0`, step `0.1`). [VERIFIED]
- `DSView/NEWS31` - release note describing buffer-mode upload semantics for stop options. [VERIFIED]

### Secondary (MEDIUM confidence)
- `.planning/phases/10-device-option-bridge-and-discovery/10-VALIDATION.md` - existing Nyquist validation pattern for nearby work. [VERIFIED]
- `.planning/phases/10-device-option-bridge-and-discovery/10-VERIFICATION.md` - confirmed live-device Phase 10 behavior and residual-risk notes. [VERIFIED]

### Tertiary (LOW confidence)
- None. All factual claims in this research were verified from the repo or explicitly marked `[ASSUMED]`. [VERIFIED]

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - all recommended components already exist in the repo and their roles are directly verified from source. [VERIFIED: workspace `Cargo.toml` files + source files]
- Architecture: MEDIUM - the need for a richer, selected-device, mode-aware model is strongly verified, but exact internal type names and exact error-code strings remain implementation choices. [VERIFIED: current code + roadmap scope] [ASSUMED]
- Pitfalls: HIGH - the major failure modes are visible directly in current Rust code and DSView's upstream mode/scaler logic. [VERIFIED: `crates/dsview-core/src/lib.rs` + `crates/dsview-cli/src/main.rs` + `DSView/libsigrok4DSL/hardware/DSL/dsl.c` + `DSView/libsigrok4DSL/hardware/DSL/dsl.h`]

**Research date:** 2026-04-13
**Valid until:** 2026-05-13 for repo-local facts; re-check before Phase 12 if upstream DSView submodule or workspace dependencies change. [VERIFIED: local repo facts] [ASSUMED]
