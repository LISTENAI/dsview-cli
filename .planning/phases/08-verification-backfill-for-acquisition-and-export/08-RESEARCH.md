# Phase 8 Research: Verification Backfill for Acquisition and Export

**Date:** 2026-04-08
**Phase:** 8 - Verification Backfill for Acquisition and Export
**Objective:** Determine the minimal truthful planning scope needed to close the reopened `RUN-*` and `EXP-*` requirement gaps for milestone re-audit without overstating existing evidence.

## Planning answer in one page

Phase 8 should be planned as a **reconciliation-plus-backfill** phase, not as a broad revalidation phase.

- `RUN-01`, `RUN-02`, and `RUN-03` already have a durable requirement-level verification artifact at `.planning/phases/04-acquisition-execution/VERIFICATION.md`.
- The current milestone audit explicitly marks all three `RUN-*` requirements as **satisfied** from that file, and separately calls out the remaining problem as documentation drift: roadmap/requirements still route `RUN-*` through future Phase 8 work.
- `EXP-01`, `EXP-02`, `EXP-03`, and `EXP-04` remain genuinely **orphaned** because Phase 5 still has no `05-VERIFICATION.md`, even though validation and hardware UAT are green.

So the minimal truthful Phase 8 scope is:

1. **Phase 4:** reconcile existing verification evidence into Phase 8 planning and traceability; do **not** plan a new Phase 4 verification artifact unless review of the current file finds a concrete audit blocker not already called out.
2. **Phase 5:** create the missing durable verification layer that closes `EXP-01` through `EXP-04` from existing summaries, validation, UAT, and current code paths.
3. **Phase 8 overall:** hand off to a fresh milestone re-audit rather than editing `.planning/v1.0-MILESTONE-AUDIT.md` directly.

## Current truth the Phase 8 plans must respect

### What the audit already says

The freshest audit at `.planning/v1.0-MILESTONE-AUDIT.md` is the key planning input.

It states:

- `RUN-01`, `RUN-02`, `RUN-03` are already **satisfied** via `.planning/phases/04-acquisition-execution/VERIFICATION.md`.
- `EXP-01` through `EXP-04` are still **orphaned** only because `05-VERIFICATION.md` is missing.
- roadmap and requirements documents are stale because they still describe Phase 8 as if Phase 4 verification does not yet exist.

That means any Phase 8 plan that says "create a new Phase 4 verification artifact" is already out of date and risks failing a plan-checker for not matching repo truth.

### What `REQUIREMENTS.md` still says

`.planning/REQUIREMENTS.md` still marks all `RUN-*` and `EXP-*` rows as `Phase 8 | Pending`.

This is stale in two different ways:

- For `RUN-*`, the requirement rows are lagging behind actual verification status.
- For `EXP-*`, the rows are still correct that closure has not happened yet, but they do not yet point to the specific missing artifact and expected handoff.

Phase 8 planning must account for this difference carefully instead of treating all seven requirements as equally unresolved.

### What `ROADMAP.md` still says

`.planning/ROADMAP.md` still says the Phase 8 success criteria include:

- "Phase 4 has a `04-VERIFICATION.md` artifact..."
- "Phase 5 has a `05-VERIFICATION.md` artifact..."

The first clause is stale because the Phase 4 verification artifact already exists at `.planning/phases/04-acquisition-execution/VERIFICATION.md`.

That means Phase 8 planning should treat the roadmap's Phase 4 clause as a **reconciliation target**, not as real remaining work.

## Minimal truthful scope for Phase 8

### Phase 4 scope: reconciliation, not net-new verification

The right minimum scope for Phase 4 is:

- verify that `.planning/phases/04-acquisition-execution/VERIFICATION.md` still provides requirement-by-requirement proof for `RUN-01`, `RUN-02`, and `RUN-03`
- verify that it cites the right existing evidence sources
- reconcile Phase 8 traceability so roadmap/requirements no longer imply that Phase 4 lacks verification
- avoid creating duplicate Phase 4 verification files or pretending the requirement is still open when the audit already says it is satisfied

Why this is enough:

- The Phase 4 verification file already contains a verdict, requirement list, requirement-by-requirement assessment, hardware UAT evidence, and linkage back to the locked Phase 4 research gates.
- The audit already accepts it as durable requirement-level proof.
- There is no current audit finding saying Phase 4 needs a second verification artifact or a Phase 4 validation artifact to close `RUN-*`.

