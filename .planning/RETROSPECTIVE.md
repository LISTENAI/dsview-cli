# Project Retrospective

*A living document updated after each milestone. Lessons feed forward into future planning.*

## Milestone: v1.1 - DSLogic Plus device options

**Shipped:** 2026-04-13
**Phases:** 4 | **Plans:** 12 | **Sessions:** not systematically tracked

### What Was Built
- DSView-backed `DSLogic Plus` option discovery across sys, core, and CLI, including the new `devices options` command.
- Mode-aware validation and friendly CLI selection for operation mode, stop option, channel mode, enabled channels, threshold volts, and filter.
- Deterministic option-aware capture execution with requested/effective reporting in JSON, text, and schema-v2 metadata.

### What Worked
- Keeping stable IDs in core and layering friendly tokens in the CLI preserved automation compatibility without hurting usability.
- The shipped `v1.0` baseline stayed intact because `v1.1` treated option work as additive seams instead of refactoring the whole capture path.
- Explicit human UAT on real hardware caught and then closed the last live-runtime confidence gap before milestone closeout.

### What Was Inefficient
- Milestone closeout started without a standalone `v1.1` milestone audit artifact, so archive-time review had to proceed on verified phase evidence alone.
- Some verification commands in planning docs lagged behind the real `dsview-cli` binary test target, creating avoidable audit noise.
- Hardware verification still required manual environment handoff instead of being schedulable earlier in the phase exit flow.

### Patterns Established
- Use owned native snapshots plus stable Rust normalization for DSView-derived device facts.
- Model requested and effective runtime values once, then reuse that structure across CLI output and metadata.
- Keep runtime apply ordering and partial-failure reporting in typed Rust orchestration rather than spreading it across C and CLI layers.

### Key Lessons
1. Put machine-stable identifiers in core and add human-friendly aliases only at the CLI boundary.
2. Treat requested-versus-effective runtime facts as a first-class model, not an output-format concern.
3. Schedule real-hardware verification as part of milestone execution, not just at final archival time.

### Cost Observations
- Model mix: not tracked in planning artifacts.
- Sessions: not tracked before retrospective creation.
- Notable: the highest-cost work in `v1.1` was live hardware verification, but it prevented shipping stale assumptions about apply-time behavior and metadata truthfulness.

---

## Cross-Milestone Trends

### Process Evolution

| Milestone | Sessions | Phases | Key Change |
|-----------|----------|--------|------------|
| v1.0 | not tracked | 9 | Proved the Rust CLI capture/export foundation and then backfilled verification discipline after the audit. |
| v1.1 | not tracked | 4 | Reused the shipped baseline to add DSView-backed device options with phase-by-phase verification and live hardware sign-off. |

### Cumulative Quality

| Milestone | Tests | Coverage | Zero-Dep Additions |
|-----------|-------|----------|-------------------|
| v1.0 | Phase verification, capture/export regressions, and manual DSLogic Plus UAT | n/a | 0 |
| v1.1 | 4 verification reports, 2 human UAT artifacts, and spawned CLI/runtime regressions | n/a | 0 |

### Top Lessons (Verified Across Milestones)

1. Keep the native boundary narrow and push automation-facing contracts into Rust-owned models.
2. Require durable verification artifacts alongside shipped code instead of trying to reconstruct proof at milestone closeout.
3. Treat real hardware checks as release-quality evidence whenever runtime behavior or export semantics change.
