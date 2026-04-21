# Phase 14: Decode Runtime Boundary and Decoder Registry - Research

**Researched:** 2026-04-14
**Domain:** `libsigrokdecode4DSL` runtime bring-up, decoder registry metadata, and CLI discovery contracts. [CITED: .planning/ROADMAP.md] [CITED: DSView/libsigrokdecode4DSL/libsigrokdecode.h]
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

### Runtime boundary
- **D-01:** Use a dedicated decode runtime instead of merging decode support into the existing capture-focused `dsview_runtime`.
- **D-02:** Keep Python and decoder-script dependencies isolated to the decode runtime path so the shipped `capture` workflow does not inherit decode-specific runtime prerequisites.

### Decoder loading and environment discovery
- **D-03:** Decoder/runtime discovery uses explicit path overrides first, then bundled fallback paths.
- **D-04:** Runtime setup must surface distinct failure categories for missing runtime library, missing decoder scripts, Python runtime issues, and decoder load failures.

### CLI discovery surface
- **D-05:** Phase 14 CLI scope is limited to `decode list` and `decode inspect <decoder-id>`.
- **D-06:** `decode inspect` should expose the metadata needed for later config and stack planning, including channels, options, annotations/rows, and stack-relevant inputs/outputs.

### Metadata strategy
- **D-07:** Preserve upstream decoder, channel, and option ids as the canonical identifiers in Rust and CLI outputs.
- **D-08:** Rust should provide a structured stable schema around upstream metadata, but should not invent a second canonical decoder-id/token system in Phase 14.

### Claude's Discretion
- Exact JSON field names for list/inspect responses, as long as upstream canonical ids remain preserved.
- Text rendering layout for `decode list` and `decode inspect`, as long as JSON remains the authoritative contract.

### Deferred Ideas (OUT OF SCOPE)
- Full offline decode execution against artifacts belongs to Phase 16.
- Decode config-file design and validation belongs to Phase 15.
- Capture+decode pipeline orchestration belongs to a later phase/milestone and should not pull decoder flags into `capture` during Phase 14.
- Live decode during acquisition remains out of scope for this phase.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| DEC-01 | User can list the available DSView protocol decoders from the CLI. [CITED: .planning/REQUIREMENTS.md] | Covered by the minimal native boundary, decode discovery decisions, and CLI `decode list` contract guidance in this document. [CITED: .planning/ROADMAP.md] |
| DEC-02 | User can inspect a decoder's channels, options, annotations, and stack-relevant metadata from the CLI. [CITED: .planning/REQUIREMENTS.md] | Covered by the recommended inspect schema, upstream-id preservation rules, and Rust-owned registry mapping guidance in this document. [CITED: .planning/phases/14-decode-runtime-boundary-and-decoder-registry/14-CONTEXT.md] |
</phase_requirements>

## Project Constraints (from CLAUDE.md)

- Treat `DSView/` as upstream dependency code and do not modify it for normal project work. [CITED: CLAUDE.md]
- Keep unsafe/native integration isolated behind a small boundary. [CITED: CLAUDE.md]
- Favor scriptable CLI behavior, explicit exit codes, and machine-readable outputs. [CITED: CLAUDE.md]
- Keep the architecture split across `dsview-cli`, `dsview-core`, and `dsview-sys`. [CITED: CLAUDE.md]
- Scope milestone work to `DSLogic Plus` unless a decode requirement explicitly expands that boundary. [CITED: CLAUDE.md]

## Summary

Phase 14 only needs a metadata boundary, not a decode-execution boundary: load the decode runtime library, optionally set Python home before init, initialize with a decoder-script directory, load all decoders, list decoder ids, inspect one decoder by canonical upstream id, expose the resolved search paths for diagnostics, and cleanly shut the runtime down. [CITED: DSView/libsigrokdecode4DSL/libsigrokdecode.h] [CITED: DSView/libsigrokdecode4DSL/srd.c] [CITED: DSView/libsigrokdecode4DSL/decoder.c]

The repo already has the right layering pattern to reuse: `dsview-sys` owns unsafe loading and raw-to-owned translation, `dsview-core` owns bundled-versus-source discovery policy, and `dsview-cli` owns JSON-first outputs plus stable machine-readable error codes. [CITED: crates/dsview-sys/src/lib.rs] [CITED: crates/dsview-core/src/lib.rs] [CITED: crates/dsview-cli/src/main.rs]

The biggest planning trap is to trust upstream decode bring-up to classify failures for you, because `srd_init(path)` will happily accept an explicit path without checking that it contains decoders, and `srd_decoder_load_all()` ignores per-decoder load failures while continuing the scan. [CITED: DSView/libsigrokdecode4DSL/srd.c] [CITED: DSView/libsigrokdecode4DSL/decoder.c] Planner-owned validation must therefore distinguish four states before CLI rendering: runtime library missing, decoder directory missing/empty, Python init failure, and decoder import/metadata failure. [CITED: .planning/phases/14-decode-runtime-boundary-and-decoder-registry/14-CONTEXT.md] [CITED: DSView/libsigrokdecode4DSL/error.c]

**Primary recommendation:** Build Phase 14 in three narrow passes: a sibling native decode runtime with explicit discovery/error taxonomy, a Rust-owned registry that preserves upstream ids verbatim, and CLI `decode list` / `decode inspect` contracts that reuse the repo's current JSON/text and stable-error patterns. [CITED: .planning/ROADMAP.md] [CITED: .planning/phases/14-decode-runtime-boundary-and-decoder-registry/14-CONTEXT.md] [CITED: crates/dsview-cli/src/main.rs]

## Standard Stack

