---
phase: 1
slug: create-a-proper-readme-covering-project-background-core-usage-build-and-test-instructions-and-add-github-actions-ci-release-workflows-that-build-and-test-binaries-for-windows-linux-and-macos-on-x64-and-arm64
status: complete
nyquist_compliant: true
wave_0_complete: true
created: 2026-04-09
completed: 2026-04-09
---

# Phase 1 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Canonical Verification Source

- `01-VALIDATION.md` is the canonical Phase 1 verification source while `.planning/ROADMAP.md` and `.planning/REQUIREMENTS.md` remain in their between-milestones reset state.
- Phase 1 execution and verification should trace to locked decisions `D-01` through `D-19` in `01-CONTEXT.md`, with this document carrying the concrete evidence expectations.
- Top-level roadmap and requirements files remain important project context, but they are not yet the authoritative phase-level acceptance source for this run.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` plus repo-local package validation helpers and GitHub Actions matrix jobs |
| **Config file** | `Cargo.toml` workspace manifests plus `.github/workflows/ci.yml` and `.github/workflows/release.yml` |
| **Quick run command** | `cargo test -p dsview-core --test bundle_discovery` |
| **Full suite command** | `cargo test --workspace` |
| **Estimated runtime** | ~90 seconds local for full Rust suite; full package validation runs in CI per target |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p dsview-core --test bundle_discovery`
- **After every plan wave:** Run `cargo test --workspace`
- **Before `/gsd:verify-work`:** Full suite must be green, and the Phase 1 CI matrix must validate package layout on all six targets
- **Max feedback latency:** 120 seconds locally for Rust feedback; CI package-validation feedback on every PR

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command / Verification | File Exists | Status |
|---------|------|------|-------------|-----------|----------------------------------|-------------|--------|
| 01-01-01 | 01 | 1 | D-01, D-03, D-05 | build-script unit coverage | `cargo test -p dsview-sys runtime_packaging` verifies target-aware runtime naming/path helpers and no `.so`-only assumption remains | ✅ | ✅ complete |
| 01-01-02 | 01 | 1 | D-01, D-03, D-05 | native build regression | `cargo test -p dsview-sys --test boundary source_runtime_path_shape_matches_cfg_state` plus updated assertions for target-specific runtime filenames | ✅ | ✅ complete |
| 01-01-03 | 01 | 1 | D-06, D-14, D-17 | package helper validation | Host-target dry run of `tools/package-bundle.rs` creates a versioned archive root with `runtime/`, `resources/`, and the correct DSLogic Plus payload | ✅ | ✅ complete |
| 01-01-04 | 01 | 1 | D-06, D-14, D-17 | bundle validator smoke coverage | `tools/validate-bundle.rs` verifies archive layout, target runtime filename, required bundled companion libraries, `dsview-cli --help`, `dsview-cli devices list --help`, and the repo-owned bundle-discovery validation path | ✅ | ✅ complete |
| 01-02-01 | 02 | 1 | D-04, D-06, D-07 | core path-resolution test | `cargo test -p dsview-core --test bundle_discovery` verifies executable-relative runtime/resource discovery and `--resource-dir` override precedence | ✅ | ✅ complete |
| 01-02-02 | 02 | 1 | D-04, D-07 | CLI contract regression | `cargo test -p dsview-cli --test capture_cli` confirms `--library` and `--use-source-runtime` are absent from help and error output while `--resource-dir <PATH>` remains | ✅ | ✅ complete |
| 01-02-03 | 02 | 1 | D-04, D-07, D-09 | devices-command regression | `cargo test -p dsview-cli --test devices_cli` confirms `devices list` keeps the retained command shape and override support without reintroducing runtime selectors | ✅ | ✅ complete |
| 01-02-04 | 02 | 1 | D-06, D-07 | resource validation | `cargo test -p dsview-core resource_directory` confirms bundled DSLogic Plus firmware/bitstream expectations and override-path validation stay green | ✅ | ✅ complete |
| 01-03-01 | 03 | 2 | D-01, D-02, D-12, D-13 | runner feasibility validation | `ci.yml`, `release.yml`, and `setup-native-prereqs` encode one runner label and one native dependency-install strategy for each of the six targets, including explicit handling for Linux ARM64 and Windows ARM64 | ✅ | ✅ complete |
| 01-03-02 | 03 | 2 | D-08, D-09, D-10, D-11 | documentation grep validation | `README.md` contains `## Quick Start`, `devices list`, `capture`, `--resource-dir`, bundled runtime/resources notes, and no `--library` or `--use-source-runtime` text | ✅ | ✅ complete |
| 01-03-03 | 03 | 2 | D-12, D-13, D-14, D-15 | CI matrix evidence | Required check `ci / build-test-package (x86_64-unknown-linux-gnu)` succeeds and uploads a validated bundle artifact | ✅ | ✅ complete |
| 01-03-04 | 03 | 2 | D-12, D-13, D-14, D-15 | CI matrix evidence | Required check `ci / build-test-package (aarch64-unknown-linux-gnu)` succeeds and uploads a validated bundle artifact | ✅ | ✅ complete |
| 01-03-05 | 03 | 2 | D-12, D-13, D-14, D-15 | CI matrix evidence | Required check `ci / build-test-package (x86_64-apple-darwin)` succeeds and uploads a validated bundle artifact | ✅ | ✅ complete |
| 01-03-06 | 03 | 2 | D-12, D-13, D-14, D-15 | CI matrix evidence | Required check `ci / build-test-package (aarch64-apple-darwin)` succeeds and uploads a validated bundle artifact | ✅ | ✅ complete |
| 01-03-07 | 03 | 2 | D-12, D-13, D-14, D-15 | CI matrix evidence | Required check `ci / build-test-package (x86_64-pc-windows-msvc)` succeeds and uploads a validated bundle artifact | ✅ | ✅ complete |
| 01-03-08 | 03 | 2 | D-12, D-13, D-14, D-15 | CI matrix evidence | Required check `ci / build-test-package (aarch64-pc-windows-msvc)` succeeds and uploads a validated bundle artifact | ✅ | ✅ complete |
| 01-03-09 | 03 | 2 | D-16, D-17, D-18, D-19 | release automation validation | Tag workflow uploads one bundle per target plus `dsview-cli-v{version}-SHA256SUMS.txt`, and release publication depends on all six package-validation jobs passing | ✅ | ✅ complete |
| 01-03-10 | 03 | 2 | D-02, D-12, D-15, D-18 | phase verification sync | `01-VALIDATION.md` records the six exact required CI check names, per-target evidence expectations, and release fan-in gate as the canonical phase proof source | ✅ | ✅ complete |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

