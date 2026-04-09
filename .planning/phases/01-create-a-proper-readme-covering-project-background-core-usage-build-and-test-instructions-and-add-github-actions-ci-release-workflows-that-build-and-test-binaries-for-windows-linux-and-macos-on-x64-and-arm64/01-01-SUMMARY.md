---
phase: 01
plan: 01
subsystem: native-portability
tags: [phase-1, portability, packaging, dsview-sys, runtime, resources, cross-platform]
dependency_graph:
  requires: []
  provides:
    - target-aware runtime naming helper (dsview_sys::runtime_library_name)
    - portable CMakeLists.txt for Linux/macOS/Windows
    - bundle packaging helper (tools/package-bundle.rs)
    - bundle validation helper (tools/validate-bundle.rs)
    - Wave 0 runtime naming tests
    - Wave 0 bundle discovery tests
  affects:
    - crates/dsview-sys/build.rs
    - crates/dsview-sys/src/lib.rs
    - crates/dsview-sys/native/CMakeLists.txt
tech_stack:
  added:
    - cargo-script pattern for standalone packaging/validation tools
  patterns:
    - target-aware build orchestration via TargetInfo struct
    - platform-conditional dependency resolution in CMake
    - shared runtime naming contract across build/package/discovery layers
key_files:
  created:
    - tools/package-bundle.rs
    - tools/validate-bundle.rs
    - crates/dsview-sys/tests/runtime_packaging.rs
  modified:
    - crates/dsview-sys/build.rs
    - crates/dsview-sys/src/lib.rs
    - crates/dsview-sys/native/CMakeLists.txt
    - crates/dsview-core/tests/bundle_discovery.rs
decisions:
  - Use cargo +stable -Zscript for packaging/validation tools instead of separate crate
  - Expose runtime_library_name() as public API for reuse across packaging and discovery
  - Make MSVC compilation explicit panic with clear message rather than silent failure
  - Keep math library linking in CMakeLists.txt rather than build.rs
metrics:
  duration_minutes: 11
  tasks_completed: 4
  commits: 5
  files_created: 4
  files_modified: 4
  tests_added: 12
  completed_at: "2026-04-09T07:47:42Z"
---

# Phase 01 Plan 01: Establish six-target native portability and bundle contract foundations

**One-liner:** Target-aware runtime naming, portable CMake build, and cargo-script packaging/validation helpers establish the foundation for six-target bundle contract.

## What Was Built

Transformed the Linux-centric source-runtime build into a six-target-ready portability foundation with:

1. **Target-aware runtime naming**: Added `TargetInfo` struct in `build.rs` that derives platform-specific decisions from Cargo environment variables. Implemented `runtime_library_name()` helper that maps Linux→`libdsview_runtime.so`, macOS→`libdsview_runtime.dylib`, Windows→`dsview_runtime.dll`. Exposed this as public API in `dsview-sys` for reuse across packaging and discovery layers.

2. **Portable CMake build**: Refactored `native/CMakeLists.txt` to support Linux, macOS, and Windows. Made pkg-config dependency resolution conditional on non-Windows platforms. Added MSVC-specific compiler flags and made GLib/libusb/fftw includes/links conditional. Math library linking now only applies on Unix systems.

3. **Bundle packaging helper**: Created `tools/package-bundle.rs` as a cargo-script tool that assembles versioned archives with `exe/runtime/resources` layout. Packages only DSLogic Plus resources (firmware and bitstreams) rather than entire DSView resource tree.

4. **Bundle validation helper**: Created `tools/validate-bundle.rs` as a cargo-script tool that validates unpacked bundle structure, verifies target-correct runtime library presence, checks required DSLogic Plus resources, and runs smoke tests (`--help`, `devices list --help`).

5. **Wave 0 tests**: Added `crates/dsview-sys/tests/runtime_packaging.rs` with 7 tests covering runtime naming consistency across platforms. Extended `crates/dsview-core/tests/bundle_discovery.rs` with 5 new tests for target-aware runtime filename contract and bundle layout validation, while preserving existing CLI-layer tests.

## Deviations from Plan