### Core
| Component | Version / Source | Purpose | Why Standard |
|-----------|------------------|---------|--------------|
| `DSView/libsigrokdecode4DSL` | Vendored repo source. [CITED: DSView/CMakeLists.txt] | Upstream decode engine, decoder registry, metadata extraction, session API, and callback model. [CITED: DSView/libsigrokdecode4DSL/libsigrokdecode.h] | It already embeds Python, loads decoder modules, exposes `srd_decoder_list()` / `srd_decoder_get_by_id()`, and populates metadata from decoder Python classes, so Phase 14 should reuse it instead of parsing decoder scripts directly. [CITED: DSView/libsigrokdecode4DSL/srd.c] [CITED: DSView/libsigrokdecode4DSL/decoder.c] |
| New sibling `dsview_decode_runtime` shared library | New repo target; separate from `dsview_runtime`. [CITED: crates/dsview-sys/native/CMakeLists.txt] [CITED: .planning/phases/14-decode-runtime-boundary-and-decoder-registry/14-CONTEXT.md] | Decode-only native seam for bring-up, list, inspect, and structured error reporting. [CITED: .planning/ROADMAP.md] | Locked decisions D-01 and D-02 require decode to stay separate from capture and to isolate Python plus decoder-script prerequisites from the shipped capture path. [CITED: .planning/phases/14-decode-runtime-boundary-and-decoder-registry/14-CONTEXT.md] |
| `dsview-sys` | Workspace crate. [CITED: Cargo.toml] | Own unsafe library loading, GSList/GVariant traversal, and conversion to owned Rust decoder metadata. [CITED: crates/dsview-sys/src/lib.rs] | The repo already uses `dsview-sys` as the only unsafe/native boundary and already decodes raw C data into owned Rust structs there. [CITED: CLAUDE.md] [CITED: crates/dsview-sys/src/lib.rs] |
| `dsview-core` | Workspace crate. [CITED: Cargo.toml] | Own decode discovery policy, runtime path selection, registry lookups, and future config/execution orchestration. [CITED: crates/dsview-core/src/lib.rs] | Existing capture bring-up and bundle/source fallback logic already live here, so Phase 14 should extend the same layer rather than move discovery into CLI or C. [CITED: crates/dsview-core/src/lib.rs] |
| `dsview-cli` | Workspace crate. [CITED: Cargo.toml] | Add `decode list` and `decode inspect` subcommands plus JSON/text rendering. [CITED: .planning/ROADMAP.md] | Existing CLI patterns already cover subcommand nesting, output-format switching, stable error envelopes, and contract tests. [CITED: crates/dsview-cli/src/main.rs] |

### Supporting
| Dependency | Detected / Source | Purpose | When to Use |
|-----------|--------------------|---------|-------------|
| Python 3 development/runtime | Python 3.13.3 and `python3-config` headers are available in this environment. [VERIFIED: local env probe] | Required because DSView builds `libsigrokdecode4DSL` with Python 3 headers/libs and upstream `srd_init()` initializes the embedded Python interpreter. [CITED: DSView/CMakeLists.txt] [CITED: DSView/libsigrokdecode4DSL/srd.c] | Needed by the decode runtime target only. [CITED: .planning/phases/14-decode-runtime-boundary-and-decoder-registry/14-CONTEXT.md] |
| `glib-2.0` | Version 2.84.1 is available in this environment. [VERIFIED: local env probe] | Required for GSList, GVariant, and search-path helpers used throughout `libsigrokdecode4DSL`. [CITED: DSView/libsigrokdecode4DSL/libsigrokdecode.h] [CITED: DSView/libsigrokdecode4DSL/srd.c] | Needed by the decode runtime target. [CITED: DSView/CMakeLists.txt] |
| Decoder scripts directory | `DSView/libsigrokdecode4DSL/decoders` exists in this checkout and contains 150 decoder subdirectories. [VERIFIED: local env probe] | Source of protocol decoder modules scanned by `srd_decoder_load_all()`. [CITED: DSView/libsigrokdecode4DSL/decoder.c] | Required for meaningful list/inspect output. [CITED: .planning/ROADMAP.md] |
| `cmake` + `pkg-config` | `cmake` 3.31.6 and `pkg-config` 1.8.1 are available in this environment. [VERIFIED: local env probe] | Match the existing source-runtime build flow already used by `dsview-sys`. [CITED: crates/dsview-sys/build.rs] [CITED: crates/dsview-sys/native/CMakeLists.txt] | Reuse for the sibling decode runtime build. [CITED: crates/dsview-sys/build.rs] |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Separate decode runtime | Merge decode symbols into `dsview_runtime` | Reject this for Phase 14 because it violates locked decisions D-01 and D-02 and would make the capture workflow inherit Python/decode packaging concerns. [CITED: .planning/phases/14-decode-runtime-boundary-and-decoder-registry/14-CONTEXT.md] |

**Build-prerequisite verification:**
```bash
python3 --version
python3-config --includes
pkg-config --modversion glib-2.0
cmake --version
```
[VERIFIED: local env probe]

## Minimal Native Boundary

