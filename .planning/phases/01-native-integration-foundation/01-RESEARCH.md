# Phase 1 Research: Native Integration Foundation

**Date:** 2026-04-03
**Phase:** 1 - Native Integration Foundation
**Goal:** Establish a stable Rust project structure and verify the lowest-risk way to reuse the `DSView/` submodule's capture stack without modifying it.

## Goal Fit

Phase 1 needs proof in three areas before broader device work starts:
1. The Rust workspace boundary must keep CLI, safe orchestration, and unsafe/native code separate.
2. The chosen native boundary must avoid pulling the DSView GUI into the CLI build.
3. The project needs an early smoke strategy that proves the boundary is real before hardware-dependent work begins.

## Findings

### 1. `libsigrok4DSL` is the most plausible native boundary

Repository evidence points to `DSView/libsigrok4DSL` as the reusable device-facing layer:
- `DSView/libsigrok4DSL/libsigrok.h` explicitly describes itself as the public header for frontends.
- `DSView/CMakeLists.txt` lists `libsigrok4DSL` sources separately from the DSView GUI sources.
- `DSView/libsigrok4DSL/output/vcd.c` implements VCD output at the library layer, not the GUI layer.
- `DSView/libsigrok4DSL/hardware/DSL/dsl.h` contains DSLogic capability/profile definitions and samplerate tables needed for later configuration work.

This supports the earlier project decision to avoid the DSView Qt application layer as the integration boundary.

### 2. The current DSView build graph is too broad to adopt directly for the CLI

`DSView/CMakeLists.txt` shows the top-level executable links against Qt, Python, FFTW, glib, zlib, libusb, and other DSView application dependencies. Reusing the full executable build would violate Phase 1's success criterion of building against the native boundary without requiring GUI integration.

The CLI should therefore treat the existing C sources and headers as the dependency boundary, not the DSView executable target.

### 3. A thin repo-owned adapter is lower risk than binding to the entire header surface immediately

`libsigrok4DSL` exposes a broad C API and internal data types. For Phase 1, the safest path is:
- keep raw FFI isolated in one crate
- expose a narrow safe Rust adapter for only the calls needed in the first milestones
- avoid generating or relying on the full API surface until the minimum bring-up path is proven

This reduces ABI drift and limits how much unsafe code leaks upward.

### 4. VCD support already exists in the native layer

`DSView/libsigrok4DSL/output/vcd.c` confirms the stack already knows how to generate VCD with:
- enabled logic channels
- samplerate-derived timescale
- channel names in header output

That is important because it means later export phases can likely reuse an existing native output path rather than inventing a parallel Rust-side exporter from scratch.

### 5. Smoke coverage can start before real hardware capture

There are existing `libsigrok4DSL/tests` files that exercise initialization and driver-level behavior, including `sr_init`. That suggests realistic Phase 1 smoke coverage can include:
- Rust crate/build success
- generated or handwritten bindings compiling against chosen headers
- a native initialization smoke path (`sr_init` / `sr_exit` or equivalent)
- dependency-path documentation checked into planning artifacts

Real device enumeration and capture should wait for Phase 2+.

## Options Compared

### Option A - Bind Rust directly to `libsigrok4DSL`

**Approach:** Create a `dsview-sys` crate that links against a built `libsigrok4DSL` boundary and exposes selected raw symbols to Rust.

**Pros**
- Aligns with the intended frontend API boundary.
- Keeps GUI code out of the Rust architecture.
- Preserves future flexibility for safe wrappers in `dsview-core`.

**Cons**
- Requires a reliable way to compile or otherwise provide the native library artifact.
- May still expose a large header/API surface unless tightly constrained.
- Needs careful handling of glib/libusb/native include paths.

**Assessment:** Best default direction if the build can be narrowed enough in Phase 1.

### Option B - Add a tiny repo-owned C shim over `libsigrok4DSL`

**Approach:** Build a minimal C adapter owned by this repo that wraps only the small set of native operations Phase 1-2 need, and bind Rust to that shim instead of the entire native surface.

**Pros**
- Strongest control over ABI exposed to Rust.
- Simplifies bindgen/manual FFI and future smoke tests.
- Helps prevent unsafe and internal struct handling from spreading upward.

**Cons**
- Adds another maintained native layer.
- Still depends on a stable way to consume `libsigrok4DSL` underneath.
- Must avoid becoming a second implementation instead of a thin wrapper.

**Assessment:** Good fallback or even preferred variant if direct binding proves too wide or too fragile.

### Option C - Reuse DSView executable/application build products

**Approach:** Build DSView as-is and link or integrate through the app target or its transitive outputs.

**Pros**
- May appear fast initially because the code already builds together upstream.

**Cons**
- Pulls in Qt and application-level dependencies.
- Violates the desired native boundary.
- Increases accidental coupling to GUI lifecycle and app assumptions.

**Assessment:** Reject for Phase 1.

## Recommended Direction

Use a Rust workspace with three crates and plan around a narrow `libsigrok4DSL` boundary:

1. `dsview-cli`
   - Clap-based command surface only.
   - No unsafe code.

2. `dsview-core`
   - Safe orchestration and domain types.
   - Owns high-level errors and later capture/session abstractions.
   - Depends on a narrow trait or adapter surface, not raw C symbols.

3. `dsview-sys`
   - Raw FFI plus the smallest possible glue needed to prove linking and initialization.
   - Sole owner of `unsafe` and native include/link details.

Implementation strategy for Phase 1 should assume this sequence:
- create the Rust workspace and crate boundaries first
- prove the native compile/link path second
- add a smoke test that exercises the chosen boundary without requiring GUI launch or hardware capture

