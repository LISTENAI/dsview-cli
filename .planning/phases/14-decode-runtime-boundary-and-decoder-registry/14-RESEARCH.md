# Phase 14 Research: Decode Runtime Boundary and Decoder Registry

**Phase:** 14
**Phase Name:** Decode Runtime Boundary and Decoder Registry
**Date:** 2026-04-14
**Status:** Ready for planning

## Research Objective

Determine the minimum technical scope needed to plan Phase 14 well: a separate decode runtime, decoder discovery/loading rules, a Rust-owned registry/inspect boundary, and a CLI discovery surface that stays compatible with upstream ids.

## Key Findings

### 1. Minimum native boundary for Phase 14

The smallest useful Phase 14 boundary is narrower than full decode execution. It only needs four native capabilities:

1. runtime load / unload for a dedicated decode runtime library
2. decode engine init / exit with explicit decoder-search path input
3. decoder registry enumeration
4. decoder inspect metadata extraction into owned C structs that Rust can copy safely

The native boundary does **not** need to support `srd_session_send()`-driven decoding yet. That belongs to Phase 16.

Recommended C-facing operations for this phase:

- `decode_runtime_load(path)` / `decode_runtime_unload()`
- `decode_runtime_init(decoder_dir, python_home?)`
- `decode_runtime_exit()`
- `decode_list(out_ptr, out_len)`
- `decode_inspect(decoder_id, out_struct)`
- `decode_free_*` helpers for any owned arrays/strings
- stable last-error retrieval for loader/init/inspect failures

### 2. Upstream API realities that matter for planning

`libsigrokdecode4DSL` already provides the right primitive shape for this work:

- `srd_init(path)` wires Python and decoder search paths
- `srd_decoder_load_all()` populates the in-memory decoder registry
- `srd_decoder_list()` exposes loaded decoder entries
- metadata lives on `struct srd_decoder` plus nested `channels`, `opt_channels`, `options`, `annotations`, `annotation_rows`, `inputs`, and `outputs`

This means Phase 14 is fundamentally a metadata-lifting exercise, not a protocol-decoding algorithm project.

### 3. Packaging and runtime-discovery implications

A separate decode runtime is the correct planning assumption for this phase.

Why:

- current `dsview_runtime` only packages capture/export-facing DSView code
- decode adds Python embedding and decoder-script search-path management
- the project has already locked the decision that `capture` should not inherit decode-specific runtime dependencies

Planner implications:

- Phase 14 should add a sibling native build target rather than mutate the existing one blindly
- runtime discovery should mirror the current explicit-path-first pattern already used by `dsview-sys`
- planner should preserve a clean distinction between:
  - runtime library missing
  - decoder directory missing
  - Python environment unavailable
  - upstream decoder load failure

### 4. Existing Rust/sys patterns worth reusing

Strong reuse candidates already exist:

- `crates/dsview-sys/src/lib.rs`
  - runtime loading wrapper style
  - owned raw-to-safe struct decoding
  - stable error typing and result conversion
- `crates/dsview-cli/src/main.rs`
  - command/subcommand organization
  - `json` vs `text` output mode pattern
- `crates/dsview-sys/native/CMakeLists.txt`
  - source-backed runtime build structure
  - include path / library dependency management pattern

The new decode boundary should follow the same layering:

- `dsview-sys`: unsafe FFI, owned metadata snapshots, runtime discovery
- `dsview-core`: typed decoder registry domain model and inspect helpers
- `dsview-cli`: `decode list` / `decode inspect`

### 5. Decoder metadata strategy

The chosen project decision to preserve upstream ids is technically sound for this phase.

Implications for planning:

- native structs should carry upstream `id` strings verbatim
- Rust should normalize structure, not semantics
- inspect output should expose required vs optional channels distinctly
- options should preserve upstream option ids, default values, and value-type information when available
- planner should avoid creating a second canonical token namespace in Phase 14

### 6. DSView frontend code should be treated as reference, not reused abstraction

Useful reference files:

- `DSView/DSView/pv/appcontrol.cpp` for startup sequencing
- `DSView/DSView/pv/storesession.cpp` for decoder/stack/config concepts
- `DSView/DSView/pv/data/decoderstack.cpp` for callback/result flow shape

But the planner must not route implementation through Qt-facing classes. Those files are helpful for understanding semantics, not for defining the CLI runtime abstraction.

## Risks And Gotchas

### Python/runtime coupling risk

The biggest Phase 14 delivery risk is not Rust code; it is getting a reliable decode runtime bring-up contract around Python + decoder search paths.

Mitigation:

- make bring-up errors first-class and specific
- keep explicit override inputs in the API
- test runtime load/init failure paths, not only the happy path

### Unsafe metadata ownership risk

Decoder metadata is nested and GLib/Python-backed upstream. A shallow pointer exposure would create fragile Rust bindings.

Mitigation:

- planner should require owned C snapshots or deep-copy conversion before safe Rust sees the data
- free helpers must be explicit

### Scope creep risk

Phase 14 can easily spill into config execution or real decode sessions.

Mitigation:

- plans should stop at registry/inspect
- no `decode run` command in this phase
- no sample-buffer/session-send implementation in this phase

### Upstream-id drift risk

If plans introduce CLI-only canonical ids now, later config and ecosystem interoperability get more complex.

Mitigation:

- keep upstream ids canonical
- any future aliases must be additive and optional

## Recommended Sequencing For The 3 Roadmap Plans

### Plan 14-01

Focus on native runtime bring-up and metadata snapshot design.

Should answer:

- what separate library/build target is added
- what C ABI shape Rust will call
- how runtime/decoder-path errors are represented
- what decoder snapshot structs exist

### Plan 14-02

Implement sys/core registry and inspect domain.

Should deliver:

- safe Rust wrappers for runtime init/list/inspect
- typed decoder/channel/option/annotation models
- error conversion and owned-memory handling

### Plan 14-03

Add CLI commands and lock output contracts.

Should deliver:

- `decode list`
- `decode inspect <decoder-id>`
- JSON/text output coverage
- failure messaging for missing runtime/decoder/id cases

## Planning Guidance

When writing plans, explicitly require:

- deep-copy metadata ownership across the FFI boundary
- runtime and decoder-path error taxonomy
- preservation of upstream ids in all schemas
- no dependence on Qt UI types
- no premature `decode run` plumbing

## Recommended Verification Focus

The checker/plans should verify:

- runtime can fail cleanly before any decoder metadata call
- list/inspect surfaces work without a device attached
- inspect output contains required/optional channels, options, and annotation metadata
- JSON output is the authoritative contract and text output is a rendering layer only

## RESEARCH COMPLETE

Wrote Phase 14 research guidance covering the minimum native boundary, packaging/discovery requirements, reusable Rust/sys patterns, upstream-id preservation strategy, major risks, and the recommended sequencing for Plans 14-01 through 14-03.