| Operation | Phase 14 | Why |
|-----------|----------|-----|
| Load decode runtime library and resolve required symbols | Required. [CITED: .planning/ROADMAP.md] | This mirrors the existing `RuntimeBridge::load()` pattern and gives Phase 14 clean `library_load_failed` versus `symbol_load_failed` behavior. [CITED: crates/dsview-sys/src/lib.rs] [CITED: crates/dsview-cli/src/main.rs] |
| Optional `python_home` setter before init | Recommended internal hook. [CITED: DSView/libsigrokdecode4DSL/libsigrokdecode.h] | Upstream exposes `srd_set_python_home()`, and DSView calls it before `srd_init()` on Windows debug builds, so Phase 14 should keep room for that knob even if it is not public CLI surface yet. [CITED: DSView/DSView/pv/appcontrol.cpp] |
| Init with explicit decoder-script directory | Required. [CITED: DSView/DSView/pv/appcontrol.cpp] | DSView's own headless-free bring-up passes the decode script directory directly to `srd_init(path)`, and locked decision D-03 requires explicit overrides to win. [CITED: DSView/DSView/pv/appcontrol.cpp] [CITED: .planning/phases/14-decode-runtime-boundary-and-decoder-registry/14-CONTEXT.md] |
| Load all decoders | Required. [CITED: DSView/DSView/pv/appcontrol.cpp] | Phase 14 must satisfy DEC-01 and DEC-02, and upstream only populates the registry after `srd_decoder_load_all()`. [CITED: DSView/libsigrokdecode4DSL/decoder.c] [CITED: .planning/REQUIREMENTS.md] |
| Enumerate registry entries | Required. [CITED: .planning/REQUIREMENTS.md] | `srd_decoder_list()` exposes the loaded registry without needing a decode session. [CITED: DSView/libsigrokdecode4DSL/libsigrokdecode.h] [CITED: DSView/libsigrokdecode4DSL/decoder.c] |
| Inspect one decoder by canonical id | Required. [CITED: .planning/REQUIREMENTS.md] | `srd_decoder_get_by_id()` plus the populated `struct srd_decoder` already provide everything Phase 14 needs for list/inspect metadata. [CITED: DSView/libsigrokdecode4DSL/libsigrokdecode.h] [CITED: DSView/libsigrokdecode4DSL/decoder.c] |
| Return search-path diagnostics | Recommended. [CITED: DSView/libsigrokdecode4DSL/libsigrokdecode.h] | `srd_searchpaths_get()` is already public, and surfacing resolved paths will make missing-decoder versus wrong-path failures debuggable. [CITED: DSView/libsigrokdecode4DSL/srd.c] |
| Translate upstream decode error names/messages | Required. [CITED: .planning/phases/14-decode-runtime-boundary-and-decoder-registry/14-CONTEXT.md] | `srd_strerror()` and `srd_strerror_name()` already map libsigrokdecode status codes into stable names/messages that Rust can wrap. [CITED: DSView/libsigrokdecode4DSL/error.c] |
| Session creation, callback registration, `srd_session_send()`, `srd_session_end()` | Not needed in Phase 14. [CITED: .planning/ROADMAP.md] | Those APIs are for execution, callbacks, and sample streaming, which belong to Phase 16 and Phase 17. [CITED: DSView/libsigrokdecode4DSL/libsigrokdecode.h] [CITED: .planning/ROADMAP.md] |

**Inspect payload should come directly from `struct srd_decoder`:** expose `id`, `name`, `longname`, `desc`, `license`, `tags`, `inputs`, `outputs`, required channels, optional channels, options, annotations, and annotation rows in owned Rust data. [CITED: DSView/libsigrokdecode4DSL/libsigrokdecode.h] [CITED: DSView/libsigrokdecode4DSL/decoder.c]

## Packaging and Discovery Decisions

- Reuse the repo's existing discovery policy shape from `RuntimeDiscoveryPaths`, but create a decode-specific variant with explicit `decode-runtime` and decoder-dir overrides first, bundled fallback second, and developer-source fallback third. [CITED: crates/dsview-core/src/lib.rs] [CITED: .planning/phases/14-decode-runtime-boundary-and-decoder-registry/14-CONTEXT.md]
- Keep the decode runtime library physically separate from the existing capture `runtime/` bundle so users who only capture do not inherit Python/decode prerequisites. [CITED: .planning/phases/14-decode-runtime-boundary-and-decoder-registry/14-CONTEXT.md] [CITED: crates/dsview-core/src/lib.rs]
- Validate the decoder-script directory in Rust before calling `srd_init(path)`, because the explicit-path branch in upstream init skips the XDG and `SIGROKDECODE_DIR` fallback logic and does not prove the directory actually contains importable decoders. [CITED: DSView/libsigrokdecode4DSL/srd.c]
- Validate the post-load registry count in Rust after `srd_decoder_load_all()`, because upstream bulk loading ignores individual decoder import failures and returns `SRD_OK` even when it only logs failures internally. [CITED: DSView/libsigrokdecode4DSL/decoder.c]
- Preserve the canonical decoder id from `srd_decoder.id`, not the filesystem directory name, because decoder module folders and canonical ids are not the same in this tree; for example, `decoders/0-i2c/pd.py` declares `id = '0:i2c'`. [CITED: DSView/libsigrokdecode4DSL/decoder.c] [CITED: DSView/libsigrokdecode4DSL/decoders/0-i2c/pd.py]
- Keep an internal optional `python_home` field in the runtime config so bundled Windows/macOS packaging can set it before init if needed, but avoid making that a Phase 14 user-facing contract unless packaging tests force it. [CITED: DSView/libsigrokdecode4DSL/libsigrokdecode.h] [CITED: DSView/DSView/pv/appcontrol.cpp]
- Prefer a bundled decoder layout that matches DSView's own discovery expectations, because DSView looks for decoders under the app data dir and then under `../share/libsigrokdecode4DSL/decoders`. [CITED: DSView/DSView/pv/config/appconfig.cpp]

## Reusable Repo Patterns

| Existing Pattern | Reuse in Phase 14 | Why |
|------------------|-------------------|-----|
| `RuntimeBridge::load()` plus `RuntimeError::{LibraryLoad, SymbolLoad, NativeCall}` | Build a `DecodeRuntimeBridge` with the same load/unload/error shape. [CITED: crates/dsview-sys/src/lib.rs] | The current sys layer already separates loader failures from native-call failures, and the CLI already knows how to turn those into stable codes. [CITED: crates/dsview-cli/src/main.rs] |
| `RuntimeDiscoveryPaths::from_executable_dir()` | Build a decode-specific discovery helper with explicit overrides and bundled/developer fallback. [CITED: crates/dsview-core/src/lib.rs] | This keeps path policy in Rust core instead of leaking it into CLI or C. [CITED: crates/dsview-core/src/lib.rs] |
| Raw-C-to-owned-Rust decoding helpers in `dsview-sys` | Decode decoder metadata into owned `Vec` / `String` / enum shapes in `dsview-sys`, not in CLI. [CITED: crates/dsview-sys/src/lib.rs] | The repo already uses this pattern for device options and validation snapshots. [CITED: crates/dsview-sys/src/lib.rs] |
| JSON-first CLI contracts with text as a secondary rendering | Make `decode list` / `decode inspect` JSON authoritative and text rendering best-effort. [CITED: crates/dsview-cli/src/main.rs] | That matches current command UX and the phase discretion notes. [CITED: .planning/phases/14-decode-runtime-boundary-and-decoder-registry/14-CONTEXT.md] |
| Stable `ErrorResponse { code, message, detail, native_error }` envelope | Add decode-specific stable error codes instead of dumping raw C/Python messages straight to stderr. [CITED: crates/dsview-cli/src/main.rs] | The current CLI already has the shape and test style for machine-readable failures. [CITED: crates/dsview-cli/src/main.rs] |
| Bundle discovery tests in `crates/dsview-core/tests/bundle_discovery.rs` | Clone this test style for decode runtime + decoder-dir discovery. [CITED: crates/dsview-core/tests/bundle_discovery.rs] | The existing tests already lock bundled layout, override precedence, and developer fallback behavior. [CITED: crates/dsview-core/tests/bundle_discovery.rs] |

