# Phase 1: create-a-proper-readme-covering-project-background-core-usage-build-and-test-instructions-and-add-github-actions-ci-release-workflows-that-build-and-test-binaries-for-windows-linux-and-macos-on-x64-and-arm64 - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-09T00:00:00Z
**Phase:** 01-readme-ci-release-workflows
**Areas discussed:** Support posture, README structure, CI coverage, Release workflow

---

## Support posture

| Option | Description | Selected |
|--------|-------------|----------|
| Linux-first and explicit about it | README, CI, and releases only promise Linux for now; other targets remain future work. | |
| Full roadmap literally in this phase | Include portability groundwork so Windows/macOS/Linux x64+arm64 become real targets before claims are made. | ✓ |
| Hybrid | Show the full matrix but skip/disable unsupported targets with blocker notes. | |
| You decide | Leave the choice open for planning. | |

**User's choice:** Full roadmap literally in this phase.
**Notes:** The phase should do the real portability work rather than settle for honest Linux-only automation.

| Option | Description | Selected |
|--------|-------------|----------|
| Full matrix truly green | CI builds/tests and release artifacts succeed for every promised target. | ✓ |
| CI green everywhere, releases narrower | All targets must build/test, but artifact publishing can stay narrower. | |
| Compile-only portability | Some targets only need to build for now. | |
| You decide | Leave the bar flexible. | |

**User's choice:** Full matrix truly green.
**Notes:** Success means Windows, Linux, and macOS on x64 and arm64 all have real CI and release success.

| Option | Description | Selected |
|--------|-------------|----------|
| Source-runtime works everywhere | `--use-source-runtime` remains the model and must work on every target. | |
| CLI builds everywhere, some targets use `--library` | Some platforms can still depend on a user-supplied runtime. | |
| Mixed by platform | Runtime model varies by operating system. | |
| User correction | Remove both runtime-selection flags and only use the repository-built runtime. | ✓ |

**User's choice:** Remove both runtime-selection flags and only use the repository-built runtime.
**Notes:** The user corrected the earlier framing: `--library` and `--use-source-runtime` should both be removed from the public CLI in this phase.

| Option | Description | Selected |
|--------|-------------|----------|
| Full auto locate/bundle resources | CLI finds packaged resources automatically and users do not pass a resource path. | |
| Keep `--resource-dir` only | Remove runtime flags, but keep resource directory entirely user-managed. | |
| Auto by default + override | Bundle resources, auto-discover them, and keep `--resource-dir` as an override. | ✓ |
| You decide | Leave resource UX open. | |

**User's choice:** Auto by default plus `--resource-dir` override.
**Notes:** Resources ship with the bundle and are discovered by relative path, but `--resource-dir` remains a stable public override.

| Option | Description | Selected |
|--------|-------------|----------|
| Stable public interface | `--resource-dir` stays documented and supported. | ✓ |
| Mostly dev/debug use | Keep it but de-emphasize it in user docs. | |
| Temporary compatibility interface | Keep it briefly, then remove it later. | |
| You decide | Leave this to planning. | |

**User's choice:** Stable public interface.
**Notes:** `--resource-dir` should remain a documented, supported override.

| Option | Description | Selected |
|--------|-------------|----------|
| Do whatever repo-side refactor is needed | Refactor `dsview-sys`, build logic, and packaging as needed while keeping `DSView/` read-only. | ✓ |
| Keep refactors minimal | Avoid large structural changes. | |
| Workflows/docs only | Treat deep refactors as out of scope. | |
| You decide | Leave this flexible. | |

**User's choice:** Do whatever repo-side refactor is needed.
**Notes:** Large repository-side changes are acceptable if they preserve the read-only `DSView/` boundary.

| Option | Description | Selected |
|--------|-------------|----------|
| Phase is not done until all 6 targets work | The roadmap promise stands unless scope changes later. | ✓ |
| Allow documented exceptions | Proven external blockers can exempt a target. | |
| Ship partial support | Move blocked targets to follow-up work. | |
| You decide | Leave this flexible. | |

**User's choice:** Phase is not done until all 6 targets work.
**Notes:** No partial-credit milestone if any target remains blocked.

---

## README structure

| Option | Description | Selected |
|--------|-------------|----------|
| Quick start first | Lead with the fastest path to using the product. | ✓ |
| Build/setup first | Explain prerequisites and packaging before usage. | |
| Reference style | Present the README as a CLI manual. | |
| Mixed | Quick start first, then full reference and architecture. | |
| You decide | Leave the structure open. | |

**User's choice:** Quick start first.
**Notes:** README should prioritize the fastest path to a successful workflow.

| Option | Description | Selected |
|--------|-------------|----------|
| Capture first | Lead with the main value path immediately. | |
| `devices list` first | Start by confirming environment and device discovery. | ✓ |
| `--help` first | Preview command surface before real usage. | |
| You decide | Leave this to planning. | |

**User's choice:** `devices list` first.
**Notes:** The README should confirm environment/device connectivity before showing capture.

