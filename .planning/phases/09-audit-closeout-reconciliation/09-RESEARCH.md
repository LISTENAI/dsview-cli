---
phase: 09-audit-closeout-reconciliation
artifact: research
created: 2026-04-08
status: complete
scope: planning
---

# Phase 9 Research: Audit Closeout Reconciliation

## Planning answer

Phase 9 is a documentation-and-traceability reconciliation phase, not a product-code phase.

The latest milestone audit says the shipped DSLogic Plus CLI flow is already functionally green end to end, and the remaining blockers are verifier-grade evidence gaps or stale planning metadata rather than missing capture/export behavior. The cleanest Phase 9 plan should therefore minimize code churn, avoid touching `DSView/`, avoid hand-editing the audit output itself, and focus on creating or reconciling only the durable planning artifacts that the next `/gsd:audit-milestone` run will actually consume.

## What changed since earlier assumptions

Current repo truth matters for Phase 9 scoping:

- Phase 8 is already complete enough to close the export orphan gap because `.planning/phases/05-export-artifacts/05-VERIFICATION.md` exists and `EXP-01` through `EXP-04` already trace to it in `.planning/REQUIREMENTS.md`.
- The remaining formal blockers called out by the audit are:
  - missing `.planning/phases/01-native-integration-foundation/01-VERIFICATION.md`
  - stale `CLI-01` through `CLI-03` rows in `.planning/REQUIREMENTS.md`
  - conflicting Phase 6 closeout records, especially `.planning/phases/06-cli-productization/06-03-SUMMARY.md` versus `.planning/phases/06-cli-productization/06-VALIDATION.md` and `.planning/phases/06-cli-productization/06-VERIFICATION.md`
  - optional roadmap and validation metadata drift
- The audit explicitly says the remaining blockers are documentation and verification-chain issues, not missing functional flow behavior.

That means Phase 9 should plan around evidence synthesis and bookkeeping truthfulness, not around implementation changes in `crates/` unless the planner discovers a real contradiction in current code paths.

## 1. Minimal artifacts and file updates needed for a passing re-audit

These are the minimum durable changes that appear necessary for `/gsd:audit-milestone` to rerun cleanly.

### Required artifacts

1. Create `.planning/phases/01-native-integration-foundation/01-VERIFICATION.md`.

This is the one missing verifier-grade artifact the audit still treats as a blocker for phase coverage and the `01 -> 02` integration chain. It should follow the same durable-verification style used by `.planning/phases/02-device-discovery-and-session-bring-up/02-VERIFICATION.md`, `.planning/phases/05-export-artifacts/05-VERIFICATION.md`, and `.planning/phases/06-cli-productization/06-VERIFICATION.md`:

- restate Phase 1 goal and requirement target(s)
- tie current code paths to the original Phase 1 summaries
- explain what the Phase 1 smoke/build evidence proves and what it intentionally does not prove
- close only the bounded claim supported by the evidence

2. Update `.planning/REQUIREMENTS.md` so `CLI-01`, `CLI-02`, and `CLI-03` no longer say `Phase 9 | Pending`.

The audit already treats those requirements as verification-passed via `.planning/phases/06-cli-productization/06-VERIFICATION.md`. The traceability table is the stale part. Minimal fix:

- remap `CLI-01`, `CLI-02`, and `CLI-03` to Phase 6 evidence
- point at `.planning/phases/06-cli-productization/06-VERIFICATION.md`
- optionally mention `.planning/phases/06-cli-productization/06-VALIDATION.md` if the table pattern wants validation listed too
- preserve the existing closeout pattern used for `RUN-*` and `EXP-*`: close through durable verification, then instruct a fresh `/gsd:audit-milestone` rerun

3. Reconcile `.planning/phases/06-cli-productization/06-03-SUMMARY.md` so it no longer says the manual shell-workflow UAT remains open.

This is the main conflicting-closeout document called out by the audit. Because `.planning/phases/06-cli-productization/06-VALIDATION.md` and `.planning/phases/06-cli-productization/06-VERIFICATION.md` already record the manual gate as passed, the summary should be updated to truthfully reflect final closeout state rather than leave a stale gate statement in place.