What would justify more than reconciliation:

- only if the planner finds a concrete mismatch inside `.planning/phases/04-acquisition-execution/VERIFICATION.md` itself, such as broken requirement mapping, missing final pass/fail statements, or evidence that no longer aligns with current repo reality
- none of those are currently called out by the audit

### Phase 5 scope: real backfill work

The right minimum scope for Phase 5 is:

- author `.planning/phases/05-export-artifacts/05-VERIFICATION.md`
- map `EXP-01` through `EXP-04` requirement-by-requirement to existing code, summaries, validation, and UAT evidence
- keep validation/UAT as evidence inputs, not substitutes for verification
- make sure the final evidence chain is durable enough that the audit can mark the export requirements satisfied without milestone-level inference

This is real missing work because:

- the audit explicitly says the export behavior is functionally green but formally unclosed without `05-VERIFICATION.md`
- Phase 5 already has strong validation and UAT artifacts, so the work is primarily evidence synthesis and requirement closure, not new implementation research

## Recommended split between 08-01 and 08-02

The cleanest split is **Phase 4 reconciliation first, Phase 5 verification backfill second**.

### 08-01 should cover Phase 4 reconciliation only

`08-01` should be narrow and truth-preserving.

Recommended objective:

- confirm the existing Phase 4 verification artifact is sufficient for `RUN-01` through `RUN-03`
- document that Phase 8 does **not** need to create a new Phase 4 verification file
- prepare the traceability handoff that will later let `REQUIREMENTS.md` stop calling `RUN-*` pending

Recommended outputs/targets:

- read and assess `.planning/phases/04-acquisition-execution/VERIFICATION.md`
- if needed, make only minimal clarifying updates to Phase 8 planning docs or Phase 4 traceability language, not a replacement verification artifact
- leave milestone audit untouched and hand off to rerun later

Why this deserves its own plan at all:

- the roadmap and requirements are stale in a way that could confuse later reconciliation
- Phase 7 showed that plan-checkers care about truthful scoping and explicit distinction between missing artifacts versus stale bookkeeping
- a small Phase 4-focused plan gives a formal place to record "this was already satisfied; Phase 8 only reconciles it"

### 08-02 should cover the real export verification backfill

`08-02` should do the substantive Phase 5 evidence closure.

Recommended objective:

- create `05-VERIFICATION.md` and use it to close `EXP-01` through `EXP-04`
- cite the existing automated and manual evidence sources precisely
- update requirement traceability only after the verification artifact exists

Recommended outputs/targets:

- `.planning/phases/05-export-artifacts/05-VERIFICATION.md`
- possibly a minimal reconciliation in `.planning/REQUIREMENTS.md` once the export verification artifact exists
- no milestone audit edits; rerun the audit instead

### Why not combine them differently?

Do **not** split by `RUN-*` versus `EXP-*` only in a symmetric way, because the repo is no longer symmetric:

- `RUN-*` already has the needed verification artifact
- `EXP-*` does not

A symmetric plan split would pressure `08-01` into busywork and increase the chance of inventing unnecessary Phase 4 deliverables.

## Exact evidence sources each plan should rely on

## 08-01 evidence sources for `RUN-01`, `RUN-02`, `RUN-03`

The Phase 4 reconciliation plan should rely on the following sources, in roughly this order.

### Primary requirement-closing source

- `.planning/phases/04-acquisition-execution/VERIFICATION.md`

This is already the accepted requirement-level artifact for all three `RUN-*` requirements.

### Constraint and acceptance-gate source

- `.planning/phases/04-acquisition-execution/04-RESEARCH.md`

This matters because the verification file explicitly ties success to the locked Phase 4 gates:

- clean finite capture requires start success, normal terminal end, logic packet, end marker, and successful cleanup
- immediate reuse after success is required
- representative failure plus actionable diagnostics is required
- immediate reuse after failure is required

Those gates are the standard the planner should preserve rather than weaken.

### Supporting summary source

- `.planning/phases/04-acquisition-execution/04-03-SUMMARY.md`

This provides the automated coverage matrix and the documented manual UAT path. It is support evidence, not the main closure artifact.

### Audit truth source

- `.planning/v1.0-MILESTONE-AUDIT.md`

Use this to justify that `RUN-*` is already satisfied and that the remaining Phase 8 work is traceability reconciliation.

### Traceability drift sources

