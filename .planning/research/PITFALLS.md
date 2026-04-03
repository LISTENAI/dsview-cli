# Pitfalls Research: DSView CLI

**Researched:** 2026-04-03
**Scope:** Risks in building a Rust CLI on top of an existing DSView/libsigrok-style stack

## Summary

Projects like this usually fail by coupling too early to the wrong layer, underestimating native integration friction, or over-scoping to feature parity before the raw capture path is reliable. The most important early discipline is to treat hardware capture correctness as the product core and to keep the DSView dependency boundary narrow and explicit.

## Pitfall 1: Binding to the wrong native layer
**Why it happens:** The GUI app is the most visible implementation, so teams bind to application/session code instead of the underlying reusable device layer.
**Warning signs:** Qt types or GUI concepts start appearing in the Rust design; build requirements unexpectedly pull in the whole app stack.
**Prevention:** Integrate at `libsigrok4DSL` or a tiny repo-owned adapter library, not the DSView GUI layer.
**Phase to address:** Phase 1

## Pitfall 2: ABI drift from submodule updates
**Why it happens:** Native headers or structs change quietly while Rust bindings remain stale.
**Warning signs:** Breakage appears after submodule bumps; crashes happen around otherwise unchanged FFI calls.
**Prevention:** Pin the submodule revision, track binding regeneration explicitly, and add adapter smoke tests.
**Phase to address:** Phase 1-2

## Pitfall 3: Accidental dependence on the full DSView build graph
**Why it happens:** Reusing the repo can tempt the project into linking against everything transitively.
**Warning signs:** CLI build docs start requiring Qt, Python, FFTW, Boost, and GUI-related setup even for simple capture use cases.
**Prevention:** Keep a minimal native dependency boundary and document only the libraries actually needed by the CLI.
**Phase to address:** Phase 1

## Pitfall 4: Hidden session assumptions from GUI-driven flows
**Why it happens:** Some lifecycle sequencing may be orchestrated in DSView-side C++ rather than obvious library APIs.
**Warning signs:** Raw library calls compile but captures do not start, stop, or flush cleanly; behavior differs from DSView despite similar parameters.
**Prevention:** Map the real acquisition path before broad CLI design and validate the smallest end-to-end capture flow first.
**Phase to address:** Phase 1-2

## Pitfall 5: Export format that is technically valid but operationally weak
**Why it happens:** Teams optimize for easy parsing rather than preserving waveform semantics.
**Warning signs:** Downstream tools lack channel names, samplerate, timing meaning, or session context.
**Prevention:** Use VCD as the canonical waveform export and add a JSON sidecar for metadata.
**Phase to address:** Phase 3

## Pitfall 6: Trusting capture success without artifact validation
**Why it happens:** The session returns success, so the project assumes exported files are usable.
**Warning signs:** Empty files, truncated captures, missing headers, or metadata not matching the actual capture.
**Prevention:** Validate artifact existence, non-empty content, and key metadata fields after every export path.
**Phase to address:** Phase 3-4

## Pitfall 7: Over-scoping to DSView parity too early
**Why it happens:** Existing GUI breadth creates pressure to support decoders, multiple devices, triggers, and viewers immediately.
**Warning signs:** Roadmap discussions keep expanding sideways before one capture/export path is complete.
**Prevention:** Keep v1 locked to `DSLogic Plus` + capture + VCD export + metadata.
**Phase to address:** All phases

## Pitfall 8: Unsafe code leaking upward
**Why it happens:** It is tempting to call native functions directly from command handlers during early bring-up.
**Warning signs:** Pointer/resource management appears in CLI modules; error handling becomes inconsistent.
**Prevention:** Confine unsafe operations to one crate/module and expose safe domain-level abstractions upward.
**Phase to address:** Phase 2

## Pitfall 9: Hardware-only verification strategy
**Why it happens:** Real devices feel like the only true test source, so the team skips fixture-based validation.
**Warning signs:** Development slows to hardware availability; regressions are hard to catch automatically.
**Prevention:** Add golden-file and fixture-based tests for export and orchestration while keeping a small set of hardware smoke tests.
**Phase to address:** Phase 3-4

## Pitfall 10: Poor operational diagnostics
**Why it happens:** Native errors are surfaced raw or collapsed into generic failures.
**Warning signs:** Users see messages like "capture failed" without knowing whether the issue was permissions, busy device, unsupported config, or export failure.
**Prevention:** Translate native errors into actionable CLI diagnostics and stable exit behavior.
**Phase to address:** Phase 4-5

## Recommendation

The project should treat three things as non-negotiable from the start:
- narrow and explicit native boundary
- single-device scoped roadmap
- output artifacts validated as products, not side effects