The minimum edit is to replace the open-gate language with wording that distinguishes:

- what `06-03` completed at execution time
- what was later closed by the successful 2026-04-08 DSLogic Plus manual UAT
- that Phase 6 is now complete

### Probably required bookkeeping updates

4. Update `.planning/ROADMAP.md` only if Phase 9 execution changes what it claims about remaining gaps.

The audit lists roadmap drift as tech debt, not as a hard blocker, but there are two roadmap details that can confuse the next audit:

- top-level Phase 8 checkbox still unchecked despite Phase 8 detail/progress saying complete
- execution order text still ends at Phase 6

Those are probably optional for audit pass, but a minimal truthful update is low-cost and reduces the chance of another “process drift” flag.

5. Update `.planning/phases/06-cli-productization/06-VALIDATION.md` frontmatter if the audit logic reads validation metadata literally.

The audit labels this as tech debt or optional metadata drift rather than the primary blocker, but the frontmatter is obviously stale:

- `status: draft`
- `nyquist_compliant: n/a`
- `wave_0_complete: false`

Given the body already says the manual gate passed and the approval is green, reconciling frontmatter is a strong candidate for the same plan that fixes the stale `06-03` summary. If skipped, the audit may still pass, but the repo would continue to carry an unnecessary contradiction.

### Not needed for closure

- Do not edit `.planning/v1.0-MILESTONE-AUDIT.md` by hand. Multiple prior closeout phases already established the pattern that audit outputs are inputs only and closure must happen by durable source artifacts plus a fresh rerun.
- Do not plan product changes in `crates/` unless new evidence disproves the current verification chain.
- Do not create a new Phase 6 verification file; `.planning/phases/06-cli-productization/06-VERIFICATION.md` already exists and is accepted by the audit.
- Do not modify `DSView/`; project instructions treat it as read-only upstream dependency code.

## 2. Should Phase 9 stay split as `09-01` and `09-02`?

Mostly yes, but the partition should be described more truthfully than the current shorthand.

The current roadmap split is:

- `09-01`: backfill verification evidence for Phase 1 native integration foundation
- `09-02`: reconcile Phase 6 validation and closeout artifacts for re-audit

That is close to correct, but the actual work naturally falls into these two buckets:

### Recommended partition

- `09-01`: Phase 1 verifier-grade evidence backfill plus DEV-01 traceability reconciliation
- `09-02`: Phase 6 closeout reconciliation plus CLI traceability/metadata cleanup

Why this is more truthful:

- The audit says Phase 1 lacks `01-VERIFICATION.md`, but Phase 9 also formally lists `DEV-01` in its requirement scope. Since Roadmap Phase 1 currently lists `DEV-01` and the audit flags the missing verifier-grade closure for that foundation, `09-01` should explicitly include whatever `REQUIREMENTS.md` or roadmap row updates are necessary to reconcile the Phase 1/Phase 2 relationship for `DEV-01`.
- The audit’s three requirement-level failures are `CLI-01`, `CLI-02`, and `CLI-03`, all of which are Phase 6 evidence / requirements-table problems, so they belong with the Phase 6 reconciliation plan rather than being spread across both plans.

### Why not split differently?

A three-plan split is probably unnecessary.

You could imagine:

- one plan for `01-VERIFICATION.md`
- one plan for `CLI-01..03` requirement-table updates
- one plan for Phase 6 summary/validation metadata cleanup

But the CLI traceability rows and Phase 6 closeout-doc conflict are the same evidence chain. Separating them would create artificial handoff overhead and a chance for partial drift.

### Why not collapse to one plan?

One plan would be less truthful because the two tasks use different primary evidence:

- Phase 1 relies on early foundation summaries, `dsview-sys` boundary code, and smoke/build evidence
- Phase 6 relies on validation/verification/manual UAT evidence and final CLI workflow docs

They are independent enough that keeping two plans preserves clear acceptance criteria and simplifies checker loops.

## 3. Primary evidence each plan should rely on