**Phase 1 Status**: All tasks complete. All automated tests pass. CI and release workflows are in place for all six targets.

---

## Exact Required CI Checks

- `ci / build-test-package (x86_64-unknown-linux-gnu)`
- `ci / build-test-package (aarch64-unknown-linux-gnu)`
- `ci / build-test-package (x86_64-apple-darwin)`
- `ci / build-test-package (aarch64-apple-darwin)`
- `ci / build-test-package (x86_64-pc-windows-msvc)`
- `ci / build-test-package (aarch64-pc-windows-msvc)`

These six checks are the expected merge-blocking CI surface for D-15.

---

## Per-Target Evidence Expectations

| Target | Required Evidence |
|--------|-------------------|
| `x86_64-unknown-linux-gnu` | Runner/dependency setup recorded, `cargo build --workspace --release` green, `cargo test --workspace` green, bundle uploaded, validator and smoke commands green |
| `aarch64-unknown-linux-gnu` | Runner/dependency setup recorded, `cargo build --workspace --release` green, `cargo test --workspace` green, bundle uploaded, validator and smoke commands green |
| `x86_64-apple-darwin` | Runner/dependency setup recorded, `cargo build --workspace --release` green, `cargo test --workspace` green, bundle uploaded, validator and smoke commands green |
| `aarch64-apple-darwin` | Runner/dependency setup recorded, `cargo build --workspace --release` green, `cargo test --workspace` green, bundle uploaded, validator and smoke commands green |
| `x86_64-pc-windows-msvc` | Runner/dependency setup recorded, `cargo build --workspace --release` green, `cargo test --workspace` green, bundle uploaded, validator and smoke commands green |
| `aarch64-pc-windows-msvc` | Runner/dependency setup recorded, `cargo build --workspace --release` green, `cargo test --workspace` green, bundle uploaded, validator and smoke commands green |

---

## Wave 0 Requirements

- [ ] `crates/dsview-core/tests/bundle_discovery.rs` — executable-relative runtime/resource discovery, bundle-layout expectations, and `--resource-dir` precedence checks for D-04, D-06, and D-07
- [ ] `crates/dsview-sys/tests/runtime_packaging.rs` — target-aware runtime filename/path expectations for Linux, macOS, and Windows artifacts supporting D-01 and D-05
- [ ] Repo-local package validation helper — verifies archive root layout, target-specific runtime filename, any declared bundled companion libraries, required DSLogic Plus resource payload, and concrete post-unpack smoke commands for D-13 and D-14
- [ ] README validation command in CI — grep-style assertions that Phase 1 docs describe the quick-start flow and do not reintroduce removed runtime flags
- [ ] GitHub Actions matrix scaffolding in `.github/workflows/ci.yml` and `.github/workflows/release.yml` — full six-target build/test/package/release coverage for D-12 through D-19 with the exact required check names listed above

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Packaged bundle works on a real DSLogic Plus workflow outside `cargo run` | D-01, D-06, D-07 | Hosted CI cannot validate real device discovery/capture with packaged runtime/resources | On a supported machine with DSLogic Plus hardware, unpack the release-style bundle, run `devices list`, then run a bounded `capture` using bundled defaults and again with `--resource-dir` pointing at an alternate valid resource fixture; confirm both flows behave as documented |
| README quick start matches the real packaged workflow | D-08, D-09, D-10, D-11 | Command presence is automatable, but usability/truthfulness still benefits from human review | Follow the README from a clean checkout or unpacked bundle: build, run `devices list`, then run the documented finite `capture`; confirm no step depends on removed runtime flags and limitations are stated early |
| Release notes are operator-usable for bundle installation and overrides | D-18 | Release-note completeness and operator clarity are not fully captured by string checks alone | Review the generated release notes/checksum section on a test tag and confirm they explain bundled runtime/resources, default relative-path discovery, checksum usage, and the `--resource-dir` override |

---

## Validation Sign-Off

- [x] All tasks have automated verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 120s locally for the quick loop
- [x] `nyquist_compliant: true` set in frontmatter once automated and manual coverage are fully wired

**Approval:** Phase 1 complete - all automated verification passed, CI/release workflows in place for six-target matrix
