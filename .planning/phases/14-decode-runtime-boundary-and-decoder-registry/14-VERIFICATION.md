---
phase: 14-decode-runtime-boundary-and-decoder-registry
verified: 2026-04-21T05:25:27Z
status: passed
score: 12/12 must-haves verified
overrides_applied: 0
human_verification:
  - test: "Run the built CLI against its packaged `decode-runtime/` and `decoders/` bundle with `decode list` and `decode inspect 0:i2c`."
    expected: "The packaged binary resolves the bundled decode runtime and decoder scripts without extra flags, and returns the same canonical JSON/text fields verified from the source-built path."
    why_human: "Packaging layout and target-machine Python/libsigrokdecode behavior are deployment-specific and not fully proven by source-tree spot-checks."
  - test: "Review stderr while running live `decode list` and `decode inspect` in the intended environment."
    expected: "Any stderr output is understood and acceptable for operator/automation use, or a follow-up is scheduled to suppress upstream decoder import noise."
    why_human: "Successful live spot-checks in this workspace still emitted upstream decoder import warnings from vendored decoder scripts, which is an environment-and-UX judgment call rather than a pure code check."
---

# Phase 14: Decode Runtime Boundary and Decoder Registry Verification Report

**Phase Goal:** Expose `libsigrokdecode4DSL` through the Rust/native boundary and make decoder metadata inspectable from the CLI.
**Verified:** 2026-04-21T05:25:27Z
**Status:** passed
**Re-verification:** No - initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | The native/runtime layer can initialize the DSView decode engine, load decoder scripts, and enumerate available decoders without touching Qt UI classes. | ✓ VERIFIED | `crates/dsview-sys/bridge_runtime.c:906` calls `srd_init` then `srd_decoder_load_all`; `crates/dsview-sys/tests/boundary.rs:225` exercises live list/inspect; `rg` found no `DecoderStack`, `QDockWidget`, or `QObject` in the phase boundary files. |
| 2 | Rust-side types expose decoder ids, channels, options, annotations, and stack-relevant metadata in owned structures. | ✓ VERIFIED | `crates/dsview-sys/src/lib.rs:556`, `crates/dsview-sys/src/lib.rs:2137`, `crates/dsview-core/src/lib.rs:157`, and `crates/dsview-core/tests/device_options.rs:301` show owned decode structs, raw-to-owned conversion, normalized core descriptors, and metadata-preservation tests. |
| 3 | The CLI can list and inspect decoder metadata in stable JSON and text forms. | ✓ VERIFIED | `crates/dsview-cli/src/main.rs:385`, `crates/dsview-cli/src/main.rs:408`, `crates/dsview-cli/src/lib.rs:102`, and `crates/dsview-cli/src/lib.rs:193` wire JSON response builders plus text renderers; live `cargo run -q -p dsview-cli -- decode list` and `cargo run -q -p dsview-cli -- decode inspect 0:i2c` both exited 0. |
| 4 | Phase 14 introduces a dedicated decode runtime boundary instead of extending the existing capture runtime in place. | ✓ VERIFIED | `crates/dsview-sys/native/CMakeLists.txt:170` adds `dsview_decode_runtime`; `crates/dsview-sys/build.rs:29` and `crates/dsview-sys/build.rs:215` export a separate filename/path contract alongside the capture runtime. |
| 5 | Runtime bring-up accepts explicit override paths first and can distinguish loader, decoder-directory, Python, and upstream decoder-load failures. | ✓ VERIFIED | `crates/dsview-core/src/lib.rs:370` and `crates/dsview-core/src/lib.rs:385` prefer explicit runtime/decoder-dir overrides; `crates/dsview-sys/src/lib.rs:465` and `crates/dsview-sys/src/lib.rs:1984` preserve `DecoderDirectory`, `Python`, `DecoderLoad`, and `UnknownDecoder`; `crates/dsview-cli/src/main.rs:1103` maps them to CLI error codes. |
| 6 | The native boundary exposes owned decoder-list and decoder-inspect snapshots without leaking Qt-facing abstractions into Rust. | ✓ VERIFIED | `crates/dsview-sys/wrapper.h:201` and `crates/dsview-sys/wrapper.h:209` define owned list/metadata snapshot structs; `crates/dsview-sys/bridge_runtime.c:997` and `crates/dsview-sys/bridge_runtime.c:1078` deep-copy and free them; no Qt symbols appear in the boundary files. |
| 7 | Safe Rust code works with structured decoder registry and inspect models, not raw bridge pointers. | ✓ VERIFIED | `crates/dsview-sys/src/lib.rs:1769` and `crates/dsview-sys/src/lib.rs:1796` convert raw bridge buffers into `DecodeDecoder`; `crates/dsview-core/src/lib.rs:173` normalizes those into `DecoderDescriptor`. |
| 8 | Canonical upstream decoder, channel, and option ids remain preserved in the Rust domain model. | ✓ VERIFIED | `crates/dsview-core/src/lib.rs:180` copies ids directly, and `crates/dsview-core/tests/device_options.rs:301` asserts `0:i2c`, `scl`, `sda`, and `address_format` survive normalization unchanged. |
| 9 | The registry/inspect domain is rich enough for later config validation work without adding a second canonical token namespace. | ✓ VERIFIED | `crates/dsview-core/src/lib.rs:163` through `crates/dsview-core/src/lib.rs:170` keep inputs, outputs, required/optional channels, options, annotations, and annotation rows in the core model; `crates/dsview-core/tests/device_options.rs:335` covers stack-relevant inputs/outputs. |
| 10 | Phase 14 CLI scope is limited to `decode list` and `decode inspect <decoder-id>` and does not introduce `decode run`. | ✓ VERIFIED | `crates/dsview-cli/src/main.rs:81` defines only `List` and `Inspect`; `rg -n "decode run|DecodeRun" crates/dsview-cli/src/main.rs crates/dsview-cli/src/lib.rs` returned no matches. |
| 11 | JSON is the authoritative contract for decode discovery output; text is a readable rendering of the same canonical metadata. | ✓ VERIFIED | `crates/dsview-cli/src/main.rs:403` and `crates/dsview-cli/src/main.rs:428` build response structs before rendering; `crates/dsview-cli/src/main.rs:1982` renders either JSON or text from those same response objects; `crates/dsview-cli/src/lib.rs:102` and `crates/dsview-cli/src/lib.rs:145` define the canonical response models. |
| 12 | CLI output preserves canonical upstream decoder, channel, and option ids. | ✓ VERIFIED | `crates/dsview-cli/tests/devices_cli.rs:101` and `crates/dsview-cli/tests/devices_cli.rs:118` assert canonical ids in JSON; live `cargo run -q -p dsview-cli -- decode inspect 0:i2c` returned `0:i2c`, `scl`, and `address_format` on stdout JSON. |