The next planner should treat these artifacts as the authoritative evidence sources, in order of importance.

### Plan `09-01`: Phase 1 verification backfill

Primary evidence sources:

- `.planning/phases/01-native-integration-foundation/01-01-SUMMARY.md`
- `.planning/phases/01-native-integration-foundation/01-02-SUMMARY.md`
- `.planning/phases/01-native-integration-foundation/01-03-SUMMARY.md`
- `.planning/ROADMAP.md` Phase 1 goal, success criteria, and requirement mapping
- `crates/dsview-sys/build.rs`
- `crates/dsview-sys/src/lib.rs`
- `crates/dsview-sys/wrapper.h`
- `crates/dsview-sys/smoke_version_shim.c`
- `Cargo.toml` workspace structure, plus `crates/dsview-cli/Cargo.toml`, `crates/dsview-core/Cargo.toml`, `crates/dsview-sys/Cargo.toml`

Supporting evidence patterns to copy from later backfills:

- `.planning/phases/02-device-discovery-and-session-bring-up/02-VERIFICATION.md`
- `.planning/phases/05-export-artifacts/05-VERIFICATION.md`

What `09-01` should claim, and no more:

- the Rust workspace and crate separation exist and still express the intended CLI -> core -> sys boundary
- the chosen native boundary is intentionally narrow and rooted in DSView/libsigrok public headers rather than GUI reuse
- the build/smoke path gives durable evidence for boundary viability and failure modes
- Phase 1 proves native-foundation readiness, not real device workflow behavior

Important caution:

Do not over-claim that Phase 1 independently closes all of `DEV-01` in user-facing terms. The audit language suggests Phase 1 matters for the foundation-to-bring-up chain, while actual `devices list` behavior is already closed through Phase 2 verification. The planner should decide whether `DEV-01` remains mapped to Phase 7/Phase 2 traceability in `REQUIREMENTS.md` while `01-VERIFICATION.md` closes phase coverage only. That is likely the least confusing interpretation.

### Plan `09-02`: Phase 6 closeout reconciliation

Primary evidence sources:

- `.planning/phases/06-cli-productization/06-VERIFICATION.md`
- `.planning/phases/06-cli-productization/06-VALIDATION.md`
- `.planning/phases/06-cli-productization/06-01-SUMMARY.md`
- `.planning/phases/06-cli-productization/06-02-SUMMARY.md`
- `.planning/phases/06-cli-productization/06-03-SUMMARY.md`
- `.planning/REQUIREMENTS.md` current stale `CLI-01..03` rows
- `.planning/ROADMAP.md` Phase 6 and Phase 9 sections
- `.planning/STATE.md` if phase-completion bookkeeping needs alignment during execution

Code/test evidence that underpins Phase 6 and can be cited secondarily:

- `crates/dsview-cli/src/main.rs`
- `crates/dsview-cli/tests/capture_cli.rs`
- `crates/dsview-core/tests/export_artifacts.rs`

What `09-02` should use them to prove:

- Phase 6 manual DSLogic Plus shell-workflow UAT passed on 2026-04-08 and is already recorded as green in validation and verification
- `CLI-01`, `CLI-02`, and `CLI-03` are already satisfied by durable Phase 6 evidence
- stale planning rows and summary wording, not shipped behavior, are the remaining gap
- any validation-frontmatter fix is reconciliation of already-green status, not new product validation

## 4. Grep-verifiable acceptance criteria for each plan

The follow-on planner/checker loop will be cleaner if the criteria are literal artifact-presence and text-traceability checks rather than subjective “looks reconciled” language.

### Plan `09-01` acceptance criteria

1. `.planning/phases/01-native-integration-foundation/01-VERIFICATION.md` exists.

Possible checker:

- `rg -n "^# Phase 01 Verification|^\*\*Status: Achieved / passed\.\*\*|Requirements:" .planning/phases/01-native-integration-foundation/01-VERIFICATION.md`

2. `01-VERIFICATION.md` explicitly references the three original Phase 1 summaries.

Possible checker:

