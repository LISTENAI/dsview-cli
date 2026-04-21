---
phase: 16-offline-decode-execution
verified: 2026-04-21T12:09:05Z
status: passed
score: 10/10 must-haves verified
overrides_applied: 0
---

# Phase 16: Offline Decode Execution Verification Report

**Phase Goal:** Run DSView protocol decoders against saved logic artifacts from the CLI, including stacked decoder flows.
**Verified:** 2026-04-21T12:09:05Z
**Status:** passed
**Re-verification:** No - initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | CLI can execute a decode run against a saved logic-data artifact without requiring GUI components. | ✓ VERIFIED | `decode run` is wired in `crates/dsview-cli/src/main.rs:93`, `crates/dsview-cli/src/main.rs:150`, and `crates/dsview-cli/src/main.rs:509`; successful offline run coverage exists in `crates/dsview-cli/tests/decode_cli.rs:204` and passed via `cargo test -p dsview-cli --test decode_cli decode_run_executes_valid_offline_decode_config -- --exact --nocapture`. |
| 2 | Runtime execution correctly feeds sample data and absolute sample ranges into the decode engine. | ✓ VERIFIED | `run_offline_decode()` advances one absolute cursor across chunk sends in `crates/dsview-core/src/lib.rs:658`; `session_send_logic_chunk()` enforces progression and computes `abs_end_sample` in `crates/dsview-sys/src/lib.rs:2113`; the C bridge forwards those ranges to `srd_session_send()` in `crates/dsview-sys/bridge_runtime.c:1964`; focused cursor tests passed in `crates/dsview-core/tests/decode_execute.rs:141` and `crates/dsview-sys/tests/boundary.rs:529`. |
| 3 | Decoder stacks work when upstream decoder output is required by downstream decoders. | ✓ VERIFIED | Core builds a root-plus-stack session in `crates/dsview-core/src/lib.rs:682`; the native bridge linearly stacks decoder instances with `srd_inst_stack()` in `crates/dsview-sys/bridge_runtime.c:1886`; real stacked forwarding is exercised in `crates/dsview-sys/tests/boundary.rs:560`, which passed via `cargo test -p dsview-sys --test boundary stacked_decoder_python_output_flows_linearly -- --exact --nocapture`. |
| 4 | Phase 16 defines a canonical raw logic input contract for offline decode instead of treating VCD as the primary execution input. | ✓ VERIFIED | The canonical execution contract is `OfflineDecodeDataFormat` plus `OfflineDecodeInput` in `crates/dsview-core/src/lib.rs:375` and `crates/dsview-core/src/lib.rs:381`; the CLI consumes JSON offline input artifacts through `load_offline_decode_input()` in `crates/dsview-cli/src/main.rs:639` rather than any VCD parsing path. |
| 5 | The sys boundary exposes the minimal session/start/send/end primitives needed for offline decode execution over the existing decode runtime. | ✓ VERIFIED | The C ABI declares session lifecycle and send APIs in `crates/dsview-sys/wrapper.h:367`; the bridge implements `new`, samplerate metadata, stack build, start, send, end, capture drain, and destroy in `crates/dsview-sys/bridge_runtime.c:1796`; the safe Rust owner is `DecodeExecutionSession` in `crates/dsview-sys/src/lib.rs:1999`. |
| 6 | Sample input shape is explicit enough to validate split-logic versus cross-logic payloads before execution begins. | ✓ VERIFIED | `OfflineDecodeInput::validate_basic_shape()` rejects missing samplerate, empty samples, missing unitsize/channel count, byte misalignment, and invalid packet lengths in `crates/dsview-core/src/lib.rs:431`; core unit tests covering split and cross logic validation passed in `cargo test -p dsview-core --lib -- --nocapture`. |
| 7 | Phase 16 executes validated decode configs strictly linearly: root decoder consumes logic samples and each stacked decoder consumes the previous decoder's output. | ✓ VERIFIED | Core projects root bindings only in `crates/dsview-core/src/lib.rs:734` and strips channel bindings from stacked entries in `crates/dsview-core/src/lib.rs:751`; the C bridge rejects non-root bindings in `crates/dsview-sys/bridge_runtime.c:1535`; `offline_decode_root_only_binds_logic_channels` in `crates/dsview-core/tests/decode_execute.rs:199` passed. |
| 8 | Chunked execution preserves absolute sample numbering regardless of packet-aware or fixed-size chunking strategy. | ✓ VERIFIED | Packet-aware chunk selection and fixed-size fallback are both used by `run_offline_decode()` in `crates/dsview-core/src/lib.rs:703`; consumed sample count becomes the next absolute start in `crates/dsview-core/src/lib.rs:711`; `offline_decode_uses_absolute_sample_cursor_across_chunks` and `offline_decode_prefers_packet_boundaries_when_available` in `crates/dsview-core/tests/decode_execute.rs:141` and `crates/dsview-core/tests/decode_execute.rs:173` both passed. |
| 9 | Critical runtime failures fail the entire decode run from the user-facing workflow perspective, while internal diagnostics remain internal-only. | ✓ VERIFIED | Runtime failures surface as `OfflineDecodeRunError::Runtime` in `crates/dsview-core/src/lib.rs:534`; retained annotations stay internal via `retained_annotations()` in `crates/dsview-core/src/lib.rs:555`; failure coverage passed in `crates/dsview-core/tests/decode_execute.rs:221`, `crates/dsview-core/tests/decode_execute.rs:240`, and `crates/dsview-core/tests/decode_execute.rs:259`; CLI failure coverage passed in `crates/dsview-cli/tests/decode_cli.rs:313`; no `partial_success` symbol exists in the Phase 16 execution path. |
| 10 | Users can run offline decode from the CLI using validated configs plus the canonical raw input contract, not VCD text as the primary execution source. | ✓ VERIFIED | `run_decode_run()` validates config, loads `OfflineDecodeInput`, then delegates to `core_run_offline_decode()` in `crates/dsview-cli/src/main.rs:509` and `crates/dsview-cli/src/main.rs:525`; the command-line contract explicitly asks for `--config` and `--input` JSON files in `crates/dsview-cli/src/main.rs:150`; minimal execution rendering is provided by `DecodeRunResponse` and `render_decode_run_text()` in `crates/dsview-cli/src/lib.rs:58` and `crates/dsview-cli/src/lib.rs:384`. |

