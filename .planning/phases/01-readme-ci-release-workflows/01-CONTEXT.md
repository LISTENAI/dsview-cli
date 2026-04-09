# Phase 1: create-a-proper-readme-covering-project-background-core-usage-build-and-test-instructions-and-add-github-actions-ci-release-workflows-that-build-and-test-binaries-for-windows-linux-and-macos-on-x64-and-arm64 - Context

**Gathered:** 2026-04-09
**Status:** Ready for planning

<domain>
## Phase Boundary

Define Phase 1 as the next-milestone portability, packaging, documentation, and automation foundation for DSView CLI. This phase must deliver a proper project README plus CI/release workflows, but those outputs must be grounded in a real repo-state change: the CLI should move from explicit runtime selection to a single source-built runtime model that works across Windows, Linux, and macOS on x64 and arm64, with honest packaged resources and release artifacts.

</domain>

<decisions>
## Implementation Decisions

### Platform Support Contract
- **D-01:** Treat the roadmap literally: Phase 1 includes whatever repository-side portability refactors are needed so Windows, Linux, and macOS on x64 and arm64 all become real supported targets.
- **D-02:** Phase completion requires the full six-target matrix to be genuinely green in CI and release automation; documented blockers or partial target success do not count as done.
- **D-03:** Keep `DSView/` read-only while allowing substantial refactors in `crates/dsview-sys`, native build logic, packaging logic, and crate boundaries if needed to achieve portable source builds.

### Runtime and Resource Model
- **D-04:** Remove both `--library` and `--use-source-runtime` from the public CLI surface; runtime selection is no longer user-configurable.
- **D-05:** The only supported runtime path after this phase is the runtime built from local source as part of this repository/toolchain.
- **D-06:** Release bundles must ship runtime resources alongside the CLI and source-built runtime, and the CLI should locate those resources automatically via stable relative-path conventions.
- **D-07:** Keep `--resource-dir` as a stable, documented public override for bundled resources.

### README Shape
- **D-08:** Structure the README as quick-start-first documentation rather than a reference manual.
- **D-09:** The primary quick-start flow is `build -> devices list -> capture`.
- **D-10:** Put platform limitations, support posture, and the new runtime/resource packaging model immediately after quick start so users learn the truth early.
- **D-11:** Keep README command coverage focused on core workflows and typical examples; rely on CLI `--help` output for exhaustive parameter detail.

### CI Expectations
- **D-12:** Use one main CI workflow that exposes the full cross-platform matrix on every push and pull request.
- **D-13:** Every target in CI must run build, tests, and packaged-artifact validation.
- **D-14:** Packaged-artifact validation must at least verify bundle structure, runtime/resource relative-path layout, and a post-unpack executable smoke check.
- **D-15:** Every matrix job is merge-blocking; there are no informational or non-blocking targets in this phase.

### Release Contract
- **D-16:** Use tag-driven official releases.
- **D-17:** Each target publishes a complete bundle containing the CLI, the source-built runtime, and bundled resources.
- **D-18:** Every release must publish checksums and platform/packaging notes, including how bundled resources work and how `--resource-dir` overrides them.
- **D-19:** If any target bundle fails to build or validate, the release fails as a whole rather than publishing a partial matrix.

### Claude's Discretion
- Exact relative-path layout inside each bundle, as long as it stays stable and documentable.
- Whether platform-specific packaging helpers are shared or split, as long as the public contract above stays intact.
- The exact smoke-check command used in CI artifact validation, as long as it exercises the unpacked bundle without assuming hosted hardware access.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase and Product Scope
- `.planning/ROADMAP.md` — Defines the new Phase 1 entry and anchors this work as the next milestone’s first phase.
- `.planning/PROJECT.md` — Captures the shipped v1 baseline, read-only `DSView/` constraint, and the expectation that future milestones build outward from the validated DSLogic Plus workflow.
- `.planning/REQUIREMENTS.md` — Records the between-milestones candidate scope and standing constraints that still govern automation, native boundaries, and scriptable CLI behavior.

### Phase-Specific Research
- `.planning/phases/01-readme-ci-release-workflows/01-RESEARCH.md` — Documents the current Linux-centric build reality, current CLI/documentation-safe examples, and the portability blockers this phase must address.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/dsview-cli/src/main.rs` — Current clap command structure for `devices list`, `devices open`, and `capture`; this is the source of truth for which workflows the README should highlight and which flags must change.
- `crates/dsview-core/src/lib.rs` — Existing `Discovery::connect()` / `Discovery::connect_auto()` and `ResourceDirectory::discover()` flow show where runtime selection and resource discovery are currently wired.
- `crates/dsview-sys/build.rs` — Central build-script entry point that already builds the source runtime and is the natural seam for platform-aware portability, artifact naming, and packaging groundwork.
- `Cargo.toml` and `crates/dsview-cli/Cargo.toml` — Workspace and CLI crate manifests define the current package boundaries the CI/release workflows must build and test.

### Established Patterns
- Native integration is intentionally isolated behind Rust layers (`dsview-cli` -> `dsview-core` -> `dsview-sys`), so portability work should keep unsafe and DSView-specific logic concentrated near `dsview-sys`.
- The product is scriptable-first and machine-readable-first, so documentation and workflow design should optimize for reproducible CLI usage rather than GUI-style setup steps.
- `DSView/` remains an upstream dependency subtree, so portability and packaging work must adapt around it rather than modifying it.

### Integration Points
- CLI surface changes land in `crates/dsview-cli/src/main.rs` and its tests in `crates/dsview-cli/tests/`.
- Runtime/resource auto-discovery and validation changes connect through `crates/dsview-core/src/lib.rs`.
- Cross-platform build, source-runtime output naming, and packaging hooks connect through `crates/dsview-sys/build.rs` and the native runtime build directory.
- Documentation and automation deliverables connect at repo root (`README.md`, `.github/workflows/`, release artifact layout).

</code_context>

<specifics>
## Specific Ideas

- The CLI should stop exposing runtime choice to users; runtime handling becomes an internal implementation detail.
- Bundled resources should make the default user path work out of the box, while `--resource-dir` stays available as an explicit override.
- README should feel task-oriented: prove the environment with `devices list`, then move into `capture`.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 01-readme-ci-release-workflows*
*Context gathered: 2026-04-09*
