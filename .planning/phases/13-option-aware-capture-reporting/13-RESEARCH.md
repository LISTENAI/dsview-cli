# Phase 13: Option-Aware Capture Reporting - Research

**Researched:** 2026-04-13 [VERIFIED: environment_context]
**Domain:** DSLogic Plus runtime option application, effective-value reporting, and capture artifact schema extension in the existing Rust workspace. [VERIFIED: .planning/phases/13-option-aware-capture-reporting/13-CONTEXT.md] [VERIFIED: crates/dsview-core/src/lib.rs] [VERIFIED: crates/dsview-cli/src/main.rs] [VERIFIED: crates/dsview-sys/src/lib.rs]
**Confidence:** MEDIUM-HIGH. [VERIFIED: .planning/phases/13-option-aware-capture-reporting/13-CONTEXT.md] [VERIFIED: crates/dsview-core/src/lib.rs] [VERIFIED: crates/dsview-cli/src/main.rs] [VERIFIED: crates/dsview-sys/src/lib.rs]

<user_constraints>
## User Constraints (from CONTEXT.md) [VERIFIED: .planning/phases/13-option-aware-capture-reporting/13-CONTEXT.md]

### Locked Decisions
### Apply strategy
- **D-01:** Apply device options in a fixed deterministic order before acquisition begins. [VERIFIED: .planning/phases/13-option-aware-capture-reporting/13-CONTEXT.md]
- **D-02:** If any apply step fails, stop immediately instead of attempting a full option rollback. [VERIFIED: .planning/phases/13-option-aware-capture-reporting/13-CONTEXT.md]
- **D-03:** On apply failure, the CLI must explicitly report which option changes already succeeded and which option failed, because the device may be left in a partially updated state. [VERIFIED: .planning/phases/13-option-aware-capture-reporting/13-CONTEXT.md]

### Apply ordering
- **D-04:** Apply mode-defining options first, then dependent device options, then channel/capture settings. [VERIFIED: .planning/phases/13-option-aware-capture-reporting/13-CONTEXT.md]
- **D-05:** Preferred apply sequence is:
  1. operation mode
  2. stop option
  3. channel mode
  4. threshold
  5. filter
  6. enabled channels
  7. sample limit
  8. sample rate [VERIFIED: .planning/phases/13-option-aware-capture-reporting/13-CONTEXT.md]

### Reporting shape
- **D-06:** CLI text output should stay concise and focus on the effective option values actually used for the run. [VERIFIED: .planning/phases/13-option-aware-capture-reporting/13-CONTEXT.md]
- **D-07:** JSON output and metadata should include both requested and effective device-option values. [VERIFIED: .planning/phases/13-option-aware-capture-reporting/13-CONTEXT.md]
- **D-08:** Effective values should at minimum include operation mode, stop option, channel mode, enabled channels, threshold volts, filter, sample rate, and sample limit. [VERIFIED: .planning/phases/13-option-aware-capture-reporting/13-CONTEXT.md]

### Apply semantics
- **D-09:** Phase 13 should explicitly apply the full validated request, including values inherited from current device state, rather than only applying fields the user explicitly changed. [VERIFIED: .planning/phases/13-option-aware-capture-reporting/13-CONTEXT.md]
- **D-10:** Deterministic full apply is preferred over partial “changed-only” apply, even when some values match the device’s prior state. [VERIFIED: .planning/phases/13-option-aware-capture-reporting/13-CONTEXT.md]

### Claude's Discretion
- Exact struct naming and serialization layout for requested/effective option facts, as long as text stays concise and JSON/metadata remain explicit. [VERIFIED: .planning/phases/13-option-aware-capture-reporting/13-CONTEXT.md]
- Exact error code and detail-field layout for partial apply failures, as long as successful steps and the failing step are both surfaced clearly. [VERIFIED: .planning/phases/13-option-aware-capture-reporting/13-CONTEXT.md]
- Whether the CLI text output uses one compact summary block or multiple lines for effective option reporting. [VERIFIED: .planning/phases/13-option-aware-capture-reporting/13-CONTEXT.md]

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within the Phase 13 boundary. [VERIFIED: .planning/phases/13-option-aware-capture-reporting/13-CONTEXT.md]
</user_constraints>

<phase_requirements>
## Phase Requirements [VERIFIED: .planning/REQUIREMENTS.md]

