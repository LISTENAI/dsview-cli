---
phase: 02-device-discovery-and-session-bring-up
plan: 03
subsystem: cli
tags: [cli, diagnostics, device-list, open, errors]
requires: [02-02]
provides:
  - Scriptable Phase 2 CLI surface for list and open/close bring-up
  - Stable JSON/text output contracts and error codes for automation
  - Source-runtime selection path alongside explicit runtime library paths
affects: [cli, core, testing]
tech-stack:
  added: [clap, serde, serde_json]
  patterns: [machine-readable CLI contract, deterministic handle selector, source-runtime toggle]
key-files:
  created: []
  modified:
    - crates/dsview-cli/Cargo.toml
    - crates/dsview-cli/src/main.rs
key-decisions:
  - "Use explicit `devices list` and `devices open` commands as the narrow Phase 2 CLI surface."
  - "Use device handles as the deterministic selector contract for Phase 2 bring-up."
  - "Support both `--library` and `--use-source-runtime` so the CLI works with either external or locally built runtime artifacts."
patterns-established:
  - "CLI Contract Pattern: list/open outputs expose stable fields and stable error codes suitable for automation."
  - "Selector Pattern: Phase 2 device targeting uses explicit non-zero handles from list output."
requirements-completed: [DEV-01, DEV-02, DEV-03]
duration: 55 min
completed: 2026-04-03
---

# Phase 02 Plan 03: Expose discovery and bring-up diagnostics in the CLI Summary

**Delivered a scriptable Phase 2 CLI that can list supported DSLogic Plus devices, target a selected handle for bring-up, and report stable machine-readable diagnostics across both external and source-built runtime paths**

## Performance

- **Duration:** 55 min
- **Completed:** 2026-04-03
- **Primary verification:** `cargo test -p dsview-cli`

## Accomplishments
- Replaced the placeholder binary with a clap-based CLI exposing `devices list` and `devices open` plus explicit `--resource-dir`, `--format`, `--library`, and `--use-source-runtime` arguments.
- Added stable JSON/text contracts for device-list output (`handle`, `stable_id`, `model`, `native_name`) and open results (`selected`, `released`, `native_last_error`, `native_init_status`).
- Added stable error-code mapping for resource setup failures, missing runtime selection, source-runtime unavailability, unsupported selection, and native bring-up failures such as firmware-missing or device-busy paths.
- Verified the source-built runtime path end-to-end by running `devices list --use-source-runtime` with the real DSView resource directory and confirming DSLogic Plus detection.

## Files Created/Modified
- `crates/dsview-cli/src/main.rs` - CLI command surface, output rendering, diagnostics mapping, and focused tests.
- `crates/dsview-cli/Cargo.toml` - Adds `clap`, `serde`, and `serde_json`.

## Decisions Made
- Kept the Phase 2 CLI non-interactive and narrow so later capture-focused commands can extend the surface cleanly.
- Used handle-based selection because it is deterministic across multiple supported devices and already available from native discovery results.
- Added `--use-source-runtime` because Phase 2 now has a locally buildable runtime artifact and should not force users to locate a manual `.so` path first.

## Deviations from Plan
- The plan proposed possibly splitting commands/output into separate modules, but the Phase 2 CLI remained small enough to keep in `crates/dsview-cli/src/main.rs` without harming readability yet.

## Issues Encountered
- Real source-runtime discovery on this machine logs `LIBUSB_ERROR_ACCESS` during profile checks, which indicates USB permission limitations may still affect full bring-up even though listing succeeds.
- The no-hardware diagnostic path and the source-runtime path needed separate validation because one proves stable error shaping while the other proves the source-built runtime artifact actually works.

## User Setup Required
- For the source-built runtime path, run with `--use-source-runtime` and a valid resource directory such as `DSView/DSView/res/`.
- For real hardware open/close bring-up, ensure the current user has USB access to the DSLogic device (for example via udev rules or an equivalent permission setup).

## Next Phase Readiness
- Phase 3 can build capture-configuration commands on top of a real device-list/open lifecycle and a stable machine-readable CLI contract.
- Manual hardware validation now has a concrete user-facing command path for both list and open/close bring-up.

---
*Phase: 02-device-discovery-and-session-bring-up*
*Completed: 2026-04-03*