- `.planning/ROADMAP.md`
- `.planning/REQUIREMENTS.md`
- `.planning/STATE.md`

These are not proof for the requirement itself; they are the stale documents that the plan must reconcile carefully later.

### Requirement-by-requirement evidence map for `RUN-*`

- `RUN-01`
  - primary proof: `.planning/phases/04-acquisition-execution/VERIFICATION.md`
  - supporting evidence inside that file: successful real finite capture command and observed `completion=clean_success`
  - research gate source: `.planning/phases/04-acquisition-execution/04-RESEARCH.md`
- `RUN-02`
  - primary proof: `.planning/phases/04-acquisition-execution/VERIFICATION.md`
  - supporting evidence inside that file: `cleanup_succeeded=true`, immediate rerun after success, immediate reuse after representative failure
  - research gate source: `.planning/phases/04-acquisition-execution/04-RESEARCH.md`
- `RUN-03`
  - primary proof: `.planning/phases/04-acquisition-execution/VERIFICATION.md`
  - supporting evidence inside that file: representative failure command exited non-zero with `code=capture_timeout` plus explicit cleanup diagnostics
  - supporting coverage context: `.planning/phases/04-acquisition-execution/04-03-SUMMARY.md`

## 08-02 evidence sources for `EXP-01`, `EXP-02`, `EXP-03`, `EXP-04`

The Phase 5 verification-backfill plan should rely on the following sources.

### Primary verification inputs

- `.planning/phases/05-export-artifacts/05-VALIDATION.md`
- `.planning/phases/05-export-artifacts/05-UAT.md`

These are the strongest existing evidence artifacts for export behavior. They already contain the detailed green evidence the audit references.

### Supporting implementation-summary sources

- `.planning/phases/05-export-artifacts/05-01-SUMMARY.md`
- `.planning/phases/05-export-artifacts/05-02-SUMMARY.md`
- `.planning/phases/05-export-artifacts/05-03-SUMMARY.md`
- `.planning/phases/05-export-artifacts/05-RESEARCH.md`

Use these to explain which code paths and validation responsibilities support each export requirement. They should not replace direct requirement assessment in `05-VERIFICATION.md`.

### Audit truth source

- `.planning/v1.0-MILESTONE-AUDIT.md`

Use this to anchor the exact gap: export is functionally green but formally orphaned because `05-VERIFICATION.md` does not exist.

### Requirement-by-requirement evidence map for `EXP-*`

- `EXP-01`
  - primary evidence: `.planning/phases/05-export-artifacts/05-VALIDATION.md` rows for 05-01-01, 05-01-02, 05-03-01, 05-03-02, and 05-03-03
  - manual confirmation: `.planning/phases/05-export-artifacts/05-UAT.md` tests 1 and 4 show successful artifact creation and immediate post-export reuse
  - summary support: `.planning/phases/05-export-artifacts/05-01-SUMMARY.md` and `.planning/phases/05-export-artifacts/05-03-SUMMARY.md`
- `EXP-02`
  - primary evidence: `.planning/phases/05-export-artifacts/05-VALIDATION.md` golden/timing rows and manual Nyquist-safe timing checks
  - manual confirmation: `.planning/phases/05-export-artifacts/05-UAT.md` test 2 confirms finite monotonic timestamps and correct channel declarations
  - research guardrail: `.planning/phases/05-export-artifacts/05-RESEARCH.md` explains why upstream VCD replay and Nyquist-safe validation are required
- `EXP-03`
  - primary evidence: `.planning/phases/05-export-artifacts/05-VALIDATION.md` metadata-sidecar rows 05-02-01, 05-02-02, 05-03-02, 05-03-03
  - manual confirmation: `.planning/phases/05-export-artifacts/05-UAT.md` tests 1, 3, and 5
  - summary support: `.planning/phases/05-export-artifacts/05-02-SUMMARY.md`
- `EXP-04`
  - primary evidence: `.planning/phases/05-export-artifacts/05-VALIDATION.md` manual metadata plausibility row plus metadata unit/integration rows
  - manual confirmation: `.planning/phases/05-export-artifacts/05-UAT.md` test 3, which checks device model, selected handle, sample rate, requested sample limit, actual sample count, enabled channels, acquisition result, and artifact paths
  - schema/source-of-truth guidance: `.planning/phases/05-export-artifacts/05-RESEARCH.md` and `.planning/phases/05-export-artifacts/05-02-SUMMARY.md`