## Architecture Patterns

### Recommended Project Structure
```text
crates/
├── dsview-sys/
│   ├── native/                 # existing capture runtime
│   ├── native-decode/          # new decode runtime target
│   └── src/decode.rs           # safe decode bridge + owned metadata
├── dsview-core/
│   └── src/decode.rs           # discovery policy + registry service
└── dsview-cli/
    └── src/main.rs             # `decode list` / `decode inspect`
```
[CITED: crates/dsview-sys/native/CMakeLists.txt] [CITED: crates/dsview-core/src/lib.rs] [CITED: crates/dsview-cli/src/main.rs]

### Pattern 1: Metadata-only runtime seam
**What:** Keep the C seam focused on init/list/inspect/exit for Phase 14 and defer sessions, callbacks, and sample streaming to later phases. [CITED: .planning/ROADMAP.md] [CITED: DSView/libsigrokdecode4DSL/libsigrokdecode.h]
**When to use:** Use this in all three Phase 14 plans so discovery/inspect land before execution concerns. [CITED: .planning/ROADMAP.md]
**Example:**
```cpp
// Source: DSView/DSView/pv/appcontrol.cpp
if (srd_init(path) != SRD_OK) {
    return false;
}
if (srd_decoder_load_all() != SRD_OK) {
    return false;
}
```
[CITED: DSView/DSView/pv/appcontrol.cpp]

### Pattern 2: Preserve upstream ids verbatim
**What:** Use `srd_decoder.id`, `srd_channel.id`, and `srd_decoder_option.id` as the canonical JSON keys and lookup ids, and keep any friendlier labels purely additive. [CITED: DSView/libsigrokdecode4DSL/libsigrokdecode.h] [CITED: .planning/phases/14-decode-runtime-boundary-and-decoder-registry/14-CONTEXT.md]
**When to use:** Use this in the Rust registry model, CLI JSON, and lookup functions. [CITED: .planning/phases/14-decode-runtime-boundary-and-decoder-registry/14-CONTEXT.md]
**Why it matters:** Existing capture surfaces add token/stable-id layers for human ergonomics, but Phase 14 explicitly forbids inventing a second canonical decoder id system. [CITED: crates/dsview-cli/src/device_options.rs] [CITED: .planning/phases/14-decode-runtime-boundary-and-decoder-registry/14-CONTEXT.md]

### Pattern 3: Inspect schema aligned with DSView concepts, not Qt objects
**What:** Model the root decoder, optional stacked metadata, channels, options, annotations, and annotation rows using the same concepts DSView serializes in session JSON, but do not reuse Qt classes like `DecoderStack` or `ProtocolDock`. [CITED: DSView/DSView/pv/storesession.cpp] [CITED: .planning/REQUIREMENTS.md]
**When to use:** Use this for `decode inspect` JSON and for Phase 15 config planning. [CITED: .planning/ROADMAP.md]
**Why it matters:** `storesession.cpp` shows the upstream concepts that later config must speak (`id`, `channel`, `options`, `stacked decoders`), while the milestone explicitly forbids making Qt decode UI code the runtime engine. [CITED: DSView/DSView/pv/storesession.cpp] [CITED: .planning/REQUIREMENTS.md] [CITED: .planning/PROJECT.md]

### Anti-Patterns to Avoid
- **Directory-name ids:** Do not key decoders by folder names like `0-i2c`; use the loaded decoder's canonical `id` field like `0:i2c`. [CITED: DSView/libsigrokdecode4DSL/decoders/0-i2c/pd.py]
- **Qt coupling:** Do not lift `ProtocolDock`, `DecoderStack`, or trace/view classes into the CLI runtime path. [CITED: .planning/REQUIREMENTS.md] [CITED: .planning/PROJECT.md]
- **Capture token reuse:** Do not apply the capture token/stable-id alias layer to decoders in Phase 14. [CITED: crates/dsview-cli/src/device_options.rs] [CITED: .planning/phases/14-decode-runtime-boundary-and-decoder-registry/14-CONTEXT.md]
- **Opaque native blobs:** Do not hide decoder metadata inside untyped JSON strings or Qt-owned objects; convert it into owned Rust types in `dsview-sys`. [CITED: crates/dsview-sys/src/lib.rs]

## Recommended Sequencing for the 3 Roadmap Plans

