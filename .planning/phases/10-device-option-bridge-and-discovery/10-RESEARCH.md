# Phase 10: Device Option Bridge and Discovery - Research

**Researched:** 2026-04-10
**Domain:** DSView/libsigrok option discovery across the native Rust boundary for `DSLogic Plus`. [VERIFIED: `.planning/ROADMAP.md`]
**Confidence:** MEDIUM

<user_constraints>
## User Constraints

- No phase-specific `CONTEXT.md` exists for Phase 10, so the effective locked constraints come from `.planning/ROADMAP.md`, `.planning/REQUIREMENTS.md`, and `CLAUDE.md`. [VERIFIED: phase init output + phase-dir listing + `CLAUDE.md`]
- Scope is limited to exposing the DSView-backed `DSLogic Plus` option surface through the Rust boundary and making supported values inspectable from the CLI. [VERIFIED: `.planning/ROADMAP.md`]
- This phase must address `OPT-01` only. [VERIFIED: `.planning/ROADMAP.md` + `.planning/REQUIREMENTS.md`]
- The shipped `v1.0` capture/export flow remains the baseline and must be preserved while adding discovery-only capability. [VERIFIED: `.planning/STATE.md` + `.planning/ROADMAP.md`]
- `DSView/` stays read-only and upstream code must not be modified for normal project work. [VERIFIED: `.planning/REQUIREMENTS.md` + `CLAUDE.md`]
- Unsafe/native integration stays isolated behind the small `dsview-sys` boundary. [VERIFIED: `CLAUDE.md` + `crates/dsview-sys/src/lib.rs`]
- The milestone remains scoped to `DSLogic Plus`; presets, repeat/loop collect behavior, advanced trigger work, protocol decode, and broader hardware support remain out of scope. [VERIFIED: `.planning/STATE.md` + `.planning/ROADMAP.md` + `.planning/REQUIREMENTS.md`]
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| OPT-01 | User can inspect the supported `DSLogic Plus` device-option values for operation mode, stop option, channel mode, threshold voltage, and filter selection from the CLI. [VERIFIED: `.planning/REQUIREMENTS.md`] | The research maps the needed config IDs and list/current-value APIs, identifies missing bridge APIs, recommends stable Rust discovery types, and defines CLI text/JSON output plus Wave 0 tests. [VERIFIED: `DSView/libsigrok4DSL/lib_main.c` + `DSView/libsigrok4DSL/hardware/DSL/dslogic.c` + `crates/dsview-sys/bridge_runtime.c` + `crates/dsview-core/src/lib.rs` + `crates/dsview-cli/src/main.rs`] |
</phase_requirements>

## Project Constraints (from CLAUDE.md)

- Treat `DSView/` as an upstream dependency and do not modify it for this phase. [VERIFIED: `CLAUDE.md`]
- Keep unsafe/native integration isolated behind a small boundary. [VERIFIED: `CLAUDE.md`]
- Favor scriptable CLI behavior, explicit exit codes, and machine-readable outputs. [VERIFIED: `CLAUDE.md`]
- Keep milestone scope to `DSLogic Plus` only until the option workflow is proven stable. [VERIFIED: `CLAUDE.md`]
- Work should stay inside the GSD workflow; for this task that means producing the planning artifact only. [VERIFIED: `CLAUDE.md`]

## Summary

The DSView public API already provides the two primitives this phase needs for most option discovery work: `ds_get_actived_device_config()` for current values and `ds_get_actived_device_config_list()` for supported-value enumeration. [VERIFIED: `DSView/libsigrok4DSL/lib_main.c`] The DSLogic driver backs the requested option surface with `SR_CONF_OPERATION_MODE`, `SR_CONF_BUFFER_OPTIONS`, `SR_CONF_CHANNEL_MODE`, `SR_CONF_FILTER`, `SR_CONF_THRESHOLD`, and `SR_CONF_VTH`, and it returns current values from `config_get()` plus supported values from `config_list()`. [VERIFIED: `DSView/libsigrok4DSL/hardware/DSL/dslogic.c` + `DSView/libsigrok4DSL/hwdriver.c`]

The main planning risk is that the current Rust bridge only exposes samplerates, sample limits, VTH current value, and channel modes, and the existing channel-mode bridge is lossy because it derives `max_enabled_channels` from label text instead of the driver's real `vld_num` field. [VERIFIED: `crates/dsview-sys/bridge_runtime.c` + `crates/dsview-sys/src/lib.rs` + `crates/dsview-core/src/lib.rs` + `DSView/libsigrok4DSL/hardware/DSL/dsl.h`] A second risk is threshold semantics: DSLogic Plus profiles advertise `SR_CONF_VTH` as the device option in `SR_CONF_DEVICE_OPTIONS`, while the driver also still implements the legacy list-valued `SR_CONF_THRESHOLD`; the GUI binds `SR_CONF_VTH` as a float with a hard-coded `0.0..5.0 V` range and `0.1 V` step. [VERIFIED: `DSView/libsigrok4DSL/hardware/DSL/dslogic.c` + `DSView/DSView/pv/prop/binding/deviceoptions.cpp`]

