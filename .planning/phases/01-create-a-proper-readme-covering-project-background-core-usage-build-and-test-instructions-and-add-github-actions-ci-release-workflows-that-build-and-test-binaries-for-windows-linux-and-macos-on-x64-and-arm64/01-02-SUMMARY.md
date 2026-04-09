---
phase: 01
plan: 02
subsystem: cli-core-contract
tags: [phase-1, cli, dsview-core, resource-discovery, runtime-discovery, packaging]
dependency_graph:
  requires:
    - target-aware runtime naming helper (dsview_sys::runtime_library_name)
    - portable CMakeLists.txt for Linux/macOS/Windows
    - bundle packaging helper (tools/package-bundle.rs)
    - bundle validation helper (tools/validate-bundle.rs)
  provides:
    - bundled runtime/resource discovery as default CLI behavior
    - removed --library and --use-source-runtime flags
    - preserved --resource-dir as only resource override
    - Wave 0 CLI contract tests
  affects:
    - crates/dsview-cli/src/main.rs
    - crates/dsview-core/src/lib.rs
    - crates/dsview-cli/tests/capture_cli.rs
    - crates/dsview-cli/tests/devices_cli.rs
    - crates/dsview-core/tests/bundle_discovery.rs
tech_stack:
  added: []
  patterns:
    - executable-relative runtime/resource discovery with developer fallback
    - single source of truth for runtime naming via dsview_sys::runtime_library_name()
key_files:
  created: []
  modified:
    - crates/dsview-core/src/lib.rs
    - crates/dsview-cli/src/main.rs
    - crates/dsview-cli/tests/capture_cli.rs
    - crates/dsview-cli/tests/devices_cli.rs
    - crates/dsview-core/tests/bundle_discovery.rs
decisions:
  - Use dsview_sys::runtime_library_name() as single source of truth for runtime naming
  - Preserve developer fallback to source runtime when bundled runtime not present
  - Make bundled resources the default with --resource-dir as explicit override only
metrics:
  duration_minutes: 3
  tasks_completed: 4
  commits: 1
  files_created: 0
  files_modified: 1
  tests_added: 0
  completed_at: "2026-04-09T07:54:12Z"
---

# Phase 01 Plan 02: Convert runtime and resource discovery into the public Phase 1 CLI contract

**One-liner:** Reconciled pre-existing bundled discovery work (commit 96008ca) with plan 01-01's target-aware runtime naming contract.

## What Was Built

This plan reconciled work that was already substantially complete in commit 96008ca (Apr 9 14:48) with the target-aware runtime naming helper added by plan 01-01.

**Pre-existing work from commit 96008ca:**
1. Removed `--library` and `--use-source-runtime` flags from CLI surface
2. Implemented `RuntimeDiscoveryPaths::discover()` and `from_executable_dir()` with bundled runtime/resource discovery
3. Added developer fallback to source runtime when bundled runtime not present
4. Preserved `--resource-dir` as the only resource override mechanism
5. Updated CLI help text to reflect bundled defaults
6. Added comprehensive test coverage in `capture_cli.rs`, `devices_cli.rs`, and `bundle_discovery.rs`

**New work in this plan execution:**
1. **Replaced duplicate runtime naming function**: Removed `platform_runtime_library_name()` from `dsview-core` and replaced all usages with `dsview_sys::runtime_library_name()` to establish single source of truth from plan 01-01's target-aware naming contract.

## Deviations from Plan

None - plan tasks were already complete except for the runtime naming integration.

## Technical Decisions

**Use dsview_sys::runtime_library_name() as single source of truth**: The pre-existing commit 96008ca had duplicated the runtime naming logic in `dsview-core`. This reconciliation removes that duplication and uses the public API from `dsview-sys` that plan 01-01 established, ensuring the target-aware naming contract is consistent across build, packaging, discovery, and test layers.

**Preserve developer fallback behavior**: The bundled runtime discovery falls back to source runtime when `<exe_dir>/runtime/<platform-runtime-name>` doesn't exist, enabling `cargo run` workflows during development without requiring bundle packaging.

## Verification

All Wave 0 tests pass:

```
cargo test -p dsview-core --test bundle_discovery
running 8 tests
test bundle_defaults_resolve_from_executable_layout ... ok
test bundle_discovery_uses_target_aware_runtime_filename ... ok
test bundle_discovery_rejects_wrong_platform_runtime ... ok
test bundle_layout_matches_packaging_contract ... ok
test developer_fallback_uses_source_runtime_and_repo_resources ... ok
test discovery_paths_feed_connect_auto_contract_without_resource_override ... ok
test resource_override_wins_over_bundled_resource_dir ... ok
test runtime_library_name_helper_is_consistent ... ok

cargo test -p dsview-cli --test capture_cli
running 6 tests
test capture_help_mentions_bundled_resource_override_and_artifact_controls ... ok
test capture_rejects_removed_runtime_selection_flags ... ok
test capture_invalid_output_path_fails_on_stderr_in_text_mode ... ok
test capture_invalid_metadata_output_fails_on_stderr_in_text_mode ... ok
test capture_conflicting_artifact_paths_fail_before_runtime_work ... ok
test capture_missing_resource_files_reports_bundle_relative_guidance ... ok

cargo test -p dsview-cli --test devices_cli
running 5 tests
test devices_help_does_not_expose_runtime_selection_flags ... ok
test devices_list_help_keeps_resource_override_only ... ok
test devices_open_help_keeps_resource_override_only ... ok
test devices_list_rejects_removed_library_flag ... ok
test devices_open_rejects_removed_use_source_runtime_flag ... ok
```

Full workspace tests pass: `cargo test --workspace` completes successfully.

CLI help output verified manually:
- `dsview-cli capture --help` shows `--resource-dir` with "bundled resources are used by default"
- No `--library` or `--use-source-runtime` flags present in any command help

## Known Limitations

None - all plan requirements satisfied.

## Commits

| Commit | Message |
|--------|---------|
| 54c18af | refactor(01-02): use target-aware runtime naming from dsview-sys |

**Note:** The bulk of this plan's work was completed in commit 96008ca (before plan 01-01 landed). This execution reconciled that pre-existing work with the target-aware runtime naming contract from plan 01-01.

## Next Steps

Plan 01-03 will consume these outputs to:
- Document the bundled runtime/resource discovery contract in README.md
- Create CI workflows that use `tools/package-bundle.rs` and `tools/validate-bundle.rs` for all six targets
- Create release workflows that publish validated bundles with checksums
- Ensure README examples reflect the new CLI surface (no runtime flags, `--resource-dir` override only)

## Self-Check: PASSED

All modified files verified:
- FOUND: crates/dsview-core/src/lib.rs

All commits verified:
- FOUND: 54c18af