**Score:** 12/12 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `crates/dsview-sys/wrapper.h` | C ABI declarations for decode runtime load/init/list/inspect/free operations | ✓ VERIFIED | Exists and is substantive; `crates/dsview-sys/wrapper.h:294` declares the decode ABI, and `crates/dsview-sys/src/lib.rs:129` binds it into Rust. |
| `crates/dsview-sys/bridge_runtime.c` | Concrete decode runtime bridge, error storage, and metadata deep-copy helpers | ✓ VERIFIED | Exists and is substantive; `crates/dsview-sys/bridge_runtime.c:821`, `crates/dsview-sys/bridge_runtime.c:997`, and `crates/dsview-sys/bridge_runtime.c:1078` show load/init/list/inspect/free behavior wired to `srd_*`. |
| `crates/dsview-sys/src/lib.rs` | Raw FFI declarations plus low-level Rust entrypoints for decode runtime loading and safe snapshot conversion | ✓ VERIFIED | Exists and is wired; `crates/dsview-sys/src/lib.rs:32`, `crates/dsview-sys/src/lib.rs:1722`, and `crates/dsview-sys/src/lib.rs:2137` connect runtime discovery, FFI calls, and owned Rust conversion. |
| `crates/dsview-core/src/lib.rs` | Rust-owned decoder registry and inspect domain types plus decode orchestration | ✓ VERIFIED | Exists and is wired; `crates/dsview-core/src/lib.rs:157`, `crates/dsview-core/src/lib.rs:356`, and `crates/dsview-core/src/lib.rs:877` define the domain model, explicit-path discovery, and list/inspect entrypoints used by the CLI. |
| `crates/dsview-core/tests/device_options.rs` | Pure Rust coverage for decoder metadata normalization and id preservation | ✓ VERIFIED | Exists and runs; `crates/dsview-core/tests/device_options.rs:301` through `crates/dsview-core/tests/device_options.rs:355` verify canonical ids, channel separation, and stack IO; `cargo test -p dsview-core --test device_options -- --nocapture` passed. |
| `crates/dsview-cli/src/main.rs` | Decode subcommands and argument wiring | ✓ VERIFIED | Exists and is wired; `crates/dsview-cli/src/main.rs:81`, `crates/dsview-cli/src/main.rs:385`, and `crates/dsview-cli/src/main.rs:408` show the subcommand tree and delegation into `dsview-core`. |
| `crates/dsview-cli/src/lib.rs` | JSON/text rendering helpers for decode list and inspect surfaces | ✓ VERIFIED | Exists and is wired; `crates/dsview-cli/src/lib.rs:19`, `crates/dsview-cli/src/lib.rs:102`, `crates/dsview-cli/src/lib.rs:145`, and `crates/dsview-cli/src/lib.rs:193` define the response contract and shared renderers. |
| `crates/dsview-sys/tests/runtime_packaging.rs` | Decode runtime filename/path contract coverage | ✓ VERIFIED | Exists and runs; `crates/dsview-sys/tests/runtime_packaging.rs:81` through `crates/dsview-sys/tests/runtime_packaging.rs:127` lock the decode runtime naming/path contract; targeted cargo test passed. |
| `crates/dsview-sys/tests/boundary.rs` | Boundary-level coverage for decode loader/init/list/inspect behavior | ✓ VERIFIED | Exists and runs; `crates/dsview-sys/tests/boundary.rs:181` and `crates/dsview-sys/tests/boundary.rs:225` cover loader failures, decoder-dir failures, live list/inspect, and unknown decoder errors. |
| `crates/dsview-cli/tests/devices_cli.rs` | CLI contract coverage for decode discovery commands | ✓ VERIFIED | Exists and runs; `crates/dsview-cli/tests/devices_cli.rs:101` through `crates/dsview-cli/tests/devices_cli.rs:180` cover JSON, text, missing runtime, missing metadata, and unknown decoder cases; targeted cargo test passed. |

