<h1 align="center">DSView CLI</h1>

<p align="center">Scriptable DSLogic Plus capture and protocol decoding without the DSView GUI.</p>

<p align="center">
  <a href="README.md">English</a> | <a href="README.zh-CN.md">简体中文</a>
</p>

<p align="center">
  <a href="https://github.com/LISTENAI/dsview-cli/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/LISTENAI/dsview-cli/actions/workflows/ci.yml/badge.svg"></a>
  <a href="https://github.com/LISTENAI/dsview-cli/releases/latest"><img alt="Latest Release" src="https://img.shields.io/github/v/release/LISTENAI/dsview-cli"></a>
  <a href="LICENSE"><img alt="License" src="https://img.shields.io/github/license/LISTENAI/dsview-cli"></a>
</p>

DSView CLI is a command-line tool for DSLogic Plus devices. It brings the DSView/libsigrok device stack into a scriptable workflow for automated capture, VCD export, and protocol decoder inspection or offline decoding.

## Features

- Capture finite logic analyzer sessions from DSLogic Plus devices.
- Export VCD waveform files plus JSON metadata for downstream tooling.
- Inspect DSView protocol decoders from the command line.
- Run offline protocol decoding from JSON logic sample input.
- Ship as self-contained release bundles with native runtimes, firmware resources, decoder scripts, and a bundled Python runtime.
- Support JSON output by default for automation, with text output for interactive shell use.

## Platform Support

| Platform | Architectures | Release bundle | One-line installer |
| --- | --- | --- | --- |
| Linux | x86_64, ARM64 | Yes | Yes |
| macOS | Intel, Apple Silicon | Yes | Yes |
| Windows | x86_64, ARM64 | Yes | Yes |

Current device support is focused on **DSLogic Plus**.

## Installation

### macOS and Linux

Install the latest published release bundle into `~/.local/opt/dsview-cli` and create a launcher in `~/.local/bin`:

```bash
curl -fsSL https://raw.githubusercontent.com/LISTENAI/dsview-cli/refs/heads/master/scripts/install.sh | sh
```

Install a specific release tag:

```bash
curl -fsSL https://raw.githubusercontent.com/LISTENAI/dsview-cli/refs/heads/master/scripts/install.sh | sh -s -- --version v1.2.1
```

Useful installer options:

```bash
curl -fsSL https://raw.githubusercontent.com/LISTENAI/dsview-cli/refs/heads/master/scripts/install.sh | sh -s -- --prefix ~/.local/opt/dsview-cli --bin-dir ~/.local/bin --version v1.2.1
curl -fsSL https://raw.githubusercontent.com/LISTENAI/dsview-cli/refs/heads/master/scripts/install.sh | sh -s -- --dry-run
```

The installer keeps the release bundle intact and smoke-tests:

```bash
dsview-cli --version
dsview-cli devices list --help
dsview-cli decode list --format json
```

### Windows

Install the latest published release bundle into `%LOCALAPPDATA%\Programs\dsview-cli`, create a launcher, and add the launcher directory to your user `PATH`:

```powershell
irm https://raw.githubusercontent.com/LISTENAI/dsview-cli/refs/heads/master/scripts/install.ps1 | iex
```

Install a specific release tag:

```powershell
$script = Join-Path $env:TEMP "dsview-cli-install.ps1"
irm https://raw.githubusercontent.com/LISTENAI/dsview-cli/refs/heads/master/scripts/install.ps1 -OutFile $script
powershell -ExecutionPolicy Bypass -File $script -Version v1.2.1
```

Restart your shell after installation so the updated `PATH` is visible.

### Manual Download

Download the `.tar.gz` bundle for your target from GitHub Releases, verify it with the release checksum file, and extract it:

```bash
VERSION=v1.2.1
TARGET=x86_64-unknown-linux-gnu
curl -LO "https://github.com/LISTENAI/dsview-cli/releases/download/$VERSION/dsview-cli-$VERSION-$TARGET.tar.gz"
curl -LO "https://github.com/LISTENAI/dsview-cli/releases/download/$VERSION/dsview-cli-$VERSION-SHA256SUMS.txt"
sha256sum --check --ignore-missing "dsview-cli-$VERSION-SHA256SUMS.txt"
tar -xzf "dsview-cli-$VERSION-$TARGET.tar.gz"
```

If `sha256sum` is not available, use `shasum -a 256` and compare the archive hash with the matching checksum entry.

Run `dsview-cli` from the extracted bundle root or put a wrapper script on your `PATH`. Keep the extracted directory together. Do not copy only the executable into another directory; the CLI discovers runtimes, decoder scripts, Python, and firmware resources relative to the executable.