- `rg -n "01-01-SUMMARY|01-02-SUMMARY|01-03-SUMMARY" .planning/phases/01-native-integration-foundation/01-VERIFICATION.md`

3. `01-VERIFICATION.md` explicitly references the narrow native-boundary evidence in `dsview-sys`.

Possible checker:

- `rg -n "crates/dsview-sys/(build\.rs|src/lib\.rs|wrapper\.h|smoke_version_shim\.c)" .planning/phases/01-native-integration-foundation/01-VERIFICATION.md`

4. `01-VERIFICATION.md` includes bounded wording that Phase 1 proves foundation viability rather than full device workflow behavior.

Possible checker:

- `rg -n "does not|doesn't|not .*device discovery|not .*capture|foundation|boundary" .planning/phases/01-native-integration-foundation/01-VERIFICATION.md`

5. If Plan `09-01` updates roadmap/state bookkeeping, Phase 9 summary should record that the audit file was left untouched.

Possible checker:

- `rg -n "audit.*untouched|fresh `/gsd:audit-milestone` rerun|do not edit .*MILESTONE-AUDIT" .planning/phases/09-audit-closeout-reconciliation/09-01-SUMMARY.md`

### Plan `09-02` acceptance criteria

1. `.planning/REQUIREMENTS.md` no longer marks `CLI-01`, `CLI-02`, or `CLI-03` as `Phase 9 | Pending`.

Possible checker:

- `rg -n "CLI-0[123].*Phase 9.*Pending" .planning/REQUIREMENTS.md`

This should return no matches.

2. `.planning/REQUIREMENTS.md` points `CLI-01`, `CLI-02`, and `CLI-03` at Phase 6 closure evidence.

Possible checker:

- `rg -n "CLI-01|CLI-02|CLI-03|06-VERIFICATION|06-VALIDATION|Phase 6" .planning/REQUIREMENTS.md`

3. `.planning/phases/06-cli-productization/06-03-SUMMARY.md` no longer says the manual shell-workflow UAT remains open or is still required.

Possible checker:

- `rg -n "still required|remaining open Phase 6 gate|manual DSLogic Plus shell-workflow UAT remains open" .planning/phases/06-cli-productization/06-03-SUMMARY.md`

This should return no matches.

4. `.planning/phases/06-cli-productization/06-03-SUMMARY.md` explicitly records that the manual UAT passed or that Phase 6 is complete.

Possible checker:

- `rg -n "manual .* passed|Phase 6 is complete|shell-workflow UAT .* passed|Phase 06 complete" .planning/phases/06-cli-productization/06-03-SUMMARY.md`

5. Optional but recommended: `.planning/phases/06-cli-productization/06-VALIDATION.md` frontmatter no longer contradicts the body.

Possible checker:

- `rg -n "^status: (draft|complete)|^wave_0_complete: (true|false)|^nyquist_compliant:" .planning/phases/06-cli-productization/06-VALIDATION.md`

Desired end state should at least remove `status: draft` and `wave_0_complete: false` if reconciliation is included.

6. Optional but recommended: roadmap checklist drift is gone.

Possible checkers:

- `rg -n "\[ \] \*\*Phase 8" .planning/ROADMAP.md`
- `rg -n "Execution Order:|1 -> 2 -> 3 -> 4 -> 5 -> 6$" .planning/ROADMAP.md`

The first should return no unchecked Phase 8 line; the second should no longer imply the roadmap ends at Phase 6.

## 5. Tech debt that should stay out of scope

Even if mentioned in the audit, these items should remain out of scope unless the user explicitly broadens Phase 9.

### Definitely out of scope

- Any edits to `.planning/v1.0-MILESTONE-AUDIT.md`
- Any changes inside `DSView/`
- Any expansion beyond `DSLogic Plus`
- Any v2 requirements such as decode support, additional export formats, presets, or broader device support
- Any new product implementation work in `crates/` just to make docs look more complete

### Likely out of scope but acceptable as optional cleanup

