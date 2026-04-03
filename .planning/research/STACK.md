# Stack Research: DSView CLI

**Researched:** 2026-04-03
**Scope:** Rust CLI that reuses `DSView/` and `libsigrok4DSL` without modifying the submodule
**Confidence:** Medium-High

## Summary

The most practical v1 stack is a Rust workspace that keeps product logic in Rust while treating the DSView submodule as an upstream device/backend dependency. Because `DSView/` is GUI-oriented and pulls in heavy Qt dependencies at the application layer, the CLI should avoid linking against DSView's GUI code directly. Instead, it should integrate at the `libsigrok4DSL` boundary and only consume the capture/export capabilities needed for `DSLogic Plus`.

For v1, prefer a thin, explicit adapter layer between Rust and the existing C stack. The CLI should own argument parsing, config validation, orchestration, and file-system UX in Rust, while capture/session execution stays close to the proven `libsigrok4DSL` behavior. The safest early deliverable is export to VCD because the DSView stack already contains a VCD output path and VCD preserves waveform semantics better than a plain tabular export for downstream analysis.

**Primary recommendation:** Build a Rust CLI with a dedicated `dsview-sys` FFI layer and a small Rust-side safe wrapper, linking only to the DSView-side capture/export libraries required for logic acquisition and VCD export.

## Recommended Stack

### Core

| Tool / Library | Purpose | Why it fits here | Confidence |
|---|---|---|---|
| Rust stable edition | Main CLI implementation | Meets the explicit project requirement and keeps command UX/testability clean | High |
| `clap` | CLI argument parsing | Mature, ergonomic, good for subcommands like `device list`, `capture run`, `export` | High |
| `anyhow` + `thiserror` | Error handling | Lets the CLI expose actionable hardware/setup errors while keeping FFI errors structured | High |
| `serde` + `toml` / `serde_json` | Config + machine-readable metadata | Useful for capture manifests and persisted presets later | High |
| Custom `-sys` crate (`bindgen`-driven where feasible) | Raw C FFI bindings | Standard Rust pattern for brownfield native integration | High |
| Safe wrapper crate over the sys layer | Encapsulate unsafe calls and lifecycle rules | Prevents unsafe/native details from leaking into CLI commands | High |

### Build / Native Integration

| Tool / Pattern | Purpose | Recommendation |
|---|---|---|
| `build.rs` | Native include/link wiring | Use to point Rust at DSView submodule headers/libs and to gate platform-specific flags |
| `cmake` crate or external prebuild step | Build or locate native artifacts | Prefer a controlled adapter-library build over trying to compile the full DSView app |
| Generated bindings checked into repo selectively | Reduce developer friction | Acceptable if headers are stable enough; regenerate deliberately when submodule changes |
| Version-pinned submodule revision | ABI stability | Treat DSView submodule revision as part of the CLI release contract |

### Export / Output

| Format | Recommendation | Why |
|---|---|---|
| VCD | v1 primary export | Already supported in `DSView/libsigrok4DSL/output/vcd.c`; preserves timing + signal transitions |
| JSON sidecar manifest | v1 strongly recommended companion output | Adds device model, samplerate, enabled channels, sample count, timestamp, and command metadata for AI workflows |
| CSV | Optional later | Easy to parse, but weaker as the canonical waveform format |

### Testing

| Tool / Pattern | Purpose | Recommendation |
|---|---|---|
| Rust unit tests | CLI/config/validation logic | Use heavily |
| Golden-file tests | Export contract verification | Essential for VCD header/body stability |
| Integration tests with fixture captures | End-to-end without hardware | Use recorded packets/snapshots where possible |
| Hardware-in-the-loop smoke tests | Validate real DSLogic Plus sessions | Keep narrow and explicit; run manually or in a dedicated lab environment |

## Integration Strategy

### Preferred approach: adapter library over direct DSView app reuse

The project should not embed or wrap the DSView GUI application. The cleaner path is:

1. Identify the minimal `libsigrok4DSL` entry points and data/session abstractions needed for:
   - device enumeration
   - DSLogic Plus selection
   - samplerate / channel configuration
   - capture lifecycle
   - VCD export
2. Expose those through a narrow native boundary.
3. Wrap that boundary in Rust with safe session/state types.
4. Keep CLI UX and output orchestration fully in Rust.

This keeps the CLI decoupled from Qt, GUI state, and DSView application assumptions.

### Direct FFI vs wrapper process vs adapter library

| Option | Use when | Verdict |
|---|---|---|
| Direct FFI into `libsigrok4DSL` | Needed functions are C-level and reasonably separable from GUI code | Best long-term fit for this project |
| Small C/C++ adapter library in this repo | Raw library surface is too awkward or unstable for direct Rust calls | Best fallback / likely practical starting point |
| Wrapper process around DSView app | No clean library boundary exists | Avoid for v1 unless forced; too brittle and GUI-coupled |

**Recommendation:** Start by validating whether `libsigrok4DSL` can be linked directly with a minimal adapter. If not, add a tiny native shim owned by the CLI repo rather than shelling out to the DSView GUI.

## What Not To Do

- Do not link the Rust CLI against DSView GUI classes or Qt-driven session objects as the main integration surface.
- Do not promise multi-device support in v1; lock to `DSLogic Plus` first.
- Do not invent a custom waveform format when VCD already exists in the dependency stack.
- Do not let unsafe/native calls spread through command handlers; keep them behind a wrapper boundary.
- Do not treat the submodule as mutable project code; pin and consume it.

## Major Risks

### Build and linking portability
`DSView/CMakeLists.txt` shows broad dependencies including Qt, glib, libusb, Python, FFTW, zlib, and Boost. If the CLI accidentally depends on the full DSView build graph, setup and release complexity will spike.

**Mitigation:** keep the integration boundary at the lowest practical native layer and document supported build environments early.

### ABI drift from submodule updates
Because the CLI depends on internal/native interfaces, submodule updates can silently break bindings.

**Mitigation:** pin the submodule revision, generate/update bindings deliberately, and add smoke tests around the adapter boundary.

### Hidden GUI assumptions
Some capture/session flows may be orchestrated in DSView-side C++ code rather than pure library code.

**Mitigation:** map the actual device session path before implementation and avoid assuming `libsigrok4DSL` alone is sufficient until verified.

## Confidence Notes

- **Rust CLI stack:** High
- **VCD as v1 export:** High
- **Direct `libsigrok4DSL` integration feasibility without a shim:** Medium
- **Need for a native adapter layer:** Medium-High
- **Avoiding DSView GUI reuse:** High
