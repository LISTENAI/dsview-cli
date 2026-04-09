# Milestones

## v1.0 MVP (Shipped: 2026-04-09)

**Phases completed:** 9 phases, 24 plans, 44 tasks

**Key accomplishments:**

- Rust workspace skeleton with separated CLI, core orchestration, and sys boundary crates
- Documented and encoded the lowest-risk Phase 1 native boundary around the public `libsigrok4DSL` frontend surface
- Added a scoped smoke path that validates the `dsview-sys` boundary through normal Cargo tests without requiring hardware or GUI launch
- Extended the native boundary from a proof-only symbol into a real DSView-backed bring-up bridge and added a source-built shared runtime so Phase 2 can run without a preexisting external `.so` artifact
- Built the safe Rust orchestration layer that validates DSLogic Plus resources, filters supported devices explicitly, and guarantees release behavior around bring-up sessions
- Delivered a scriptable Phase 2 CLI that can list supported DSLogic Plus devices, target a selected handle for bring-up, and report stable machine-readable diagnostics across both external and source-built runtime paths
- Built the Rust-side capture configuration model so DSLogic Plus requests can be validated and normalized before any acquisition work begins
- Extended the Phase 2 runtime bridge so an opened DSLogic Plus session can expose capture capabilities and accept only validated sample rate, sample limit, and channel-enable settings
- Added focused Phase 3 validation coverage so DSLogic Plus configuration rules are exercised without hardware and the source-runtime integration path keeps a live regression check
- Added the first real DSLogic Plus capture command by wiring DSView acquisition callbacks through the sys/core/cli seam and enforcing a finite-run clean-success contract
- Hardened Phase 4 acquisition cleanup and diagnostics so failed or timed-out runs attempt deterministic teardown, surface stable failure categories, and keep the worktree build path usable for verification
- Completed the Phase 4 validation layer by adding hardware-free acquisition coverage across core, sys, and CLI seams, then documenting a verifier-ready preflight-first manual UAT path for DSLogic Plus.
- Upstream VCD replay now runs through `dsview-sys` before artifact publication, while `dsview-core` only exports clean-success captures and surfaces stable export facts for later metadata work.
- Versioned capture metadata now ships alongside the exported VCD, and the CLI reports both artifact paths with stable export-stage failure codes for automation.
- Phase 05-03 now has verifier-ready automated validation coverage across sys, core, and CLI layers, and the DSLogic Plus hardware UAT rerun has passed with sane real-hardware VCD timestamps, metadata plausibility, and immediate device reuse.
- Phase 06-01 is complete: the final `capture` command now exposes a locked JSON-vs-text result contract, and text-mode success reports the produced VCD plus metadata artifacts directly instead of only printing `ok`.
- Phase 06-02 is complete: the implementation already landed in commit `f93d35f`, and the same acceptance criteria were re-verified at current HEAD without needing to disturb later overlapping Phase 6 edits.
- Phase 06-03 completed the automated closeout work for the final capture-to-export command surface, and Phase 6 is now complete because the later 2026-04-08 DSLogic Plus manual shell-workflow UAT also passed.
- Phase 2 device discovery and bring-up now has durable verification and validation artifacts tied directly to shipped code, original summaries, and recorded source-runtime hardware evidence
- Backfilled durable CAP-01 through CAP-04 proof with a truthful Phase 3 verification record, minimal validation rationale, and final Phase 7 requirement reconciliation
- Phase 8 now has an explicit record that `RUN-01` through `RUN-03` were already closed by the existing Phase 4 verification artifact, so this plan only reconciles traceability instead of inventing duplicate verification work.
- Phase 5 export behavior now has a durable requirement-level verification artifact, and the EXP traceability rows point at that closure evidence for the next milestone re-audit.
- Phase 1 now has a durable verification artifact that closes the native integration foundation without over-claiming later user-facing CLI behavior
- Phase 9 closeout now traces the shipped CLI workflow back to Phase 6 evidence, aligns the stale Phase 6 closeout records, and removes the audit-listed roadmap drift without touching the audit file itself.

---