None - plan executed exactly as written.

## Technical Decisions

**Cargo-script pattern for tools**: Chose `cargo +stable -Zscript` invocation pattern for packaging and validation helpers instead of creating a separate `tools/` crate. This keeps the helpers standalone and directly executable while avoiding workspace complexity. CI workflows can invoke them with a single command.

**Public runtime naming API**: Exposed `runtime_library_name()` as public function in `dsview-sys` rather than keeping it build-script-internal. This creates a single source of truth for the naming contract that packaging helpers, discovery logic, and tests all reference.

**Explicit MSVC panic**: Made MSVC compilation in `build_static_object_archive()` fail with an explicit panic message rather than attempting to use `cl` and failing silently. This makes the current limitation clear and provides a concrete extension point for future MSVC support.

**CMake-owned math linking**: Kept math library linking in `CMakeLists.txt` via `UNIX` check rather than adding it to `build.rs`. This keeps platform-specific link decisions colocated with the native build configuration.

## Verification

All Wave 0 tests pass:

```
cargo test -p dsview-sys --test runtime_packaging
running 7 tests
test linux_runtime_naming ... ok
test macos_runtime_naming ... ok
test runtime_library_name_is_stable ... ok
test runtime_library_name_has_no_path_separators ... ok
test windows_runtime_naming ... ok
test runtime_library_name_is_valid_filename ... ok
test runtime_library_name_matches_current_platform ... ok

cargo test -p dsview-core --test bundle_discovery
running 8 tests
test bundle_discovery_rejects_wrong_platform_runtime ... ok
test bundle_defaults_resolve_from_executable_layout ... ok
test bundle_discovery_uses_target_aware_runtime_filename ... ok
test discovery_paths_feed_connect_auto_contract_without_resource_override ... ok
test bundle_layout_matches_packaging_contract ... ok
test resource_override_wins_over_bundled_resource_dir ... ok
test developer_fallback_uses_source_runtime_and_repo_resources ... ok
test runtime_library_name_helper_is_consistent ... ok
```

Workspace builds cleanly:
```
cargo check --workspace
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.94s
```

## Known Limitations

- MSVC compilation for dsview-sys shims is not yet implemented (explicit panic with clear message)
- Windows dependency resolution in CMakeLists.txt assumes vcpkg or manual paths (documented in comments)
- Packaging and validation helpers have not been tested on actual Windows or macOS systems yet (local Linux validation only)
- Bundle validation smoke tests only verify `--help` commands, not actual device discovery or capture (hardware-independent validation only)

## Commits

| Commit | Message |
|--------|---------|
| 02a3985 | refactor(01-01): add target-aware runtime naming and portable build orchestration |
| 3dbb956 | refactor(01-01): make CMakeLists.txt portable for Linux/macOS/Windows |
| 80432e6 | feat(01-01): add bundle packaging and validation helpers |
| ba4a563 | test(01-01): add Wave 0 tests for runtime naming and bundle discovery |
| 914f026 | chore(01-01): remove unused needs_m_link helper |

## Next Steps

Plan 01-02 will consume these outputs to:
- Update CLI to use `RuntimeDiscoveryPaths::from_executable_dir()` with the target-aware runtime naming
- Remove `--library` and `--use-source-runtime` from public CLI surface
- Keep `--resource-dir` as the only resource override mechanism
- Wire the bundle discovery contract into `dsview-cli` and `dsview-core` public APIs

Plan 01-03 will consume these outputs to:
- Use `tools/package-bundle.rs` in CI matrix jobs for all six targets
- Use `tools/validate-bundle.rs` in CI to verify unpacked bundle layout and smoke tests
- Document the bundle contract in README.md
- Wire the same packaging/validation flow into tag-driven release automation

## Self-Check: PASSED

All created files verified:
- FOUND: tools/package-bundle.rs
- FOUND: tools/validate-bundle.rs
- FOUND: crates/dsview-sys/tests/runtime_packaging.rs

All commits verified:
- FOUND: 02a3985
- FOUND: 3dbb956
- FOUND: 80432e6
- FOUND: ba4a563
- FOUND: 914f026