**Primary recommendation:** Add narrow C bridge snapshot APIs for each discovery concern, keep all pointer and `GVariant` handling in `dsview-sys`, model threshold as a float-range capability plus current value, and model channel modes per operation mode instead of as a single flat list. [VERIFIED: `crates/dsview-sys/bridge_runtime.c` + `crates/dsview-core/src/lib.rs` + `DSView/libsigrok4DSL/hardware/DSL/dslogic.c`] [CITED: https://docs.gtk.org/glib/method.Variant.lookup_value.html] [CITED: https://docs.gtk.org/glib/method.Variant.get_fixed_array.html]

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `dsview-sys` | `0.1.0` [VERIFIED: `crates/dsview-sys/Cargo.toml`] | Own all unsafe FFI, dynamic `ds_*` loading, and C-shim translation. [VERIFIED: `crates/dsview-sys/src/lib.rs` + `crates/dsview-sys/bridge_runtime.c`] | This already matches the project's "small unsafe boundary" rule and existing runtime loading pattern. [VERIFIED: `CLAUDE.md` + `crates/dsview-sys/src/lib.rs`] |
| `dsview-core` | `0.1.0` [VERIFIED: `crates/dsview-core/Cargo.toml`] | Normalize native discovery results into typed Rust domain objects for CLI consumption. [VERIFIED: `crates/dsview-core/src/lib.rs`] | This crate already owns safe orchestration and should absorb option normalization without leaking native details into the CLI. [VERIFIED: `crates/dsview-core/src/lib.rs`] |
| `dsview-cli` | `0.1.0` [VERIFIED: `crates/dsview-cli/Cargo.toml`] | Provide stable text and JSON inspection output. [VERIFIED: `crates/dsview-cli/src/main.rs`] | This crate already has stable `json|text` rendering conventions and device-scoped commands. [VERIFIED: `crates/dsview-cli/src/main.rs`] |
| `DSView/libsigrok4DSL` public `ds_*` API | local submodule snapshot [VERIFIED: repo layout + `DSView/libsigrok4DSL/libsigrok.h`] | Supplies current-value and list-enumeration APIs without changing upstream code. [VERIFIED: `DSView/libsigrok4DSL/lib_main.c` + `DSView/libsigrok4DSL/libsigrok.h`] | The roadmap explicitly requires consuming upstream `DSView/` without modifying it. [VERIFIED: `.planning/ROADMAP.md` + `.planning/REQUIREMENTS.md`] |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `clap` | `4.6.0` [VERIFIED: `Cargo.lock` + `crates/dsview-cli/Cargo.toml`] | CLI command/flag parsing and help text. [VERIFIED: `crates/dsview-cli/src/main.rs`] | Use for a new discovery subcommand under `devices`, not for ad-hoc parsing. [VERIFIED: `crates/dsview-cli/src/main.rs`] |
| `serde` | `1.0.228` [VERIFIED: `Cargo.lock` + `crates/dsview-cli/Cargo.toml` + `crates/dsview-core/Cargo.toml`] | Serialize stable JSON discovery responses. [VERIFIED: `crates/dsview-cli/src/main.rs` + `crates/dsview-core/src/lib.rs`] | Use for machine-readable discovery output only. [VERIFIED: `crates/dsview-cli/src/main.rs`] |
| `serde_json` | `1.0.149` [VERIFIED: `Cargo.lock` + `crates/dsview-cli/Cargo.toml` + `crates/dsview-core/Cargo.toml`] | Pretty-print JSON responses and metadata. [VERIFIED: `crates/dsview-cli/src/main.rs` + `crates/dsview-core/src/lib.rs`] | Reuse the existing pretty JSON pattern instead of inventing another formatter. [VERIFIED: `crates/dsview-cli/src/main.rs`] |
| `thiserror` | `2.0.18` [VERIFIED: `Cargo.lock` + `crates/dsview-sys/Cargo.toml` + `crates/dsview-core/Cargo.toml`] | Stable internal error typing across `dsview-sys` and `dsview-core`. [VERIFIED: `crates/dsview-sys/src/lib.rs` + `crates/dsview-core/src/lib.rs`] | Use for discovery-specific error taxonomy if new error cases are needed. [VERIFIED: existing crate usage] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Adding more unsafe Rust to parse raw `GVariant` data in `dsview-core` or `dsview-cli` | Keep all `GVariant` handling in `dsview-sys` C helpers and return plain C structs to Rust | The existing project architecture and current bridge pattern already isolate unsafe/native translation in `dsview-sys`, which is lower risk to the `v1.0` baseline. [VERIFIED: `CLAUDE.md` + `crates/dsview-sys/bridge_runtime.c` + `crates/dsview-sys/src/lib.rs`] |
| Reusing `CaptureCapabilities` as the public discovery model | Add a dedicated device-option discovery model in `dsview-core` | `CaptureCapabilities` is optimized for Phase 9 capture validation and collapses some distinctions that matter here, especially per-operation-mode channel modes and threshold representation. [VERIFIED: `crates/dsview-core/src/lib.rs` + `crates/dsview-core/src/capture_config.rs`] |
| Deriving automation IDs from DSView labels | Use explicit stable IDs in Rust, backed by raw numeric codes where needed | Labels are human-facing strings and some current bridge logic already has to parse text heuristically, which is fragile. [VERIFIED: `crates/dsview-sys/bridge_runtime.c` + `DSView/libsigrok4DSL/hardware/DSL/dsl.h`] |

**Installation:**
```bash
# No new dependencies are required for Phase 10.
# Existing workspace crates and native build prerequisites are already present.
```

**Version verification:** Workspace crate versions were verified from `Cargo.toml` and resolved dependency versions were verified from `Cargo.lock`. [VERIFIED: `Cargo.toml` + `Cargo.lock`]

## Architecture Patterns

### Recommended Project Structure

```text
crates/
├── dsview-sys/
│   ├── wrapper.h          # Public C ABI for discovery snapshots
│   ├── bridge_runtime.c   # All GVariant/list copying and restore-on-exit logic
│   └── src/lib.rs         # Safe Rust wrappers over plain C structs
├── dsview-core/
│   ├── src/lib.rs         # Discovery entry points on top of RuntimeBridge
│   └── src/device_options.rs  # New typed option discovery model and normalization
└── dsview-cli/
    ├── src/main.rs        # New `devices options` subcommand and stable renderers
    └── tests/device_options_cli.rs  # New CLI coverage for text/JSON discovery
```

This structure keeps the current crate layering intact and avoids coupling Phase 10 discovery work to the Phase 11 validation model. [VERIFIED: `CLAUDE.md` + current workspace layout]

### Pattern 1: Split current-value access from supported-value access

**What:** Use `ds_get_actived_device_config()` for the device's current setting and `ds_get_actived_device_config_list()` for the supported list or range backing that setting. [VERIFIED: `DSView/libsigrok4DSL/lib_main.c` + `DSView/libsigrok4DSL/hardware/DSL/dslogic.c`]

**When to use:** Use this for operation mode, stop option, channel mode, filter, and threshold because DSView exposes current values and supported values through different API paths. [VERIFIED: `DSView/libsigrok4DSL/hardware/DSL/dslogic.c`]

**Example:**
```c
// Source: crates/dsview-sys/bridge_runtime.c
status = g_bridge_api.ds_get_actived_device_config_list(NULL, SR_CONF_CHANNEL_MODE, &data);
items = (struct sr_list_item *)(uintptr_t)g_variant_get_uint64(data);
while (items != NULL && items[index].id >= 0) {
    out_modes[index].id = items[index].id;
    strncpy(out_modes[index].name, items[index].name, sizeof(out_modes[index].name) - 1);
    index++;
}
```
[VERIFIED: `crates/dsview-sys/bridge_runtime.c`]

### Pattern 2: Copy list data in C before releasing the `GVariant`

**What:** Treat DSView list enumeration as an unsafe/native concern and copy every list item into fixed C structs before unrefing the `GVariant`. [VERIFIED: `crates/dsview-sys/bridge_runtime.c`] [CITED: https://docs.gtk.org/glib/method.Variant.get_fixed_array.html]

**When to use:** Use this for operation mode, stop option, channel mode, filter, and any samplerate-like `a{sv}` list. [VERIFIED: `DSView/libsigrok4DSL/hardware/DSL/dsl.c` + `DSView/libsigrok4DSL/hardware/DSL/dslogic.c`]

**Example:**
```c
// Source: crates/dsview-sys/bridge_runtime.c
samplerates = g_variant_lookup_value(data, "samplerates", G_VARIANT_TYPE("at"));
values = g_variant_get_fixed_array(samplerates, &count, sizeof(guint64));
for (i = 0; i < out_list->count; i++) {
    out_list->values[i] = values[i];
}
g_variant_unref(samplerates);
g_variant_unref(data);
```
[VERIFIED: `crates/dsview-sys/bridge_runtime.c`] [CITED: https://docs.gtk.org/glib/method.Variant.lookup_value.html] [CITED: https://docs.gtk.org/glib/method.Variant.get_fixed_array.html]

### Pattern 3: Model channel modes per operation mode

**What:** Represent channel modes as a mapping from operation mode to a list of channel-mode entries instead of as one flat list. [VERIFIED: `DSView/libsigrok4DSL/hardware/DSL/dslogic.c`]

**When to use:** Use this because `SR_CONF_CHANNEL_MODE` enumeration depends on `devc->stream`, which in turn depends on the selected operation mode. [VERIFIED: `DSView/libsigrok4DSL/hardware/DSL/dslogic.c`]

**Recommended Rust shape:**
```rust
// Recommendation derived from the verified DSView config behavior.
pub struct DeviceOptionSnapshot {
    pub operation_modes: EnumOptionSet,
    pub stop_options: EnumOptionSet,
    pub channel_modes_by_operation_mode: Vec<ChannelModeGroup>,
    pub threshold: ThresholdOptionSet,
    pub filters: EnumOptionSet,
}
```
[VERIFIED: `DSView/libsigrok4DSL/hardware/DSL/dslogic.c` + `crates/dsview-core/src/lib.rs`] [ASSUMED]

### Pattern 4: Normalize stable IDs in Rust, not in C

**What:** Keep the C bridge numeric and lossless, then normalize stable automation IDs in `dsview-core`. [VERIFIED: existing sys/core layering]

**When to use:** Use named stable IDs for options with small, known vocabularies like operation mode and filter, and numeric-code-backed IDs for channel modes so the first public contract does not depend on parsing human labels. [VERIFIED: `DSView/libsigrok4DSL/hardware/DSL/dslogic.c` + `DSView/libsigrok4DSL/hardware/DSL/dsl.h`] [ASSUMED]

**Recommended stable IDs:**

| Option | Stable ID strategy | Why |
|--------|--------------------|-----|
| Operation mode | `buffer`, `stream`, `internal-test` | DSView exposes stable numeric codes and English labels for these values. [VERIFIED: `DSView/libsigrok4DSL/hardware/DSL/dslogic.c`] |
| Stop option | `stop-immediately`, `upload-captured-data` | Small finite set with stable meaning. [VERIFIED: `DSView/libsigrok4DSL/hardware/DSL/dslogic.c`] |
| Filter | `none`, `1-sample-clock` | Small finite set with stable meaning. [VERIFIED: `DSView/libsigrok4DSL/hardware/DSL/dslogic.c`] |
| Channel mode | `channel-mode-<native_code>` | Avoid coupling public IDs to localizable or reformatted labels. [VERIFIED: current labels are human text only; no exported semantic ID exists in the public bridge] [ASSUMED] |
| Threshold | float-range metadata plus current value, not an enum ID | DSLogic Plus GUI uses `SR_CONF_VTH` as a float option, not a list picker. [VERIFIED: `DSView/DSView/pv/prop/binding/deviceoptions.cpp` + `DSView/libsigrok4DSL/hardware/DSL/dslogic.c`] |

### Anti-Patterns to Avoid

- **Leaking raw `GVariant` or `sr_list_item *` into Rust:** Those values are only safe while the native object is alive and should be copied inside `dsview-sys`. [VERIFIED: `crates/dsview-sys/bridge_runtime.c`] [CITED: https://docs.gtk.org/glib/method.Variant.lookup_value.html]
- **Inferring channel limits from label text in new code:** The current helper parses the trailing `xN` pattern, which does not work for buffer labels like `Use Channels 0~15 (Max 100MHz)`. [VERIFIED: `crates/dsview-sys/bridge_runtime.c` + `DSView/libsigrok4DSL/hardware/DSL/dsl.h`]
- **Assuming every option is a list:** `SR_CONF_VTH` is a float current value for DSLogic Plus device options, while `SR_CONF_THRESHOLD` is a separate legacy list-valued path. [VERIFIED: `DSView/libsigrok4DSL/hardware/DSL/dslogic.c` + `DSView/DSView/pv/prop/binding/deviceoptions.cpp`]
- **Reusing Phase 9 capture capability types as the public discovery schema:** They already flatten option relationships that matter for discovery. [VERIFIED: `crates/dsview-core/src/lib.rs` + `crates/dsview-core/src/capture_config.rs`]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Native option-list parsing | A generic Rust-side parser over raw `GVariant` pointers | Narrow C bridge functions that copy into plain structs | This keeps pointer lifetimes, type checks, and `GVariant` ownership inside the existing unsafe boundary. [VERIFIED: `crates/dsview-sys/bridge_runtime.c` + `crates/dsview-sys/src/lib.rs`] [CITED: https://docs.gtk.org/glib/method.Variant.lookup_value.html] |
| Channel-mode capacity derivation | New string parsing heuristics over human labels | Bridge `vld_num` and `max_samplerate` directly from `channel_modes[]` semantics, or at minimum carry the raw native code and current valid count without pretending the label is structured data | The driver already has the real capacity data and the label format is inconsistent across stream and buffer modes. [VERIFIED: `DSView/libsigrok4DSL/hardware/DSL/dsl.h` + current fallback in `crates/dsview-sys/bridge_runtime.c`] |
| Automation IDs | IDs derived from translated GUI strings | Explicit Rust normalization tables and numeric-code-backed IDs | GUI labels can be translated, reformatted, or duplicated; numeric codes and explicit IDs are safer. [VERIFIED: `DSView/DSView/pv/dialogs/deviceoptions.cpp` + `DSView/libsigrok4DSL/hardware/DSL/dslogic.c`] |
| Threshold discovery | Pretending threshold is always a discrete enum | A threshold capability that can express `float-range` and optionally carry a legacy discrete list if queried | DSLogic Plus device options currently favor `SR_CONF_VTH`, not just `SR_CONF_THRESHOLD`. [VERIFIED: `DSView/libsigrok4DSL/hardware/DSL/dslogic.c` + `DSView/DSView/pv/prop/binding/deviceoptions.cpp`] |

**Key insight:** The bridge should translate DSView's native option surface into a lossless, typed snapshot, but the public automation contract should not expose DSView's pointer tricks, GUI localization rules, or label-shape assumptions. [VERIFIED: `DSView/libsigrok4DSL/hardware/DSL/dslogic.c` + `crates/dsview-sys/bridge_runtime.c` + `DSView/DSView/pv/dialogs/deviceoptions.cpp`]

## Common Pitfalls

### Pitfall 1: `SR_CONF_VTH` and `SR_CONF_THRESHOLD` are not the same thing

**What goes wrong:** Planning assumes threshold discovery is just another enum list, but DSLogic Plus device options expose `SR_CONF_VTH` as a float current value while the driver still separately supports list-valued `SR_CONF_THRESHOLD`. [VERIFIED: `DSView/libsigrok4DSL/hardware/DSL/dslogic.c` + `DSView/DSView/pv/prop/binding/deviceoptions.cpp`]

**Why it happens:** `SR_CONF_DEVICE_OPTIONS` returns `hwoptions_pro` for VTH-capable devices, and that array includes `SR_CONF_VTH` instead of `SR_CONF_THRESHOLD`. [VERIFIED: `DSView/libsigrok4DSL/hardware/DSL/dslogic.c`]

**How to avoid:** Treat threshold discovery as a dedicated branch in the model and confirm whether the CLI contract should expose a float range, the legacy threshold list, or both. [VERIFIED: code paths above] [ASSUMED]

**Warning signs:** Any design that only has `values: [{id,label}]` for threshold but no place for `current_volts`, `min_volts`, `max_volts`, or `step_volts`. [VERIFIED: current DSView GUI binds `SR_CONF_VTH` as a double] [ASSUMED]

### Pitfall 2: Channel modes are operation-mode dependent

**What goes wrong:** A single `channel_modes` list is cached and treated as universally valid. [VERIFIED: current `capture_capabilities()` returns one flat vector in Rust]

**Why it happens:** `config_list(SR_CONF_CHANNEL_MODE)` filters the static `channel_modes[]` table by `devc->stream`, and `devc->stream` is tied to the selected operation mode. [VERIFIED: `DSView/libsigrok4DSL/hardware/DSL/dslogic.c`]

**How to avoid:** Snapshot channel modes per operation mode, or explicitly switch operation mode during discovery and always restore the original device state before release. [VERIFIED: operation-mode and channel-mode setters in `DSView/libsigrok4DSL/hardware/DSL/dslogic.c`] [ASSUMED]

**Warning signs:** Buffer mode reports stream-only labels, or the CLI shows one channel-mode list regardless of the selected operation mode. [VERIFIED: `config_list()` behavior in `DSView/libsigrok4DSL/hardware/DSL/dslogic.c`] [ASSUMED]

### Pitfall 3: Pointer-encoded lists are easy to misuse

**What goes wrong:** Code stores or forwards the pointer returned from `g_variant_get_uint64()` instead of copying out the `sr_list_item` records immediately. [VERIFIED: DSView encodes several option lists as `g_variant_new_uint64((uint64_t)&list)` in `config_list()`]

**Why it happens:** DSView uses a non-obvious pattern where enum-like lists are exposed as pointers smuggled through a `uint64` `GVariant`. [VERIFIED: `DSView/libsigrok4DSL/hardware/DSL/dslogic.c`]

**How to avoid:** Keep this pattern confined to C helpers in `dsview-sys`, copy into owned output structs, and unref the `GVariant` after copying. [VERIFIED: current channel-mode bridge pattern in `crates/dsview-sys/bridge_runtime.c`]

**Warning signs:** Rust structs containing raw pointers, or bridge code that unrefs `data` before copying the list elements. [VERIFIED: current safe pattern copies before `g_variant_unref(data)` in `crates/dsview-sys/bridge_runtime.c`] [CITED: https://docs.gtk.org/glib/method.Variant.lookup_value.html]

### Pitfall 4: Labels are for display, not identity

**What goes wrong:** Public IDs or validation rules depend on strings like `Use 16 Channels (Max 20MHz)`. [VERIFIED: `DSView/libsigrok4DSL/hardware/DSL/dsl.h`]

**Why it happens:** DSView exposes human-readable `descr` text in channel-mode lists and the GUI translates some labels for display. [VERIFIED: `DSView/libsigrok4DSL/hardware/DSL/dsl.h` + `DSView/DSView/pv/dialogs/deviceoptions.cpp`]

**How to avoid:** Keep raw numeric code plus label, and generate stable public IDs in Rust. [VERIFIED: current device stable-ID pattern already exists in `crates/dsview-core/src/lib.rs`] [ASSUMED]

**Warning signs:** Public JSON that uses only `label` and drops the raw code. [VERIFIED: current design conventions rely on explicit stable IDs for devices and errors in `crates/dsview-core/src/lib.rs` + `crates/dsview-cli/src/main.rs`] [ASSUMED]

### Pitfall 5: `SR_ERR_NA` should mean "option unavailable", not "bridge failed"

**What goes wrong:** Discovery fails completely when an option is device-specific or not applicable. [VERIFIED: existing `capture_capabilities()` already treats `SR_ERR_NA` on `VTH` as optional]

**Why it happens:** The same public API serves different device families and option sets. [VERIFIED: `DSView/libsigrok4DSL/hwdriver.c` + `DSView/libsigrok4DSL/hardware/DSL/dslogic.c`]

**How to avoid:** Preserve optionality in the Rust model and report unavailable options explicitly in JSON/text instead of collapsing them into generic runtime errors. [VERIFIED: existing optional VTH handling in `crates/dsview-sys/src/lib.rs`] [ASSUMED]

**Warning signs:** A single unsupported key prevents the CLI from printing the rest of the discovery surface. [VERIFIED: API can return `SR_ERR_NA` per key] [ASSUMED]

## Code Examples

Verified patterns from inspected sources:

### Current-value lookup for DSView-backed config

```c
// Source: DSView/libsigrok4DSL/lib_main.c
return sr_config_get(lib_ctx.actived_device_instance->driver,
                     lib_ctx.actived_device_instance,
                     ch,
                     cg,
                     key,
                     data);
```
[VERIFIED: `DSView/libsigrok4DSL/lib_main.c`]

### List enumeration for DSLogic option lists

```c
// Source: DSView/libsigrok4DSL/hardware/DSL/dslogic.c
case SR_CONF_OPERATION_MODE:
    *data = g_variant_new_uint64((uint64_t)&opmode_list);
    break;
case SR_CONF_BUFFER_OPTIONS:
    *data = g_variant_new_uint64((uint64_t)&bufoption_list);
    break;
case SR_CONF_FILTER:
    *data = g_variant_new_uint64((uint64_t)&filter_list);
    break;
```
[VERIFIED: `DSView/libsigrok4DSL/hardware/DSL/dslogic.c`]

### Samplerate list shape (`a{sv}` with `samplerates`)

```c
// Source: DSView/libsigrok4DSL/hardware/DSL/dsl.c
g_variant_builder_init(&gvb, G_VARIANT_TYPE("a{sv}"));
gvar = g_variant_new_from_data(G_VARIANT_TYPE("at"),
       devc->profile->dev_caps.samplerates + devc->samplerates_min_index,
       (devc->samplerates_max_index - devc->samplerates_min_index + 1) * sizeof(uint64_t),
       TRUE, NULL, NULL);
g_variant_builder_add(&gvb, "{sv}", "samplerates", gvar);
*data = g_variant_builder_end(&gvb);
```
[VERIFIED: `DSView/libsigrok4DSL/hardware/DSL/dsl.c`]

### Existing stable text-vs-JSON CLI rendering pattern

```rust
// Source: crates/dsview-cli/src/main.rs
match format {
    OutputFormat::Json => println!("{}", serde_json::to_string_pretty(response).unwrap()),
    OutputFormat::Text => {
        for device in &response.devices {
            println!("{}\t{}\t{}\t{}", device.handle, device.stable_id, device.model, device.native_name);
        }
    }
}
```
[VERIFIED: `crates/dsview-cli/src/main.rs`]

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Phase 9 exposes only sample rate, sample limit, total/valid channels, active channel mode, hardware depth, VTH current value, samplerates, and channel modes. [VERIFIED: `crates/dsview-sys/src/lib.rs` + `crates/dsview-core/src/lib.rs`] | Phase 10 should add a dedicated option-discovery snapshot that includes operation modes, stop options, channel modes grouped by operation mode, threshold capability, and filter options. [VERIFIED: phase goal in `.planning/ROADMAP.md`] [ASSUMED] | Planned on 2026-04-10 for milestone `v1.1`. [VERIFIED: `.planning/STATE.md` + `.planning/ROADMAP.md`] | This avoids overloading the capture-validation model and gives later phases a stable base for configuration and validation work. [VERIFIED: roadmap phase ordering] [ASSUMED] |
| Current bridge infers some channel-mode metadata from names. [VERIFIED: `crates/dsview-sys/bridge_runtime.c`] | The safer approach is to bridge explicit metadata instead of parsing labels. [VERIFIED: `DSView/libsigrok4DSL/hardware/DSL/dsl.h`] [ASSUMED] | Needed immediately for Phase 10 to avoid publishing unstable discovery facts. [VERIFIED: current bridge limitation] [ASSUMED] | This reduces drift between what DSView actually supports and what the CLI claims. [VERIFIED: underlying driver already stores explicit fields] [ASSUMED] |

**Deprecated/outdated:**

- Reusing `CaptureCapabilities` as the public discovery schema is outdated for this phase because it does not model operation-mode-scoped option sets or threshold-as-range semantics. [VERIFIED: `crates/dsview-core/src/lib.rs` + `crates/dsview-core/src/capture_config.rs`] [ASSUMED]
- Relying on label parsing for `max_enabled_channels` is outdated because the DSLogic driver already has `vld_num` and some labels do not encode a trailing `xN`. [VERIFIED: `DSView/libsigrok4DSL/hardware/DSL/dsl.h` + `crates/dsview-sys/bridge_runtime.c`]

## Assumptions Log

> List all claims tagged `[ASSUMED]` in this research. The planner and discuss-phase use this
> section to identify decisions that need user confirmation before execution.

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | The public Phase 10 Rust model should represent threshold as a float-range capability plus current value instead of only as a discrete enum. | `## Summary`, `## Architecture Patterns`, `## Common Pitfalls` | Medium - planner may build the wrong CLI/schema shape for threshold discovery. |
| A2 | The safest public ID for channel modes in Phase 10 is a numeric-code-backed string such as `channel-mode-<native_code>`. | `## Architecture Patterns` | Medium - downstream CLI compatibility may change if prettier semantic IDs are later preferred. |
| A3 | Discovery should snapshot channel modes per operation mode, possibly by temporarily switching operation mode and restoring it before release. | `## Common Pitfalls`, `## Open Questions` | High - implementation complexity and device-state safety depend on this approach. |
| A4 | The CLI should add a `devices options`-style discovery subcommand rather than attaching this to `capture`. | `## Open Questions`, `## Code Examples` | Low - command naming can be changed later without changing the native bridge design. |
| A5 | `CaptureCapabilities` should not be reused as the public discovery model for Phase 10. | `## Standard Stack`, `## State of the Art` | Low - a planner could still choose to extend it, but that increases coupling risk. |

## Open Questions (RESOLVED)

1. **What should "threshold voltage values" mean for `OPT-01`?**
   - Resolution: Phase 10 will expose threshold as a truthful `SR_CONF_VTH` capability snapshot with `kind = "voltage-range"`, `id = "threshold:vth-range"`, `current_volts`, `min_volts = 0.0`, `max_volts = 5.0`, and `step_volts = 0.1`. If legacy `SR_CONF_THRESHOLD` list metadata is available, it is supplementary raw metadata only and not the authoritative discovery contract. [VERIFIED: plan decisions in `.planning/phases/10-device-option-bridge-and-discovery/10-01-PLAN.md` + `.planning/phases/10-device-option-bridge-and-discovery/10-02-PLAN.md`]
   - Why this was chosen: This matches the DSLogic Plus GUI/device-option contract today and avoids inventing future setter semantics during a discovery-only phase. [VERIFIED: `DSView/libsigrok4DSL/hardware/DSL/dslogic.c` + `DSView/DSView/pv/prop/binding/deviceoptions.cpp`]

2. **Should Phase 10 discovery surface `Internal Test` as an operation mode?**
   - Resolution: Yes. Phase 10 discovery will preserve all DSView operation modes exposed by the driver, including `Internal Test`, while later configuration phases can decide which modes are supported for user selection. [VERIFIED: plan decisions in `.planning/phases/10-device-option-bridge-and-discovery/10-02-PLAN.md`]
   - Why this was chosen: The phase goal is truthful option inspection, and filtering discovery output to the later Phase 12 selection subset would hide real upstream capability. [VERIFIED: `.planning/ROADMAP.md` + `.planning/REQUIREMENTS.md`]

3. **How should channel modes for both buffer and stream be discovered without modifying `DSView/`?**
   - Resolution: The approved Phase 10 strategy is temporary operation-mode switching inside `dsview-sys` during discovery, with original operation mode and current channel mode captured first and restored on every success and failure path. This behavior is blocking-test-covered in Plan 10-01. [VERIFIED: plan decisions in `.planning/phases/10-device-option-bridge-and-discovery/10-01-PLAN.md`]
   - Why this was chosen: `SR_CONF_CHANNEL_MODE` is operation-mode dependent in upstream DSView, and the current public `ds_*` surface does not offer an all-modes snapshot. Controlled switching plus restore-on-exit is the narrowest read-only-compatible approach. [VERIFIED: `DSView/libsigrok4DSL/hardware/DSL/dslogic.c` + `DSView/libsigrok4DSL/libsigrok.h`]

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo` | Rust build and test execution | Yes [VERIFIED: local command probe] | `1.94.1` [VERIFIED: local command probe] | - |
| `rustc` | Rust build and test execution | Yes [VERIFIED: local command probe] | `1.94.1` [VERIFIED: local command probe] | - |
| `cc` | `dsview-sys` shim compilation | Yes [VERIFIED: local command probe] | `14.2.0` [VERIFIED: local command probe] | - |
| `cmake` | source-backed runtime build in `dsview-sys` | Yes [VERIFIED: local command probe] | `3.31.6` [VERIFIED: local command probe] | Build only the shim path if source runtime is not needed. [VERIFIED: `crates/dsview-sys/build.rs`] |
| `pkg-config` | native dependency resolution for source runtime build | Yes [VERIFIED: local command probe] | `1.8.1` [VERIFIED: local command probe] | None for source-runtime builds. [VERIFIED: `crates/dsview-sys/build.rs`] |
| `glib-2.0` | `GVariant`-using bridge and source runtime build | Yes [VERIFIED: local `pkg-config --modversion`] | `2.84.1` [VERIFIED: local `pkg-config --modversion`] | None for native build; required. [VERIFIED: `crates/dsview-sys/build.rs`] |
| `libusb-1.0` | source-backed DSView runtime build | Yes [VERIFIED: local `pkg-config --modversion`] | `1.0.27` [VERIFIED: local `pkg-config --modversion`] | None for native build; required. [VERIFIED: `crates/dsview-sys/build.rs`] |
| `fftw3` | source-backed DSView runtime build | Yes [VERIFIED: local `pkg-config --modversion`] | `3.3.10` [VERIFIED: local `pkg-config --modversion`] | None for source-runtime builds. [VERIFIED: `crates/dsview-sys/build.rs`] |
| `cargo-nextest` | optional faster test runner | No [VERIFIED: local command probe] | - | Use `cargo test`, which already works in this workspace. [VERIFIED: local test runs] |

**Missing dependencies with no fallback:**
- None for the currently recommended `cargo test`-based workflow. [VERIFIED: local test runs succeeded]

**Missing dependencies with fallback:**
- `cargo-nextest` is absent, but standard `cargo test` is sufficient for this phase. [VERIFIED: local command probe + local test runs]

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness via `cargo test`. [VERIFIED: workspace layout + local test runs] |
| Config file | none - standard Cargo test layout. [VERIFIED: repo file scan] |
| Quick run command | `cargo test -p dsview-cli --test devices_cli -- --nocapture` [VERIFIED: local run succeeded] |
| Full suite command | `cargo test` [VERIFIED: workspace structure] [ASSUMED] |

### Phase Requirements -> Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| OPT-01 | Native bridge enumerates operation modes, stop options, filter options, and threshold metadata/current values for an opened `DSLogic Plus` device snapshot. [VERIFIED: phase goal + native API mapping] | sys integration | `cargo test -p dsview-sys --test device_options -- --nocapture` | No - Wave 0. [VERIFIED: current tests inventory] |
| OPT-01 | Core normalizes raw option snapshots into stable IDs, preserves raw numeric codes, and groups channel modes by operation mode. [VERIFIED: phase goal + current core layering] | core unit/integration | `cargo test -p dsview-core --test device_options -- --nocapture` | No - Wave 0. [VERIFIED: current tests inventory] |
| OPT-01 | CLI prints deterministic text and JSON discovery output for a selected device handle. [VERIFIED: phase goal + current CLI render pattern] | CLI integration | `cargo test -p dsview-cli --test device_options_cli -- --nocapture` | No - Wave 0. [VERIFIED: current tests inventory] |

### Sampling Rate

- **Per task commit:** `cargo test -p dsview-sys --test device_options -- --nocapture && cargo test -p dsview-core --test device_options -- --nocapture && cargo test -p dsview-cli --test device_options_cli -- --nocapture` [ASSUMED]
- **Per wave merge:** `cargo test -p dsview-sys && cargo test -p dsview-core && cargo test -p dsview-cli` [ASSUMED]
- **Phase gate:** Full relevant workspace tests plus at least one manual discovery run against a real `DSLogic Plus` before `/gsd-verify-work`. [ASSUMED]

### Wave 0 Gaps

- [ ] `crates/dsview-sys/tests/device_options.rs` - bridge coverage for enum-list copying, threshold metadata/current values, and restore-on-exit channel-mode enumeration. [VERIFIED: missing from current tests inventory] [ASSUMED]
- [ ] `crates/dsview-core/tests/device_options.rs` - normalization coverage for stable IDs, option ordering, and per-operation-mode channel-mode grouping. [VERIFIED: missing from current tests inventory] [ASSUMED]
- [ ] `crates/dsview-cli/tests/device_options_cli.rs` - text/JSON golden-style assertions for deterministic option discovery output. [VERIFIED: missing from current tests inventory] [ASSUMED]
- [ ] A small sys-level helper or fixture strategy to exercise list copying without needing a live hardware device for every test. [VERIFIED: current sys tests already use synthetic export fixtures, but no option-discovery fixture exists] [ASSUMED]

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no [VERIFIED: phase scope is local device discovery only] | not applicable for this phase. [VERIFIED: `.planning/ROADMAP.md`] |
| V3 Session Management | no [VERIFIED: phase scope is local device discovery only] | not applicable for this phase. [VERIFIED: `.planning/ROADMAP.md`] |
| V4 Access Control | no [VERIFIED: phase scope is local single-user CLI discovery] | not applicable for this phase. [VERIFIED: `.planning/ROADMAP.md`] |
| V5 Input Validation | yes [VERIFIED: CLI accepts a device handle and bridge parses untyped native data] | Validate config key expectations with typed bridge functions and stable Rust structs; keep user-facing parsing in `clap`. [VERIFIED: `crates/dsview-cli/src/main.rs` + `crates/dsview-sys/bridge_runtime.c`] [ASSUMED] |
| V6 Cryptography | no [VERIFIED: no cryptographic behavior appears in phase scope or code paths] | none - do not introduce custom crypto. [VERIFIED: current scope/code inspection] |

### Known Threat Patterns for this stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Wrong `GVariant` type or null payload from native API | Tampering / DoS | Validate each option through narrow bridge functions, check `NULL`, and return structured runtime errors instead of dereferencing blindly. [VERIFIED: existing bridge helpers already do this for current getters in `crates/dsview-sys/bridge_runtime.c`] |
| Human-readable labels used as machine identity | Spoofing / Tampering | Emit stable Rust IDs separately from labels and preserve raw numeric codes in JSON for traceability. [VERIFIED: existing stable-ID pattern in `crates/dsview-core/src/lib.rs` + `crates/dsview-cli/src/main.rs`] [ASSUMED] |
| Discovery mutates device state and fails to restore it | Tampering / DoS | If operation-mode switching is required for full channel-mode enumeration, capture original state first and restore on every success/failure path before release. [VERIFIED: current driver setters mutate `op_mode`, `stream`, `ch_mode`, and samplerate-related state in `DSView/libsigrok4DSL/hardware/DSL/dslogic.c`] [ASSUMED] |
| Device-specific unsupported keys reported as generic bridge failure | DoS | Treat `SR_ERR_NA` as unavailable-option state and keep the rest of discovery intact. [VERIFIED: existing optional `VTH` handling in `crates/dsview-sys/src/lib.rs`] |

## Sources

### Primary (HIGH confidence)

- `.planning/ROADMAP.md` - phase goal, success criteria, ordering, and scope constraints. [VERIFIED: local file read]
- `.planning/REQUIREMENTS.md` - `OPT-01` requirement and out-of-scope constraints. [VERIFIED: local file read]
- `CLAUDE.md` - project-specific architecture and safety constraints. [VERIFIED: local file read]
- `DSView/libsigrok4DSL/lib_main.c` - public `ds_*` current/list config APIs. [VERIFIED: local file read]
- `DSView/libsigrok4DSL/libsigrok.h` - public config declarations and config metadata structures. [VERIFIED: local file read]
- `DSView/libsigrok4DSL/hardware/DSL/dslogic.c` - DSLogic option IDs, lists, getters, setters, and dynamic list behavior. [VERIFIED: local file read]
- `DSView/libsigrok4DSL/hardware/DSL/dsl.h` - channel-mode metadata and DSLogic Plus profiles. [VERIFIED: local file read]
- `DSView/libsigrok4DSL/hardware/DSL/dsl.c` - samplerate list shape and shared config-list implementation. [VERIFIED: local file read]
- `DSView/libsigrok4DSL/hwdriver.c` - config-info datatype mapping. [VERIFIED: local file read]
- `DSView/DSView/pv/deviceagent.cpp` - GUI-facing current/list config usage. [VERIFIED: local file read]
- `DSView/DSView/pv/dialogs/deviceoptions.cpp` - GUI option display and channel-mode radio population. [VERIFIED: local file read]
- `DSView/DSView/pv/prop/binding/deviceoptions.cpp` - GUI binding of `SR_CONF_VTH` as a double and list options as enums/lists. [VERIFIED: local file read]
- `crates/dsview-sys/wrapper.h` - current exported bridge ABI. [VERIFIED: local file read]
- `crates/dsview-sys/bridge_runtime.c` - current bridge implementation and ownership pattern. [VERIFIED: local file read]
- `crates/dsview-sys/src/lib.rs` - current Rust wrapper surface and existing optional `VTH` behavior. [VERIFIED: local file read]
- `crates/dsview-core/src/lib.rs` - current discovery/capability orchestration and stable-ID conventions. [VERIFIED: local file read]
- `crates/dsview-core/src/capture_config.rs` - current validation model boundaries. [VERIFIED: local file read]
- `crates/dsview-cli/src/main.rs` - current CLI command organization and stable output conventions. [VERIFIED: local file read]
- `Cargo.lock` and crate `Cargo.toml` files - resolved dependency versions and test stack. [VERIFIED: local file read]
- `https://docs.gtk.org/glib/method.Variant.lookup_value.html` - ownership semantics for `g_variant_lookup_value()`. [CITED: https://docs.gtk.org/glib/method.Variant.lookup_value.html]
- `https://docs.gtk.org/glib/method.Variant.get_fixed_array.html` - fixed-array access semantics for `g_variant_get_fixed_array()`. [CITED: https://docs.gtk.org/glib/method.Variant.get_fixed_array.html]

### Secondary (MEDIUM confidence)

- None. [VERIFIED: this research relied on local source and official GLib docs only]

### Tertiary (LOW confidence)

- None. [VERIFIED: no unverified web-search-only sources were used]

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - the workspace stack, native bridge shape, and dependency versions are directly verified from the repo and local toolchain. [VERIFIED: local file reads + local command probes]
- Architecture: MEDIUM - the DSView option/config mechanics are verified, but threshold contract shape and full per-operation-mode channel-mode enumeration still require one product decision and one sys-level proof. [VERIFIED: local file reads] [ASSUMED]
- Pitfalls: HIGH - the pointer/list ownership pattern, threshold split, and channel-mode dependency are all directly visible in current source. [VERIFIED: local file reads + cited GLib docs]

**Research date:** 2026-04-10
**Valid until:** 2026-05-10 for repo structure and dependency versions, or until upstream `DSView/` or the workspace bridge changes. [VERIFIED: local repo state] [ASSUMED]