| Option | Description | Selected |
|--------|-------------|----------|
| `build -> devices list -> capture` | Quick start starts from source build, then discovery, then capture. | ✓ |
| `build -> help -> devices list -> capture` | Insert help preview before discovery. | |
| `install -> devices list -> capture` | Lead with end-user install flow. | |
| You decide | Leave quick-start sequencing open. | |

**User's choice:** `build -> devices list -> capture`.
**Notes:** This matches the current repo-first delivery model.

| Option | Description | Selected |
|--------|-------------|----------|
| Put limitations right after quick start | Explain platform truth and packaging model immediately after the main path. | ✓ |
| Put limitations after build/setup | Explain constraints later in the README. | |
| Put limitations near the end | Keep the top flow as smooth as possible. | |
| You decide | Leave this to planning. | |

**User's choice:** Put limitations right after quick start.
**Notes:** README should make platform status and runtime/resource changes visible early.

| Option | Description | Selected |
|--------|-------------|----------|
| Core commands in README, details in `--help` | README stays workflow-focused and avoids becoming a flag encyclopedia. | ✓ |
| Full CLI manual in README | Document most parameters inline. | |
| Mostly flows, little command detail | Push users to `--help` for almost everything. | |
| You decide | Leave the balance open. | |

**User's choice:** Core commands in README, details in `--help`.
**Notes:** README should show representative commands and leave exhaustive flag documentation to the CLI.

---

## CI coverage

| Option | Description | Selected |
|--------|-------------|----------|
| Build + tests + packaged artifact validation | Every target must validate the built product and the final bundle shape. | ✓ |
| Build + tests | Package validation can wait for release workflow. | |
| Layered target depth | Some targets only build while others run fuller checks. | |
| You decide | Leave this to planning. | |

**User's choice:** Build + tests + packaged artifact validation.
**Notes:** CI must validate final delivery shape, not just compilation.

| Option | Description | Selected |
|--------|-------------|----------|
| One full matrix | One main CI reports the full cross-platform truth on each push/PR. | ✓ |
| Layered workflows | Light CI plus heavier full-platform verification. | |
| Per-OS workflows | Split Linux, macOS, and Windows into separate workflows. | |
| You decide | Leave organization open. | |

**User's choice:** One full matrix.
**Notes:** Avoids hiding failures behind optional heavy workflows.

| Option | Description | Selected |
|--------|-------------|----------|
| Structure + basic executable validation | Verify archive contents, relative layout, and unpacked smoke execution. | ✓ |
| Structure only | Check archive layout but do not run unpacked artifacts. | |
| Near-real quick-start replay | Re-run a larger automated user flow after unpack. | |
| You decide | Leave smoke depth open. | |

**User's choice:** Structure + basic executable validation.
**Notes:** CI should verify bundle structure and unpacked executability without pretending hosted runners can do hardware validation.

| Option | Description | Selected |
|--------|-------------|----------|
| All matrix jobs block merges | Any target failure blocks the pull request. | ✓ |
| Some non-main targets can be non-blocking | Allow informational failures during transition. | |
| Only build/tests block, artifact validation is softer | Partial enforcement. | |
| You decide | Leave merge policy open. | |

**User's choice:** All matrix jobs block merges.
**Notes:** This matches the six-target completion bar.

---

## Release workflow

| Option | Description | Selected |
|--------|-------------|----------|
| Full bundle per target | Publish CLI + source-built runtime + resources together. | ✓ |
| CLI binary only | Other pieces are installed or fetched separately. | |
| Split artifacts | Publish CLI, runtime, and resources independently. | |
| You decide | Leave artifact model open. | |

**User's choice:** Full bundle per target.
**Notes:** Release artifacts should be usable with the default relative-path model.

| Option | Description | Selected |
|--------|-------------|----------|
| Tag-driven releases | Official multi-platform releases happen from version tags. | ✓ |
| Manual release trigger | Publish from manual workflow dispatch. | |
| Both | Tags for official releases, manual for preflight. | |
| You decide | Leave this to planning. | |

**User's choice:** Tag-driven releases.
**Notes:** Release flow should align with explicit versioning.

| Option | Description | Selected |
|--------|-------------|----------|
| Checksums + packaging notes | Release includes checksums and notes about platform packaging and resource overrides. | ✓ |
| Minimal bundle-only release | Keep notes brief. | |
| Add installer metadata/scripts too | Expand release surface further. | |
| You decide | Leave release metadata open. | |

**User's choice:** Checksums + packaging notes.
**Notes:** Notes should mention bundled resources, `--resource-dir`, and target-specific expectations.

| Option | Description | Selected |
|--------|-------------|----------|
| Fail whole release if any target fails | Do not publish partial platform results. | ✓ |
| Publish successful targets only | Partial matrix publication is acceptable. | |
| Draft until complete | Hold release open until the matrix is fixed. | |
| You decide | Leave failure policy open. | |

**User's choice:** Fail whole release if any target fails.
**Notes:** Keeps release behavior aligned with the all-target support promise.

## Claude's Discretion

- Exact bundle layout and packaging implementation details.
- Exact smoke-check command used for unpacked artifact validation.
- Internal refactor shape needed to make source-built runtime portable while preserving the `DSView/` read-only boundary.

## Deferred Ideas

None.