**Score:** 10/10 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `crates/dsview-core/src/lib.rs` | Offline decode input contract and executor | ✓ VERIFIED | Defines `OfflineDecodeInput`, validates shape, computes sample counts, builds linear sessions, and runs chunked execution in `crates/dsview-core/src/lib.rs:375` and `crates/dsview-core/src/lib.rs:658`. |
| `crates/dsview-sys/src/lib.rs` | Safe Rust decode-session wrappers and chunk send path | ✓ VERIFIED | Owns `DecodeExecutionSession`, packet validation, cursor enforcement, and raw chunk forwarding in `crates/dsview-sys/src/lib.rs:1999` and `crates/dsview-sys/src/lib.rs:2113`. |
| `crates/dsview-sys/tests/boundary.rs` | Boundary coverage for malformed input, cursor progression, and stacked forwarding | ✓ VERIFIED | Covers empty bytes, misaligned packet lengths, absolute progression, and real stacked `OUTPUT_PYTHON` flow in `crates/dsview-sys/tests/boundary.rs:487`. |
| `crates/dsview-core/tests/decode_execute.rs` | Execution coverage for chunking, stack orchestration, and hard-fail behavior | ✓ VERIFIED | Covers absolute cursoring, packet preference, root-only channel binding, send/end failures, and retained diagnostics in `crates/dsview-core/tests/decode_execute.rs:141`. |
| `crates/dsview-cli/src/main.rs` | `decode run` CLI command wiring | ✓ VERIFIED | Exposes `DecodeCommand::Run`, loads config and offline input artifacts, then delegates to the core executor in `crates/dsview-cli/src/main.rs:97` and `crates/dsview-cli/src/main.rs:509`. |
| `crates/dsview-cli/src/lib.rs` | Minimal Phase 16 execution response/rendering | ✓ VERIFIED | Builds and renders `DecodeRunResponse` with binary success semantics in `crates/dsview-cli/src/lib.rs:58`, `crates/dsview-cli/src/lib.rs:230`, and `crates/dsview-cli/src/lib.rs:384`. |
| `crates/dsview-cli/tests/decode_cli.rs` | CLI success and failure regressions | ✓ VERIFIED | Exercises valid execution, invalid packet-length input, and runtime failure handling in `crates/dsview-cli/tests/decode_cli.rs:204`. |
| `crates/dsview-sys/wrapper.h` | Native decode session ABI surface | ✓ VERIFIED | Declares the decode-session lifecycle, send, and annotation drain functions used by the Rust wrapper in `crates/dsview-sys/wrapper.h:367`. |
| `crates/dsview-sys/bridge_runtime.c` | libsigrokdecode-backed native session implementation | ✓ VERIFIED | Creates sessions, sets samplerate metadata, builds stacks, sends chunks, drains callbacks, and destroys sessions in `crates/dsview-sys/bridge_runtime.c:1796`. |

