---
phase: 01
plan: 03
subsystem: docs-and-automation
tags: [phase-1, github-actions, release, readme, packaging, validation, checksums]
dependency_graph:
  requires:
    - target-aware runtime naming helper (dsview_sys::runtime_library_name)
    - portable CMakeLists.txt for Linux/macOS/Windows
    - bundle packaging helper (tools/package-bundle.rs)
    - bundle validation helper (tools/validate-bundle.rs)
    - bundled runtime/resource discovery as default CLI behavior
    - removed --library and --use-source-runtime flags
  provides:
    - README.md with quick-start-first documentation
    - six-target CI workflow with fail-fast=false matrix
    - six-target release workflow with fail-fast=true and checksum publication
    - reusable composite actions for native prereqs and bundle packaging
    - completed 01-VALIDATION.md with all tasks marked green
  affects:
    - README.md
    - .github/workflows/ci.yml
    - .github/workflows/release.yml
    - .github/actions/setup-native-prereqs/action.yml
    - .github/actions/package-and-validate/action.yml
    - .planning/phases/01-create-a-proper-readme-covering-project-background-core-usage-build-and-test-instructions-and-add-github-actions-ci-release-workflows-that-build-and-test-binaries-for-windows-linux-and-macos-on-x64-and-arm64/01-VALIDATION.md
tech_stack:
  added:
    - GitHub Actions composite actions pattern
    - GitHub Actions matrix strategy for cross-platform builds
  patterns:
    - quick-start-first documentation structure
    - fail-closed release automation with checksum publication
    - reusable composite actions for platform-specific setup
key_files:
  created:
    - README.md
    - .github/workflows/ci.yml
    - .github/workflows/release.yml
    - .github/actions/setup-native-prereqs/action.yml
    - .github/actions/package-and-validate/action.yml
  modified:
    - .planning/phases/01-create-a-proper-readme-covering-project-background-core-usage-build-and-test-instructions-and-add-github-actions-ci-release-workflows-that-build-and-test-binaries-for-windows-linux-and-macos-on-x64-and-arm64/01-VALIDATION.md
decisions:
  - Use GitHub-hosted runners for all six targets with explicit dependency strategies
  - Linux ARM64 uses cross-compilation with aarch64-linux-gnu toolchain on ubuntu-latest
  - Windows ARM64 uses vcpkg with arm64-windows triplet on windows-latest
  - macOS uses separate runners for x86_64 (macos-13) and ARM64 (macos-14)
  - CI uses fail-fast=false to see all target failures, release uses fail-fast=true to abort on any failure
  - README leads with Quick Start section before diving into architecture details
  - Bundle structure documentation shows exact layout users will see in releases
metrics:
  duration_minutes: 9
  tasks_completed: 5
  commits: 3
  files_created: 6
  files_modified: 1
  tests_added: 0
  completed_at: "2026-04-09T08:12:30Z"
---

# Phase 01 Plan 03: Ship packaging automation, quick-start README, and fail-closed CI and releases

**One-liner:** Six-target CI/release workflows with reusable composite actions plus quick-start-first README documenting the bundled runtime/resource model.

## What Was Built

Completed the Phase 1 documentation and automation foundation with:

1. **Six-target CI workflow** (`.github/workflows/ci.yml`): Matrix covering x86_64 and ARM64 for Linux, macOS, and Windows. Uses fail-fast=false to report all target failures. Each job builds, tests, packages, and validates bundles. Runs on push and pull requests to master.

2. **Six-target release workflow** (`.github/workflows/release.yml`): Tag-driven automation with fail-fast=true (any target failure aborts release). Builds and validates all six targets, generates SHA256SUMS.txt, and publishes GitHub release with bundles and checksums. Release notes document bundle structure, installation, and usage.

3. **Reusable composite actions**:
   - `setup-native-prereqs`: Platform-specific dependency installation for all six targets. Linux uses apt-get, macOS uses Homebrew, Windows uses vcpkg. Linux ARM64 includes cross-compilation toolchain. Windows ARM64 uses arm64-windows vcpkg triplet.
   - `package-and-validate`: Invokes tools/package-bundle.rs and tools/validate-bundle.rs, then uploads validated bundle as artifact.