1. **`14-01-PLAN.md` - native boundary and packaging first:** build the sibling decode runtime target, add decode discovery config, validate decoder-dir existence/count, and lock the decode-specific error categories before any Rust registry work. [CITED: .planning/ROADMAP.md] [CITED: DSView/libsigrokdecode4DSL/srd.c] [CITED: DSView/libsigrokdecode4DSL/decoder.c]
2. **`14-02-PLAN.md` - Rust registry second:** add owned decoder summary/inspect structs, lookup-by-id, option-type conversion, annotation/row mapping, and unit tests that prove canonical ids survive round-trips unchanged. [CITED: .planning/ROADMAP.md] [CITED: DSView/libsigrokdecode4DSL/libsigrokdecode.h] [CITED: DSView/libsigrokdecode4DSL/util.c]
3. **`14-03-PLAN.md` - CLI contract last:** add `decode list` / `decode inspect`, JSON/text rendering, and stable decode error codes after the registry and discovery contracts are already fixed underneath. [CITED: .planning/ROADMAP.md] [CITED: crates/dsview-cli/src/main.rs]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Decoder-script parsing | A custom Python/regex parser over `decoders/*/pd.py` | `libsigrokdecode4DSL` loader plus `struct srd_decoder` metadata | Upstream already imports the decoder modules, validates API version/methods, and extracts channels/options/annotations/rows for you. [CITED: DSView/libsigrokdecode4DSL/decoder.c] |
| Decoder ids | A second token or alias system | `srd_decoder.id` as the canonical id | The folder name is not the canonical id in this repo, and Phase 14 explicitly locks upstream ids as canonical. [CITED: DSView/libsigrokdecode4DSL/decoders/0-i2c/pd.py] [CITED: .planning/phases/14-decode-runtime-boundary-and-decoder-registry/14-CONTEXT.md] |
| Stack-compatibility rules | Custom hand-maintained mapping tables | Upstream `inputs` / `outputs` metadata | `srd_inst_stack()` already checks matching input/output strings, so Phase 14 inspect should expose those exact strings for Phase 15 validation. [CITED: DSView/libsigrokdecode4DSL/instance.c] [CITED: DSView/libsigrokdecode4DSL/libsigrokdecode.h] |
| Option typing | Ad-hoc string parsing in CLI | GVariant-backed typing from upstream defaults/values mapped once in `dsview-sys` | Upstream option conversion only supports string, int64, and double, so the Rust bridge should decode those once and keep CLI rendering simple. [CITED: DSView/libsigrokdecode4DSL/util.c] [CITED: DSView/libsigrokdecode4DSL/decoder.c] |
| GUI runtime behavior | Qt decode UI classes | `libsigrokdecode4DSL` plus Rust-owned models | The milestone explicitly forbids reusing Qt decode panel code as the engine. [CITED: .planning/REQUIREMENTS.md] [CITED: .planning/PROJECT.md] |

**Key insight:** Upstream `libsigrokdecode4DSL` already does the hard parts that are easy to underestimate in this domain: Python embedding, decoder import validation, metadata extraction, stack I/O wiring, and callback plumbing. [CITED: DSView/libsigrokdecode4DSL/srd.c] [CITED: DSView/libsigrokdecode4DSL/decoder.c] [CITED: DSView/libsigrokdecode4DSL/instance.c]

## Common Pitfalls

### Pitfall 1: Missing decoder directories look like an empty registry instead of a hard failure
**What goes wrong:** Phase 14 can appear to initialize successfully but return zero decoders if the planner only calls `srd_init(path)` and `srd_decoder_load_all()` without extra validation. [CITED: DSView/libsigrokdecode4DSL/srd.c] [CITED: DSView/libsigrokdecode4DSL/decoder.c]
**Why it happens:** The explicit-path branch of `srd_init(path)` only prepends the provided path to Python's module search path, and `srd_decoder_load_all()` ignores individual decoder-load failures while continuing the scan. [CITED: DSView/libsigrokdecode4DSL/srd.c] [CITED: DSView/libsigrokdecode4DSL/decoder.c]
**How to avoid:** Validate the decoder directory in Rust before init, then validate that the loaded registry count is greater than zero after bulk load. [CITED: DSView/libsigrokdecode4DSL/srd.c] [CITED: DSView/libsigrokdecode4DSL/decoder.c]
**Warning signs:** Search paths look correct but `decode list` is empty and no stable failure code explains why. [CITED: DSView/libsigrokdecode4DSL/srd.c]

### Pitfall 2: Repeated init without guaranteed exit breaks the process-wide runtime
**What goes wrong:** Tests or repeated CLI operations in the same process can fail unexpectedly if the runtime is initialized twice without a matching exit. [CITED: DSView/libsigrokdecode4DSL/srd.c]
**Why it happens:** Upstream tracks initialization globally through `max_session_id` and treats repeated `srd_init()` calls without `srd_exit()` as an error. [CITED: DSView/libsigrokdecode4DSL/srd.c]
**How to avoid:** Give the Rust bridge RAII-style shutdown on drop and add tests that exercise repeated load/list/inspect cycles. [CITED: crates/dsview-core/src/lib.rs] [CITED: DSView/libsigrokdecode4DSL/srd.c]
**Warning signs:** The first command works, later commands in the same process report a generic init error. [CITED: DSView/libsigrokdecode4DSL/srd.c] [CITED: DSView/libsigrokdecode4DSL/error.c]

### Pitfall 3: Canonical id drift if the planner keys decoders by folder names or aliases
**What goes wrong:** Later config files and CLI inspect output become incompatible with upstream metadata if Phase 14 exposes synthetic ids. [CITED: .planning/phases/14-decode-runtime-boundary-and-decoder-registry/14-CONTEXT.md]
**Why it happens:** Decoder module folder names are not the same as canonical ids in this checkout, and the capture CLI's existing token/stable-id pattern is easy to over-apply. [CITED: DSView/libsigrokdecode4DSL/decoders/0-i2c/pd.py] [CITED: crates/dsview-cli/src/device_options.rs]
**How to avoid:** Key everything off upstream ids from the loaded registry and keep any display labels additive only. [CITED: DSView/libsigrokdecode4DSL/libsigrokdecode.h] [CITED: .planning/phases/14-decode-runtime-boundary-and-decoder-registry/14-CONTEXT.md]
**Warning signs:** JSON contains decoder keys that cannot be found by `srd_decoder_get_by_id()`. [CITED: DSView/libsigrokdecode4DSL/libsigrokdecode.h]

### Pitfall 4: Over-normalizing options or annotation metadata too early
**What goes wrong:** Phase 15 and Phase 16 lose information they need if Phase 14 flattens option types, drops annotation indices, or hides row ids. [CITED: .planning/ROADMAP.md]
**Why it happens:** Upstream options are typed through GVariant defaults/values, annotations can be two- or three-element tuples with explicit or implicit numeric classes, and annotation rows carry stable row ids plus class membership. [CITED: DSView/libsigrokdecode4DSL/decoder.c] [CITED: DSView/libsigrokdecode4DSL/util.c]
**How to avoid:** Preserve option type, default, allowed values, annotation numeric class, raw annotation strings, row id, row description, and row class list in the inspect schema. [CITED: DSView/libsigrokdecode4DSL/decoder.c]
**Warning signs:** Phase 15 needs to reopen Python decoder files or DSView source to recover metadata that `decode inspect` should already have exposed. [CITED: .planning/phases/14-decode-runtime-boundary-and-decoder-registry/14-CONTEXT.md]

