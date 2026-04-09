# DSView CLI

Command-line tool for using DSLogic Plus devices without the DSView GUI. Capture logic analyzer data and export machine-readable waveform files for downstream analysis.

## Quick Start

### Build from source

```bash
# Clone with submodules
git clone --recursive https://github.com/yourusername/dsview-cli.git
cd dsview-cli

# Build release binary
cargo build --release

# The CLI and bundled runtime are now in target/release/
```

### List connected devices

```bash
./target/release/dsview-cli devices list --format text
```

Example output:
```
Device 1 (handle: 1)
  Model: DSLogic Plus
  ID: dslogic-plus
  Native: DSLogic Plus [0]
```

### Capture waveform data

```bash
./target/release/dsview-cli capture \
  --handle 1 \
  --sample-rate-hz 100000000 \
  --sample-limit 2048 \
  --channels 0,1,2,3 \
  --output capture.vcd
```

This captures 2048 samples at 100 MHz from channels 0-3 and exports to `capture.vcd` (Value Change Dump format) with a JSON metadata sidecar at `capture.json`.

## How It Works

### Bundled Runtime and Resources

The CLI uses a **repository-built runtime** model:

- **During development**: `cargo run` automatically uses the runtime built from the `DSView/` submodule
- **In release bundles**: The CLI, runtime library, and DSLogic Plus resources are packaged together
- **Resource discovery**: The CLI finds bundled resources relative to the executable location

Release bundle structure:
```
dsview-cli-v0.1.0-x86_64-unknown-linux-gnu/
├── dsview-cli                    # CLI executable
├── runtime/
│   └── libdsview_runtime.so      # Platform-specific runtime (.so/.dylib/.dll)
└── resources/
    ├── DSLogicPlus.fw            # Firmware
    ├── DSLogic.fw                # Firmware fallback
    ├── DSLogicPlus.bin           # Bitstream
    └── DSLogicPlus-pgl12.bin     # Bitstream
```

### Resource Override

Use `--resource-dir` to point to a different resource directory:

```bash
./dsview-cli devices list --resource-dir /path/to/custom/resources
```

This is the only resource-related flag. The CLI no longer exposes runtime library selection.

## Supported Devices

**Current support**: DSLogic Plus only

The v1.0 milestone validates the capture-and-export workflow for DSLogic Plus. Future releases may add support for other DSLogic family devices.

## Platform Support

Pre-built release bundles are available for:

- **Linux**: x86_64, ARM64
- **macOS**: x86_64 (Intel), ARM64 (Apple Silicon)
- **Windows**: x86_64, ARM64

All platforms use the same CLI interface and bundle structure.

## Build Prerequisites

### Linux (Ubuntu/Debian)

```bash
sudo apt-get install \
  build-essential \
  cmake \
  pkg-config \
  libglib2.0-dev \
  libusb-1.0-0-dev \
  libfftw3-dev
```

### macOS

```bash
brew install cmake pkg-config glib libusb fftw
```

### Windows

Install dependencies via vcpkg:

```powershell
vcpkg install glib:x64-windows libusb:x64-windows fftw3:x64-windows pkgconf:x64-windows
```

## Testing

Run the full test suite:

```bash
cargo test --workspace
```

Run specific test suites:

```bash
# Bundle discovery tests
cargo test -p dsview-core --test bundle_discovery

# CLI contract tests
cargo test -p dsview-cli --test capture_cli
cargo test -p dsview-cli --test devices_cli
```

## Commands

### `devices list`

List all connected DSLogic Plus devices.

**Options:**
- `--format <json|text>`: Output format (default: json)
- `--resource-dir <PATH>`: Override bundled resource directory

### `devices open`

Open a device by handle and verify initialization.

**Options:**
- `--handle <HANDLE>`: Device handle from `devices list`
- `--format <json|text>`: Output format (default: json)
- `--resource-dir <PATH>`: Override bundled resource directory

### `capture`

Run a bounded capture and export waveform data.

**Required options:**
- `--handle <HANDLE>`: Device handle from `devices list`
- `--sample-rate-hz <HZ>`: Sample rate in Hz (e.g., 100000000 for 100 MHz)
- `--sample-limit <SAMPLES>`: Number of samples to capture
- `--channels <IDX,IDX,...>`: Comma-separated channel indices (e.g., 0,1,2,3)
- `--output <PATH>`: Output VCD file path (must end with .vcd)

**Optional:**
- `--metadata-output <PATH>`: JSON metadata path (defaults to .vcd path with .json extension)
- `--wait-timeout-ms <MS>`: Capture timeout in milliseconds (default: 10000)
- `--poll-interval-ms <MS>`: Status polling interval (default: 50)
- `--format <json|text>`: Output format (default: json)
- `--resource-dir <PATH>`: Override bundled resource directory

## Output Formats

### VCD (Value Change Dump)

Standard waveform interchange format. Compatible with GTKWave and other waveform viewers.

### JSON Metadata

Capture metadata sidecar includes:
- Capture configuration (sample rate, channels, limits)
- Acquisition summary (packets, terminal events, status)
- Artifact paths (VCD and metadata locations)
- Timestamps (ISO 8601 format)

Example:
```json
{
  "capture": {
    "sample_rate_hz": 100000000,
    "requested_sample_limit": 2048,
    "actual_sample_count": 2048,
    "enabled_channels": [0, 1, 2, 3]
  },
  "acquisition": {
    "completion": "clean_success",
    "terminal_event": "normal_end"
  },
  "artifacts": {
    "vcd_path": "/path/to/capture.vcd",
    "metadata_path": "/path/to/capture.json"
  }
}
```

## Development Notes

### DSView Submodule

The `DSView/` directory is a git submodule containing the upstream DSView project. This provides the device communication stack (`libsigrok4DSL`) that the CLI integrates with.

**Important**: Treat `DSView/` as read-only dependency code. Do not modify it for normal development work.

### Architecture

- **dsview-cli**: Command-line interface and user-facing commands
- **dsview-core**: Safe Rust orchestration for device/session/config flows
- **dsview-sys**: Native bindings to DSView/libsigrok integration boundary

The native integration is intentionally isolated behind Rust layers to keep unsafe code contained.

### Packaging

Release bundles are created using `tools/package-bundle.py` and validated with `tools/validate-bundle.py`. CI uses these Python helpers to ensure consistent bundle structure across all platforms without relying on unstable Cargo script support.

## License

[Your license here]

## Contributing

[Your contribution guidelines here]