## What Phase 7 teaches that can make a Phase 8 plan get rejected

Phase 7 established a few rules that Phase 8 should follow closely.

### 1. Do not treat summaries alone as durable proof

Phase 7 repeatedly required each requirement to be mapped to:

- a requirement-specific verification section
- at least one implementation/code path
- at least one phase summary or validation/UAT source
- a clear sufficiency judgment

For Phase 8 that means:

- `04-03-SUMMARY.md` alone is not enough for `RUN-*`
- `05-01/02/03-SUMMARY.md` alone are not enough for `EXP-*`
- the plan must create or rely on a requirement-level verification artifact, not only summaries

### 2. Do not overstate partial manual evidence

Phase 7 explicitly refused to let partial UAT stand in for full proof on `CAP-03`/`CAP-04`.

For Phase 8 that means:

- do not claim export correctness from "artifact exists" alone
- keep `EXP-02` tied to the documented timing/channel-semantic checks, including the Nyquist-safe guidance
- keep `EXP-04` tied to observed metadata facts, not only requested config
- do not imply broader hardware coverage than the existing UAT actually records

### 3. Do not edit the old milestone audit by hand

Phase 7 plans explicitly treated `.planning/v1.0-MILESTONE-AUDIT.md` as an input artifact and required a fresh `/gsd:audit-milestone` rerun instead.

Phase 8 should do the same.

A plan-checker may reject Phase 8 if it proposes hand-editing the audit to mark requirements closed.

### 4. Do not invent missing work when the repo already contains the artifact

This is the biggest Phase 8-specific trap.

The audit already says Phase 4 verification exists and passes. If `08-01` is written as if Phase 4 still lacks verification, the plan will conflict with current repo truth.

That kind of conflict is exactly the sort of stale-assumption issue a plan-checker can reject.

### 5. Reconcile related requirement rows together, but only after artifacts exist

Phase 7's second plan updated `REQUIREMENTS.md` only after both backfilled verification and validation artifacts existed.

For Phase 8 that implies:

- do not prematurely update `REQUIREMENTS.md` to mark `EXP-*` closed before `05-VERIFICATION.md` exists
- if `RUN-*` rows are reconciled, do it in a way that respects the already-existing Phase 4 verification artifact and does not mix in speculative export closure
- the final requirements reconciliation should likely happen only once both the Phase 4 reconciliation and Phase 5 verification-backfill work are complete

## Concrete plan-checker risks for Phase 8

A plan-checker is likely to reject or flag Phase 8 if the plan does any of the following:

- says Phase 4 needs a new `04-VERIFICATION.md` even though `.planning/phases/04-acquisition-execution/VERIFICATION.md` already exists and the audit already accepts it
- treats `05-VALIDATION.md` or `05-UAT.md` as if they already close `EXP-*` without a `05-VERIFICATION.md`
- updates `.planning/v1.0-MILESTONE-AUDIT.md` directly instead of requiring a rerun
- changes unrelated requirement rows while reconciling `RUN-*` and `EXP-*`
- claims broader hardware proof than the recorded export UAT actually provides
- weakens the locked Phase 4 success criteria from `.planning/phases/04-acquisition-execution/04-RESEARCH.md`
- weakens the Phase 5 timing semantics standard by dropping the Nyquist-safe/manual plausibility caveats around `EXP-02`
- collapses requested metadata values and observed metadata facts, which would overstate `EXP-04`
- fails to call out the stale assumptions still present in roadmap/requirements/state

## Concrete planning guidance for a milestone re-audit that passes cleanly

### 1. Plan Phase 8 as two different kinds of work

Use this mental model:

- `08-01` = prove that the existing Phase 4 verification artifact is already sufficient and define the reconciliation boundary
- `08-02` = create the missing Phase 5 requirement-level verification artifact

This keeps the scope honest and aligns with the audit's current findings.

### 2. Make `05-VERIFICATION.md` the core deliverable of Phase 8

The milestone audit itself says the cleanest next step is to backfill `.planning/phases/05-export-artifacts/05-VERIFICATION.md`.

That should be the main deliverable Phase 8 is planned around.

The file should:

- follow the durable verification style already used by Phase 4, Phase 6, and Phase 7 artifacts
- include `## Verdict`
- include requirement-by-requirement sections for `EXP-01` through `EXP-04`
- cite both automated and manual evidence explicitly
- distinguish validation evidence from verification judgment
- end with a final pass/fail decision for each export requirement