| ID | Description | Research Support |
|----|-------------|------------------|
| RUN-04 | Capture applies the selected DSView-compatible device options before acquisition begins. [VERIFIED: .planning/REQUIREMENTS.md] | Add a core-layer option-aware capture path that consumes the already validated Phase 11/12 request, exposes public setters from `dsview-sys` for the missing DSView options, applies the full request in the locked order, and stops immediately with partial-apply detail if any setter fails. [VERIFIED: .planning/phases/13-option-aware-capture-reporting/13-CONTEXT.md] [VERIFIED: crates/dsview-core/src/device_option_validation.rs] [VERIFIED: crates/dsview-core/src/lib.rs] [VERIFIED: crates/dsview-sys/src/lib.rs] [VERIFIED: crates/dsview-sys/bridge_runtime.c] |
| RUN-05 | CLI success output and machine-readable metadata record the effective device option values used for the run. [VERIFIED: .planning/REQUIREMENTS.md] | Extend the existing serde-backed capture response and metadata schema with a shared requested/effective facts model, build those facts from the same core source, and keep text output focused on effective values only. [VERIFIED: .planning/phases/13-option-aware-capture-reporting/13-CONTEXT.md] [VERIFIED: crates/dsview-cli/src/main.rs] [VERIFIED: crates/dsview-core/src/lib.rs] [VERIFIED: crates/dsview-core/tests/export_artifacts.rs] [CITED: https://serde.rs/field-attrs.html] |
</phase_requirements>

## Summary

Phase 13 is not a new validation phase; the codebase already has a full selected-device validation model and a capture-side resolver that produces a complete `DeviceOptionValidationRequest` with stable IDs, native codes, aligned sample limits, and preserved current-state fallbacks before runtime work begins. [VERIFIED: .planning/phases/11-device-option-validation-model/11-02-SUMMARY.md] [VERIFIED: .planning/phases/12-cli-device-option-surface/12-02-SUMMARY.md] [VERIFIED: crates/dsview-cli/src/capture_device_options.rs] [VERIFIED: crates/dsview-core/src/device_option_validation.rs]

The implementation gap is the runtime/application layer. Today `dsview-core` only applies enabled channels, sample limit, and sample rate, while `dsview-sys` only exposes setters for those three values. Operation mode, stop option, channel mode, threshold volts, and filter are discoverable and validatable, but they are not yet publicly settable through the Rust/native boundary. [VERIFIED: crates/dsview-core/src/lib.rs] [VERIFIED: crates/dsview-sys/src/lib.rs] [VERIFIED: crates/dsview-sys/bridge_runtime.c]

The biggest planning risk is that the current capture path still assumes the device’s current channel mode is the truth. `run_capture(...)` re-validates `CaptureConfigRequest` against the current runtime state, and `acquisition_preflight(...)` mutates device configuration by calling `apply_capture_config(...)` before the main capture session starts. For an option-aware path, that would either reject valid requested mode changes or apply untracked changes before the deterministic Phase 13 sequence begins. [VERIFIED: crates/dsview-core/src/lib.rs]

**Primary recommendation:** Plan Phase 13 around one new core “option-aware capture” path that accepts the already validated device-option request, derives both the runtime setter sequence and the requested/effective reporting facts from that single source, bypasses the current mutating preflight behavior for option-aware runs, and expands the `dsview-sys` mock/test surface before wiring CLI output and metadata. [VERIFIED: .planning/phases/13-option-aware-capture-reporting/13-CONTEXT.md] [VERIFIED: crates/dsview-core/src/device_option_validation.rs] [VERIFIED: crates/dsview-core/src/lib.rs] [VERIFIED: crates/dsview-cli/src/main.rs] [VERIFIED: crates/dsview-sys/bridge_runtime.c]

## Project Constraints (from CLAUDE.md)

- Treat `DSView/` as upstream dependency code and do not modify it for normal project work. [VERIFIED: CLAUDE.md]
- Keep unsafe/native integration isolated behind a small boundary. [VERIFIED: CLAUDE.md]
- Favor scriptable CLI behavior, explicit exit codes, and machine-readable outputs. [VERIFIED: CLAUDE.md]
- Scope the milestone to `DSLogic Plus` only. [VERIFIED: CLAUDE.md] [VERIFIED: .planning/PROJECT.md]
- Preserve the shipped `v1.0` capture/export workflow while adding richer option control and reporting. [VERIFIED: CLAUDE.md] [VERIFIED: .planning/PROJECT.md] [VERIFIED: .planning/ROADMAP.md]
- Keep GSD workflow artifacts in sync; planning should stay inside the GSD flow rather than recommending ad-hoc repo edits. [VERIFIED: CLAUDE.md]

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `dsview-sys` | `0.1.0` [VERIFIED: crates/dsview-sys/Cargo.toml] | Native bridge for DSView/libsigrok4DSL getters/setters, capture start/stop, and VCD export. [VERIFIED: crates/dsview-sys/src/lib.rs] | This crate is already the project’s only unsafe/native boundary and already owns option discovery plus the existing capture setters, so new DSLogic option setters belong here rather than in `dsview-core` or `dsview-cli`. [VERIFIED: CLAUDE.md] [VERIFIED: crates/dsview-sys/src/lib.rs] [VERIFIED: crates/dsview-sys/bridge_runtime.c] |
| `dsview-core` | `0.1.0` [VERIFIED: crates/dsview-core/Cargo.toml] | Safe orchestration for validation, capture sequencing, export, and metadata. [VERIFIED: crates/dsview-core/src/lib.rs] | This crate already owns `Discovery`, `CaptureRunRequest`, `CaptureExportRequest`, `CaptureMetadata`, and the stable validation model, so the Phase 13 orchestration and requested/effective fact model should live here. [VERIFIED: crates/dsview-core/src/lib.rs] [VERIFIED: crates/dsview-core/src/device_option_validation.rs] |
| `dsview-cli` | `0.1.0` [VERIFIED: crates/dsview-cli/Cargo.toml] | User-facing capture command, error mapping, and text/JSON success rendering. [VERIFIED: crates/dsview-cli/src/main.rs] | The CLI already owns capture response shapes and error serialization patterns locked by integration tests, so Phase 13 should extend those types instead of inventing a new output channel. [VERIFIED: crates/dsview-cli/src/main.rs] [VERIFIED: crates/dsview-cli/tests/capture_cli.rs] |
| `serde` | `1.0.228` [VERIFIED: Cargo.lock] | Stable serialization for CLI JSON responses and metadata sidecars. [VERIFIED: crates/dsview-cli/src/main.rs] [VERIFIED: crates/dsview-core/src/lib.rs] | The project already relies on serde-derived structs for machine-readable output, and official serde field attributes support additive nested reporting without hand-rolled JSON. [VERIFIED: crates/dsview-cli/src/main.rs] [VERIFIED: crates/dsview-core/src/lib.rs] [CITED: https://serde.rs/field-attrs.html] |
| `serde_json` | `1.0.149` [VERIFIED: Cargo.lock] | JSON encoding for CLI success/error output and metadata emission. [VERIFIED: crates/dsview-cli/src/main.rs] [VERIFIED: crates/dsview-core/src/lib.rs] | JSON is already the stable automation surface for this project, so requested/effective option facts should extend the existing structs rather than introducing a second format. [VERIFIED: CLAUDE.md] [VERIFIED: crates/dsview-cli/src/main.rs] |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `thiserror` | `2.0.18` [VERIFIED: Cargo.lock] | Typed error enums across core and sys layers. [VERIFIED: crates/dsview-core/Cargo.toml] [VERIFIED: crates/dsview-sys/Cargo.toml] | Use for the new partial-apply error enum and any richer bridge error mapping instead of raw strings. [VERIFIED: crates/dsview-core/src/lib.rs] |
| `time` | `0.3.47` [VERIFIED: Cargo.lock] | Metadata timestamp formatting. [VERIFIED: crates/dsview-core/Cargo.toml] [VERIFIED: crates/dsview-core/src/lib.rs] | Reuse when extending metadata rather than changing timestamp formatting behavior. [VERIFIED: crates/dsview-core/src/lib.rs] |
| `assert_cmd` | `2.2.0` [VERIFIED: Cargo.lock] | Spawned CLI regression coverage. [VERIFIED: crates/dsview-cli/Cargo.toml] [VERIFIED: crates/dsview-cli/tests/capture_cli.rs] | Use for end-to-end capture output/error contract tests after the new reporting fields land. [VERIFIED: crates/dsview-cli/tests/capture_cli.rs] |
| `predicates` | `3.1.4` [VERIFIED: Cargo.lock] | CLI stdout/stderr assertions. [VERIFIED: Cargo.lock] [VERIFIED: crates/dsview-cli/tests/capture_cli.rs] | Use for concise text-mode assertions and JSON substring checks in spawned CLI tests. [VERIFIED: crates/dsview-cli/tests/capture_cli.rs] |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Extending `dsview-sys` with new public setters. [VERIFIED: crates/dsview-sys/src/lib.rs] | Modifying the upstream `DSView/` submodule. [VERIFIED: .planning/PROJECT.md] | Out of scope and contrary to project constraints. [VERIFIED: CLAUDE.md] [VERIFIED: .planning/PROJECT.md] |
| Driving reporting from one shared core facts model. [VERIFIED: crates/dsview-core/src/lib.rs] | Building separate requested/effective shapes independently in CLI response code and metadata code. [VERIFIED: crates/dsview-cli/src/main.rs] [VERIFIED: crates/dsview-core/src/lib.rs] | Separate shaping increases drift risk because Phase 13 must keep text, JSON, and metadata explicit and aligned. [VERIFIED: .planning/phases/13-option-aware-capture-reporting/13-CONTEXT.md] [VERIFIED: crates/dsview-cli/src/main.rs] [VERIFIED: crates/dsview-core/src/lib.rs] |
| Reusing `ValidatedDeviceOptionRequest` native codes for apply-time setters. [VERIFIED: crates/dsview-core/src/device_option_validation.rs] | Re-resolving labels or stable IDs back into runtime values during apply. [VERIFIED: crates/dsview-cli/src/capture_device_options.rs] | Re-resolution would duplicate the Phase 11/12 mapping logic and create extra failure modes even though the validated request already carries native codes. [VERIFIED: .planning/phases/11-device-option-validation-model/11-02-SUMMARY.md] [VERIFIED: crates/dsview-core/src/device_option_validation.rs] |

**Installation:** No new third-party crate is required for Phase 13 planning; the existing workspace dependencies and local Rust/C build toolchain are sufficient. [VERIFIED: crates/dsview-cli/Cargo.toml] [VERIFIED: crates/dsview-core/Cargo.toml] [VERIFIED: crates/dsview-sys/Cargo.toml] [VERIFIED: cargo test]

**Version verification:** The versions above are the versions currently declared and locked in this workspace, verified from `Cargo.toml`, `Cargo.lock`, and live local tool output. [VERIFIED: Cargo.lock] [VERIFIED: crates/dsview-cli/Cargo.toml] [VERIFIED: crates/dsview-core/Cargo.toml] [VERIFIED: crates/dsview-sys/Cargo.toml] [VERIFIED: cargo --version] [VERIFIED: rustc --version]

## Architecture Patterns

### Recommended Project Structure
```text
crates/
├── dsview-sys/
│   ├── src/lib.rs              # Public runtime setters/getters for DSView config keys
│   ├── bridge_runtime.c        # SR_CONF_* apply shims and test mock extensions
│   └── tests/device_options.rs # Low-level setter/readback/order regression harness
├── dsview-core/
│   ├── src/device_option_validation.rs # Existing validated request model reused by Phase 13
│   ├── src/lib.rs                      # Option-aware capture orchestration + metadata facts
│   ├── tests/export_artifacts.rs       # Metadata schema / requested-effective reporting tests
│   └── tests/acquisition.rs            # Capture error-shape regression tests
└── dsview-cli/
    ├── src/main.rs              # Capture success/error response rendering
    └── tests/capture_cli.rs     # Spawned CLI output contract tests
```
This structure keeps new native setters in `dsview-sys`, orchestration/report facts in `dsview-core`, and only presentation concerns in `dsview-cli`, which matches the existing crate boundaries. [VERIFIED: CLAUDE.md] [VERIFIED: crates/dsview-sys/src/lib.rs] [VERIFIED: crates/dsview-core/src/lib.rs] [VERIFIED: crates/dsview-cli/src/main.rs]

### Pattern 1: Use the validated device-option request as the single apply contract
**What:** `ValidatedDeviceOptionRequest` already contains the stable IDs, native codes, aligned sample-limit result, and sorted enabled-channel list that Phase 13 needs for both runtime application and reporting. [VERIFIED: crates/dsview-core/src/device_option_validation.rs]

**When to use:** Use this type as the input to the new option-aware core capture path whenever Phase 12 flags participated in request resolution, including values inherited from current device state. [VERIFIED: .planning/phases/13-option-aware-capture-reporting/13-CONTEXT.md] [VERIFIED: crates/dsview-cli/src/capture_device_options.rs] [VERIFIED: crates/dsview-core/src/device_option_validation.rs]

**Example:**
```rust
pub struct ValidatedDeviceOptionRequest {
    pub operation_mode_id: String,
    pub operation_mode_code: i16,
    pub stop_option_id: Option<String>,
    pub stop_option_code: Option<i16>,
    pub channel_mode_id: String,
    pub channel_mode_code: i16,
    pub sample_rate_hz: u64,
    pub requested_sample_limit: u64,
    pub effective_sample_limit: u64,
    pub enabled_channels: Vec<u16>,
    pub threshold_volts: Option<f64>,
    pub filter_id: Option<String>,
    pub filter_code: Option<i16>,
}
```
Source: `crates/dsview-core/src/device_option_validation.rs`. [VERIFIED: crates/dsview-core/src/device_option_validation.rs]

### Pattern 2: Split readiness checks from the mutating apply sequence
**What:** The option-aware path should not reuse the current `acquisition_preflight(...)` side effects as the main Phase 13 apply step because the existing preflight already mutates the device by calling `apply_capture_config(...)` before session preparation. [VERIFIED: crates/dsview-core/src/lib.rs]

**When to use:** Any Phase 13 path that must provide deterministic apply ordering, partial-apply reporting, or truthful effective-value reporting before acquisition starts. [VERIFIED: .planning/phases/13-option-aware-capture-reporting/13-CONTEXT.md] [VERIFIED: crates/dsview-core/src/lib.rs]

**Example:**
```rust
if let Ok(validated) = capabilities.validate_request(request) {
    config_apply_ready = self
        .apply_capture_config(&validated, capabilities.total_channel_count)
        .is_ok();
}
```
This is the current hidden mutation inside `acquisition_preflight(...)`; Phase 13 should branch away from this pattern for option-aware runs. [VERIFIED: crates/dsview-core/src/lib.rs]

### Pattern 3: Build requested/effective reporting from one shared serde-backed core model
**What:** The project already serializes capture artifacts through nested serde structs in `dsview-core` and `dsview-cli`; Phase 13 should extend that pattern with one shared requested/effective facts model so text, CLI JSON, and metadata stay aligned. [VERIFIED: crates/dsview-core/src/lib.rs] [VERIFIED: crates/dsview-cli/src/main.rs] [CITED: https://serde.rs/field-attrs.html]

**When to use:** Use this pattern for `RUN-05`, especially when the same fact set must appear in CLI JSON and metadata but text should stay concise. [VERIFIED: .planning/phases/13-option-aware-capture-reporting/13-CONTEXT.md]

**Example:**
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CaptureMetadata {
    pub schema_version: u32,
    pub tool: MetadataToolInfo,
    pub capture: MetadataCaptureInfo,
    pub acquisition: MetadataAcquisitionInfo,
    pub artifacts: MetadataArtifactInfo,
}
```
Source: `crates/dsview-core/src/lib.rs`. [VERIFIED: crates/dsview-core/src/lib.rs]

### Anti-Patterns to Avoid
- **Reusing `run_capture(...)` unchanged for option-aware requests:** The current path re-validates `CaptureConfigRequest` against the device’s current active channel mode, which can reject a mode change that Phase 11 already validated against the selected requested mode. [VERIFIED: crates/dsview-core/src/lib.rs] [VERIFIED: crates/dsview-core/src/capture_config.rs] [VERIFIED: crates/dsview-core/src/device_option_validation.rs]
- **Letting preflight perform untracked mutations:** The current readiness probe already applies capture config once; Phase 13 cannot produce honest partial-apply reporting if setters run before the tracked sequence starts. [VERIFIED: crates/dsview-core/src/lib.rs]
- **Reporting only labels or only text for partial failures:** D-03 requires explicit machine-readable detail about which steps succeeded and which failed. [VERIFIED: .planning/phases/13-option-aware-capture-reporting/13-CONTEXT.md]
- **Hand-copying requested/effective values into both CLI and metadata independently:** The current code already has separate success structs in CLI and metadata structs in core, so duplicate assembly would create drift unless both are sourced from one shared facts object. [VERIFIED: crates/dsview-cli/src/main.rs] [VERIFIED: crates/dsview-core/src/lib.rs]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Device-option compatibility rules. [VERIFIED: crates/dsview-core/src/device_option_validation.rs] | A second matrix of CLI-side `if/else` compatibility checks. [VERIFIED: crates/dsview-cli/src/main.rs] | `DeviceOptionValidationCapabilities::validate_request(...)`. [VERIFIED: crates/dsview-core/src/device_option_validation.rs] | The validator already enforces operation-mode, stop-option, channel-mode, channel-count, sample-rate, threshold, and filter constraints and returns stable error codes. [VERIFIED: .planning/phases/11-device-option-validation-model/11-02-SUMMARY.md] [VERIFIED: crates/dsview-core/src/device_option_validation.rs] |
| Stable-ID-to-native-code mapping at apply time. [VERIFIED: crates/dsview-core/src/device_option_validation.rs] | Re-parsing labels or token strings during the setter sequence. [VERIFIED: crates/dsview-cli/src/capture_device_options.rs] | The native codes already stored in `ValidatedDeviceOptionRequest`. [VERIFIED: crates/dsview-core/src/device_option_validation.rs] | Labels are presentation data; the validated request is already the runtime-safe contract. [VERIFIED: crates/dsview-core/src/device_option_validation.rs] |
| Sample-limit alignment and effective-limit math. [VERIFIED: crates/dsview-core/src/device_option_validation.rs] | New ad-hoc arithmetic in the apply/reporting path. [VERIFIED: crates/dsview-core/src/capture_config.rs] | `effective_sample_limit` from the validated request plus the existing alignment helpers. [VERIFIED: crates/dsview-core/src/device_option_validation.rs] [VERIFIED: crates/dsview-core/src/capture_config.rs] | Recomputing this later risks drifting from the Phase 11 rules. [VERIFIED: .planning/phases/11-device-option-validation-model/11-02-SUMMARY.md] |
| Machine-readable success payloads. [VERIFIED: crates/dsview-cli/src/main.rs] | Manual JSON/string assembly. [VERIFIED: crates/dsview-core/src/lib.rs] | Existing serde structs in `dsview-core` and `dsview-cli`. [VERIFIED: crates/dsview-core/src/lib.rs] [VERIFIED: crates/dsview-cli/src/main.rs] | The current JSON surfaces are already serde-backed and testable. [VERIFIED: crates/dsview-cli/tests/capture_cli.rs] [VERIFIED: crates/dsview-core/tests/export_artifacts.rs] |
| Rollback after mid-sequence failure. [VERIFIED: .planning/phases/13-option-aware-capture-reporting/13-CONTEXT.md] | A custom rollback engine that tries to restore prior device state. [VERIFIED: .planning/phases/13-option-aware-capture-reporting/13-CONTEXT.md] | Stop immediately, surface the succeeded steps plus the failing step, and leave recovery to the next explicit open/validate flow. [VERIFIED: .planning/phases/13-option-aware-capture-reporting/13-CONTEXT.md] [VERIFIED: crates/dsview-cli/src/main.rs] | D-02 and D-03 explicitly prefer operational honesty over speculative rollback. [VERIFIED: .planning/phases/13-option-aware-capture-reporting/13-CONTEXT.md] |

**Key insight:** Phase 13 should add runtime sequencing and reporting, not a second rules engine or a second schema assembly path. [VERIFIED: .planning/phases/13-option-aware-capture-reporting/13-CONTEXT.md] [VERIFIED: crates/dsview-core/src/device_option_validation.rs] [VERIFIED: crates/dsview-core/src/lib.rs] [VERIFIED: crates/dsview-cli/src/main.rs]

## Common Pitfalls

### Pitfall 1: Hidden preflight mutation breaks honest apply reporting
**What goes wrong:** The code mutates the device during readiness checks before the tracked Phase 13 apply sequence begins, so failure reporting cannot truthfully say which Phase 13 steps succeeded. [VERIFIED: crates/dsview-core/src/lib.rs]

**Why it happens:** `acquisition_preflight(...)` currently calls `apply_capture_config(...)` as a boolean readiness probe. [VERIFIED: crates/dsview-core/src/lib.rs]

**How to avoid:** For option-aware runs, split preflight into non-mutating environment checks and a single tracked apply sequence that records each successful step in order. [VERIFIED: crates/dsview-core/src/lib.rs] [VERIFIED: .planning/phases/13-option-aware-capture-reporting/13-CONTEXT.md]

**Warning signs:** Tests or logs show setters running before the new partial-apply error path records any successful step names. [VERIFIED: crates/dsview-core/src/lib.rs] [ASSUMED]

### Pitfall 2: Baseline capture validation rejects a valid requested channel mode
**What goes wrong:** A request that was valid in Phase 11/12 can still fail later because `validate_capture_config(...)` and `CaptureCapabilities::active_mode()` are keyed to the device’s current active mode, not the requested mode that Phase 13 intends to apply. [VERIFIED: crates/dsview-core/src/lib.rs] [VERIFIED: crates/dsview-core/src/capture_config.rs] [VERIFIED: crates/dsview-core/src/device_option_validation.rs]

**Why it happens:** The baseline capture path predates option-aware runtime apply and assumes the current device mode remains unchanged. [VERIFIED: .planning/ROADMAP.md] [VERIFIED: crates/dsview-core/src/lib.rs]

**How to avoid:** Derive the capture config for option-aware runs directly from `ValidatedDeviceOptionRequest` or a converted core struct, and do not re-run the old current-state validator on raw CLI config after Phase 11 validation already succeeded. [VERIFIED: crates/dsview-core/src/device_option_validation.rs] [VERIFIED: crates/dsview-core/src/lib.rs]

**Warning signs:** A mode-changing request fails with `sample_rate_unsupported` or `enabled_channels_exceed_mode_limit` even though the selected requested mode should allow it. [VERIFIED: crates/dsview-core/src/device_option_validation.rs] [ASSUMED]

### Pitfall 3: The current sys-layer mock is too narrow for full Phase 13 coverage
**What goes wrong:** Tests look green for discovery/validation, but there is no low-level proof that the full Phase 13 setter order or partial-apply failure path works. [VERIFIED: crates/dsview-sys/tests/device_options.rs] [VERIFIED: crates/dsview-sys/bridge_runtime.c]

**Why it happens:** `dsview_test_mock_set_config(...)` currently handles only `SR_CONF_OPERATION_MODE` and `SR_CONF_CHANNEL_MODE` and records call counts only for those keys, even though Phase 13 also needs threshold, filter, stop option, sample limit, and sample rate application. [VERIFIED: crates/dsview-sys/bridge_runtime.c]

**How to avoid:** Expand the C mock/test seam first so setters can record values, fail on chosen steps, and assert exact order for all Phase 13 config keys. [VERIFIED: crates/dsview-sys/bridge_runtime.c] [VERIFIED: crates/dsview-sys/tests/device_options.rs]

**Warning signs:** Phase 13 tests only assert final JSON text while never asserting the underlying key order or failing-step boundaries in `dsview-sys`. [VERIFIED: crates/dsview-sys/tests/device_options.rs] [ASSUMED]

### Pitfall 4: Effective reporting drifts between CLI output and metadata
**What goes wrong:** One surface reports the requested values while another surface reports the effective values, or field names diverge between CLI JSON and metadata. [VERIFIED: .planning/phases/13-option-aware-capture-reporting/13-CONTEXT.md]

**Why it happens:** The current code builds CLI success payloads in `dsview-cli` and metadata in `dsview-core`, and neither surface currently contains device-option facts. [VERIFIED: crates/dsview-cli/src/main.rs] [VERIFIED: crates/dsview-core/src/lib.rs]

**How to avoid:** Add one core requested/effective facts model and thread it into both the CLI response and `CaptureMetadata` so only text formatting remains CLI-specific. [VERIFIED: crates/dsview-cli/src/main.rs] [VERIFIED: crates/dsview-core/src/lib.rs]

**Warning signs:** The planner proposes separate “add CLI JSON fields” and “add metadata fields” tasks with independent field lists. [VERIFIED: .planning/phases/13-option-aware-capture-reporting/13-CONTEXT.md] [ASSUMED]

### Pitfall 5: Local live-runtime verification is unreliable on this machine
**What goes wrong:** Automated tests or ad-hoc local commands fail in libusb/runtime initialization before the Phase 13 logic can run, which can hide whether a failure came from the new feature or from the environment. [VERIFIED: cargo run]

**Why it happens:** On this machine, `cargo run -p dsview-cli -- devices list --format json` fails during `ds_lib_init` with `LIBUSB_ERROR_OTHER`, while the existing automated capture CLI coverage relies on an env-gated debug fixture and the sys-layer discovery coverage relies on a C mock option API. [VERIFIED: cargo run] [VERIFIED: .planning/phases/12-cli-device-option-surface/12-03-SUMMARY.md] [VERIFIED: crates/dsview-cli/tests/capture_cli.rs] [VERIFIED: crates/dsview-sys/tests/device_options.rs]

**How to avoid:** Use the existing fixture/mock seams for automated Phase 13 regression coverage and reserve true hardware verification for a workstation with working libusb/device access. [VERIFIED: .planning/phases/12-cli-device-option-surface/12-03-SUMMARY.md] [VERIFIED: crates/dsview-cli/tests/capture_cli.rs] [VERIFIED: cargo run]

**Warning signs:** A new test or manual step depends on `devices list` or live capture enumeration without a fixture and fails with `native_call_failed` before reaching Phase 13 logic. [VERIFIED: cargo run]

## Code Examples

Verified patterns from the current codebase and official docs:

### Reuse the validated request that already carries stable IDs and native codes
```rust
let request = resolve_capture_device_option_request(
    &snapshot,
    &capabilities,
    &args.device_options,
    args.sample_rate_hz,
    args.sample_limit,
    &args.channels,
)?;
capabilities.validate_request(&request)?;
```
This Phase 12 pattern already produces the canonical request that Phase 13 should consume instead of rebuilding the option facts later. [VERIFIED: crates/dsview-cli/src/main.rs] [VERIFIED: crates/dsview-cli/src/capture_device_options.rs] [VERIFIED: crates/dsview-core/src/device_option_validation.rs]

### Rich runtime failures already use structured CLI detail fields
```rust
ErrorResponse {
    code: "capture_cleanup_failed",
    message: format!(
        "capture cleanup failed after {during}; the device may require re-open validation"
    ),
    detail: Some(format!(
        "terminal_event={}, stop_error={:?}, release_error={:?}",
        terminal_event_name(summary),
        cleanup.stop_error,
        cleanup.release_error
    )),
    native_error: Some(summary.last_error.name()),
    terminal_event: Some(terminal_event_name(summary)),
    cleanup: Some(capture_cleanup_response(cleanup)),
}
```
Use the same pattern for a new partial-apply failure response instead of inventing an opaque string-only error. [VERIFIED: crates/dsview-cli/src/main.rs]

### Metadata is already produced from nested serde structs in `dsview-core`
```rust
Ok(CaptureMetadata {
    schema_version: 1,
    tool: MetadataToolInfo {
        name: request.tool_name.clone(),
        version: request.tool_version.clone(),
    },
    capture: MetadataCaptureInfo {
        timestamp_utc: capture_timestamp_utc(request.capture_started_at)?,
        device_model: request.device_model.clone(),
        device_stable_id: request.device_stable_id.clone(),
        selected_handle: request.selected_handle.raw(),
        sample_rate_hz: request.validated_config.sample_rate_hz,
        requested_sample_limit: request.validated_config.requested_sample_limit,
        actual_sample_count: export.sample_count,
        enabled_channels: request.validated_config.enabled_channels.clone(),
    },
    acquisition: MetadataAcquisitionInfo { /* ... */ },
    artifacts: MetadataArtifactInfo { /* ... */ },
})
```
Phase 13 should extend this builder with requested/effective device-option facts rather than bypassing it. [VERIFIED: crates/dsview-core/src/lib.rs]

### `BTreeSet` remains the right source for deterministic channel ordering
```rust
enabled_channels: channels.iter().copied().collect::<BTreeSet<_>>(),
```
The codebase already uses `BTreeSet` for channel selection, and Rust’s `BTreeSet` iterators yield items in order, which supports deterministic requested/effective reporting for enabled channels. [VERIFIED: crates/dsview-cli/src/capture_device_options.rs] [CITED: https://doc.rust-lang.org/std/collections/struct.BTreeSet.html]

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Capture runtime apply only sets enabled channels, sample limit, and sample rate through `apply_capture_config(...)`. [VERIFIED: crates/dsview-core/src/lib.rs] [VERIFIED: crates/dsview-sys/src/lib.rs] | Phase 13 needs a fuller DSLogic option-apply sequence that also sets operation mode, stop option, channel mode, threshold volts, and filter before those capture settings. [VERIFIED: .planning/ROADMAP.md] [VERIFIED: .planning/phases/13-option-aware-capture-reporting/13-CONTEXT.md] | Planned for Phase 13. [VERIFIED: .planning/ROADMAP.md] | The missing work is in runtime orchestration and reporting, not in user-facing token parsing. [VERIFIED: .planning/phases/12-cli-device-option-surface/12-02-SUMMARY.md] [VERIFIED: .planning/phases/13-option-aware-capture-reporting/13-CONTEXT.md] |
| Capture success output currently reports completion plus artifact paths only. [VERIFIED: crates/dsview-cli/src/main.rs] | Phase 13 must add concise effective-option text plus explicit requested/effective JSON facts. [VERIFIED: .planning/phases/13-option-aware-capture-reporting/13-CONTEXT.md] | Planned for Phase 13. [VERIFIED: .planning/ROADMAP.md] | Text stays concise while machine-readable surfaces become explicit enough for automation and post-run analysis. [VERIFIED: .planning/phases/13-option-aware-capture-reporting/13-CONTEXT.md] |
| Metadata schema version `1` currently contains `tool`, `capture`, `acquisition`, and `artifacts`, but no device-option fact block. [VERIFIED: crates/dsview-core/src/lib.rs] [VERIFIED: crates/dsview-core/tests/export_artifacts.rs] | Phase 13 should add requested/effective device-option facts in metadata rather than leaving them only in CLI output. [VERIFIED: .planning/REQUIREMENTS.md] [VERIFIED: .planning/phases/13-option-aware-capture-reporting/13-CONTEXT.md] | Planned for Phase 13. [VERIFIED: .planning/ROADMAP.md] | Automation that consumes only the sidecar can learn the effective hardware state without parsing text output. [VERIFIED: .planning/REQUIREMENTS.md] [VERIFIED: .planning/phases/13-option-aware-capture-reporting/13-CONTEXT.md] |
| Phase 12 already resolves friendly tokens and validates the full selected-device request before acquisition. [VERIFIED: .planning/phases/12-cli-device-option-surface/12-02-SUMMARY.md] [VERIFIED: crates/dsview-cli/src/capture_device_options.rs] | Phase 13 should build directly on that request instead of changing the token/help/discovery surface again. [VERIFIED: .planning/phases/13-option-aware-capture-reporting/13-CONTEXT.md] | 2026-04-13 in Phase 12. [VERIFIED: .planning/phases/12-cli-device-option-surface/12-02-SUMMARY.md] | Planning can stay focused on apply/report sequencing and regression coverage. [VERIFIED: .planning/phases/13-option-aware-capture-reporting/13-CONTEXT.md] |

**Deprecated/outdated:**
- Treating `validate_capture_config(...)` as the final authority for option-aware capture is outdated because it validates against the current active mode instead of the selected requested mode. [VERIFIED: crates/dsview-core/src/lib.rs] [VERIFIED: crates/dsview-core/src/capture_config.rs] [VERIFIED: crates/dsview-core/src/device_option_validation.rs]
- Keeping capture success JSON limited to completion and artifact paths is outdated for `RUN-05`. [VERIFIED: .planning/REQUIREMENTS.md] [VERIFIED: crates/dsview-cli/src/main.rs]
- Assuming the existing sys mock can already prove full Phase 13 apply order is outdated because it only records operation-mode and channel-mode set counts today. [VERIFIED: crates/dsview-sys/bridge_runtime.c] [VERIFIED: crates/dsview-sys/tests/device_options.rs]

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Adding a required requested/effective device-option block to metadata should probably bump `schema_version` from `1` to `2` even if the change is additive. [ASSUMED] | Open Questions | Medium; if the project treats additive metadata fields as schema-compatible, the planner should avoid unnecessary consumer churn. |
| A2 | The cleanest implementation will introduce one new shared core facts struct for requested/effective option reporting rather than threading many loose fields through CLI and export APIs. [ASSUMED] | Summary / Architecture Patterns | Low; naming and shape can change without changing the underlying work. |

## Open Questions (RESOLVED)

1. **Should the metadata schema version bump when the new device-option block is added?**  
   **RESOLVED:** Yes. Phase 13 should bump `CaptureMetadata.schema_version` from `1` to `2` because the metadata sidecar gains a required `device_options` reporting contract that automation consumers must understand explicitly. [VERIFIED: crates/dsview-core/src/lib.rs] [VERIFIED: crates/dsview-core/tests/export_artifacts.rs] [VERIFIED: .planning/REQUIREMENTS.md]

2. **How much of the effective fact set should be read back from the runtime versus trusted from successful setters?**  
   **RESOLVED:** Read back operation mode, stop option, channel mode, threshold volts, filter, sample rate, and sample limit from the runtime, then derive the stable-ID form from those readback codes through the validated capability catalog/current snapshot. Use the successfully applied validated channel list as `effective.enabled_channels` instead of adding a new enabled-channel getter in Phase 13. Also, when no validated device-option request exists, build inherited effective facts from the current device snapshot plus current runtime sample settings so the baseline path still reports effective values. [VERIFIED: crates/dsview-sys/src/lib.rs] [VERIFIED: crates/dsview-sys/bridge_runtime.c] [VERIFIED: crates/dsview-core/src/device_option_validation.rs] [VERIFIED: .planning/phases/13-option-aware-capture-reporting/13-CONTEXT.md]

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo` | Build/test execution for all Phase 13 work. [VERIFIED: cargo metadata] | ✓ [VERIFIED: cargo --version] | `1.94.1` [VERIFIED: cargo --version] | — |
| `rustc` | Compile Rust workspace changes. [VERIFIED: cargo metadata] | ✓ [VERIFIED: rustc --version] | `1.94.1` [VERIFIED: rustc --version] | — |
| `cmake` | `dsview-sys` native bridge build. [VERIFIED: crates/dsview-sys/Cargo.toml] [VERIFIED: cargo test] | ✓ [VERIFIED: cmake --version] | `3.31.6` [VERIFIED: cmake --version] | — |
| `pkg-config` | Native dependency discovery in local builds. [VERIFIED: cargo test] | ✓ [VERIFIED: pkg-config --version] | `1.8.1` [VERIFIED: pkg-config --version] | — |
| `DSView/` submodule checkout | Source-backed runtime build and project dependency boundary. [VERIFIED: .planning/PROJECT.md] [VERIFIED: CLAUDE.md] | ✓ [VERIFIED: test -d DSView] | `e93e644f` [VERIFIED: git -C DSView rev-parse --short HEAD] | — |
| Source-backed DSView runtime library | Automated tests and local runtime bring-up. [VERIFIED: cargo test] | ✓ [VERIFIED: cargo test] | Built at test time in `target/debug/build/.../libdsview_runtime.so`. [VERIFIED: cargo test] | — |
| Live libusb-backed runtime/device access | Manual end-to-end hardware validation. [VERIFIED: .planning/ROADMAP.md] | ✗ on this machine for ad-hoc local runs. [VERIFIED: cargo run] | `ds_lib_init` currently fails with `LIBUSB_ERROR_OTHER`. [VERIFIED: cargo run] | Use the existing debug-only CLI fixture and the `dsview-sys` mock option API for automated verification; reserve true hardware checks for a machine with working device/libusb access. [VERIFIED: .planning/phases/12-cli-device-option-surface/12-03-SUMMARY.md] [VERIFIED: crates/dsview-cli/tests/capture_cli.rs] [VERIFIED: crates/dsview-sys/tests/device_options.rs] [VERIFIED: cargo run] |

**Missing dependencies with no fallback:**
- None for planning and automated test coverage. [VERIFIED: cargo test] [VERIFIED: cargo metadata]

**Missing dependencies with fallback:**
- Live hardware/libusb access is not currently usable for local ad-hoc verification, but the existing fixture/mock seams provide deterministic automated fallback coverage. [VERIFIED: cargo run] [VERIFIED: .planning/phases/12-cli-device-option-surface/12-03-SUMMARY.md] [VERIFIED: crates/dsview-cli/tests/capture_cli.rs] [VERIFIED: crates/dsview-sys/tests/device_options.rs]

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust `cargo test` across library tests, binary unit tests, and spawned integration tests. [VERIFIED: cargo metadata] |
| Config file | None; the workspace uses Cargo target discovery plus per-target test files. [VERIFIED: cargo metadata] |
| Quick run command | `cargo test -p dsview-sys --test device_options -- --nocapture && cargo test -p dsview-core --test export_artifacts -- --nocapture` [VERIFIED: cargo test] |
| Full suite command | `cargo test --workspace -- --nocapture` [VERIFIED: cargo test] |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| RUN-04 | Option-aware capture applies the full validated request in the locked deterministic order and stops with partial-apply detail on the first failure. [VERIFIED: .planning/REQUIREMENTS.md] [VERIFIED: .planning/phases/13-option-aware-capture-reporting/13-CONTEXT.md] | Sys-layer mock regression + core orchestration test. [VERIFIED: crates/dsview-sys/tests/device_options.rs] [VERIFIED: crates/dsview-core/tests/acquisition.rs] | `cargo test -p dsview-sys --test device_options -- --nocapture` and `cargo test -p dsview-core --test acquisition -- --nocapture` [VERIFIED: cargo test] | Harness exists, but Phase 13-specific setter/order cases are missing. [VERIFIED: crates/dsview-sys/tests/device_options.rs] [VERIFIED: crates/dsview-core/tests/acquisition.rs] |
| RUN-05 | CLI JSON/text and metadata record requested/effective device-option facts without drifting from each other. [VERIFIED: .planning/REQUIREMENTS.md] [VERIFIED: .planning/phases/13-option-aware-capture-reporting/13-CONTEXT.md] | Core export-schema regression + spawned CLI output contract test. [VERIFIED: crates/dsview-core/tests/export_artifacts.rs] [VERIFIED: crates/dsview-cli/tests/capture_cli.rs] | `cargo test -p dsview-core --test export_artifacts -- --nocapture` and `cargo test -p dsview-cli --test capture_cli -- --nocapture` [VERIFIED: cargo test] | Harness exists, but no requested/effective-option assertions exist yet. [VERIFIED: crates/dsview-core/tests/export_artifacts.rs] [VERIFIED: crates/dsview-cli/tests/capture_cli.rs] |

### Sampling Rate
- **Per task commit:** Run the narrowest touched target, with `dsview-sys/tests/device_options.rs` for setter sequencing and `dsview-core/tests/export_artifacts.rs` for schema/output shaping. [VERIFIED: cargo test] [VERIFIED: crates/dsview-sys/tests/device_options.rs] [VERIFIED: crates/dsview-core/tests/export_artifacts.rs]
- **Per wave merge:** `cargo test -p dsview-sys --test device_options -- --nocapture && cargo test -p dsview-core --test export_artifacts -- --nocapture && cargo test -p dsview-cli --test capture_cli -- --nocapture` [VERIFIED: cargo test]
- **Phase gate:** `cargo test --workspace -- --nocapture` must be green before `/gsd-verify-work`. [VERIFIED: cargo test] [VERIFIED: .planning/config.json]

### Wave 0 Gaps
- [ ] Expand `crates/dsview-sys/tests/device_options.rs` and the C mock setter to cover stop option, filter, threshold volts, sample limit, sample rate, failing-step injection, and exact call order. [VERIFIED: crates/dsview-sys/tests/device_options.rs] [VERIFIED: crates/dsview-sys/bridge_runtime.c]
- [ ] Add core tests that prove the option-aware path does not rely on the current active mode once a Phase 11 validated request exists. [VERIFIED: crates/dsview-core/src/lib.rs] [VERIFIED: crates/dsview-core/src/device_option_validation.rs] [VERIFIED: crates/dsview-core/tests/acquisition.rs]
- [ ] Add metadata schema assertions for requested/effective device-option facts in `crates/dsview-core/tests/export_artifacts.rs`. [VERIFIED: crates/dsview-core/tests/export_artifacts.rs]
- [ ] Add spawned CLI assertions for concise text output and explicit JSON requested/effective reporting in `crates/dsview-cli/tests/capture_cli.rs`. [VERIFIED: crates/dsview-cli/tests/capture_cli.rs]

## Security Domain

### Applicable ASVS Categories
| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no. [VERIFIED: .planning/PROJECT.md] | Not applicable to this local CLI capture workflow. [VERIFIED: .planning/PROJECT.md] |
| V3 Session Management | no. [VERIFIED: .planning/PROJECT.md] | Not applicable to this local CLI capture workflow. [VERIFIED: .planning/PROJECT.md] |
| V4 Access Control | no. [VERIFIED: .planning/PROJECT.md] | Device selection is a local handle choice, not an authorization system. [VERIFIED: crates/dsview-cli/src/main.rs] [VERIFIED: .planning/PROJECT.md] |
| V5 Input Validation | yes. [VERIFIED: .planning/REQUIREMENTS.md] [VERIFIED: crates/dsview-core/src/device_option_validation.rs] | Keep option parsing in clap and semantic enforcement in `DeviceOptionValidationCapabilities::validate_request(...)`, plus stable error mapping in the CLI. [VERIFIED: crates/dsview-cli/src/main.rs] [VERIFIED: crates/dsview-core/src/device_option_validation.rs] |
| V6 Cryptography | no. [VERIFIED: .planning/PROJECT.md] | No crypto primitive is added or required in this phase. [VERIFIED: .planning/PROJECT.md] |

### Known Threat Patterns for this stack
| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Unsupported or ambiguous device-option input reaches runtime setters. [VERIFIED: crates/dsview-cli/src/capture_device_options.rs] [VERIFIED: crates/dsview-core/src/device_option_validation.rs] | Tampering | Keep friendly-token parsing in `capture_device_options.rs`, then require a successful Phase 11 validation result before any runtime apply begins. [VERIFIED: crates/dsview-cli/src/capture_device_options.rs] [VERIFIED: crates/dsview-core/src/device_option_validation.rs] |
| Partial device reconfiguration leaves hardware in a misleading state after a failed setter. [VERIFIED: .planning/phases/13-option-aware-capture-reporting/13-CONTEXT.md] | Tampering / Repudiation | Use deterministic order, stop on first failure, and surface the successful steps plus the failing step in machine-readable error output. [VERIFIED: .planning/phases/13-option-aware-capture-reporting/13-CONTEXT.md] [VERIFIED: crates/dsview-cli/src/main.rs] |
| Requested/effective reporting disagrees across text, JSON, and metadata. [VERIFIED: .planning/phases/13-option-aware-capture-reporting/13-CONTEXT.md] | Integrity | Build all requested/effective facts once in core, serialize with serde, and lock the contract with core plus CLI tests. [VERIFIED: crates/dsview-core/src/lib.rs] [VERIFIED: crates/dsview-cli/src/main.rs] [VERIFIED: crates/dsview-core/tests/export_artifacts.rs] [VERIFIED: crates/dsview-cli/tests/capture_cli.rs] |
| Artifact/metadata path or write failures mask capture success state. [VERIFIED: crates/dsview-core/src/lib.rs] [VERIFIED: crates/dsview-core/tests/export_artifacts.rs] | Denial of Service / Integrity | Keep using the existing validated artifact-path helpers and atomic metadata writes when extending the schema. [VERIFIED: crates/dsview-core/src/lib.rs] [VERIFIED: crates/dsview-core/tests/export_artifacts.rs] |

## Sources

### Primary (HIGH confidence)
- `crates/dsview-core/src/lib.rs` - current capture run path, `apply_capture_config(...)`, preflight side effects, export builder, and metadata schema. [VERIFIED: crates/dsview-core/src/lib.rs]
- `crates/dsview-core/src/device_option_validation.rs` - validated request contract and stable validation model that Phase 13 should reuse. [VERIFIED: crates/dsview-core/src/device_option_validation.rs]
- `crates/dsview-cli/src/main.rs` - current capture success/error rendering and Phase 12 validation entrypoint. [VERIFIED: crates/dsview-cli/src/main.rs]
- `crates/dsview-cli/src/capture_device_options.rs` - current request-resolution layer from CLI tokens to selected-device validation input. [VERIFIED: crates/dsview-cli/src/capture_device_options.rs]
- `crates/dsview-sys/src/lib.rs` and `crates/dsview-sys/bridge_runtime.c` - current getter/setter surface and native mock/test capabilities. [VERIFIED: crates/dsview-sys/src/lib.rs] [VERIFIED: crates/dsview-sys/bridge_runtime.c]
- `.planning/phases/11-device-option-validation-model/11-02-SUMMARY.md`, `.planning/phases/11-device-option-validation-model/11-03-SUMMARY.md`, `.planning/phases/12-cli-device-option-surface/12-02-SUMMARY.md`, `.planning/phases/12-cli-device-option-surface/12-03-SUMMARY.md` - prior phase decisions and test seams that Phase 13 inherits. [VERIFIED: .planning/phases/11-device-option-validation-model/11-02-SUMMARY.md] [VERIFIED: .planning/phases/11-device-option-validation-model/11-03-SUMMARY.md] [VERIFIED: .planning/phases/12-cli-device-option-surface/12-02-SUMMARY.md] [VERIFIED: .planning/phases/12-cli-device-option-surface/12-03-SUMMARY.md]
- `https://serde.rs/field-attrs.html` - official serde field attribute guidance for additive nested serialization choices. [CITED: https://serde.rs/field-attrs.html]
- `https://doc.rust-lang.org/std/collections/struct.BTreeSet.html` - official Rust documentation confirming deterministic ordered iteration for `BTreeSet`. [CITED: https://doc.rust-lang.org/std/collections/struct.BTreeSet.html]
- `cargo test -p dsview-sys --test device_options -- --nocapture`, `cargo test -p dsview-core --test export_artifacts -- --nocapture`, and `cargo test -p dsview-cli --test capture_cli -- --nocapture` - current green evidence for the existing test harnesses. [VERIFIED: cargo test]
- `cargo run -q -p dsview-cli -- devices list --format json` - current local runtime/libusb failure evidence for environment planning. [VERIFIED: cargo run]

### Secondary (MEDIUM confidence)
- None. [VERIFIED: research scope]

### Tertiary (LOW confidence)
- None. [VERIFIED: research scope]

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - the crates, versions, toolchain, and existing boundaries are directly visible in `Cargo.toml`, `Cargo.lock`, `cargo metadata`, and the current code. [VERIFIED: Cargo.lock] [VERIFIED: cargo metadata] [VERIFIED: crates/dsview-cli/Cargo.toml] [VERIFIED: crates/dsview-core/Cargo.toml] [VERIFIED: crates/dsview-sys/Cargo.toml]
- Architecture: MEDIUM-HIGH - the key risks and recommended split are strongly supported by current code paths, but the final naming/layout of new core facts and metadata versioning remains discretionary. [VERIFIED: crates/dsview-core/src/lib.rs] [VERIFIED: crates/dsview-core/src/device_option_validation.rs] [VERIFIED: crates/dsview-sys/bridge_runtime.c] [ASSUMED]
- Pitfalls: HIGH - the most important pitfalls are directly observable in the current implementation and local environment behavior. [VERIFIED: crates/dsview-core/src/lib.rs] [VERIFIED: crates/dsview-sys/bridge_runtime.c] [VERIFIED: cargo run]

**Research date:** 2026-04-13. [VERIFIED: environment_context]
**Valid until:** 2026-05-13 for codebase-derived findings, or sooner if Phase 13 implementation changes the capture orchestration, metadata schema, or sys-layer mock surface. [VERIFIED: .planning/STATE.md] [VERIFIED: crates/dsview-core/src/lib.rs] [VERIFIED: crates/dsview-sys/bridge_runtime.c]