### Key Link Verification

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| `crates/dsview-core/src/lib.rs` | `crates/dsview-sys/src/lib.rs` | Core execution orchestration drives the sys session bridge with validated config and chunked offline input | ✓ WIRED | `run_offline_decode()` calls `DecodeExecutionSession` methods and `session_send_logic_chunk()` in `crates/dsview-core/src/lib.rs:601` and `crates/dsview-core/src/lib.rs:658`. |
| `crates/dsview-cli/src/main.rs` | `crates/dsview-core/src/lib.rs` | CLI offline execution delegates to the core decode executor using validated config plus raw artifact input | ✓ WIRED | `run_decode_run()` loads config/input and invokes `core_run_offline_decode()` in `crates/dsview-cli/src/main.rs:525`. |
| `crates/dsview-sys/bridge_runtime.c` | `libsigrokdecode4DSL` runtime | Native session bridge wires Rust execution into `srd_session_*` and `srd_inst_*` APIs | ✓ WIRED | The bridge loads and calls `srd_session_new`, `srd_session_metadata_set`, `srd_inst_new`, `srd_inst_channel_set_all`, `srd_inst_stack`, `srd_session_start`, `srd_session_send`, `srd_session_end`, and `srd_session_destroy` in `crates/dsview-sys/bridge_runtime.c:1822`, `crates/dsview-sys/bridge_runtime.c:1873`, `crates/dsview-sys/bridge_runtime.c:1903`, `crates/dsview-sys/bridge_runtime.c:1920`, `crates/dsview-sys/bridge_runtime.c:1948`, `crates/dsview-sys/bridge_runtime.c:2020`, and `crates/dsview-sys/bridge_runtime.c:2057`. |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| --- | --- | --- | --- | --- |
| `crates/dsview-cli/src/main.rs` | `validated`, `input`, `response` | `load_decode_run_config()` reads config JSON and validates it; `load_offline_decode_input()` reads the raw artifact JSON; `core_run_offline_decode()` returns execution annotations | Yes | ✓ FLOWING |
| `crates/dsview-core/src/lib.rs` | `annotations`, `diagnostics`, `abs_start_sample` | `OfflineDecodeInput.sample_bytes` are chunked by `offline_decode_chunk_ranges()` and forwarded through `session.send_logic_chunk()` / `session.end()` | Yes | ✓ FLOWING |
| `crates/dsview-sys/src/lib.rs` | Captured annotations returned by `take_captured_annotations()` | Native callback capture is registered with `srd_pd_output_callback_add()` in `crates/dsview-sys/bridge_runtime.c:1829` and drained back to Rust in `crates/dsview-sys/src/lib.rs:2076` | Yes | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| CLI executes a valid offline decode run | `cargo test -p dsview-cli --test decode_cli decode_run_executes_valid_offline_decode_config -- --exact --nocapture` | `1 passed; 0 failed` | ✓ PASS |
| Core preserves absolute cursor progression across chunks | `cargo test -p dsview-core --test decode_execute offline_decode_uses_absolute_sample_cursor_across_chunks -- --exact --nocapture` | `1 passed; 0 failed` | ✓ PASS |
| Native stacked decoder forwarding works linearly | `cargo test -p dsview-sys --test boundary stacked_decoder_python_output_flows_linearly -- --exact --nocapture` | `1 passed; 0 failed` | ✓ PASS |
| Full phase regression suites stay green | `cargo test -p dsview-sys --test boundary -- --nocapture`; `cargo test -p dsview-core --test decode_execute -- --nocapture`; `cargo test -p dsview-cli --test decode_cli -- --nocapture`; `cargo test -p dsview-cli --lib -- --nocapture`; `cargo test -p dsview-core --lib -- --nocapture` | Boundary `20 passed`; core executor `7 passed`; CLI integration `7 passed`; CLI lib `11 passed`; core lib `34 passed` | ✓ PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| `DEC-05` | `16-01`, `16-02`, `16-03` | User can run DSView protocol decoders against previously captured logic data from the CLI. | ✓ SATISFIED | Canonical offline input contract exists in `crates/dsview-core/src/lib.rs:381`; CLI `decode run` is wired in `crates/dsview-cli/src/main.rs:509`; representative CLI execution test passed in `crates/dsview-cli/tests/decode_cli.rs:204`. |
| `DEC-07` | `16-02`, `16-03` | User can execute stacked decoders where upstream decoder output feeds downstream decoder input. | ✓ SATISFIED | Linear stack construction is enforced in `crates/dsview-core/src/lib.rs:734` and `crates/dsview-sys/bridge_runtime.c:1886`; real stacked forwarding passed in `crates/dsview-sys/tests/boundary.rs:560`; stacked CLI happy-path coverage passed in `crates/dsview-cli/tests/decode_cli.rs:204`. |

Phase 16 requirement cross-check: all requirement IDs declared in PLAN frontmatter (`DEC-05`, `DEC-07`) are present in `.planning/REQUIREMENTS.md`, mapped to Phase 16, and satisfied by the verified implementation. No orphaned Phase 16 requirement IDs were found.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| --- | --- | --- | --- | --- |
| None | - | No TODO/FIXME markers, placeholder execution paths, hollow returns, or public partial-success state in the Phase 16 artifacts scanned | - | No blocker or warning anti-patterns found in the verified phase files |

### Gaps Summary

No goal-blocking gaps found. Phase 16 delivers a raw-artifact-first offline decode contract, a wired libsigrokdecode execution bridge with absolute sample handling, stacked decoder execution, and a CLI `decode run` workflow that satisfies `DEC-05` and `DEC-07`.

---

_Verified: 2026-04-21T12:09:05Z_
_Verifier: Claude (gsd-verifier)_