## Quick Start

### List Devices

```bash
dsview-cli devices list --format text
```

Example output:

```text
Device 1 (handle: 1)
  Model: DSLogic Plus
  ID: dslogic-plus
  Native: DSLogic Plus [0]
```

### Inspect Capture Options

```bash
dsview-cli devices options --handle 1 --format text
```

Use this before capture to discover valid tokens for operation mode, stop option, channel mode, threshold, and filters.

### Capture a Waveform

```bash
dsview-cli capture \
  --handle 1 \
  --sample-rate-hz 100000000 \
  --sample-limit 2048 \
  --channels 0,1,2,3 \
  --output capture.vcd
```

This captures 2048 samples at 100 MHz from channels 0-3 and writes:

- `capture.vcd` - waveform data in Value Change Dump format.
- `capture.json` - metadata sidecar with capture settings, completion state, and artifact paths.

Optional capture overrides use the same tokens reported by `devices options`:

```bash
dsview-cli capture \
  --handle 1 \
  --operation-mode buffer \
  --sample-rate-hz 100000000 \
  --sample-limit 2048 \
  --channels 0,1 \
  --output capture.vcd
```

### List Protocol Decoders

```bash
dsview-cli decode list --format text
```

Inspect one decoder to see its channels, options, annotations, and stack compatibility:

```bash
dsview-cli decode inspect i2c --format json
```

### Validate and Run Offline Decoding

Create a decoder config:

```json
{
  "version": 1,
  "decoder": {
    "id": "i2c",
    "channels": {
      "scl": 0,
      "sda": 1
    },
    "options": {}
  },
  "stack": []
}
```

Validate it against the bundled decoder registry:

```bash
dsview-cli decode validate --config decode.json --format text
```

Create a logic sample input. This is the low-level decode input format, not the VCD file or capture metadata sidecar:

```json
{
  "samplerate_hz": 100000000,
  "format": "split_logic",
  "sample_bytes": [3, 1, 3, 2, 3],
  "unitsize": 1
}
```

Run offline decoding from the config and sample input:

```bash
dsview-cli decode run \
  --config decode.json \
  --input logic-input.json \
  --output decode-report.json \
  --format json
```

The offline decode input uses `samplerate_hz`, `format`, `sample_bytes`, `unitsize`, and optional `logic_packet_lengths`. `sample_bytes` is a JSON byte array where each byte is one packed logic sample for `split_logic` inputs. Use `decode inspect` to confirm the channel IDs and option names expected by a decoder.

## Command Reference

### `devices list`

List connected DSLogic Plus devices.

```bash
dsview-cli devices list [--format json|text] [--resource-dir PATH]
```

### `devices open`

Open a device by handle and verify initialization.

```bash
dsview-cli devices open --handle HANDLE [--format json|text] [--resource-dir PATH]
```

### `devices options`

Read capture option capabilities from a device.

```bash
dsview-cli devices options --handle HANDLE [--format json|text] [--resource-dir PATH]
```

### `capture`

Run a bounded capture and export a VCD file.

```bash
dsview-cli capture \
  --handle HANDLE \
  --sample-rate-hz HZ \
  --sample-limit SAMPLES \
  --channels IDX[,IDX...] \
  --output PATH.vcd \
  [--metadata-output PATH.json] \
  [--operation-mode TOKEN] \
  [--stop-option TOKEN] \
  [--channel-mode TOKEN] \
  [--threshold-volts VOLTS] \
  [--filter TOKEN] \
  [--wait-timeout-ms MS] \
  [--poll-interval-ms MS] \
  [--format json|text] \
  [--resource-dir PATH]
```

### `decode list`

List bundled protocol decoders.

```bash
dsview-cli decode list [--format json|text] [--decode-runtime PATH] [--decoder-dir PATH]
```

### `decode inspect`

Inspect one decoder descriptor.

```bash
dsview-cli decode inspect DECODER_ID [--format json|text] [--decode-runtime PATH] [--decoder-dir PATH]
```

### `decode validate`

Validate a JSON decode config without running a decode session.

```bash
dsview-cli decode validate --config PATH [--format json|text] [--decode-runtime PATH] [--decoder-dir PATH]
```

### `decode run`

Run an offline decode session and optionally write the canonical report to a file.

```bash
dsview-cli decode run \
  --config PATH \
  --input PATH \
  [--output PATH] \
  [--format json|text] \
  [--decode-runtime PATH] \
  [--decoder-dir PATH]
```

## Release Bundle Layout

Release archives are relocatable as long as the extracted directory remains intact:

