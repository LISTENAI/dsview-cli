# Phase 12: CLI Device Option Surface - Research

**Researched:** 2026-04-13 [VERIFIED: environment_context]
**Domain:** Rust CLI option parsing, DSLogic Plus device-option selection, and pre-acquisition validation wiring [VERIFIED: .planning/phases/12-cli-device-option-surface/12-CONTEXT.md] [VERIFIED: crates/dsview-cli/src/main.rs] [VERIFIED: crates/dsview-core/src/device_option_validation.rs]
**Confidence:** MEDIUM-HIGH [VERIFIED: research synthesis]

<user_constraints>
## User Constraints (from CONTEXT.md) [VERIFIED: .planning/phases/12-cli-device-option-surface/12-CONTEXT.md]

### Locked Decisions
- **D-01:** Extend the existing `capture` command instead of introducing a new top-level command or a nested `capture run` structure.
- **D-02:** Keep `devices options` as the inspection entrypoint rather than adding a separate capture-option inspection command in Phase 12.
- **D-03:** Expose human-readable, stable CLI tokens for option values instead of requiring internal stable IDs such as `operation-mode:0`.
- **D-04:** Internally map those human-readable CLI tokens onto the existing Phase 10/11 stable IDs and native codes rather than changing the core validation model.
- **D-05:** Make `devices options` output more directly aligned with the future `capture` flags so users can read valid values and copy them into `capture`.
- **D-06:** Keep `capture --help` concise and use it to point users toward `devices options` for the full supported-value surface and compatibility context.
- **D-07:** All new device-option flags should be optional; if a flag is omitted, the CLI should preserve the current device value rather than forcing the user to restate every option.
- **D-08:** More specific option flags may infer their parent mode automatically. Example: if a user passes a channel-mode token that belongs to buffer mode, the CLI may infer the matching operation mode rather than requiring an additional explicit parent-mode flag.

### Claude's Discretion
- Exact token spellings for each CLI-facing value, as long as they remain human-readable and stable.
- Whether help output includes inline examples, grouped sections, or “see `devices options`” references.
- Exact text/json layout changes in `devices options`, as long as it becomes more capture-oriented and remains machine-readable.

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within the Phase 12 boundary.
</user_constraints>

<phase_requirements>
## Phase Requirements [VERIFIED: .planning/REQUIREMENTS.md]