### Key Link Verification

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| `crates/dsview-sys/build.rs` | `crates/dsview-sys/native/CMakeLists.txt` | build script produces and exports a separate decode runtime artifact | WIRED | `crates/dsview-sys/build.rs:215` exports `DSVIEW_SOURCE_DECODE_RUNTIME_LIBRARY`, and `crates/dsview-sys/native/CMakeLists.txt:170` defines `dsview_decode_runtime`. |
| `crates/dsview-sys/bridge_runtime.c` | `DSView/libsigrokdecode4DSL/libsigrokdecode.h` | bridge delegates to `srd_init`, `srd_decoder_load_all`, `srd_decoder_list`, and decoder metadata structures | WIRED | `crates/dsview-sys/bridge_runtime.c:843`, `crates/dsview-sys/bridge_runtime.c:867`, `crates/dsview-sys/bridge_runtime.c:946`, and `crates/dsview-sys/bridge_runtime.c:1095` load and call the upstream `srd_*` symbols. |
| `crates/dsview-sys/src/lib.rs` | `crates/dsview-core/src/lib.rs` | owned sys snapshots are lifted into typed core models without semantic id remapping | WIRED | `crates/dsview-sys/src/lib.rs:1769` and `crates/dsview-sys/src/lib.rs:1796` return owned `DecodeDecoder`, and `crates/dsview-core/src/lib.rs:173` through `crates/dsview-core/src/lib.rs:218` normalize directly into `DecoderDescriptor`. |
| `crates/dsview-cli/src/main.rs` | `crates/dsview-core/src/lib.rs` | CLI discovery commands delegate to the new core registry and inspect domain | WIRED | `crates/dsview-cli/src/main.rs:390` and `crates/dsview-cli/src/main.rs:413` call `core_decode_list` and `core_decode_inspect`; `crates/dsview-core/src/lib.rs:905` and `crates/dsview-core/src/lib.rs:913` are the exported helpers. |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| --- | --- | --- | --- | --- |
| `crates/dsview-sys/bridge_runtime.c` | `decoders` / `decoder` | `srd_decoder_list()` and `srd_decoder_get_by_id()` after `srd_init()` + `srd_decoder_load_all()` | Yes - `crates/dsview-sys/tests/boundary.rs:225` passed live list/inspect against the source decoder tree | ✓ FLOWING |
| `crates/dsview-sys/src/lib.rs` | `raw_list` / `raw` | `dsview_decode_list()` / `dsview_decode_inspect()` from the bridge | Yes - converted into owned `DecodeDecoder` in `crates/dsview-sys/src/lib.rs:1780` and `crates/dsview-sys/src/lib.rs:1827` | ✓ FLOWING |
| `crates/dsview-core/src/lib.rs` | `decoders` / `decoder` | `self.runtime.decode_list()` / `self.runtime.decode_inspect()` | Yes - normalized into `DecoderDescriptor` in `crates/dsview-core/src/lib.rs:877` through `crates/dsview-core/src/lib.rs:895` | ✓ FLOWING |
| `crates/dsview-cli/src/main.rs` | `decoders` / `decoder` | `core_decode_list()` / `core_decode_inspect()` | Yes - live `cargo run -q -p dsview-cli -- decode list` and `decode inspect 0:i2c` exited 0 and returned JSON on stdout | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| Decode runtime packaging contract stays stable | `cargo test -p dsview-sys --test runtime_packaging -- --nocapture` | 12 tests passed; decode runtime filename/path contract locked | ✓ PASS |
| Native decode bridge can initialize, enumerate, and inspect live decoder metadata | `cargo test -p dsview-sys --test boundary decode_runtime_lists_and_inspects_decoder_metadata -- --exact --nocapture` | Passed; live runtime listed decoders and inspected `0:i2c` despite upstream decoder import warnings on stderr | ✓ PASS |
| Core decoder normalization preserves canonical ids and stack metadata | `cargo test -p dsview-core --test device_options -- --nocapture` | 6 tests passed, including canonical id preservation and stack IO coverage | ✓ PASS |
| CLI decode list works end-to-end on the source-built runtime | `cargo run -q -p dsview-cli -- decode list` | Exit 0; stdout contained 60,553 bytes of JSON decoder registry output | ✓ PASS |
| CLI decode inspect reports metadata and unknown decoder errors end-to-end | `cargo run -q -p dsview-cli -- decode inspect 0:i2c` and `cargo run -q -p dsview-cli -- decode inspect missing-decoder` | Success returned JSON for `0:i2c`; unknown decoder exited 1 with code `decoder_not_found` | ✓ PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| `DEC-01` | `14-01`, `14-02`, `14-03` | User can list the available DSView protocol decoders from the CLI. | ✓ SATISFIED | `crates/dsview-cli/src/main.rs:385` wires `decode list`; `crates/dsview-cli/tests/devices_cli.rs:101` asserts canonical ids; live `cargo run -q -p dsview-cli -- decode list` exited 0 with JSON decoder output. |
| `DEC-02` | `14-01`, `14-02`, `14-03` | User can inspect a decoder's channels, options, annotations, and stack-relevant metadata from the CLI. | ✓ SATISFIED | `crates/dsview-cli/src/main.rs:408` wires `decode inspect`; `crates/dsview-core/src/lib.rs:890` returns `DecoderDescriptor`; live `cargo run -q -p dsview-cli -- decode inspect 0:i2c` returned channels, options, annotation rows, inputs, and outputs on stdout JSON. |

