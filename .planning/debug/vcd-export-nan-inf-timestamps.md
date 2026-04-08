# /gsd:debug vcd-export-nan-inf-timestamps

## Context
- Issue: bounded real-hardware DSLogic Plus captures exported VCD files containing invalid timestamps like `#-nan` and `#inf`.
- Expected behavior: VCD timestamps should be finite, monotonic, and plausible for the configured sample rate.
- Observed behavior: capture and cleanup reported success, metadata was written, and the device remained reusable, but the VCD body was malformed.

## Reproduction
- Confirmed the reported artifact in `./.tmp/manual-uat/run-01.vcd`.
- The captured file contained:
  - `$timescale 1 ms $end`
  - `#-nan 1!`
  - `#inf`

## Hypotheses
1. Rust passed an invalid samplerate into the export bridge.
2. The native bridge dropped samplerate metadata while retaining packets.
3. The upstream VCD output module received packets in an order that allowed logic samples before samplerate metadata was known.

## Evidence
### Rust request path is valid
- `crates/dsview-core/src/lib.rs` builds `VcdExportRequest` directly from validated capture config.
- `crates/dsview-sys/src/lib.rs` rejects zero samplerates before crossing the FFI boundary.
- The failing repro used `--sample-rate-hz 1000000`, so hypothesis 1 was not supported.

### Native retention does preserve samplerate when metadata is present
- `crates/dsview-sys/bridge_runtime.c` records `SR_CONF_SAMPLERATE` from retained `SR_DF_META` packets into both the retained packet and `g_recorded_stream.samplerate_hz`.
- The bridge also already backfilled samplerate onto retained meta packets whose `samplerate_hz` field was zero.
- That weakened hypothesis 2 as a primary cause.

### Upstream VCD exporter emits NaN/Inf if logic arrives before samplerate
- `DSView/libsigrok4DSL/output/vcd.c` computes timestamps as:
  - `(double)ctx->samplecount / ctx->samplerate * ctx->period`
- If `ctx->samplerate` is still zero when the first logic packet is processed, the first timestamp becomes `0 / 0 => NaN` and the final timestamp becomes `n / 0 => Inf`.
- The module only learns samplerate from either:
  - a prior `SR_DF_META` packet, or
  - `sr_config_get(..., SR_CONF_SAMPLERATE, ...)` on the replay device.
- The malformed VCD exactly matches this failure mode, supporting hypothesis 3.

## Root Cause
- The recorded export replay path in `crates/dsview-sys/bridge_runtime.c` preserved retained packet order verbatim.
- Real hardware captures can retain `LOGIC` before `META`.
- Synthetic tests stayed green because the helper replay paths always constructed `META -> LOGIC -> END`, so they never exercised the hardware ordering hazard.
- When `LOGIC` was replayed before any samplerate metadata reached the upstream VCD module, `ctx->samplerate` remained zero and the exporter wrote `#-nan` / `#inf`.

## Fix Implemented
### `crates/dsview-sys/bridge_runtime.c`
- Updated `dsview_bridge_export_stream(...)` to compute a `replay_samplerate_hz` from the recorded stream when available, otherwise from the export request.
- Before replaying the first retained `LOGIC` packet, the bridge now synthesizes and emits a samplerate `META` packet if no metadata has been replayed yet.
- Existing retained `META` packets are still replayed, with zero-valued samplerates backfilled from `replay_samplerate_hz`.
- This keeps packet retention intact while enforcing the upstream VCD module's requirement that samplerate be known before the first logic sample.

## Validation
### Automated
- `cargo test -p dsview-sys` ✅
- `cargo build -p dsview-cli` ✅

### Manual hardware validation
- Rebuilt `dsview-cli` on `master` with `cargo build --manifest-path /home/seasonyuu/projects/dsview-cli/Cargo.toml -p dsview-cli`.
- Re-ran the bounded DSLogic Plus repro on real hardware:
  - `cargo run --manifest-path /home/seasonyuu/projects/dsview-cli/Cargo.toml -p dsview-cli -- capture --use-source-runtime --resource-dir /home/seasonyuu/projects/dsview-cli/DSView/DSView/res --handle 1 --sample-rate-hz 1000000 --sample-limit 1024 --channels 0 --wait-timeout-ms 10000 --poll-interval-ms 50 --format json --output /home/seasonyuu/projects/dsview-cli/.tmp/manual-uat/run-01.vcd`
- Observed capture result remained `clean_success`, with metadata still reporting:
  - `sample_rate_hz: 1000000`
  - `actual_sample_count: 64`
  - `completion: clean_success`
  - `end_packet_status: ok`
- Regenerated `./.tmp/manual-uat/run-01.vcd` now contains finite timestamps:
  - `$comment ... at 1 MHz`
  - `$timescale 1 us $end`
  - `#0 1!`
  - `#64`
- Confirmed the malformed markers are gone: no `#-nan`, no `#inf`.
- Immediate device reuse still succeeds after export:
  - `cargo run --manifest-path /home/seasonyuu/projects/dsview-cli/Cargo.toml -p dsview-cli -- devices open --use-source-runtime --resource-dir /home/seasonyuu/projects/dsview-cli/DSView/DSView/res --handle 1 --format json` ✅

## Status
- Root cause: confirmed
- Fix: implemented in `crates/dsview-sys/bridge_runtime.c`
- Automated validation: passed
- Real hardware repro confirmation: passed