### Pitfall 5: Letting Qt references leak into the runtime design
**What goes wrong:** The implementation becomes harder to bundle, harder to test, and more coupled to GUI-only concerns like labels, view indices, and row visibility toggles. [CITED: .planning/REQUIREMENTS.md] [CITED: DSView/DSView/pv/storesession.cpp]
**Why it happens:** `storesession.cpp` is useful as a semantics reference, but it also serializes UI-specific fields such as `label`, `view_index`, and `show` state that are not Phase 14 runtime requirements. [CITED: DSView/DSView/pv/storesession.cpp]
**How to avoid:** Reuse the metadata concepts from `storesession.cpp` but stop the Phase 14 contract at decoder/core metadata, not UI state. [CITED: DSView/DSView/pv/storesession.cpp] [CITED: .planning/ROADMAP.md]
**Warning signs:** The planner starts talking about `ProtocolDock`, traces, or row-visibility state in the native boundary. [CITED: DSView/DSView/pv/storesession.cpp] [CITED: .planning/REQUIREMENTS.md]

### Pitfall 6: Forgetting that stack compatibility is metadata-driven, not GUI-driven
**What goes wrong:** Later config validation accepts impossible stacks or rejects valid ones because it invents custom compatibility logic. [CITED: .planning/ROADMAP.md]
**Why it happens:** Upstream stacking checks whether at least one output string from the lower decoder matches one input string from the upper decoder, and it only warns when they do not match. [CITED: DSView/libsigrokdecode4DSL/instance.c]
**How to avoid:** Expose `inputs` and `outputs` verbatim in Phase 14 inspect and let Phase 15 add strict validation on top of those exact strings. [CITED: DSView/libsigrokdecode4DSL/libsigrokdecode.h] [CITED: DSView/libsigrokdecode4DSL/instance.c]
**Warning signs:** Phase 15 starts maintaining a parallel table of stackable decoder pairs. [CITED: DSView/libsigrokdecode4DSL/instance.c]

## Code Examples

Verified patterns from current repo sources:

### Reuse the existing Rust connect/load shape
```rust
// Source: crates/dsview-core/src/lib.rs
let resources = ResourceDirectory::discover(resource_dir)?;
let runtime = RuntimeBridge::load(library_path).map_err(BringUpError::Runtime)?;
runtime
    .set_firmware_resource_dir(resources.path())
    .map_err(BringUpError::Runtime)?;
runtime.init().map_err(BringUpError::Runtime)?;
```
[CITED: crates/dsview-core/src/lib.rs]

### Mirror DSView's decode bring-up ordering
```cpp
// Source: DSView/DSView/pv/appcontrol.cpp
if (srd_init(path) != SRD_OK) {
    return false;
}
if (srd_decoder_load_all() != SRD_OK) {
    return false;
}
```
[CITED: DSView/DSView/pv/appcontrol.cpp]

### Reuse the repo's JSON-first CLI rendering split
```rust
// Source: crates/dsview-cli/src/main.rs
match format {
    OutputFormat::Json => println!("{}", serde_json::to_string_pretty(payload).unwrap()),
    OutputFormat::Text => println!("ok"),
}
```
[CITED: crates/dsview-cli/src/main.rs]

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Treat decode as a future concern outside the shipped CLI | Active milestone `v1.2` now plans a dedicated decode foundation with separate discovery, config, execution, and reporting phases. [CITED: .planning/ROADMAP.md] | 2026-04-14 milestone definition. [CITED: .planning/ROADMAP.md] | Phase 14 should optimize for inspectable metadata now so later phases do not need to redesign ids or registry shapes. [CITED: .planning/ROADMAP.md] |
| Friendly token layers for human-facing capture options | Canonical upstream ids for decode metadata are now a locked Phase 14 decision. [CITED: crates/dsview-cli/src/device_options.rs] [CITED: .planning/phases/14-decode-runtime-boundary-and-decoder-registry/14-CONTEXT.md] | Locked on 2026-04-14 during phase discussion. [CITED: .planning/phases/14-decode-runtime-boundary-and-decoder-registry/14-CONTEXT.md] | Decoder list/inspect should be easier to compose into later config files without remapping ids. [CITED: .planning/phases/14-decode-runtime-boundary-and-decoder-registry/14-CONTEXT.md] |
| One runtime for everything | Separate capture and decode runtime paths for this milestone. [CITED: crates/dsview-sys/native/CMakeLists.txt] [CITED: .planning/phases/14-decode-runtime-boundary-and-decoder-registry/14-CONTEXT.md] | Locked on 2026-04-14 during phase discussion. [CITED: .planning/phases/14-decode-runtime-boundary-and-decoder-registry/14-CONTEXT.md] | Decode packaging can evolve without destabilizing the shipped capture workflow. [CITED: .planning/phases/14-decode-runtime-boundary-and-decoder-registry/14-CONTEXT.md] |

**Deprecated/outdated for this milestone:**
- Reusing Qt `DecoderStack` UI code as the engine is explicitly out of scope and should be treated as outdated planning guidance for `v1.2`. [CITED: .planning/REQUIREMENTS.md] [CITED: .planning/PROJECT.md]
- Flattening decoder-specific flags into `capture` is explicitly out of scope and should not leak into Phase 14 plans. [CITED: .planning/REQUIREMENTS.md] [CITED: .planning/PROJECT.md]

## Assumptions Log

All claims in this research were verified against local code, planning docs, or live environment probes in this session. [VERIFIED: codebase grep]

## Open Questions