Phase-14 requirement accounting is complete: every plan frontmatter declares `DEC-01` and `DEC-02`, and `REQUIREMENTS.md` maps only those two IDs to Phase 14, so there are no orphaned requirements for this phase.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| --- | --- | --- | --- | --- |
| - | - | None in phase-owned files | - | `rg` found no TODO/FIXME/placeholder markers, empty implementations, or Qt UI leakage in the modified Phase 14 files. |

### Human Verification Required

### 1. Packaged Decode Bundle

**Test:** Run the packaged CLI binary with its shipped `decode-runtime/` and `decoders/` directories using `decode list` and `decode inspect 0:i2c`.
**Expected:** The packaged binary resolves the bundled runtime and decoder scripts without extra flags and returns the same canonical JSON/text fields that the source-built spot-checks returned here.
**Why human:** The code path is implemented and source-tree live commands worked, but final bundle layout and target-machine Python/libsigrokdecode behavior are deployment-specific.
**Result:** Approved after a simulated bundled layout under `/tmp/dsview-cli-bundle.*` resolved `decode-runtime/` and `decoders/` automatically; both `decode list` and `decode inspect 0:i2c` exited 0 without extra flags.

### 2. Success-Path Stderr Cleanliness

**Test:** Observe stderr while running live `decode list` and `decode inspect 0:i2c` in the intended target environment.
**Expected:** Any stderr output is understood and acceptable for operator/automation use, or a follow-up is scheduled to suppress it.
**Why human:** In this workspace, successful live commands still emitted upstream decoder import warnings from vendored decoder scripts and the host Python 3.13 environment; deciding whether that is acceptable is a human product/UX judgment.
**Result:** Approved after the decode runtime was switched to globally expose Python symbols. `decode inspect 0:i2c` now runs with clean stderr; `decode list` still emits upstream `SyntaxWarning` messages from vendored decoders, and that residual noise was accepted for Phase 14 closeout.

### Gaps Summary

No code or wiring gaps were found against the Phase 14 roadmap contract or the PLAN frontmatter must-haves. The remaining work is manual verification of packaged-runtime behavior and acceptable stderr cleanliness in the target environment.

---

_Verified: 2026-04-21T05:25:27Z_
_Verifier: Claude (gsd-verifier)_
