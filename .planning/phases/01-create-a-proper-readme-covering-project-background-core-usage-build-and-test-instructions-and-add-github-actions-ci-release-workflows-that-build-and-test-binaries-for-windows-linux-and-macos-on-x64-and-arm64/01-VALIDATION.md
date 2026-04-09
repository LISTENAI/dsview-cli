---
phase: 1
slug: create-a-proper-readme-covering-project-background-core-usage-build-and-test-instructions-and-add-github-actions-ci-release-workflows-that-build-and-test-binaries-for-windows-linux-and-macos-on-x64-and-arm64
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-09
---

# Phase 1 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` plus repo-local package validation helpers and GitHub Actions matrix jobs |
| **Config file** | `Cargo.toml` workspace manifests plus `.github/workflows/` files added in this phase |
| **Quick run command** | `cargo test -p dsview-core --test bundle_discovery` |
| **Full suite command** | `cargo test --workspace` |
| **Estimated runtime** | ~90 seconds local for full Rust suite; matrix package validation runs in CI |

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
| 01-01-01 | 01 | 1 | D-01, D-03, D-05 | build-script unit coverage | `cargo test -p dsview-sys runtime_path` verifies target-aware runtime naming/path helpers and no `.so`-only assumption remains | ❌ W0 | ⬜ pending |
| 01-01-02 | 01 | 1 | D-01, D-03, D-05 | native build regression | `cargo test -p dsview-sys --test boundary source_runtime_path_shape_matches_cfg_state` plus updated assertions for target-specific runtime filenames | ✅ | ⬜ pending |
| 01-01-03 | 01 | 1 | D-01, D-02, D-05 | build validation | CI logs show `cargo build --workspace --release` succeeds on `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `x86_64-apple-darwin`, `aarch64-apple-darwin`, `x86_64-pc-windows-msvc`, and `aarch64-pc-windows-msvc` | ❌ W0 | ⬜ pending |
| 01-02-01 | 02 | 1 | D-04, D-06, D-07 | core path-resolution test | `cargo test -p dsview-core --test bundle_discovery` verifies executable-relative runtime/resource discovery and `--resource-dir` override precedence | ❌ W0 | ⬜ pending |
| 01-02-02 | 02 | 1 | D-04, D-07 | CLI contract regression | `cargo test -p dsview-cli --test capture_cli` confirms `--library` and `--use-source-runtime` are absent from help and error output while `--resource-dir <PATH>` remains | ✅ | ⬜ pending |
| 01-02-03 | 02 | 1 | D-06, D-07 | resource validation | `cargo test -p dsview-core resource_directory` confirms bundled DSLogic Plus firmware/bitstream expectations and override-path validation stay green | ✅ | ⬜ pending |
| 01-03-01 | 03 | 2 | D-12, D-13, D-14, D-15 | package layout validation | CI package helper verifies each unpacked bundle contains `dsview-cli[.exe]`, `runtime/`, `resources/`, and the target-appropriate runtime filename | ❌ W0 | ⬜ pending |
| 01-03-02 | 03 | 2 | D-12, D-13, D-14 | post-unpack smoke check | CI runs unpacked `dsview-cli --help`, `dsview-cli devices list --help`, and bundle validation commands from a clean temp directory on every target | ❌ W0 | ⬜ pending |
| 01-03-03 | 03 | 2 | D-16, D-17, D-18, D-19 | release automation validation | Tag workflow uploads one bundle per target plus `dsview-cli-v{version}-SHA256SUMS.txt`, and release publication depends on all matrix jobs passing | ❌ W0 | ⬜ pending |
| 01-04-01 | 04 | 2 | D-08, D-09, D-10, D-11 | documentation grep validation | `README.md` contains `## Quick Start`, `devices list`, `capture`, `--resource-dir`, and explains bundled runtime/resources without mentioning `--library` or `--use-source-runtime` | ❌ W0 | ⬜ pending |
| 01-04-02 | 04 | 2 | D-10, D-18 | docs/release-note validation | Workflow or release-note template contains bundled runtime/resources notes, relative-path discovery, `--resource-dir` override guidance, and `DSLogic Plus`-only support language | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/dsview-core/tests/bundle_discovery.rs` — executable-relative runtime/resource discovery, bundle-layout expectations, and `--resource-dir` precedence checks for D-04, D-06, and D-07
- [ ] `crates/dsview-sys/tests/runtime_packaging.rs` or equivalent `crates/dsview-sys` unit coverage — target-aware runtime filename/path expectations for Linux, macOS, and Windows artifacts supporting D-01 and D-05
- [ ] Repo-local package validation helper (shell script, Rust binary, or test harness) — verifies archive root layout, target-specific runtime filename, and required DSLogic Plus resource payload for D-13 and D-14
- [ ] README validation command in CI — grep-style assertions that Phase 1 docs describe the quick-start flow and do not reintroduce removed runtime flags
- [ ] GitHub Actions matrix scaffolding in `.github/workflows/ci.yml` and `.github/workflows/release.yml` — full six-target build/test/package/release coverage for D-12 through D-19

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Packaged bundle works on a real DSLogic Plus workflow outside `cargo run` | D-01, D-06, D-07 | Hosted CI cannot validate real device discovery/capture with packaged runtime/resources | On a supported machine with DSLogic Plus hardware, unpack the release-style bundle, run `devices list`, then run a bounded `capture` using bundled defaults and again with `--resource-dir` pointing at an alternate valid resource fixture; confirm both flows behave as documented |
| README quick start matches the real packaged workflow | D-08, D-09, D-10, D-11 | Command presence is automatable, but usability/truthfulness still benefits from human review | Follow the README from a clean checkout or unpacked bundle: build, run `devices list`, then run the documented finite `capture`; confirm no step depends on removed runtime flags and limitations are stated early |
| Release notes are operator-usable for bundle installation and overrides | D-18 | Release-note completeness and operator clarity are not fully captured by string checks alone | Review the generated release notes/checksum section on a test tag and confirm they explain bundled runtime/resources, default relative-path discovery, checksum usage, and the `--resource-dir` override |

---

## Validation Sign-Off

- [ ] All tasks have automated verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s locally for the quick loop
- [ ] `nyquist_compliant: true` set in frontmatter once automated and manual coverage are fully wired

**Approval:** pending