If direct Rust-to-`libsigrok4DSL` binding becomes too broad during plan 01-02, pivot to a tiny repo-owned shim instead of expanding Rust's unsafe surface.

## Plan 01-02 Boundary Decision

Plan 01-02 uses the public frontend header `DSView/libsigrok4DSL/libsigrok.h` as the supported Rust-facing boundary and intentionally scopes Phase 1 to a single public symbol: `sr_get_lib_version_string()`.

This decision is based on current DSView evidence:
- `libsigrok.h` marks `sr_get_lib_version_string()` with `SR_API`, so it is explicitly part of the public frontend surface.
- `sr_init` and `sr_exit` are present in `libsigrok-internal.h` with `SR_PRIV`, so they should not be treated as the stable Rust-facing boundary for Phase 1.
- `DSView/CMakeLists.txt` compiles `libsigrok4DSL` sources into the DSView application target rather than exposing a standalone in-tree library artifact, so Phase 1 can prove header and boundary selection now without pretending that a reusable native library is already packaged.

### Supported Phase 1 path

- `DSView/` is treated as a read-only upstream dependency.
- `crates/dsview-sys/wrapper.h` includes only `DSView/libsigrok4DSL/libsigrok.h`.
- `crates/dsview-sys/src/lib.rs` declares only the raw symbol needed for the current proof point.
- `crates/dsview-sys/build.rs` validates that the submodule and public header exist, emits include metadata for downstream inspection, and fails with explicit messages when the native prerequisite files are missing.
- The CLI does **not** link against the DSView GUI executable target as part of this Phase 1 proof.

### Deferred work and revalidation rules

Phase 1 does not yet claim a finished linkable `libsigrok4DSL` artifact. Later work must either:
- provide a standalone native library path for the public `libsigrok4DSL` surface, or
- introduce a tiny repo-owned shim that wraps the needed internal lifecycle calls without coupling Rust to DSView GUI code.

Revalidate the boundary when any of the following changes happen:
- the `DSView/` submodule revision changes
- `DSView/libsigrok4DSL/libsigrok.h` changes the declaration or visibility of `sr_get_lib_version_string()`
- `DSView/CMakeLists.txt` changes how `libsigrok4DSL` sources are built or exported
- Phase 01-03 moves from header/build proof to executable native smoke coverage


### Phase 1 Smoke Coverage Expectations

Plan 01-03 uses `cargo test -p dsview-sys` as the scoped smoke command for the native boundary.

What this smoke currently proves:
- the Rust-side `dsview-sys` boundary is wired through the normal Cargo workflow
- the build script still finds the `DSView/` submodule and `libsigrok4DSL` public headers
- a non-GUI runtime smoke path can call a real boundary symbol, `sr_get_lib_version_string()`, when the local machine has enough native headers to build the scoped shim

What this smoke does **not** prove:
- real device discovery
- capture execution
- correctness of internal lifecycle calls such as `sr_init` / `sr_exit`
- availability of a reusable standalone `libsigrok4DSL` library artifact for later phases

The runtime smoke remains intentionally non-hardware and non-GUI. When glib development headers are unavailable, the build script skips the runtime shim and the tests still verify the compile-time boundary plus the documented skip condition.

Revalidate or update the smoke path when any of these change:
- the `DSView/` submodule revision changes
- `DSView/libsigrok4DSL/libsigrok.h` or `DSView/libsigrok4DSL/version.h` changes the public version symbol or macro values
- `crates/dsview-sys/build.rs` changes include paths, shim compilation rules, or cfg names
- the project introduces a true standalone native library path and can replace the scoped shim with direct runtime linkage
- Phase 2 starts depending on lifecycle calls, device enumeration, or other behavior beyond this version-string proof point

### ABI drift
- Pin the `DSView/` submodule revision as part of the product contract.
- Treat header changes as explicit integration events.
- Keep generated bindings or handwritten signatures tightly scoped.

### Accidental GUI dependency
- Do not link against the DSView executable target.
- Do not allow Qt types, GUI lifecycle concepts, or DSView app modules into `dsview-core` or `dsview-cli`.
- Document exactly which native headers, libraries, and system deps the CLI path requires.

### Unsafe code spreading upward
- Confine all `unsafe` to `dsview-sys`.
- Expose safe wrappers or narrow adapter traits upward.
- Keep plan tasks explicit about which crate owns what boundary.

### False confidence from build-only success
- Build success alone is not enough.
- Add a smoke path that calls into the native layer and fails clearly if the boundary is misconfigured.

## Planning Implications

Phase 1 plans should be structured as:
- **01-01:** Create workspace and crate boundaries with explicit ownership rules.
- **01-02:** Prove and document the native build/link strategy against `DSView/` without GUI integration.
- **01-03:** Add minimal smoke coverage for the chosen integration path and codify guardrails around submodule pinning/ABI assumptions.

The plans should avoid promising real capture execution, device enumeration, or protocol decode in this phase. The deliverable is a proven foundation, not feature breadth.

## Open Uncertainties

- Whether `libsigrok4DSL` can be built independently with a trimmed dependency set, or whether a local shim/build script will be needed immediately.
- Whether bindgen is practical against the current headers without excessive surface area.
- Which exact native symbols form the smallest stable smoke path beyond `sr_init`/`sr_exit`.

## Recommendation Summary

Plan Phase 1 around a Rust workspace plus a narrow `libsigrok4DSL` integration boundary, with the option to insert a tiny repo-owned shim if direct binding proves too broad. Reject any plan that depends on the DSView GUI build target or allows unsafe/native concerns outside a dedicated sys layer.