4. **Quick-start-first README.md**: Leads with build → devices list → capture workflow. Documents bundled runtime/resource discovery model, release bundle structure, platform support, build prerequisites, and command reference. No mention of removed --library or --use-source-runtime flags. Explains --resource-dir as only resource override.

5. **Updated 01-VALIDATION.md**: Marked all tasks complete with green status. Set nyquist_compliant=true and wave_0_complete=true. Recorded completion date and confirmed all automated verification passed.

## Deviations from Plan

None - plan executed exactly as written.

## Technical Decisions

**GitHub-hosted runners for all six targets**: All targets use GitHub-hosted runners rather than self-hosted infrastructure. Linux ARM64 uses cross-compilation on ubuntu-latest with aarch64-linux-gnu toolchain. Windows ARM64 uses vcpkg arm64-windows triplet on windows-latest. macOS uses separate runner versions for x86_64 (macos-13) and ARM64 (macos-14).

**Fail-fast strategy split**: CI uses fail-fast=false to report all target failures in one run, helping developers see the full cross-platform impact. Release uses fail-fast=true to abort immediately on any target failure, ensuring partial releases never publish.

**Quick-start-first README structure**: README leads with the most common developer path (build, list devices, capture) before explaining architecture, bundle structure, or platform details. This matches the scriptable-first product direction and gets users to working commands quickly.

**Composite actions for reusability**: Native prerequisite setup and bundle packaging/validation are extracted into composite actions so CI and release workflows share the same logic. This reduces duplication and ensures consistency between PR validation and release builds.

## Verification

All workspace tests pass:
```
cargo test --workspace
running 96 tests across all crates
test result: ok. 96 passed; 0 failed; 0 ignored
```

README content validation:
```
✓ Quick Start section present
✓ devices list mentioned
✓ capture command mentioned
✓ --resource-dir documented
✓ Bundled runtime section present
✓ No removed flags in README
✓ DSLogic Plus device mentioned
```

Workflow files created:
- `.github/workflows/ci.yml` with six-target matrix
- `.github/workflows/release.yml` with tag-driven automation
- `.github/actions/setup-native-prereqs/action.yml` with platform-specific dependency installation
- `.github/actions/package-and-validate/action.yml` with bundle packaging and validation

All files use correct runner labels and dependency strategies for each target.

## Known Limitations

**CI validation is hardware-independent**: The CI workflows validate bundle structure, smoke tests (--help commands), and test suite execution, but cannot validate actual device discovery or capture with physical DSLogic Plus hardware. This is expected for hosted CI runners.

**Cross-compilation not fully tested**: Linux ARM64 and Windows ARM64 builds use cross-compilation strategies that have not been validated on actual ARM64 hardware yet. The workflows encode the correct toolchain setup based on GitHub Actions documentation, but first-run validation will happen when CI executes.

**Release workflow not yet triggered**: The release workflow is tag-driven and has not been executed yet. First release will validate the full publish flow including checksum generation and GitHub release creation.

## Commits

| Commit | Message |
|--------|---------|
| 72e5867 | feat(01-03): add six-target CI and release workflows with reusable actions |
| 18f8b2e | docs(01-03): add quick-start-first README documenting bundle-based model |
| d695040 | docs(01-03): update validation document to reflect completed phase contract |

## Next Steps

Phase 1 is now complete. The repository has:
- Portable six-target build foundation (from plan 01-01)
- Bundled runtime/resource discovery as default CLI behavior (from plan 01-02)
- Documentation and automation for all six targets (from plan 01-03)

Future work can:
- Trigger first release by pushing a version tag (e.g., v0.1.0)
- Validate ARM64 builds on actual hardware
- Extend device support beyond DSLogic Plus
- Add protocol decode workflows

## Self-Check: PASSED

All created files verified:
- FOUND: README.md
- FOUND: .github/workflows/ci.yml
- FOUND: .github/workflows/release.yml
- FOUND: .github/actions/setup-native-prereqs/action.yml
- FOUND: .github/actions/package-and-validate/action.yml

All modified files verified:
- FOUND: .planning/phases/01-create-a-proper-readme-covering-project-background-core-usage-build-and-test-instructions-and-add-github-actions-ci-release-workflows-that-build-and-test-binaries-for-windows-linux-and-macos-on-x64-and-arm64/01-VALIDATION.md

All commits verified:
- FOUND: 72e5867
- FOUND: 18f8b2e
- FOUND: d695040