1. **Does the first shipped decode bundle need a public Python-home override, or is an internal runtime-config field enough?** [CITED: DSView/libsigrokdecode4DSL/libsigrokdecode.h] [CITED: DSView/DSView/pv/appcontrol.cpp]
   - What we know: upstream exposes `srd_set_python_home()` and DSView calls it before `srd_init()` in a Windows debug path. [CITED: DSView/libsigrokdecode4DSL/libsigrokdecode.h] [CITED: DSView/DSView/pv/appcontrol.cpp]
   - What's unclear: the current CLI bundle plan for non-Linux platforms does not yet say whether Python will be system-provided or bundled. [CITED: .planning/ROADMAP.md]
   - Recommendation: wire the field through the native/Rust config now, but keep the public CLI surface focused on runtime-library and decoder-dir overrides unless packaging tests prove `python_home` must be user-controlled. [CITED: .planning/phases/14-decode-runtime-boundary-and-decoder-registry/14-CONTEXT.md] [CITED: DSView/libsigrokdecode4DSL/libsigrokdecode.h]

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Python 3 runtime | Embedded decode runtime init. [CITED: DSView/libsigrokdecode4DSL/srd.c] | Yes. [VERIFIED: local env probe] | 3.13.3. [VERIFIED: local env probe] | None for Phase 14 bundle planning. [CITED: DSView/CMakeLists.txt] |
| Python 3 headers/config | Building the sibling decode runtime. [CITED: DSView/CMakeLists.txt] | Yes. [VERIFIED: local env probe] | `python3-config --includes` resolves `/usr/include/python3.13`. [VERIFIED: local env probe] | CMake `find_package(Python3 COMPONENTS Development)` or `find_package(PythonLibs 3)` already matches upstream. [CITED: DSView/CMakeLists.txt] |
| `glib-2.0` | `libsigrokdecode4DSL` GSList/GVariant APIs. [CITED: DSView/libsigrokdecode4DSL/libsigrokdecode.h] | Yes. [VERIFIED: local env probe] | 2.84.1. [VERIFIED: local env probe] | None. [CITED: DSView/CMakeLists.txt] |
| `cmake` | Source-build flow for the sibling decode runtime. [CITED: crates/dsview-sys/build.rs] | Yes. [VERIFIED: local env probe] | 3.31.6. [VERIFIED: local env probe] | None in current repo build flow. [CITED: crates/dsview-sys/build.rs] |
| `pkg-config` | Dependency discovery in the existing native build flow. [CITED: crates/dsview-sys/build.rs] | Yes. [VERIFIED: local env probe] | 1.8.1. [VERIFIED: local env probe] | `pkgconf` would also work because the build script already supports both. [CITED: crates/dsview-sys/build.rs] |
| Decoder scripts checkout | Developer fallback and local source-run discovery. [CITED: .planning/phases/14-decode-runtime-boundary-and-decoder-registry/14-CONTEXT.md] | Yes. [VERIFIED: local env probe] | 150 decoder directories present. [VERIFIED: local env probe] | Bundle the decoder directory next to the CLI for shipped builds. [CITED: DSView/DSView/pv/config/appconfig.cpp] |

**Missing dependencies with no fallback:**
- None detected in this environment for planning and local source builds. [VERIFIED: local env probe]

**Missing dependencies with fallback:**
- None detected in this environment. [VERIFIED: local env probe]

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness via `cargo test`. [CITED: Cargo.toml] [VERIFIED: codebase grep] |
| Config file | None; the workspace uses Cargo defaults. [CITED: Cargo.toml] |
| Quick run command | `cargo test -p dsview-core --test bundle_discovery`. [CITED: crates/dsview-core/tests/bundle_discovery.rs] |
| Full suite command | `cargo test --workspace`. [CITED: Cargo.toml] |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| DEC-01 | Discover decode runtime paths, initialize registry, and list decoder summaries with stable JSON/text output. [CITED: .planning/REQUIREMENTS.md] | integration + CLI contract | `cargo test -p dsview-cli decode_list_ -x` as the recommended naming pattern for the new tests. [CITED: crates/dsview-cli/src/main.rs] | No - add in Wave 0. [VERIFIED: codebase grep] |
| DEC-02 | Inspect one decoder and return canonical ids plus channels, options, annotations, rows, inputs, and outputs. [CITED: .planning/REQUIREMENTS.md] | sys/core integration + CLI contract | `cargo test -p dsview-core decode_inspect_ -x` as the recommended naming pattern for the new tests. [CITED: crates/dsview-core/src/lib.rs] | No - add in Wave 0. [VERIFIED: codebase grep] |

### Sampling Rate
- **Per task commit:** run the narrowest affected test target plus `cargo test -p dsview-core --test bundle_discovery` whenever discovery logic changes. [CITED: crates/dsview-core/tests/bundle_discovery.rs]
- **Per wave merge:** run `cargo test --workspace`. [CITED: Cargo.toml]
- **Phase gate:** full workspace green before `/gsd-verify-work`. [CITED: .planning/config.json]

### Wave 0 Gaps
- [ ] `crates/dsview-sys/tests/decode_runtime.rs` - decode runtime load/init/load-all/list/inspect boundary coverage for DEC-01 and DEC-02. [VERIFIED: codebase grep]
- [ ] `crates/dsview-core/tests/decode_registry.rs` - discovery precedence, registry lookup, canonical-id preservation, and inspect-model coverage. [VERIFIED: codebase grep]
- [ ] `crates/dsview-cli/tests/decode_cli.rs` or equivalent unit coverage in `crates/dsview-cli/src/main.rs` - JSON/text and stable error-code contracts for `decode list` / `decode inspect`. [VERIFIED: codebase grep]

## Security Domain