```text
dsview-cli-<version>-<target>/
|-- dsview-cli[.exe]
|-- runtime/
|   `-- libdsview_runtime.so | libdsview_runtime.dylib | dsview_runtime.dll
|-- decode-runtime/
|   `-- libdsview_decode_runtime.so | libdsview_decode_runtime.dylib | dsview_decode_runtime.dll
|-- decoders/
|   `-- DSView protocol decoder scripts
|-- python/
|   `-- bundled Python runtime and standard library used by protocol decoding
|-- resources/
|   |-- DSLogicPlus.fw
|   |-- DSLogic.fw
|   |-- DSLogicPlus.bin
|   `-- DSLogicPlus-pgl12.bin
`-- *.dll, python*.dll, vcruntime*.dll on Windows when needed
```

The bundled Python runtime is intentionally slimmed for redistribution. Test packages, `ensurepip`, GUI modules such as `tkinter`, and other unnecessary files are omitted.

Platform notes:

- Linux bundles include the linked `libpython*.so*` under `python/lib` and set the decode runtime search path to use it.
- macOS bundles include the required Python framework or dylib content under `python/`, rewrite Python links to bundle-relative paths, and ad-hoc sign modified runtime files.
- Windows bundles include Python DLLs at the bundle root and Python library content under `python/`.

## Build From Source

Clone with submodules:

```bash
git clone --recursive https://github.com/LISTENAI/dsview-cli.git
cd dsview-cli
```

Install native prerequisites.

Linux (Ubuntu/Debian):

```bash
sudo apt-get update
sudo apt-get install -y \
  build-essential \
  cmake \
  pkg-config \
  libglib2.0-dev \
  libusb-1.0-0-dev \
  libfftw3-dev
```

macOS:

```bash
brew install cmake pkg-config glib libusb fftw
```

Windows with vcpkg:

```powershell
vcpkg install glib:x64-windows libusb:x64-windows fftw3:x64-windows pkgconf:x64-windows
```

For Windows ARM64 source builds, use the `arm64-windows` triplet instead.

Build and test:

```bash
cargo build --release
cargo test --workspace
```

A source build is useful for development, but it is not the same as a fully relocatable release archive. Release archives are produced by the packaging workflow so native runtimes, decoder scripts, firmware resources, and Python runtime files are assembled and validated together.

## Development Notes

The upstream DSView project is included as a git submodule and provides the native device communication stack used by this CLI. Treat that submodule as dependency code for normal development work.

Workspace responsibilities:

- `dsview-cli` - command-line interface and user-facing command contracts.
- `dsview-core` - safe Rust orchestration for discovery, capture, decode, and validation flows.
- `dsview-sys` - native bindings and C/C++ integration boundary for DSView/libsigrok components.

The native boundary is intentionally isolated so unsafe and platform-specific behavior stays behind a narrow Rust API.

## Packaging and CI

CI builds every supported target, packages a release archive, validates the archive structure, and smoke-tests command discovery from the extracted bundle.

Validation checks include:

- Executable, capture runtime, decode runtime, decoder scripts, firmware resources, and Python runtime are present.
- Unix decode runtime links resolve to bundled Python paths.
- Windows bundles include native DLL and Python DLL dependencies.
- `dsview-cli --help`, `devices list --help`, and `decode list --format json` work from the extracted bundle.

Windows ARM64 CI uses GitHub-hosted `windows-11-arm` runners, which are public preview infrastructure as of April 2026. Runner image details may change over time.

## Troubleshooting

### `libpython*.so*` or Python framework not found

Use a current full release bundle and keep the extracted directory intact. This error usually means an old bundle was used or only the executable was copied away from its sibling directories.

### `decode list` fails or returns no decoders

Check that `decode-runtime/`, `decoders/`, and `python/` are still present next to the executable. If you use `--decode-runtime` or `--decoder-dir`, both paths must match the same decoder runtime build.

### Linux cannot access the USB device as a normal user

Install a udev rule, reload rules, and reconnect the device:

```bash
printf '%s\n' 'SUBSYSTEM=="usb", ATTRS{idVendor}=="2a0e", MODE="0666"' | \
  sudo tee /etc/udev/rules.d/99-dsview-cli.rules >/dev/null
sudo udevadm control --reload-rules
sudo udevadm trigger
```

### Windows command is not found after installation

Restart your shell so the updated user `PATH` is loaded, or run the generated launcher by full path.

## Contributing

Issues and pull requests are welcome. Please keep user-facing command output stable for JSON consumers, and update documentation when behavior changes.

## License

The Rust code in this repository is licensed under the Apache License, Version 2.0. See `LICENSE`.

The upstream DSView submodule and bundled third-party components keep their own licenses. Vendored DSView/libsigrok sources ship their existing license files and are not relicensed by this repository.