- Full Nyquist metadata normalization across older phases (`01`, `02`, `03`, `04`, `07`, `08`)
- Adding missing `VALIDATION.md` artifacts for phases that currently pass without them, such as Phase 1 or Phase 4
- Broad roadmap polish beyond the specific stale checklist/execution-order lines the audit already identified
- Reworking historical summaries beyond the minimum statements needed to remove direct contradictions

### Residual risk that should be documented, not fixed in Phase 9

- Source-runtime proof still depends on local native prerequisites and USB permissions
- Historical planning artifacts may preserve execution-time state that later required reconciliation; Phase 9 should correct contradictions, not rewrite history wholesale

## Practical planning guidance

If you want the subsequent planner/checker loop to stay precise, Phase 9 should be framed as “close only the minimum durable artifacts that the audit actually reads.”

That means:

- `09-01` should create one new file: `.planning/phases/01-native-integration-foundation/01-VERIFICATION.md`, plus only the minimal bookkeeping updates needed to record that backfill truthfully.
- `09-02` should update the stale `CLI-*` traceability rows and reconcile the conflicting Phase 6 closeout docs, with validation frontmatter and roadmap drift treated as same-plan optional cleanup if they are cheap and directly reduce audit ambiguity.
- Both plans should preserve the established closeout rule: audit outputs are regenerated, not hand-edited.

## Direct answers

### Q1. What exact artifacts and file updates are minimally necessary to make a rerun of `/gsd:audit-milestone` pass?

Minimum likely set:

- create `.planning/phases/01-native-integration-foundation/01-VERIFICATION.md`
- update `CLI-01`, `CLI-02`, and `CLI-03` rows in `.planning/REQUIREMENTS.md` to close through Phase 6 evidence
- update `.planning/phases/06-cli-productization/06-03-SUMMARY.md` so it no longer says the manual UAT is still open

Recommended same-pass cleanup:

- reconcile stale frontmatter in `.planning/phases/06-cli-productization/06-VALIDATION.md`
- fix Phase 8 / execution-order roadmap drift in `.planning/ROADMAP.md`

### Q2. Should Phase 9 stay split exactly as roadmap suggests?

Yes, keep two plans, but describe them more truthfully:

- `09-01`: Phase 1 verification backfill plus any minimal Phase 1/DEV-01 traceability reconciliation
- `09-02`: Phase 6 closeout reconciliation plus `CLI-01..03` traceability and optional metadata drift cleanup

### Q3. What existing summaries, tests, validation docs, and code paths should each plan rely on?

- `09-01`: rely on `01-01-SUMMARY.md`, `01-02-SUMMARY.md`, `01-03-SUMMARY.md`, `crates/dsview-sys/build.rs`, `crates/dsview-sys/src/lib.rs`, `crates/dsview-sys/wrapper.h`, `crates/dsview-sys/smoke_version_shim.c`, and workspace manifests
- `09-02`: rely on `06-VERIFICATION.md`, `06-VALIDATION.md`, `06-01-SUMMARY.md`, `06-02-SUMMARY.md`, `06-03-SUMMARY.md`, plus the underlying `crates/dsview-cli/src/main.rs`, `crates/dsview-cli/tests/capture_cli.rs`, and `crates/dsview-core/tests/export_artifacts.rs`

### Q4. What acceptance criteria can be made grep-verifiable?

- `09-01`: require existence of `01-VERIFICATION.md`, references to all three Phase 1 summaries, references to key `dsview-sys` boundary files, and bounded wording about what Phase 1 does not prove
- `09-02`: require no `CLI-01..03` `Phase 9 | Pending` rows in `.planning/REQUIREMENTS.md`, positive references to `06-VERIFICATION.md`, no “manual UAT still open” wording in `06-03-SUMMARY.md`, and ideally no contradictory draft/false frontmatter in `06-VALIDATION.md`

### Q5. Which tech-debt items should remain out of scope?

Out of scope:

- editing the milestone audit by hand
- any code changes in `crates/` unless evidence proves docs are lying
- any `DSView/` changes
- broader historical validation/Nyquist cleanup across old phases
- v2 features or broader runtime/product hardening not required for audit closeout