### Applicable ASVS Categories
| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no. [CITED: .planning/ROADMAP.md] | None in scope for this local CLI registry phase. [CITED: .planning/ROADMAP.md] |
| V3 Session Management | no. [CITED: .planning/ROADMAP.md] | None in scope because Phase 14 does not create user sessions or auth sessions. [CITED: .planning/ROADMAP.md] |
| V4 Access Control | no. [CITED: .planning/ROADMAP.md] | None in scope for local list/inspect commands. [CITED: .planning/ROADMAP.md] |
| V5 Input Validation | yes. [CITED: .planning/phases/14-decode-runtime-boundary-and-decoder-registry/14-CONTEXT.md] | Validate runtime-library paths, decoder-dir paths, and decoder-id lookups in Rust before calling into native code. [CITED: crates/dsview-cli/src/main.rs] [CITED: crates/dsview-core/src/lib.rs] |
| V6 Cryptography | no. [CITED: .planning/ROADMAP.md] | None; do not invent crypto concerns where the phase has none. [CITED: .planning/ROADMAP.md] |

### Known Threat Patterns for this stack
| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Shared-library path hijack through runtime overrides | Tampering / Elevation of Privilege | Only load from explicit user-selected paths or bundled/developer fallback paths, and report the resolved runtime path in diagnostics. [CITED: crates/dsview-core/src/lib.rs] [CITED: crates/dsview-sys/src/lib.rs] |
| Arbitrary Python code execution from untrusted decoder directories | Tampering / Elevation of Privilege | Treat decoder-dir overrides as trusted local inputs, validate them explicitly, and keep decode-path overrides separate from the default capture workflow. [CITED: DSView/libsigrokdecode4DSL/decoder.c] [CITED: .planning/phases/14-decode-runtime-boundary-and-decoder-registry/14-CONTEXT.md] |
| Decoder-id spoofing via alias layers or directory-name keys | Spoofing / Tampering | Use upstream `id` fields from the loaded registry as canonical identifiers everywhere in JSON and Rust lookups. [CITED: DSView/libsigrokdecode4DSL/libsigrokdecode.h] [CITED: DSView/libsigrokdecode4DSL/decoders/0-i2c/pd.py] |

## Sources

### Primary (HIGH confidence)
- `.planning/phases/14-decode-runtime-boundary-and-decoder-registry/14-CONTEXT.md` - locked decisions, scope, and planner expectations. [CITED: .planning/phases/14-decode-runtime-boundary-and-decoder-registry/14-CONTEXT.md]
- `.planning/ROADMAP.md` - Phase 14 goals, success criteria, and plan sequencing. [CITED: .planning/ROADMAP.md]
- `.planning/REQUIREMENTS.md` - DEC-01 and DEC-02 scope. [CITED: .planning/REQUIREMENTS.md]
- `crates/dsview-sys/native/CMakeLists.txt` - current capture runtime boundary and packaging style. [CITED: crates/dsview-sys/native/CMakeLists.txt]
- `crates/dsview-sys/build.rs` - existing source-runtime build and dependency-discovery flow. [CITED: crates/dsview-sys/build.rs]
- `crates/dsview-sys/src/lib.rs` - load/unload/error-shaping and owned-struct patterns to mirror. [CITED: crates/dsview-sys/src/lib.rs]
- `crates/dsview-core/src/lib.rs` and `crates/dsview-core/tests/bundle_discovery.rs` - bundled/developer discovery policy and supporting tests. [CITED: crates/dsview-core/src/lib.rs] [CITED: crates/dsview-core/tests/bundle_discovery.rs]
- `crates/dsview-cli/src/main.rs` and `crates/dsview-cli/src/device_options.rs` - command structure, JSON/text output, and stable error envelope patterns. [CITED: crates/dsview-cli/src/main.rs] [CITED: crates/dsview-cli/src/device_options.rs]
- `DSView/libsigrokdecode4DSL/libsigrokdecode.h`, `srd.c`, `decoder.c`, `session.c`, `instance.c`, `type_decoder.c`, `util.c`, and `error.c` - upstream decode runtime API, metadata extraction, session model, and error helpers. [CITED: DSView/libsigrokdecode4DSL/libsigrokdecode.h] [CITED: DSView/libsigrokdecode4DSL/srd.c] [CITED: DSView/libsigrokdecode4DSL/decoder.c] [CITED: DSView/libsigrokdecode4DSL/session.c] [CITED: DSView/libsigrokdecode4DSL/instance.c] [CITED: DSView/libsigrokdecode4DSL/type_decoder.c] [CITED: DSView/libsigrokdecode4DSL/util.c] [CITED: DSView/libsigrokdecode4DSL/error.c]
- `DSView/DSView/pv/appcontrol.cpp`, `DSView/DSView/pv/storesession.cpp`, and `DSView/DSView/pv/config/appconfig.cpp` - upstream decode bring-up order, metadata concepts, and decoder-path discovery expectations. [CITED: DSView/DSView/pv/appcontrol.cpp] [CITED: DSView/DSView/pv/storesession.cpp] [CITED: DSView/DSView/pv/config/appconfig.cpp]
- `DSView/CMakeLists.txt` - upstream Python and decode-source linkage expectations. [CITED: DSView/CMakeLists.txt]
- Local environment probes for Python, GLib, CMake, pkg-config, and decoder-directory availability. [VERIFIED: local env probe]

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - every recommendation is grounded in current repo structure, locked decisions, or upstream vendored source. [CITED: .planning/phases/14-decode-runtime-boundary-and-decoder-registry/14-CONTEXT.md] [CITED: DSView/CMakeLists.txt]
- Architecture: HIGH - the repo already ships the same layer split and discovery/error patterns on the capture side. [CITED: crates/dsview-core/src/lib.rs] [CITED: crates/dsview-sys/src/lib.rs] [CITED: crates/dsview-cli/src/main.rs]
- Pitfalls: HIGH - the main risks are visible directly in the upstream decode sources and current CLI/runtime patterns. [CITED: DSView/libsigrokdecode4DSL/srd.c] [CITED: DSView/libsigrokdecode4DSL/decoder.c] [CITED: DSView/libsigrokdecode4DSL/instance.c]

**Research date:** 2026-04-14. [VERIFIED: local env probe]
**Valid until:** 2026-05-14 for planning purposes, unless the vendored `DSView/` submodule or bundle layout changes first. [CITED: .planning/phases/14-decode-runtime-boundary-and-decoder-registry/14-CONTEXT.md]