### 3. Keep Phase 4 work narrowly framed as evidence sufficiency review

If `08-01` needs deliverables, they should be minimal and framed like this:

- inventory the existing Phase 4 evidence sources
- confirm `.planning/phases/04-acquisition-execution/VERIFICATION.md` still matches the current requirement and audit truth
- record that Phase 8 does not need a duplicate Phase 4 verification artifact
- prepare the later traceability reconciliation for `RUN-*`

Avoid writing Phase 8 as if it must recreate the whole Phase 4 verification story from scratch.

### 4. Preserve the exact evidence hierarchy

For re-audit, the strongest truthful hierarchy is:

- verification artifact closes requirement
- validation/UAT artifacts provide underlying evidence
- summaries explain implementation and execution history
- audit rerun re-evaluates closure

Do not invert that hierarchy by asking the audit to infer closure from validation/UAT alone.

### 5. Keep stale-document reconciliation explicit

Phase 8 must call out that the following assumptions are stale and must be handled carefully:

- `.planning/ROADMAP.md` still frames Phase 8 as adding a Phase 4 verification artifact, but that artifact already exists
- `.planning/REQUIREMENTS.md` still lists `RUN-*` as `Phase 8 | Pending`, even though the audit already marks them satisfied via Phase 4 verification
- `.planning/STATE.md` still says "Execute Phase 8 ... if the fresh audit still reports reopened RUN/EXP gaps," but the fresh audit only left `EXP-*` orphaned and flagged `RUN-*` as traceability drift

This should be explicit in the plans so later reconciliation does not accidentally over-correct.

### 6. Do not over-claim what export evidence proves

To make the re-audit pass without overstating evidence:

- claim `EXP-01` from successful VCD creation and post-export reuse evidence
- claim `EXP-02` from both automated golden/timing checks and the manual plausibility checks already documented
- claim `EXP-03` from metadata sidecar generation and CLI artifact reporting evidence
- claim `EXP-04` only from observed metadata field evidence, not from requested config alone

Especially for `EXP-02` and `EXP-04`, the verification artifact should preserve the existing caveats instead of flattening them into stronger claims than the repo supports.

### 7. Hand off to re-audit instead of self-certifying closure

The Phase 8 plans should end the same way Phase 7 did:

- update traceability only after the relevant verification artifact exists
- leave `.planning/v1.0-MILESTONE-AUDIT.md` untouched
- explicitly require a fresh `/gsd:audit-milestone` rerun

That is the cleanest path to a truthful milestone re-audit pass.

## Recommended planning shape for the actual Phase 8 plans

### 08-01 recommended objective

Reconcile Phase 4 acquisition verification for audit closure by confirming the existing verification artifact already closes `RUN-01` through `RUN-03` and by scoping any remaining work to traceability drift rather than duplicate backfill.

### 08-01 recommended must-haves

- state that `.planning/phases/04-acquisition-execution/VERIFICATION.md` already exists and is the primary requirement-closing artifact
- state that `.planning/v1.0-MILESTONE-AUDIT.md` already marks `RUN-*` satisfied
- forbid hand-editing the audit
- require any later requirement-row updates to happen only as reconciliation, not as new proof generation
- forbid inventing a duplicate `04-VERIFICATION.md`

### 08-02 recommended objective

Backfill durable Phase 5 verification so `EXP-01` through `EXP-04` can be closed from persistent requirement-level evidence instead of validation/UAT alone.

### 08-02 recommended must-haves

- create `.planning/phases/05-export-artifacts/05-VERIFICATION.md`
- map each `EXP-*` requirement to specific evidence sources from summaries, validation, UAT, and current export semantics
- preserve Nyquist-safe and observed-fact caveats where they matter
- update `REQUIREMENTS.md` only after `05-VERIFICATION.md` exists
- forbid hand-editing the milestone audit; require rerun instead

## Bottom line

What you need to know to plan Phase 8 well is mostly about **scope truth**:

- Phase 4 is not missing verification anymore; it is missing reconciliation.
- Phase 5 is still missing verification and is the real closure target.
- The audit already distinguishes those two cases clearly.
- A good Phase 8 plan will mirror that asymmetry, rely on requirement-level evidence rather than summaries, preserve the existing hardware/validation caveats, and end with a fresh milestone audit rerun rather than manual audit edits.