| ID | Description | Research Support |
|----|-------------|------------------|
| OPT-02 | User can choose `Buffer Mode` or `Stream Mode` for a `DSLogic Plus` capture run from the CLI. | Add an optional operation-mode flag on `capture`, resolve friendly tokens to Phase 11 stable IDs, and feed the resolved value into `DeviceOptionValidationRequest`. [VERIFIED: .planning/phases/12-cli-device-option-surface/12-CONTEXT.md] [VERIFIED: crates/dsview-core/src/device_option_validation.rs] |
| OPT-03 | User can choose the DSLogic stop option for operation modes that support it. | Keep stop-option selection optional, resolve it against the selected or inferred operation mode, and rely on Phase 11 `StopOptionIncompatibleWithMode` validation instead of new rule code. [VERIFIED: .planning/REQUIREMENTS.md] [VERIFIED: crates/dsview-core/src/device_option_validation.rs] |
| OPT-04 | User can choose a DSLogic channel mode that determines valid channel count and maximum sample rate. | Add a channel-mode flag on `capture`, infer operation mode when absent, and keep sample-rate/channel-count enforcement in the Phase 11 validator. [VERIFIED: .planning/phases/12-cli-device-option-surface/12-CONTEXT.md] [VERIFIED: crates/dsview-core/src/device_option_validation.rs] |
| OPT-05 | User can choose which logic channels are enabled for a run within the selected channel-mode limit. | Reuse the existing `--channels` flag and route it through the richer Phase 11 device-option validator once channel mode is resolved. [VERIFIED: crates/dsview-cli/src/main.rs] [VERIFIED: crates/dsview-core/src/device_option_validation.rs] |
| OPT-06 | User can choose the `DSLogic Plus` threshold voltage from the CLI. | Add an optional threshold-voltage flag, keep primitive parsing in clap, and keep range/step validation in Phase 11. [VERIFIED: .planning/REQUIREMENTS.md] [VERIFIED: crates/dsview-core/src/device_option_validation.rs] [CITED: https://docs.rs/clap/latest/clap/_derive/index.html] |
| OPT-07 | User can choose the DSLogic filter option from the CLI. | Add an optional filter flag, resolve the user token via the device-option snapshot, and keep allowlist enforcement in `UnknownFilter`. [VERIFIED: .planning/REQUIREMENTS.md] [VERIFIED: crates/dsview-core/src/device_option_validation.rs] |
</phase_requirements>

## Summary

Phase 12 is a CLI-surface and validation-wiring phase, not a runtime apply phase: the roadmap and context both keep hardware application and effective-value reporting in Phase 13, while Phase 12 focuses on how users specify, inspect, and understand DSLogic Plus option values from the existing `capture` and `devices options` commands. [VERIFIED: .planning/ROADMAP.md] [VERIFIED: .planning/phases/12-cli-device-option-surface/12-CONTEXT.md] [VERIFIED: .planning/REQUIREMENTS.md]

The current codebase already has the critical ingredients needed for planning: `capture` is a flat clap command with existing `--sample-rate-hz`, `--sample-limit`, and `--channels` flags; `devices options` already exposes current values plus DSView-derived option lists; and Phase 11 added `Discovery::validate_device_option_request(...)` with stable error codes for operation mode, stop option, channel mode, channel counts, sample rates, threshold, and filter compatibility. [VERIFIED: crates/dsview-cli/src/main.rs] [VERIFIED: crates/dsview-cli/src/device_options.rs] [VERIFIED: crates/dsview-core/src/lib.rs] [VERIFIED: crates/dsview-core/src/device_option_validation.rs]

The main planning implication is that Phase 12 needs a CLI-side resolution layer, not a new validation engine. That layer should accept optional human-readable tokens, merge omissions with current device values, infer parent operation mode from child selections when the parent is absent, convert the result into the existing Phase 11 stable IDs, and then call the existing validator before the unchanged capture runtime path proceeds. [VERIFIED: .planning/phases/12-cli-device-option-surface/12-CONTEXT.md] [VERIFIED: crates/dsview-core/src/device_option_validation.rs] [VERIFIED: crates/dsview-core/src/lib.rs]

**Primary recommendation:** Extend `CaptureArgs` with a flattened optional device-option flag group, build a dedicated CLI token resolver keyed by Phase 10/11 stable IDs, reuse `devices options` as the authoritative inspection surface, and keep all compatibility enforcement inside `DeviceOptionValidationRequest` / `validate_device_option_request(...)`. [VERIFIED: .planning/phases/12-cli-device-option-surface/12-CONTEXT.md] [VERIFIED: crates/dsview-cli/src/main.rs] [VERIFIED: crates/dsview-core/src/lib.rs] [VERIFIED: crates/dsview-core/src/device_option_validation.rs]

## Project Constraints (from CLAUDE.md)

- Treat `DSView/` as upstream dependency code and do not modify it for normal project work. [VERIFIED: CLAUDE.md]
- Keep unsafe/native integration isolated behind a small boundary. [VERIFIED: CLAUDE.md]
- Favor scriptable CLI behavior, explicit exit codes, and machine-readable outputs. [VERIFIED: CLAUDE.md]
- Scope `v1` / current milestone work to `DSLogic Plus` only. [VERIFIED: CLAUDE.md] [VERIFIED: .planning/PROJECT.md]
- Preserve the shipped `v1.0` capture/export workflow while adding richer option control. [VERIFIED: CLAUDE.md] [VERIFIED: .planning/PROJECT.md]
- Keep GSD workflow artifacts in sync; Phase 12 planning should not recommend ad-hoc repo edits outside the GSD flow. [VERIFIED: CLAUDE.md]

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `clap` | `4.6.0` | Existing CLI parser, help generator, derive macros, and enum/parser attributes for `dsview-cli`. [VERIFIED: Cargo.lock] | The project already depends on clap derive, and official docs cover `Args`, `ValueEnum`, `next_help_heading`, aliases, and typed parsers needed for a grouped, scriptable Phase 12 flag surface. [VERIFIED: crates/dsview-cli/Cargo.toml] [CITED: https://docs.rs/clap/latest/clap/_derive/index.html] [CITED: https://docs.rs/clap/latest/clap/builder/struct.PossibleValue.html] [CITED: https://docs.rs/clap/latest/clap/builder/trait.TypedValueParser.html] |
| `dsview-core` | `0.1.0` | Existing selected-device inspection and validation APIs. [VERIFIED: crates/dsview-core/Cargo.toml] | Phase 11 already centralized DSLogic Plus compatibility rules here, so Phase 12 should reuse `inspect_device_options`, `load_device_option_validation_capabilities`, and `validate_device_option_request` instead of duplicating them in the CLI. [VERIFIED: crates/dsview-core/src/lib.rs] [VERIFIED: crates/dsview-core/src/device_option_validation.rs] |
| `serde` | `1.0.228` | Stable serialization for JSON responses and machine-readable CLI output. [VERIFIED: Cargo.lock] | `devices options` and error/success responses already use serde-backed structs, so capture-oriented inspection output can extend that pattern without a new serialization dependency. [VERIFIED: crates/dsview-cli/src/device_options.rs] [VERIFIED: crates/dsview-cli/src/main.rs] |
| `serde_json` | `1.0.149` | JSON rendering for automation-facing CLI output. [VERIFIED: Cargo.lock] | The existing CLI treats JSON as the stable automation surface, and Phase 12 inspection output should continue that contract. [VERIFIED: crates/dsview-cli/src/main.rs] [VERIFIED: crates/dsview-cli/tests/device_options_cli.rs] |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `assert_cmd` | `2.2.0` | Integration-style CLI tests for `--help`, parse errors, and stdout/stderr contracts. [VERIFIED: Cargo.lock] | Use for Phase 12 end-to-end CLI cases that assert the new `capture` flags, concise help text, and user-facing diagnostics. [VERIFIED: crates/dsview-cli/tests/capture_cli.rs] [VERIFIED: crates/dsview-cli/tests/device_options_cli.rs] |
| `predicates` | `3.1.4` | Readable stdout/stderr assertions in CLI tests. [VERIFIED: Cargo.lock] | Use alongside `assert_cmd` for text/JSON contract assertions and for regressions around concise help and copyable `devices options` output. [VERIFIED: crates/dsview-cli/tests/capture_cli.rs] [VERIFIED: crates/dsview-cli/tests/device_options_cli.rs] |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Extending the existing `capture` command | A new top-level command or nested `capture run` structure | Rejected because the context explicitly locks Phase 12 to the existing `capture` command shape. [VERIFIED: .planning/phases/12-cli-device-option-surface/12-CONTEXT.md] |
| Reusing clap plus runtime resolution | Adding a second parser crate or a bespoke CLI grammar | Unnecessary because clap is already in the workspace and the hard part is device-aware resolution, not primitive parsing. [VERIFIED: crates/dsview-cli/Cargo.toml] [CITED: https://docs.rs/clap/latest/clap/_derive/index.html] |
| Runtime token resolution keyed by stable IDs | Compile-time `ValueEnum` for every device option value | Poor fit because many valid values are selected-device and operation-mode dependent, while Phase 10/11 already provide the authoritative runtime data and stable IDs. [VERIFIED: crates/dsview-core/src/device_options.rs] [VERIFIED: crates/dsview-core/src/device_option_validation.rs] [VERIFIED: .planning/phases/12-cli-device-option-surface/12-CONTEXT.md] |

**Installation:** No new third-party crate is required for Phase 12 planning; the existing workspace dependencies and toolchain are sufficient. [VERIFIED: crates/dsview-cli/Cargo.toml] [VERIFIED: crates/dsview-core/Cargo.toml] [VERIFIED: cargo test]

**Version verification:** The versions above are the versions currently declared and locked in this workspace, verified from `Cargo.toml` and `Cargo.lock`. [VERIFIED: Cargo.lock] [VERIFIED: crates/dsview-cli/Cargo.toml] [VERIFIED: crates/dsview-core/Cargo.toml]

## Architecture Patterns

### Recommended Project Structure
```text
crates/dsview-cli/src/
├── main.rs                    # Existing clap command surface and capture dispatch
├── device_options.rs          # Existing `devices options` response and text renderer
└── capture_device_options.rs  # Recommended Phase 12 resolver/token-mapping helper
```
[VERIFIED: crates/dsview-cli/src/main.rs] [VERIFIED: crates/dsview-cli/src/device_options.rs] [ASSUMED]

### Pattern 1: Flat `capture` flags with a dedicated help section
**What:** Add a flattened optional args struct for device-option flags under the existing `capture` command rather than creating a new command branch. [VERIFIED: .planning/phases/12-cli-device-option-surface/12-CONTEXT.md] [VERIFIED: crates/dsview-cli/src/main.rs]

**When to use:** Use this for operation mode, stop option, channel mode, threshold voltage, and filter selection, while keeping existing `--sample-rate-hz`, `--sample-limit`, and `--channels` intact. [VERIFIED: crates/dsview-cli/src/main.rs] [VERIFIED: .planning/REQUIREMENTS.md]

**Example:**
```rust
#[derive(Args, Debug)]
#[command(next_help_heading = "Device options")]
struct CaptureDeviceOptionArgs {
    #[arg(long = "operation-mode", value_name = "TOKEN")]
    operation_mode: Option<String>,
}
```
This pattern matches clap's documented derive attributes and the project's current `Args` / `flatten` usage. [CITED: https://docs.rs/clap/latest/clap/_derive/index.html] [VERIFIED: crates/dsview-cli/src/main.rs]

### Pattern 2: Resolve partial CLI input into a full Phase 11 request
**What:** Build a CLI-only resolver that takes optional user flags plus selected-device facts, preserves current values for omitted flags, performs limited parent inference, and emits a full `DeviceOptionValidationRequest`. [VERIFIED: .planning/phases/12-cli-device-option-surface/12-CONTEXT.md] [VERIFIED: crates/dsview-core/src/device_option_validation.rs]

**When to use:** Run this after parsing the command line and after validating `--handle`, but before `run_capture(...)` or any future apply-time work. [VERIFIED: crates/dsview-cli/src/main.rs] [VERIFIED: crates/dsview-core/src/lib.rs]

**Example:**
```rust
let snapshot = discovery.inspect_device_options(handle)?;
let capabilities = discovery.load_device_option_validation_capabilities(handle)?;
let validated = discovery.validate_device_option_request(handle, &request)?;
```
All three APIs already exist; Phase 12 needs to add the `request`-building layer between them. [VERIFIED: crates/dsview-core/src/lib.rs]

### Pattern 3: Keep inspection and validation as separate responsibilities
**What:** Use `devices options` to teach the user which tokens and constraints exist, and keep `DeviceOptionValidationCapabilities::validate_request(...)` as the only semantic rule gate. [VERIFIED: .planning/phases/12-cli-device-option-surface/12-CONTEXT.md] [VERIFIED: crates/dsview-cli/src/device_options.rs] [VERIFIED: crates/dsview-core/src/device_option_validation.rs]

**When to use:** Use `devices options` for copyable option discovery and `capture` for user input, but do not move DSView rule tables into clap metadata or text rendering code. [VERIFIED: .planning/phases/12-cli-device-option-surface/12-CONTEXT.md] [VERIFIED: crates/dsview-cli/src/device_options.rs] [VERIFIED: crates/dsview-core/src/device_option_validation.rs]

**Example:**
```rust
let response = build_device_options_response(&snapshot);
render_device_options_success(args.runtime.format, &response);
```
That existing split is already in place; Phase 12 should reshape the response, not replace the command. [VERIFIED: crates/dsview-cli/src/main.rs] [VERIFIED: crates/dsview-cli/src/device_options.rs]

### Anti-Patterns to Avoid
- **New command tree for capture options:** This violates locked decision D-01 and would force unnecessary help and test churn. [VERIFIED: .planning/phases/12-cli-device-option-surface/12-CONTEXT.md]
- **Label parsing as the source of truth:** Phase 10 and Phase 11 stabilized IDs and native codes precisely so later phases would not recover meaning from mutable DSView labels. [VERIFIED: crates/dsview-core/src/device_options.rs] [VERIFIED: crates/dsview-core/tests/device_options.rs] [VERIFIED: .planning/phases/11-device-option-validation-model/11-03-SUMMARY.md]
- **Duplicate validation logic in `dsview-cli`:** The Phase 11 validator already covers mode compatibility, channel limits, sample rates, threshold range/step, and filter allowlists. [VERIFIED: crates/dsview-core/src/device_option_validation.rs] [VERIFIED: .planning/phases/11-device-option-validation-model/11-VERIFICATION.md]
- **Using `cargo test -p dsview-cli --lib` as the main proof command:** Phase 11 verification showed that the meaningful CLI unit tests live in the binary target (`src/main.rs`), so `--bin dsview-cli` is the correct command. [VERIFIED: .planning/phases/11-device-option-validation-model/11-VERIFICATION.md] [VERIFIED: cargo test]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| DSLogic Plus compatibility rules | A second matrix of mode/filter/threshold/channel constraints in `dsview-cli` | `Discovery::validate_device_option_request(...)` and `DeviceOptionValidationCapabilities::validate_request(...)` [VERIFIED: crates/dsview-core/src/lib.rs] [VERIFIED: crates/dsview-core/src/device_option_validation.rs] | The validator already owns the stable error taxonomy and the DSView-derived rule set. [VERIFIED: .planning/phases/11-device-option-validation-model/11-VERIFICATION.md] |
| User-facing option discovery | A new inspection command or hardcoded README table | `devices options` plus a capture-oriented response schema [VERIFIED: .planning/phases/12-cli-device-option-surface/12-CONTEXT.md] [VERIFIED: crates/dsview-cli/src/device_options.rs] | The command already has JSON and deterministic text output, so Phase 12 should evolve that surface instead of splitting user guidance across commands. [VERIFIED: crates/dsview-cli/tests/device_options_cli.rs] |
| Channel/sample-limit arithmetic | Ad-hoc math in Phase 12 parsing code | Existing Phase 11 validation and shared sample-limit alignment helpers [VERIFIED: crates/dsview-core/src/device_option_validation.rs] [VERIFIED: crates/dsview-core/src/capture_config.rs] | Phase 11 already fixed overflow-safe alignment and capacity checks; duplicating the math risks drift. [VERIFIED: .planning/phases/11-device-option-validation-model/11-03-SUMMARY.md] |
| Friendly token mapping | Runtime regexes over DSView labels | An explicit CLI token map keyed by stable IDs/native codes [VERIFIED: .planning/phases/12-cli-device-option-surface/12-CONTEXT.md] [VERIFIED: crates/dsview-core/tests/device_options.rs] [ASSUMED] | Stable IDs are fixed and already normalized; a token map built on top of them preserves D-04 without making label wording part of the contract. [VERIFIED: .planning/phases/12-cli-device-option-surface/12-CONTEXT.md] [ASSUMED] |

**Key insight:** Phase 12 should add a translation layer, not a new domain model: Phase 10 already normalized IDs, Phase 11 already validates full requests, and the missing piece is CLI ergonomics plus deterministic token-to-ID resolution. [VERIFIED: crates/dsview-core/src/device_options.rs] [VERIFIED: crates/dsview-core/src/device_option_validation.rs] [VERIFIED: .planning/phases/12-cli-device-option-surface/12-CONTEXT.md]

## Common Pitfalls

### Pitfall 1: Trying to make clap the source of truth for device-dependent values
**What goes wrong:** The parser or help text advertises values that are not valid for the selected device or selected operation mode. [VERIFIED: crates/dsview-core/src/device_option_validation.rs] [CITED: https://docs.rs/clap/latest/clap/_derive/index.html]

**Why it happens:** clap's `ValueEnum`, `PossibleValue`, and typed parser features are excellent for static token sets, but Phase 12 values depend on runtime-selected device state and DSView capability probing. [CITED: https://docs.rs/clap/latest/clap/_derive/index.html] [CITED: https://docs.rs/clap/latest/clap/builder/struct.PossibleValue.html] [VERIFIED: crates/dsview-core/src/lib.rs]

**How to avoid:** Use clap for primitive shape and help grouping, then resolve tokens against `inspect_device_options(...)` / validation capabilities after the handle is known. [VERIFIED: crates/dsview-core/src/lib.rs] [VERIFIED: crates/dsview-cli/src/main.rs]

**Warning signs:** `capture --help` starts listing DSView labels or mode-specific values that cannot be correct before `--handle` is loaded. [VERIFIED: .planning/phases/12-cli-device-option-surface/12-CONTEXT.md] [ASSUMED]

### Pitfall 2: Losing the "preserve current value" behavior
**What goes wrong:** Omitted flags turn into `None` or fake defaults instead of the selected device's current values, causing validation failures or surprising behavior. [VERIFIED: .planning/phases/12-cli-device-option-surface/12-CONTEXT.md] [VERIFIED: crates/dsview-core/src/device_option_validation.rs]

**Why it happens:** Phase 11 requests require concrete `operation_mode_id` and `channel_mode_id`, so Phase 12 must fill those from current device state before validation. [VERIFIED: crates/dsview-core/src/device_option_validation.rs]

**How to avoid:** Resolve omissions from `CurrentDeviceOptionValues` after opening the selected handle and before calling `validate_device_option_request(...)`. [VERIFIED: crates/dsview-core/src/device_options.rs] [VERIFIED: crates/dsview-core/src/lib.rs]

**Warning signs:** A plain baseline `capture` command starts failing because device-option flags were absent. [VERIFIED: .planning/PROJECT.md] [ASSUMED]

### Pitfall 3: Letting inference override explicit parent flags
**What goes wrong:** A child selection such as channel mode or stop option silently wins over an explicitly passed operation mode, which makes the CLI hard to trust in automation. [VERIFIED: .planning/phases/12-cli-device-option-surface/12-CONTEXT.md] [ASSUMED]

**Why it happens:** D-08 permits inference when the parent is omitted, but it does not authorize silently changing an explicit parent flag. [VERIFIED: .planning/phases/12-cli-device-option-surface/12-CONTEXT.md]

**How to avoid:** Infer only when the parent flag is absent and the child maps to exactly one parent; otherwise return the existing stable incompatibility error from Phase 11. [VERIFIED: .planning/phases/12-cli-device-option-surface/12-CONTEXT.md] [VERIFIED: crates/dsview-core/src/device_option_validation.rs]

**Warning signs:** The implementation mutates a parsed parent flag before validation or never surfaces `channel_mode_incompatible` / `stop_option_incompatible` anymore. [VERIFIED: crates/dsview-core/src/device_option_validation.rs] [ASSUMED]

### Pitfall 4: Implying Phase 12 applies runtime device settings
**What goes wrong:** Help text, success output, or tests accidentally claim that the runtime used the selected option values even though apply-time work is deferred to Phase 13. [VERIFIED: .planning/ROADMAP.md] [VERIFIED: .planning/phases/12-cli-device-option-surface/12-CONTEXT.md]

**Why it happens:** The `capture` command already runs real acquisitions, so it is easy to blur "accepted and validated" with "applied to hardware". [VERIFIED: crates/dsview-cli/src/main.rs] [VERIFIED: crates/dsview-core/src/lib.rs]

**How to avoid:** Keep Phase 12 focused on parsing, inspection, request construction, and validation wiring; leave runtime option mutation and effective-value reporting to Phase 13. [VERIFIED: .planning/ROADMAP.md] [VERIFIED: .planning/phases/12-cli-device-option-surface/12-CONTEXT.md]

**Warning signs:** Phase 12 starts editing `apply_capture_config(...)` or adding success metadata about effective device options. [VERIFIED: crates/dsview-core/src/lib.rs] [VERIFIED: .planning/ROADMAP.md]

### Pitfall 5: Missing the right test targets
**What goes wrong:** CLI logic changes look green even though the relevant `main.rs` tests never ran. [VERIFIED: .planning/phases/11-device-option-validation-model/11-VERIFICATION.md]

**Why it happens:** `dsview-cli` has both integration tests in `tests/` and binary unit tests in `src/main.rs`; Phase 11 verification explicitly found that `--lib` misses the binary tests. [VERIFIED: crates/dsview-cli/tests/capture_cli.rs] [VERIFIED: crates/dsview-cli/src/main.rs] [VERIFIED: .planning/phases/11-device-option-validation-model/11-VERIFICATION.md]

**How to avoid:** Treat `cargo test -p dsview-cli --bin dsview-cli -- --nocapture` as the quick unit command for validation mapping, and keep `capture_cli` / `device_options_cli` as the integration proof points. [VERIFIED: cargo test] [VERIFIED: crates/dsview-cli/tests/capture_cli.rs] [VERIFIED: crates/dsview-cli/tests/device_options_cli.rs]

**Warning signs:** A plan or verification checklist references only `cargo test -p dsview-cli --lib ...`. [VERIFIED: .planning/phases/11-device-option-validation-model/11-VERIFICATION.md]

## Code Examples

Verified patterns from official docs and the current codebase:

### Group new flags under a dedicated help heading without changing the command tree
```rust
#[derive(Args, Debug)]
#[command(next_help_heading = "Device options")]
struct CaptureDeviceOptionArgs {
    #[arg(long = "filter", value_name = "TOKEN")]
    filter: Option<String>,
}
```
This is the clap-supported way to group flags while staying inside the existing derive-based command structure. [CITED: https://docs.rs/clap/latest/clap/_derive/index.html] [VERIFIED: crates/dsview-cli/src/main.rs]

### Keep comma-delimited channels as the enabled-channel surface
```rust
#[arg(
    long = "channels",
    value_delimiter = ',',
    value_name = "IDX[,IDX...]",
    help = "Comma-separated logic channel indexes to enable, for example 0,1,2,3"
)]
channels: Vec<u16>,
```
Phase 12 does not need a new channel-selection syntax; it needs the richer validator to interpret the existing one against the chosen channel mode. [VERIFIED: crates/dsview-cli/src/main.rs] [VERIFIED: .planning/REQUIREMENTS.md]

### Reuse the Phase 11 validation entrypoint before runtime work
```rust
let validated = discovery
    .validate_device_option_request(handle, &request)
    .map_err(|error| command_error(args.runtime.format, classify_validation_error(&error)))?;
```
The entrypoint already exists and already returns the stable validation taxonomy Phase 12 needs for user-facing diagnostics. [VERIFIED: crates/dsview-core/src/lib.rs] [VERIFIED: crates/dsview-cli/src/main.rs]

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `devices options` exposes normalized stable IDs and labels, but `capture` does not yet accept device-option flags. [VERIFIED: crates/dsview-cli/src/device_options.rs] [VERIFIED: crates/dsview-cli/src/main.rs] | Add human-readable capture flags that map back to the existing stable IDs rather than changing the Phase 10/11 model. [VERIFIED: .planning/phases/12-cli-device-option-surface/12-CONTEXT.md] [ASSUMED] | Planned for Phase 12. [VERIFIED: .planning/ROADMAP.md] | Users get copyable CLI tokens while the core model stays stable for automation and later apply/reporting work. [VERIFIED: .planning/phases/12-cli-device-option-surface/12-CONTEXT.md] [ASSUMED] |
| Baseline capture validation only checked sample rate, sample limit, and channels. [VERIFIED: crates/dsview-core/src/capture_config.rs] | Phase 11 added full device-option validation for operation mode, stop option, channel mode, threshold, and filter compatibility. [VERIFIED: crates/dsview-core/src/device_option_validation.rs] [VERIFIED: .planning/phases/11-device-option-validation-model/11-VERIFICATION.md] | 2026-04-13 in Phase 11. [VERIFIED: .planning/phases/11-device-option-validation-model/11-VERIFICATION.md] | Phase 12 can focus on CLI ergonomics and request construction instead of inventing a new rules engine. [VERIFIED: crates/dsview-core/src/lib.rs] [VERIFIED: .planning/phases/11-device-option-validation-model/11-02-SUMMARY.md] |

**Deprecated/outdated:**
- Treating internal IDs such as `operation-mode:0` as the end-user CLI contract is outdated for Phase 12 because D-03 explicitly requires human-readable stable tokens instead. [VERIFIED: .planning/phases/12-cli-device-option-surface/12-CONTEXT.md]
- Using `cargo test -p dsview-cli --lib ...` as the primary CLI proof command is outdated because it misses the binary unit tests in `src/main.rs`. [VERIFIED: .planning/phases/11-device-option-validation-model/11-VERIFICATION.md]

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | The best implementation split is a new CLI-side helper module such as `crates/dsview-cli/src/capture_device_options.rs` rather than keeping all resolver logic in `main.rs`. [ASSUMED] | Architecture Patterns | Low-to-medium; the plan may need a different file layout, but the core work stays the same. |
| A2 | Friendly token spellings should be short slug-style aliases layered on top of stable IDs rather than exact DSView labels. [ASSUMED] | Don't Hand-Roll / Open Questions | Medium; different token choices change tests, help text, and `devices options` output shape. |
| A3 | `devices options` should likely expose both the CLI token and the underlying stable ID in JSON during `v1.1` to avoid regressing machine-readability. [ASSUMED] | Open Questions | Medium; if the planner chooses token-only JSON, downstream automation docs and fixtures must be updated carefully. |

## Open Questions (RESOLVED)

1. **Should `devices options` expose both CLI tokens and stable IDs, or only CLI tokens?**  
   **RESOLVED:** Expose both `token` and `stable_id` in JSON for `v1.1`, while letting text output optimize for copy-paste ergonomics. This keeps the command machine-readable for automation and aligned with the capture-facing token surface. [VERIFIED: .planning/phases/12-cli-device-option-surface/12-CONTEXT.md] [VERIFIED: .planning/phases/12-cli-device-option-surface/12-01-SUMMARY.md]

2. **Should Phase 12 say anything at success time about deferred apply behavior?**  
   **RESOLVED:** Keep success output unchanged in Phase 12 and make the Phase 13 boundary clear through help text, tests, and plan notes rather than transitional success metadata. [VERIFIED: .planning/ROADMAP.md] [VERIFIED: .planning/phases/12-cli-device-option-surface/12-CONTEXT.md]

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo` | Build and run Phase 12 unit/integration tests | ✓ [VERIFIED: `command -v cargo && cargo --version`] | `1.94.1` [VERIFIED: `cargo --version`] | — |
| `rustc` | Compile `dsview-cli` / `dsview-core` during Phase 12 work | ✓ [VERIFIED: `command -v rustc && rustc --version`] | `1.94.1` [VERIFIED: `rustc --version`] | — |
| `pkg-config` | Native build support already used by the workspace | ✓ [VERIFIED: `command -v pkg-config && pkg-config --version`] | `1.8.1` [VERIFIED: `pkg-config --version`] | — |

**Missing dependencies with no fallback:** None found in the current environment. [VERIFIED: environment probe]

**Missing dependencies with fallback:** None needed for Phase 12 planning. [VERIFIED: environment probe]

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust `cargo test` with integration tests via `assert_cmd` + `predicates`, plus binary unit tests in `src/main.rs`. [VERIFIED: crates/dsview-cli/tests/capture_cli.rs] [VERIFIED: crates/dsview-cli/src/main.rs] |
| Config file | None detected; the workspace uses Cargo defaults. [VERIFIED: codebase grep] |
| Quick run command | `cargo test -p dsview-cli --bin dsview-cli -- --nocapture` [VERIFIED: cargo test] |
| Full suite command | `cargo test --workspace -- --nocapture` [VERIFIED: .planning/phases/11-device-option-validation-model/11-03-SUMMARY.md] |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| OPT-02 | `capture` accepts operation-mode tokens, keeps help concise, and reports stable validation errors for incompatible combinations. [VERIFIED: .planning/REQUIREMENTS.md] | Integration + binary unit | `cargo test -p dsview-cli --test capture_cli -- --nocapture` and `cargo test -p dsview-cli --bin dsview-cli -- --nocapture` [VERIFIED: cargo test] | Harness exists in `crates/dsview-cli/tests/capture_cli.rs` and `crates/dsview-cli/src/main.rs`; Phase 12-specific cases are missing. [VERIFIED: crates/dsview-cli/tests/capture_cli.rs] [VERIFIED: crates/dsview-cli/src/main.rs] |
| OPT-03 | Stop-option tokens validate only for compatible operation modes. [VERIFIED: .planning/REQUIREMENTS.md] | Binary unit + integration | `cargo test -p dsview-cli --bin dsview-cli -- --nocapture` and `cargo test -p dsview-core --test device_option_validation -- --nocapture` [VERIFIED: cargo test] | Existing binary/core harness exists; Phase 12 CLI cases are missing. [VERIFIED: crates/dsview-cli/src/main.rs] [VERIFIED: crates/dsview-core/tests/device_option_validation.rs] |
| OPT-04 | Channel-mode tokens resolve into mode-aware validation for sample-rate and enabled-channel limits. [VERIFIED: .planning/REQUIREMENTS.md] | Integration + core regression | `cargo test -p dsview-cli --test capture_cli -- --nocapture` and `cargo test -p dsview-core --test device_option_validation -- --nocapture` [VERIFIED: cargo test] | Existing harness exists; add Phase 12 cases. [VERIFIED: crates/dsview-cli/tests/capture_cli.rs] [VERIFIED: crates/dsview-core/tests/device_option_validation.rs] |
| OPT-05 | Existing `--channels` continues working but is now checked against the resolved channel mode. [VERIFIED: .planning/REQUIREMENTS.md] [VERIFIED: crates/dsview-cli/src/main.rs] | Integration + core regression | `cargo test -p dsview-cli --test capture_cli -- --nocapture` and `cargo test -p dsview-core --test device_option_validation -- --nocapture` [VERIFIED: cargo test] | Existing harness exists; add Phase 12 cases. [VERIFIED: crates/dsview-cli/tests/capture_cli.rs] [VERIFIED: crates/dsview-core/tests/device_option_validation.rs] |
| OPT-06 | Threshold-voltage input parses and fails cleanly on range/step violations. [VERIFIED: .planning/REQUIREMENTS.md] | Binary unit + integration | `cargo test -p dsview-cli --bin dsview-cli -- --nocapture` and `cargo test -p dsview-cli --test capture_cli -- --nocapture` [VERIFIED: cargo test] | Existing harness exists; add threshold-specific parse/diagnostic cases. [VERIFIED: crates/dsview-cli/src/main.rs] [VERIFIED: crates/dsview-cli/tests/capture_cli.rs] |
| OPT-07 | Filter tokens resolve through the selected-device allowlist and fail with stable codes when unsupported. [VERIFIED: .planning/REQUIREMENTS.md] | Binary unit + integration | `cargo test -p dsview-cli --bin dsview-cli -- --nocapture` and `cargo test -p dsview-cli --test capture_cli -- --nocapture` [VERIFIED: cargo test] | Existing harness exists; add filter-specific cases. [VERIFIED: crates/dsview-cli/src/main.rs] [VERIFIED: crates/dsview-cli/tests/capture_cli.rs] |

### Sampling Rate
- **Per task commit:** `cargo test -p dsview-cli --bin dsview-cli -- --nocapture` [VERIFIED: cargo test]
- **Per wave merge:** `cargo test -p dsview-cli --test capture_cli --test device_options_cli --test devices_cli -- --nocapture` [VERIFIED: cargo test]
- **Phase gate:** `cargo test --workspace -- --nocapture` [VERIFIED: .planning/phases/11-device-option-validation-model/11-03-SUMMARY.md]

### Wave 0 Gaps
- [ ] Extend `crates/dsview-cli/tests/capture_cli.rs` with happy-path and failure-path cases for `--operation-mode`, `--stop-option`, `--channel-mode`, `--threshold-volts`, and `--filter`. [VERIFIED: crates/dsview-cli/tests/capture_cli.rs]
- [ ] Extend `crates/dsview-cli/tests/device_options_cli.rs` so `devices options` asserts capture-oriented text/JSON output, not just raw stable-ID sections. [VERIFIED: crates/dsview-cli/tests/device_options_cli.rs]
- [ ] Add resolver-focused unit tests in the CLI layer so parent inference, current-value preservation, and token-to-stable-ID mapping do not depend solely on integration tests. [ASSUMED]
- [ ] Keep `crates/dsview-core/tests/device_option_validation.rs` as the semantic rule regression suite; do not fork semantic validation into CLI-only tests. [VERIFIED: crates/dsview-core/tests/device_option_validation.rs]

## Security Domain

### Applicable ASVS Categories
| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no [VERIFIED: phase scope review] | Not part of this local hardware CLI phase. [VERIFIED: .planning/PROJECT.md] |
| V3 Session Management | no [VERIFIED: phase scope review] | Not part of this non-daemon CLI phase. [VERIFIED: .planning/PROJECT.md] |
| V4 Access Control | no [VERIFIED: phase scope review] | The phase does not introduce user/account authorization boundaries. [VERIFIED: .planning/PROJECT.md] |
| V5 Input Validation | yes [VERIFIED: phase scope review] | Use clap for primitive parsing and `DeviceOptionValidationCapabilities::validate_request(...)` for selected-device semantic validation. [CITED: https://docs.rs/clap/latest/clap/_derive/index.html] [VERIFIED: crates/dsview-core/src/device_option_validation.rs] |
| V6 Cryptography | no [VERIFIED: phase scope review] | No cryptographic behavior is introduced in this phase. [VERIFIED: .planning/PROJECT.md] |

### Known Threat Patterns for Rust CLI device-option selection
| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Invalid numeric or enum-like input for threshold, channels, or token flags | Tampering | Parse primitives with clap, then reject semantic violations with the stable Phase 11 validation taxonomy before acquisition begins. [CITED: https://docs.rs/clap/latest/clap/_derive/index.html] [VERIFIED: crates/dsview-core/src/device_option_validation.rs] |
| Mode/child mismatches such as a buffer-only stop option in stream mode | Tampering | Keep parent inference narrow and let the validator surface `stop_option_incompatible` / `channel_mode_incompatible` instead of guessing silently. [VERIFIED: crates/dsview-core/src/device_option_validation.rs] [VERIFIED: .planning/phases/12-cli-device-option-surface/12-CONTEXT.md] |
| Over-capacity sample-limit or enabled-channel requests after a channel-mode change | Denial of Service | Reuse Phase 11 capacity checks and aligned sample-limit math rather than duplicating it in CLI code. [VERIFIED: crates/dsview-core/src/device_option_validation.rs] [VERIFIED: crates/dsview-core/src/capture_config.rs] |

## Sources

### Primary (HIGH confidence)
- `.planning/phases/12-cli-device-option-surface/12-CONTEXT.md` - locked decisions, scope boundary, and discretionary areas. [VERIFIED: local file]
- `.planning/REQUIREMENTS.md` - Phase 12 requirement targets `OPT-02` through `OPT-07`. [VERIFIED: local file]
- `.planning/ROADMAP.md` - milestone/phase goal, boundary with Phase 13, and success criteria. [VERIFIED: local file]
- `.planning/PROJECT.md` and `CLAUDE.md` - project constraints and workflow guardrails. [VERIFIED: local file]
- `.planning/phases/11-device-option-validation-model/11-02-SUMMARY.md`, `.planning/phases/11-device-option-validation-model/11-03-SUMMARY.md`, and `.planning/phases/11-device-option-validation-model/11-VERIFICATION.md` - Phase 11 guarantees and the correct verification commands. [VERIFIED: local file]
- `crates/dsview-cli/src/main.rs`, `crates/dsview-cli/src/device_options.rs`, `crates/dsview-core/src/lib.rs`, `crates/dsview-core/src/device_option_validation.rs`, `crates/dsview-core/src/device_options.rs`, and related tests - current surface, existing APIs, and regression harnesses. [VERIFIED: codebase grep]
- `cargo test -p dsview-cli --bin dsview-cli -- --nocapture`, `cargo test -p dsview-cli --test capture_cli --test device_options_cli --test devices_cli -- --nocapture`, and `cargo test -p dsview-core --test device_option_validation -- --nocapture` - current green test evidence. [VERIFIED: cargo test]
- `https://docs.rs/clap/latest/clap/_derive/index.html` - clap derive attributes, help headings, and parser/value metadata. [CITED: https://docs.rs/clap/latest/clap/_derive/index.html]
- `https://docs.rs/clap/latest/clap/builder/struct.PossibleValue.html` - clap value aliases and user-facing token metadata. [CITED: https://docs.rs/clap/latest/clap/builder/struct.PossibleValue.html]
- `https://docs.rs/clap/latest/clap/builder/trait.TypedValueParser.html` - clap typed parser transformation hooks. [CITED: https://docs.rs/clap/latest/clap/builder/trait.TypedValueParser.html]

### Secondary (MEDIUM confidence)
- None; the primary sources were sufficient for this phase. [VERIFIED: research process]

### Tertiary (LOW confidence)
- None; all unverified recommendations are isolated in the Assumptions Log. [VERIFIED: research process]

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - the relevant crates, versions, and test harness are all directly verifiable in the workspace, and the external clap guidance comes from official docs. [VERIFIED: Cargo.lock] [VERIFIED: cargo test] [CITED: https://docs.rs/clap/latest/clap/_derive/index.html]
- Architecture: MEDIUM-HIGH - the core flow (`capture` + `devices options` + Phase 11 validator) is well verified, but exact token spellings and the cleanest CLI module split remain discretionary. [VERIFIED: .planning/phases/12-cli-device-option-surface/12-CONTEXT.md] [VERIFIED: crates/dsview-cli/src/main.rs] [ASSUMED]
- Pitfalls: HIGH - the main regressions are grounded in locked phase decisions, verified Phase 11 behavior, and existing test-target structure. [VERIFIED: .planning/phases/12-cli-device-option-surface/12-CONTEXT.md] [VERIFIED: .planning/phases/11-device-option-validation-model/11-VERIFICATION.md]

**Research date:** 2026-04-13 [VERIFIED: environment_context]
**Valid until:** 2026-05-13 for codebase-specific planning, with earlier refresh only if the clap or workspace dependency surface changes. [VERIFIED: research synthesis] [ASSUMED]
