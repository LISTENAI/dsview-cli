use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

use clap::{Args, Parser, Subcommand, ValueEnum};
use dsview_cli::{
    DecodeInspectResponse, DecodeListResponse, build_decode_inspect_response,
    build_decode_list_response,
    build_device_options_response,
    capture_device_options::{
        resolve_capture_device_option_request, CaptureDeviceOptionParseError,
    },
    render_decode_inspect_text, render_decode_list_text, render_device_options_text,
    DeviceOptionsResponse,
};
use dsview_core::{
    DecodeBringUpError,
    DecoderRuntimeError, DecoderRuntimeErrorCode,
    describe_native_error, resolve_capture_artifact_paths,
    decode_inspect as core_decode_inspect, decode_list as core_decode_list,
    validated_capture_config_from_device_options, AcquisitionSummary, AcquisitionTerminalEvent,
    BringUpError, CaptureArtifactPathError, CaptureCleanup, CaptureCompletion, CaptureConfigError,
    CaptureConfigRequest, CaptureDeviceOptionFacts, CaptureExportError, CaptureRunError,
    CaptureRunRequest, ChannelModeGroupSnapshot, ChannelModeOptionSnapshot,
    CurrentDeviceOptionValues, DeviceIdentitySnapshot, DeviceOptionApplyFailure,
    DeviceOptionValidationCapabilities, DeviceOptionValidationError, DeviceOptionsSnapshot,
    Discovery, EnumOptionSnapshot, MetadataAcquisitionInfo, MetadataArtifactInfo,
    MetadataCaptureInfo, MetadataToolInfo, NativeErrorCode, OperationModeValidationCapabilities,
    RuntimeError, SelectionHandle, SupportedDevice, ThresholdCapabilitySnapshot,
    ValidatedCaptureConfig,
};
use serde::Serialize;

const BUILD_VERSION: &str = match option_env!("DSVIEW_BUILD_VERSION") {
    Some(version) => version,
    None => env!("CARGO_PKG_VERSION"),
};

#[cfg(debug_assertions)]
const TEST_DEVICE_OPTIONS_FIXTURE_ENV: &str = "DSVIEW_CLI_TEST_DEVICE_OPTIONS_FIXTURE";
#[cfg(debug_assertions)]
const TEST_DECODE_FIXTURE_ENV: &str = "DSVIEW_CLI_TEST_DECODE_FIXTURE";

#[derive(Parser, Debug)]
#[command(version = BUILD_VERSION)]
#[command(name = "dsview-cli")]
#[command(about = "Scriptable DSLogic bring-up CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Devices(DeviceArgs),
    Decode(DecodeArgs),
    Capture(CaptureArgs),
}

#[derive(Args, Debug)]
struct DeviceArgs {
    #[command(subcommand)]
    command: DeviceCommand,
}

#[derive(Subcommand, Debug)]
enum DeviceCommand {
    List(ListArgs),
    Open(OpenArgs),
    Options(OptionsArgs),
}

#[derive(Args, Debug)]
struct DecodeArgs {
    #[command(subcommand)]
    command: DecodeCommand,
}

#[derive(Subcommand, Debug)]
enum DecodeCommand {
    List(DecodeListArgs),
    Inspect(DecodeInspectArgs),
}

#[derive(Args, Debug)]
struct SharedDecodeArgs {
    #[arg(
        long = "decode-runtime",
        value_name = "PATH",
        help = "Path to the decoder runtime shared library; bundled decoder runtime is used by default"
    )]
    decode_runtime: Option<PathBuf>,
    #[arg(
        long = "decoder-dir",
        value_name = "PATH",
        help = "Directory containing decoder scripts; bundled decoder scripts are used by default"
    )]
    decoder_dir: Option<PathBuf>,
    #[arg(
        long,
        value_enum,
        default_value_t = OutputFormat::Json,
        help = "Output format: json is stable for automation, text is for direct shell use"
    )]
    format: OutputFormat,
}

#[derive(Args, Debug)]
struct DecodeListArgs {
    #[command(flatten)]
    decode: SharedDecodeArgs,
}

#[derive(Args, Debug)]
struct DecodeInspectArgs {
    #[command(flatten)]
    decode: SharedDecodeArgs,
    #[arg(value_name = "DECODER_ID", help = "Canonical upstream decoder id from `decode list`")]
    decoder_id: String,
}

#[derive(Args, Debug)]
struct SharedRuntimeArgs {
    #[arg(
        long = "resource-dir",
        value_name = "PATH",
        help = "Directory containing DSLogic Plus firmware and bitstream resources; bundled resources are used by default"
    )]
    resource_dir: Option<PathBuf>,
    #[arg(
        long,
        value_enum,
        default_value_t = OutputFormat::Json,
        help = "Output format: json is stable for automation, text is for direct shell use"
    )]
    format: OutputFormat,
}

#[derive(Args, Debug)]
struct ListArgs {
    #[command(flatten)]
    runtime: SharedRuntimeArgs,
}

#[derive(Args, Debug)]
struct OpenArgs {
    #[command(flatten)]
    runtime: SharedRuntimeArgs,
    #[arg(
        long,
        value_name = "HANDLE",
        help = "Selection handle returned by `devices list`"
    )]
    handle: u64,
}

#[derive(Args, Debug)]
struct OptionsArgs {
    #[command(flatten)]
    runtime: SharedRuntimeArgs,
    #[arg(
        long,
        value_name = "HANDLE",
        help = "Selection handle returned by `devices list`"
    )]
    handle: u64,
}

#[derive(Args, Debug)]
struct CaptureArgs {
    #[command(flatten)]
    runtime: SharedRuntimeArgs,
    #[arg(
        long,
        value_name = "HANDLE",
        help = "Selection handle returned by `devices list`"
    )]
    handle: u64,
    #[arg(
        long = "sample-rate-hz",
        value_name = "HZ",
        help = "Requested capture sample rate in hertz"
    )]
    sample_rate_hz: u64,
    #[arg(
        long = "sample-limit",
        value_name = "SAMPLES",
        help = "Requested sample count before the finite capture stops"
    )]
    sample_limit: u64,
    #[arg(
        long = "channels",
        value_delimiter = ',',
        value_name = "IDX[,IDX...]",
        help = "Comma-separated logic channel indexes to enable, for example 0,1,2,3"
    )]
    channels: Vec<u16>,
    #[command(flatten)]
    device_options: CaptureDeviceOptionArgs,
    #[arg(
        long = "output",
        value_name = "PATH",
        help = "Final VCD artifact path; must end with .vcd"
    )]
    output: PathBuf,
    #[arg(
        long = "metadata-output",
        value_name = "PATH",
        help = "Optional metadata JSON path; defaults to the VCD path with a .json extension"
    )]
    metadata_output: Option<PathBuf>,
    #[arg(
        long = "wait-timeout-ms",
        default_value_t = 10_000,
        help = "Maximum time to wait for capture completion before aborting"
    )]
    wait_timeout_ms: u64,
    #[arg(
        long = "poll-interval-ms",
        default_value_t = 50,
        help = "Polling interval for checking capture progress while waiting"
    )]
    poll_interval_ms: u64,
}

#[derive(Args, Debug, Clone, Default, PartialEq)]
#[command(next_help_heading = "Device options")]
struct CaptureDeviceOptionArgs {
    #[arg(
        long = "operation-mode",
        value_name = "TOKEN",
        help = "Operation mode token, for example `buffer`; see `devices options --handle <HANDLE>` for supported tokens and compatibility"
    )]
    operation_mode: Option<String>,
    #[arg(
        long = "stop-option",
        value_name = "TOKEN",
        help = "Stop option token; see `devices options --handle <HANDLE>` for supported tokens and compatibility"
    )]
    stop_option: Option<String>,
    #[arg(
        long = "channel-mode",
        value_name = "TOKEN",
        help = "Channel mode token; see `devices options --handle <HANDLE>` for supported tokens and compatibility"
    )]
    channel_mode: Option<String>,
    #[arg(
        long = "threshold-volts",
        value_name = "VOLTS",
        help = "Threshold voltage in volts; see `devices options --handle <HANDLE>` for supported values and compatibility"
    )]
    threshold_volts: Option<f64>,
    #[arg(
        long = "filter",
        value_name = "TOKEN",
        help = "Filter token; see `devices options --handle <HANDLE>` for supported tokens and compatibility"
    )]
    filter: Option<String>,
}

impl dsview_cli::capture_device_options::CaptureDeviceOptionInput for CaptureDeviceOptionArgs {
    fn operation_mode(&self) -> Option<&str> {
        self.operation_mode.as_deref()
    }

    fn stop_option(&self) -> Option<&str> {
        self.stop_option.as_deref()
    }

    fn channel_mode(&self) -> Option<&str> {
        self.channel_mode.as_deref()
    }

    fn threshold_volts(&self) -> Option<f64> {
        self.threshold_volts
    }

    fn filter(&self) -> Option<&str> {
        self.filter.as_deref()
    }
}

impl CaptureDeviceOptionArgs {
    fn has_overrides(&self) -> bool {
        self.operation_mode.is_some()
            || self.stop_option.is_some()
            || self.channel_mode.is_some()
            || self.threshold_volts.is_some()
            || self.filter.is_some()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum OutputFormat {
    Json,
    Text,
}

#[derive(Serialize)]
struct DeviceListResponse {
    devices: Vec<DeviceRecord>,
}

#[derive(Serialize)]
struct DeviceRecord {
    handle: u64,
    stable_id: &'static str,
    model: &'static str,
    native_name: String,
}

#[derive(Serialize)]
struct OpenResponse {
    selected: DeviceRecord,
    released: bool,
    native_last_error: String,
    native_init_status: Option<i32>,
}

#[derive(Serialize)]
struct CaptureResponse {
    selected_handle: u64,
    completion: &'static str,
    saw_logic_packet: bool,
    saw_end_packet: bool,
    saw_terminal_normal_end: bool,
    cleanup_succeeded: bool,
    device_options: CaptureDeviceOptionFacts,
    artifacts: CaptureArtifactsResponse,
}

#[derive(Serialize)]
struct CaptureArtifactsResponse {
    vcd_path: String,
    metadata_path: String,
}

#[derive(Serialize, Debug, PartialEq, Eq)]
pub(crate) struct CaptureCleanupResponse {
    pub(crate) stop_attempted: bool,
    pub(crate) stop_succeeded: bool,
    pub(crate) callbacks_cleared: bool,
    pub(crate) release_succeeded: bool,
    pub(crate) stop_error: Option<String>,
    pub(crate) clear_callbacks_error: Option<String>,
    pub(crate) release_error: Option<String>,
}

#[derive(Serialize, Debug, PartialEq, Eq)]
pub(crate) struct ErrorResponse {
    pub(crate) code: &'static str,
    pub(crate) message: String,
    pub(crate) detail: Option<String>,
    pub(crate) native_error: Option<&'static str>,
    pub(crate) terminal_event: Option<&'static str>,
    pub(crate) cleanup: Option<CaptureCleanupResponse>,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Devices(args) => match args.command {
            DeviceCommand::List(args) => run_list(args),
            DeviceCommand::Open(args) => run_open(args),
            DeviceCommand::Options(args) => run_options(args),
        },
        Command::Decode(args) => match args.command {
            DecodeCommand::List(args) => run_decode_list(args),
            DecodeCommand::Inspect(args) => run_decode_inspect(args),
        },
        Command::Capture(args) => run_capture(args),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(FailedCommand { format, error }) => {
            render_error(format, &error);
            ExitCode::from(1)
        }
    }
}

fn run_decode_list(args: DecodeListArgs) -> Result<(), FailedCommand> {
    #[cfg(debug_assertions)]
    let decoders = if let Some(mode) = decode_test_fixture_mode() {
        decode_list_from_fixture(mode)
    } else {
        core_decode_list(
            args.decode.decode_runtime.as_deref(),
            args.decode.decoder_dir.as_deref(),
        )
    }
    .map_err(|error| command_error(args.decode.format, classify_decode_error(&error)))?;

    #[cfg(not(debug_assertions))]
    let decoders = core_decode_list(
        args.decode.decode_runtime.as_deref(),
        args.decode.decoder_dir.as_deref(),
    )
    .map_err(|error| command_error(args.decode.format, classify_decode_error(&error)))?;
    let response = build_decode_list_response(&decoders);
    render_decode_list_success(args.decode.format, &response);
    Ok(())
}

fn run_decode_inspect(args: DecodeInspectArgs) -> Result<(), FailedCommand> {
    #[cfg(debug_assertions)]
    let decoder = if let Some(mode) = decode_test_fixture_mode() {
        decode_inspect_from_fixture(mode, &args.decoder_id)
    } else {
        core_decode_inspect(
            args.decode.decode_runtime.as_deref(),
            args.decode.decoder_dir.as_deref(),
            &args.decoder_id,
        )
    }
    .map_err(|error| command_error(args.decode.format, classify_decode_error(&error)))?;

    #[cfg(not(debug_assertions))]
    let decoder = core_decode_inspect(
        args.decode.decode_runtime.as_deref(),
        args.decode.decoder_dir.as_deref(),
        &args.decoder_id,
    )
    .map_err(|error| command_error(args.decode.format, classify_decode_error(&error)))?;
    let response = build_decode_inspect_response(&decoder);
    render_decode_inspect_success(args.decode.format, &response);
    Ok(())
}

#[cfg(debug_assertions)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DecodeTestFixtureMode {
    Registry,
    MissingRuntime,
    MissingMetadata,
}

#[cfg(debug_assertions)]
fn decode_test_fixture_mode() -> Option<DecodeTestFixtureMode> {
    match std::env::var(TEST_DECODE_FIXTURE_ENV).ok().as_deref() {
        Some("registry") => Some(DecodeTestFixtureMode::Registry),
        Some("missing-runtime") => Some(DecodeTestFixtureMode::MissingRuntime),
        Some("missing-metadata") => Some(DecodeTestFixtureMode::MissingMetadata),
        _ => None,
    }
}

#[cfg(debug_assertions)]
fn decode_list_from_fixture(
    mode: DecodeTestFixtureMode,
) -> Result<Vec<dsview_core::DecoderDescriptor>, DecodeBringUpError> {
    match mode {
        DecodeTestFixtureMode::Registry => Ok(vec![decode_fixture_descriptor()]),
        DecodeTestFixtureMode::MissingRuntime => Err(DecodeBringUpError::BundledRuntimeMissing {
            path: PathBuf::from("bundle/decode-runtime/libdsview_decode_runtime.so"),
            executable_dir: PathBuf::from("bundle"),
        }),
        DecodeTestFixtureMode::MissingMetadata => Err(DecodeBringUpError::DecoderScriptsMissing {
            path: PathBuf::from("bundle/decoders"),
        }),
    }
}

#[cfg(debug_assertions)]
fn decode_inspect_from_fixture(
    mode: DecodeTestFixtureMode,
    decoder_id: &str,
) -> Result<dsview_core::DecoderDescriptor, DecodeBringUpError> {
    match mode {
        DecodeTestFixtureMode::Registry => {
            if decoder_id == "0:i2c" {
                Ok(decode_fixture_descriptor())
            } else {
                Err(DecodeBringUpError::UnknownDecoder {
                    decoder_id: decoder_id.to_string(),
                })
            }
        }
        DecodeTestFixtureMode::MissingRuntime => Err(DecodeBringUpError::BundledRuntimeMissing {
            path: PathBuf::from("bundle/decode-runtime/libdsview_decode_runtime.so"),
            executable_dir: PathBuf::from("bundle"),
        }),
        DecodeTestFixtureMode::MissingMetadata => Err(DecodeBringUpError::DecoderScriptsMissing {
            path: PathBuf::from("bundle/decoders"),
        }),
    }
}

#[cfg(debug_assertions)]
fn decode_fixture_descriptor() -> dsview_core::DecoderDescriptor {
    dsview_core::DecoderDescriptor {
        id: "0:i2c".to_string(),
        name: "0:I2C".to_string(),
        longname: "Inter-Integrated Circuit".to_string(),
        description: "Two-wire serial bus".to_string(),
        license: "gplv2+".to_string(),
        inputs: vec![dsview_core::DecoderInputDescriptor {
            id: "logic".to_string(),
        }],
        outputs: vec![
            dsview_core::DecoderOutputDescriptor {
                id: "i2c".to_string(),
            },
            dsview_core::DecoderOutputDescriptor {
                id: "i2c-messages".to_string(),
            },
        ],
        tags: vec!["serial".to_string(), "embedded".to_string()],
        required_channels: vec![dsview_core::DecoderChannelDescriptor {
            id: "scl".to_string(),
            name: "SCL".to_string(),
            description: "Clock".to_string(),
            order: 0,
            channel_type: 0,
            idn: Some("clk".to_string()),
        }],
        optional_channels: vec![dsview_core::DecoderChannelDescriptor {
            id: "sda".to_string(),
            name: "SDA".to_string(),
            description: "Data".to_string(),
            order: 1,
            channel_type: 0,
            idn: Some("data".to_string()),
        }],
        options: vec![dsview_core::DecoderOptionDescriptor {
            id: "address_format".to_string(),
            idn: Some("address_format".to_string()),
            description: Some("Whether addresses render as 7-bit or 8-bit".to_string()),
            default_value: Some("7-bit".to_string()),
            values: vec!["7-bit".to_string(), "8-bit".to_string()],
        }],
        annotations: vec![dsview_core::DecoderAnnotationDescriptor {
            id: "start".to_string(),
            label: Some("START".to_string()),
            description: Some("Start condition".to_string()),
            annotation_type: 0,
        }],
        annotation_rows: vec![dsview_core::DecoderAnnotationRowDescriptor {
            id: "frames".to_string(),
            description: Some("Frame events".to_string()),
            annotation_classes: vec![0],
        }],
    }
}

struct FailedCommand {
    format: OutputFormat,
    error: ErrorResponse,
}

fn run_list(args: ListArgs) -> Result<(), FailedCommand> {
    let discovery = connect_runtime(&args.runtime)?;
    let devices = discovery
        .list_supported_devices()
        .map_err(|error| command_error(args.runtime.format, classify_error(&error)))?;

    let response = DeviceListResponse {
        devices: devices.iter().map(device_record).collect(),
    };
    render_device_list_success(args.runtime.format, &response);
    Ok(())
}

fn run_open(args: OpenArgs) -> Result<(), FailedCommand> {
    let discovery = connect_runtime(&args.runtime)?;
    let handle = SelectionHandle::new(args.handle)
        .ok_or_else(|| command_error(args.runtime.format, invalid_handle_error()))?;
    let opened = discovery
        .open_device(handle)
        .map_err(|error| command_error(args.runtime.format, classify_error(&error)))?;
    let response = OpenResponse {
        selected: device_record(opened.device()),
        released: true,
        native_last_error: opened.last_error().name().to_string(),
        native_init_status: opened.init_status(),
    };
    opened
        .release()
        .map_err(|error| command_error(args.runtime.format, classify_error(&error)))?;

    render_json_or_ok(args.runtime.format, &response);
    Ok(())
}

fn run_capture(args: CaptureArgs) -> Result<(), FailedCommand> {
    let artifact_paths =
        resolve_capture_artifact_paths(&args.output, args.metadata_output.as_ref()).map_err(
            |error| command_error(args.runtime.format, classify_artifact_path_error(&error)),
        )?;
    let handle = SelectionHandle::new(args.handle)
        .ok_or_else(|| command_error(args.runtime.format, invalid_handle_error()))?;
    let mut validated_device_options = None;
    let mut device_options_snapshot = None;
    if uses_device_option_validation(&args) {
        let (snapshot, capabilities) =
            if let Some((snapshot, capabilities)) = capture_test_fixture(handle) {
                (snapshot, capabilities)
            } else {
                let discovery = connect_runtime(&args.runtime)?;
                let snapshot = discovery
                    .inspect_device_options(handle)
                    .map_err(|error| command_error(args.runtime.format, classify_error(&error)))?;
                let capabilities = discovery
                    .load_device_option_validation_capabilities(handle)
                    .map_err(|error| command_error(args.runtime.format, classify_error(&error)))?;
                (snapshot, capabilities)
            };
        device_options_snapshot = Some(snapshot.clone());
        let request = resolve_capture_device_option_request(
            &snapshot,
            &capabilities,
            &args.device_options,
            args.sample_rate_hz,
            args.sample_limit,
            &args.channels,
        )
        .map_err(|error| {
            command_error(
                args.runtime.format,
                classify_capture_device_option_parse_error(&error),
            )
        })?;
        let validated = capabilities.validate_request(&request).map_err(|error| {
            command_error(args.runtime.format, classify_validation_error(&error))
        })?;
        validated_device_options = Some(validated);
    }
    #[cfg(debug_assertions)]
    if let Some(mode) = capture_test_fixture_mode(handle) {
        let validated_config = fixture_validated_capture_config(
            validated_device_options.as_ref(),
            &device_options_snapshot,
            &args,
        )?;
        return run_capture_with_test_fixture(
            args.runtime.format,
            handle,
            &artifact_paths,
            validated_config,
            validated_device_options,
            device_options_snapshot,
            mode,
        );
    }
    let discovery = connect_runtime(&args.runtime)?;
    let config_request = CaptureConfigRequest {
        sample_rate_hz: args.sample_rate_hz,
        sample_limit: args.sample_limit,
        enabled_channels: args.channels.iter().copied().collect::<BTreeSet<_>>(),
    };
    let run_request = CaptureRunRequest {
        selection_handle: handle,
        config: config_request,
        validated_device_options: validated_device_options.clone(),
        wait_timeout: Duration::from_millis(args.wait_timeout_ms),
        poll_interval: Duration::from_millis(args.poll_interval_ms),
    };
    let validated_config = if let Some(validated_device_options) = validated_device_options.as_ref()
    {
        validated_capture_config_from_device_options(validated_device_options)
    } else {
        discovery
            .validate_capture_config(handle, &run_request.config)
            .map_err(|error| {
                command_error(args.runtime.format, classify_capture_config_error(&error))
            })?
    };
    let capture_started_at = std::time::SystemTime::now();
    let result = discovery
        .run_capture(&run_request)
        .map_err(|error| command_error(args.runtime.format, classify_capture_error(&error)))?;
    let device_options_snapshot = if let Some(snapshot) = device_options_snapshot {
        snapshot
    } else {
        discovery
            .inspect_device_options(handle)
            .map_err(|error| command_error(args.runtime.format, classify_error(&error)))?
    };
    let export = discovery
        .export_clean_capture_vcd(&dsview_core::CaptureExportRequest {
            capture: result.clone(),
            validated_config,
            vcd_path: artifact_paths.vcd_path,
            metadata_path: Some(artifact_paths.metadata_path),
            tool_name: env!("CARGO_PKG_NAME").to_string(),
            tool_version: BUILD_VERSION.to_string(),
            capture_started_at,
            device_model: "DSLogic Plus".to_string(),
            device_stable_id: "dslogic-plus".to_string(),
            selected_handle: handle,
            validated_device_options: validated_device_options.clone(),
            device_options_snapshot,
        })
        .map_err(|error| command_error(args.runtime.format, classify_export_error(&error)))?;

    let response = CaptureResponse {
        selected_handle: args.handle,
        completion: completion_name(result.completion),
        saw_logic_packet: result.summary.saw_logic_packet,
        saw_end_packet: result.summary.saw_end_packet,
        saw_terminal_normal_end: result.summary.saw_terminal_normal_end,
        cleanup_succeeded: result.cleanup.succeeded(),
        device_options: export.metadata.device_options.clone(),
        artifacts: CaptureArtifactsResponse {
            vcd_path: export.vcd_path.display().to_string(),
            metadata_path: export.metadata_path.display().to_string(),
        },
    };
    render_capture_success(args.runtime.format, &response);
    Ok(())
}

#[cfg(debug_assertions)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CaptureTestFixtureMode {
    Success,
    ApplyFailureFilter,
}

#[cfg(debug_assertions)]
fn capture_test_fixture_mode(handle: SelectionHandle) -> Option<CaptureTestFixtureMode> {
    if handle.raw() != 7 {
        return None;
    }

    match std::env::var(TEST_DEVICE_OPTIONS_FIXTURE_ENV)
        .ok()
        .as_deref()
    {
        Some("apply-failure-filter") => Some(CaptureTestFixtureMode::ApplyFailureFilter),
        Some(_) => Some(CaptureTestFixtureMode::Success),
        None => None,
    }
}

#[cfg(debug_assertions)]
fn fixture_validated_capture_config(
    validated_device_options: Option<&dsview_core::ValidatedDeviceOptionRequest>,
    device_options_snapshot: &Option<DeviceOptionsSnapshot>,
    args: &CaptureArgs,
) -> Result<ValidatedCaptureConfig, FailedCommand> {
    if let Some(validated_device_options) = validated_device_options {
        return Ok(validated_capture_config_from_device_options(
            validated_device_options,
        ));
    }

    let channel_mode_id = device_options_snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.current.channel_mode_code)
        .ok_or_else(|| {
            command_error(
                args.runtime.format,
                fixture_error_response(
                    "capture_test_fixture_invalid",
                    "debug capture fixture is missing the current channel mode",
                ),
            )
        })?;
    Ok(ValidatedCaptureConfig {
        sample_rate_hz: args.sample_rate_hz,
        requested_sample_limit: args.sample_limit,
        effective_sample_limit: args.sample_limit,
        enabled_channels: args.channels.clone(),
        channel_mode_id,
    })
}

#[cfg(debug_assertions)]
fn run_capture_with_test_fixture(
    format: OutputFormat,
    handle: SelectionHandle,
    artifact_paths: &dsview_core::CaptureArtifactPaths,
    validated_config: ValidatedCaptureConfig,
    validated_device_options: Option<dsview_core::ValidatedDeviceOptionRequest>,
    device_options_snapshot: Option<DeviceOptionsSnapshot>,
    mode: CaptureTestFixtureMode,
) -> Result<(), FailedCommand> {
    match mode {
        CaptureTestFixtureMode::ApplyFailureFilter => {
            let error = DeviceOptionApplyFailure {
                applied_steps: vec![
                    dsview_core::DeviceOptionApplyStep::OperationMode,
                    dsview_core::DeviceOptionApplyStep::StopOption,
                    dsview_core::DeviceOptionApplyStep::ChannelMode,
                    dsview_core::DeviceOptionApplyStep::ThresholdVolts,
                ],
                failed_step: dsview_core::DeviceOptionApplyStep::Filter,
                runtime_error: RuntimeError::NativeCall {
                    operation: "ds_set_filter",
                    code: NativeErrorCode::Arg,
                },
            };
            Err(command_error(
                format,
                classify_device_option_apply_failure(&error),
            ))
        }
        CaptureTestFixtureMode::Success => {
            let device_options_snapshot = device_options_snapshot.ok_or_else(|| {
                command_error(
                    format,
                    fixture_error_response(
                        "capture_test_fixture_invalid",
                        "debug capture fixture is missing the device option snapshot",
                    ),
                )
            })?;
            let effective_device_options = validated_device_options
                .as_ref()
                .map(fixture_effective_device_option_state)
                .or_else(|| {
                    fixture_inherited_effective_device_option_state(
                        &device_options_snapshot,
                        &validated_config,
                    )
                });
            let capture = dsview_core::CaptureRunSummary {
                completion: CaptureCompletion::CleanSuccess,
                summary: fixture_acquisition_summary(),
                cleanup: fixture_capture_cleanup(),
                effective_device_options,
            };
            let export_request = dsview_core::CaptureExportRequest {
                capture: capture.clone(),
                validated_config: validated_config.clone(),
                vcd_path: artifact_paths.vcd_path.clone(),
                metadata_path: Some(artifact_paths.metadata_path.clone()),
                tool_name: env!("CARGO_PKG_NAME").to_string(),
                tool_version: BUILD_VERSION.to_string(),
                capture_started_at: std::time::UNIX_EPOCH,
                device_model: "DSLogic Plus".to_string(),
                device_stable_id: "dslogic-plus".to_string(),
                selected_handle: handle,
                validated_device_options: validated_device_options.clone(),
                device_options_snapshot: device_options_snapshot.clone(),
            };
            let facts = dsview_core::build_capture_device_option_facts(&export_request).map_err(|detail| {
                command_error(
                    format,
                    fixture_error_response(
                        "capture_test_fixture_invalid",
                        &format!("debug capture fixture failed to build device option facts: {detail}"),
                    ),
                )
            })?;
            write_capture_test_fixture_artifacts(artifact_paths, handle, &validated_config, &facts)
                .map_err(|detail| {
                    command_error(
                        format,
                        fixture_error_response("capture_test_fixture_write_failed", &detail),
                    )
                })?;
            let response = CaptureResponse {
                selected_handle: handle.raw(),
                completion: completion_name(capture.completion),
                saw_logic_packet: capture.summary.saw_logic_packet,
                saw_end_packet: capture.summary.saw_end_packet,
                saw_terminal_normal_end: capture.summary.saw_terminal_normal_end,
                cleanup_succeeded: capture.cleanup.succeeded(),
                device_options: facts,
                artifacts: CaptureArtifactsResponse {
                    vcd_path: artifact_paths.vcd_path.display().to_string(),
                    metadata_path: artifact_paths.metadata_path.display().to_string(),
                },
            };
            render_capture_success(format, &response);
            Ok(())
        }
    }
}

#[cfg(debug_assertions)]
fn fixture_acquisition_summary() -> AcquisitionSummary {
    AcquisitionSummary {
        callback_registration_active: false,
        start_status: 0,
        saw_collect_task_start: true,
        saw_device_running: true,
        saw_device_stopped: true,
        saw_terminal_normal_end: true,
        saw_terminal_end_by_detached: false,
        saw_terminal_end_by_error: false,
        terminal_event: AcquisitionTerminalEvent::NormalEnd,
        saw_logic_packet: true,
        saw_end_packet: true,
        end_packet_status: None,
        saw_end_packet_ok: true,
        saw_data_error_packet: false,
        last_error: NativeErrorCode::Ok,
        is_collecting: false,
    }
}

#[cfg(debug_assertions)]
fn fixture_capture_cleanup() -> CaptureCleanup {
    CaptureCleanup {
        callbacks_cleared: true,
        release_succeeded: true,
        ..CaptureCleanup::default()
    }
}

#[cfg(debug_assertions)]
fn fixture_effective_device_option_state(
    validated: &dsview_core::ValidatedDeviceOptionRequest,
) -> dsview_core::EffectiveDeviceOptionState {
    dsview_core::EffectiveDeviceOptionState {
        operation_mode_code: Some(validated.operation_mode_code),
        stop_option_code: validated.stop_option_code,
        channel_mode_code: Some(validated.channel_mode_code),
        threshold_volts: validated.threshold_volts,
        filter_code: validated.filter_code,
        enabled_channels: validated.enabled_channels.clone(),
        sample_limit: Some(validated.effective_sample_limit),
        sample_rate_hz: Some(validated.sample_rate_hz),
    }
}

#[cfg(debug_assertions)]
fn fixture_inherited_effective_device_option_state(
    snapshot: &DeviceOptionsSnapshot,
    validated_config: &ValidatedCaptureConfig,
) -> Option<dsview_core::EffectiveDeviceOptionState> {
    Some(dsview_core::EffectiveDeviceOptionState {
        operation_mode_code: snapshot.current.operation_mode_code,
        stop_option_code: snapshot.current.stop_option_code,
        channel_mode_code: snapshot.current.channel_mode_code,
        threshold_volts: snapshot.threshold.current_volts,
        filter_code: snapshot.current.filter_code,
        enabled_channels: validated_config.enabled_channels.clone(),
        sample_limit: Some(validated_config.effective_sample_limit),
        sample_rate_hz: Some(validated_config.sample_rate_hz),
    })
}

#[cfg(debug_assertions)]
fn write_capture_test_fixture_artifacts(
    artifact_paths: &dsview_core::CaptureArtifactPaths,
    handle: SelectionHandle,
    validated_config: &ValidatedCaptureConfig,
    facts: &CaptureDeviceOptionFacts,
) -> Result<(), String> {
    if let Some(parent) = artifact_paths.vcd_path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    if let Some(parent) = artifact_paths.metadata_path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let vcd_bytes = b"$date fixture $end\n$version dsview-cli fixture $end\n#0\n";
    fs::write(&artifact_paths.vcd_path, vcd_bytes).map_err(|error| error.to_string())?;

    let metadata = dsview_core::CaptureMetadata {
        schema_version: 2,
        tool: MetadataToolInfo {
            name: env!("CARGO_PKG_NAME").to_string(),
            version: BUILD_VERSION.to_string(),
        },
        capture: MetadataCaptureInfo {
            timestamp_utc: "1970-01-01T00:00:00Z".to_string(),
            device_model: "DSLogic Plus".to_string(),
            device_stable_id: "dslogic-plus".to_string(),
            selected_handle: handle.raw(),
            sample_rate_hz: validated_config.sample_rate_hz,
            requested_sample_limit: validated_config.requested_sample_limit,
            actual_sample_count: validated_config.effective_sample_limit,
            enabled_channels: validated_config.enabled_channels.clone(),
        },
        acquisition: MetadataAcquisitionInfo {
            completion: "clean_success".to_string(),
            terminal_event: "normal_end".to_string(),
            saw_logic_packet: true,
            saw_end_packet: true,
            end_packet_status: Some("ok".to_string()),
        },
        artifacts: MetadataArtifactInfo {
            vcd_path: artifact_paths.vcd_path.display().to_string(),
            metadata_path: artifact_paths.metadata_path.display().to_string(),
        },
        device_options: facts.clone(),
    };
    let metadata_bytes = serde_json::to_vec_pretty(&metadata).map_err(|error| error.to_string())?;
    fs::write(&artifact_paths.metadata_path, metadata_bytes).map_err(|error| error.to_string())
}

#[cfg(debug_assertions)]
fn fixture_error_response(code: &'static str, message: &str) -> ErrorResponse {
    ErrorResponse {
        code,
        message: message.to_string(),
        detail: None,
        native_error: None,
        terminal_event: None,
        cleanup: None,
    }
}

fn run_options(args: OptionsArgs) -> Result<(), FailedCommand> {
    let handle = SelectionHandle::new(args.handle)
        .ok_or_else(|| command_error(args.runtime.format, invalid_handle_error()))?;
    let discovery = connect_runtime(&args.runtime)?;
    let snapshot = discovery
        .inspect_device_options(handle)
        .map_err(|error| command_error(args.runtime.format, classify_error(&error)))?;
    let response = build_device_options_response(&snapshot);
    render_device_options_success(args.runtime.format, &response);
    Ok(())
}

fn connect_runtime(args: &SharedRuntimeArgs) -> Result<Discovery, FailedCommand> {
    Discovery::connect_auto(args.resource_dir.as_deref())
        .map_err(|error| command_error(args.format, classify_error(&error)))
}

fn classify_decode_error(error: &DecodeBringUpError) -> ErrorResponse {
    match error {
        DecodeBringUpError::CurrentExecutableUnavailable { detail } => ErrorResponse {
            code: "decode_current_executable_unavailable",
            message: format!(
                "could not determine the executable location used for decoder runtime discovery: {detail}"
            ),
            detail: Some(
                "The CLI resolves bundled `decode-runtime/` and `decoders/` relative to the executable; rerun from a normal filesystem location or pass explicit decode paths."
                    .to_string(),
            ),
            native_error: None,
            terminal_event: None,
            cleanup: None,
        },
        DecodeBringUpError::BundledRuntimeMissing {
            path,
            executable_dir,
        } => ErrorResponse {
            code: "decode_runtime_missing",
            message: format!(
                "decoder runtime `{}` was not found relative to executable directory `{}`",
                path.display(),
                executable_dir.display()
            ),
            detail: Some(
                "Build or unpack the CLI with its sibling `decode-runtime/` directory, or pass `--decode-runtime <PATH>`."
                    .to_string(),
            ),
            native_error: None,
            terminal_event: None,
            cleanup: None,
        },
        DecodeBringUpError::MissingDecoderDirectory { path } => ErrorResponse {
            code: "decoder_scripts_missing",
            message: format!("decoder scripts directory `{}` is missing", path.display()),
            detail: Some(
                "Provide `--decoder-dir <PATH>` pointing at a valid DSView decoder scripts directory."
                    .to_string(),
            ),
            native_error: None,
            terminal_event: None,
            cleanup: None,
        },
        DecodeBringUpError::UnreadableDecoderDirectory { path } => ErrorResponse {
            code: "decoder_scripts_unreadable",
            message: format!("decoder scripts directory `{}` is not readable", path.display()),
            detail: Some(
                "Check filesystem permissions or provide `--decoder-dir <PATH>` to a readable decoder scripts directory."
                    .to_string(),
            ),
            native_error: None,
            terminal_event: None,
            cleanup: None,
        },
        DecodeBringUpError::DecoderScriptsMissing { path } => ErrorResponse {
            code: "decoder_metadata_missing",
            message: format!(
                "decoder scripts directory `{}` did not yield any decoder metadata",
                path.display()
            ),
            detail: Some(
                "Check that the directory contains DSView decoder scripts and that the decoder runtime can import them."
                    .to_string(),
            ),
            native_error: None,
            terminal_event: None,
            cleanup: None,
        },
        DecodeBringUpError::UnknownDecoder { decoder_id } => ErrorResponse {
            code: "decoder_not_found",
            message: format!("decoder `{decoder_id}` was not found in the loaded registry"),
            detail: Some(
                "Run `decode list` to inspect the canonical upstream decoder ids before calling `decode inspect <decoder-id>`."
                    .to_string(),
            ),
            native_error: None,
            terminal_event: None,
            cleanup: None,
        },
        DecodeBringUpError::Runtime(runtime) => classify_decode_runtime_error(runtime),
    }
}

fn classify_decode_runtime_error(error: &DecoderRuntimeError) -> ErrorResponse {
    match error {
        DecoderRuntimeError::LibraryLoad { path, detail } => ErrorResponse {
            code: "decode_runtime_load_failed",
            message: format!("failed to load decoder runtime `{}`: {detail}", path.display()),
            detail: None,
            native_error: None,
            terminal_event: None,
            cleanup: None,
        },
        DecoderRuntimeError::SymbolLoad { path, detail } => ErrorResponse {
            code: "decode_runtime_symbol_load_failed",
            message: format!(
                "`{}` is missing required decoder runtime symbols: {detail}",
                path.display()
            ),
            detail: None,
            native_error: None,
            terminal_event: None,
            cleanup: None,
        },
        DecoderRuntimeError::BridgeNotLoaded => ErrorResponse {
            code: "decode_runtime_not_loaded",
            message: "the decoder runtime bridge is not loaded".to_string(),
            detail: None,
            native_error: None,
            terminal_event: None,
            cleanup: None,
        },
        DecoderRuntimeError::NativeCall {
            operation,
            code,
            detail,
        } => ErrorResponse {
            code: match code {
                DecoderRuntimeErrorCode::DecoderDirectory => "decoder_scripts_missing",
                DecoderRuntimeErrorCode::Python => "decode_python_failed",
                DecoderRuntimeErrorCode::DecoderLoad => "decoder_load_failed",
                DecoderRuntimeErrorCode::UnknownDecoder => "decoder_not_found",
                DecoderRuntimeErrorCode::OutOfMemory => "decode_out_of_memory",
                DecoderRuntimeErrorCode::Upstream
                | DecoderRuntimeErrorCode::Unknown(_) => "decode_runtime_failed",
            },
            message: format!("decoder runtime operation `{operation}` failed: {detail}"),
            detail: None,
            native_error: None,
            terminal_event: None,
            cleanup: None,
        },
        other => ErrorResponse {
            code: "decode_runtime_error",
            message: other.to_string(),
            detail: None,
            native_error: None,
            terminal_event: None,
            cleanup: None,
        },
    }
}

fn device_record(device: &SupportedDevice) -> DeviceRecord {
    DeviceRecord {
        handle: device.selection_handle.raw(),
        stable_id: device.stable_id,
        model: device.kind.display_name(),
        native_name: device.name.clone(),
    }
}

fn uses_device_option_validation(args: &CaptureArgs) -> bool {
    !args.channels.is_empty() || args.device_options.has_overrides()
}

#[cfg(debug_assertions)]
fn capture_test_fixture(
    handle: SelectionHandle,
) -> Option<(DeviceOptionsSnapshot, DeviceOptionValidationCapabilities)> {
    if std::env::var_os(TEST_DEVICE_OPTIONS_FIXTURE_ENV).is_none() || handle.raw() != 7 {
        return None;
    }

    Some((
        capture_test_device_options_snapshot(),
        capture_test_validation_capabilities(),
    ))
}

#[cfg(not(debug_assertions))]
fn capture_test_fixture(
    _handle: SelectionHandle,
) -> Option<(DeviceOptionsSnapshot, DeviceOptionValidationCapabilities)> {
    None
}

fn capture_test_device_options_snapshot() -> DeviceOptionsSnapshot {
    DeviceOptionsSnapshot {
        device: DeviceIdentitySnapshot {
            selection_handle: 7,
            native_handle: 42,
            stable_id: "dslogic-plus".to_string(),
            kind: "DSLogic Plus".to_string(),
            name: "DSLogic Plus".to_string(),
        },
        current: CurrentDeviceOptionValues {
            operation_mode_id: Some("operation-mode:0".to_string()),
            operation_mode_code: Some(0),
            stop_option_id: Some("stop-option:1".to_string()),
            stop_option_code: Some(1),
            filter_id: Some("filter:1".to_string()),
            filter_code: Some(1),
            channel_mode_id: Some("channel-mode:20".to_string()),
            channel_mode_code: Some(20),
        },
        operation_modes: vec![
            EnumOptionSnapshot {
                id: "operation-mode:0".to_string(),
                native_code: 0,
                label: "Buffer Mode".to_string(),
            },
            EnumOptionSnapshot {
                id: "operation-mode:1".to_string(),
                native_code: 1,
                label: "Stream Mode".to_string(),
            },
        ],
        stop_options: vec![
            EnumOptionSnapshot {
                id: "stop-option:0".to_string(),
                native_code: 0,
                label: "Immediate".to_string(),
            },
            EnumOptionSnapshot {
                id: "stop-option:1".to_string(),
                native_code: 1,
                label: "Stop after samples".to_string(),
            },
        ],
        filters: vec![
            EnumOptionSnapshot {
                id: "filter:1".to_string(),
                native_code: 1,
                label: "Off".to_string(),
            },
            EnumOptionSnapshot {
                id: "filter:2".to_string(),
                native_code: 2,
                label: "1 Sample".to_string(),
            },
        ],
        channel_modes_by_operation_mode: vec![
            ChannelModeGroupSnapshot {
                operation_mode_id: "operation-mode:0".to_string(),
                operation_mode_code: 0,
                current_channel_mode_id: Some("channel-mode:20".to_string()),
                current_channel_mode_code: Some(20),
                channel_modes: vec![
                    ChannelModeOptionSnapshot {
                        id: "channel-mode:20".to_string(),
                        native_code: 20,
                        label: "Buffer 100x16".to_string(),
                        max_enabled_channels: 16,
                    },
                    ChannelModeOptionSnapshot {
                        id: "channel-mode:21".to_string(),
                        native_code: 21,
                        label: "Buffer 200x8".to_string(),
                        max_enabled_channels: 8,
                    },
                ],
            },
            ChannelModeGroupSnapshot {
                operation_mode_id: "operation-mode:1".to_string(),
                operation_mode_code: 1,
                current_channel_mode_id: None,
                current_channel_mode_code: None,
                channel_modes: vec![ChannelModeOptionSnapshot {
                    id: "channel-mode:30".to_string(),
                    native_code: 30,
                    label: "Stream 100x16".to_string(),
                    max_enabled_channels: 16,
                }],
            },
        ],
        threshold: ThresholdCapabilitySnapshot {
            id: "threshold:vth-range".to_string(),
            kind: "voltage-range".to_string(),
            current_volts: Some(1.8),
            min_volts: 0.7,
            max_volts: 5.0,
            step_volts: 0.1,
            legacy_metadata: None,
        },
    }
}

fn capture_test_validation_capabilities() -> DeviceOptionValidationCapabilities {
    DeviceOptionValidationCapabilities {
        device: capture_test_device_options_snapshot().device,
        current: CurrentDeviceOptionValues {
            operation_mode_id: Some("operation-mode:0".to_string()),
            operation_mode_code: Some(0),
            stop_option_id: Some("stop-option:1".to_string()),
            stop_option_code: Some(1),
            filter_id: Some("filter:1".to_string()),
            filter_code: Some(1),
            channel_mode_id: Some("channel-mode:20".to_string()),
            channel_mode_code: Some(20),
        },
        total_channel_count: 16,
        hardware_sample_capacity: 16 * 4096,
        sample_limit_alignment: 1024,
        operation_modes: vec![
            OperationModeValidationCapabilities {
                id: "operation-mode:0".to_string(),
                native_code: 0,
                label: "Buffer Mode".to_string(),
                stop_option_ids: vec!["stop-option:0".to_string(), "stop-option:1".to_string()],
                channel_modes: vec![
                    dsview_core::ChannelModeValidationCapabilities {
                        id: "channel-mode:20".to_string(),
                        native_code: 20,
                        label: "Buffer 100x16".to_string(),
                        max_enabled_channels: 16,
                        supported_sample_rates: vec![100_000_000],
                    },
                    dsview_core::ChannelModeValidationCapabilities {
                        id: "channel-mode:21".to_string(),
                        native_code: 21,
                        label: "Buffer 200x8".to_string(),
                        max_enabled_channels: 8,
                        supported_sample_rates: vec![100_000_000],
                    },
                ],
            },
            OperationModeValidationCapabilities {
                id: "operation-mode:1".to_string(),
                native_code: 1,
                label: "Stream Mode".to_string(),
                stop_option_ids: vec![],
                channel_modes: vec![dsview_core::ChannelModeValidationCapabilities {
                    id: "channel-mode:30".to_string(),
                    native_code: 30,
                    label: "Stream 100x16".to_string(),
                    max_enabled_channels: 16,
                    supported_sample_rates: vec![100_000_000],
                }],
            },
        ],
        filters: vec![
            EnumOptionSnapshot {
                id: "filter:1".to_string(),
                native_code: 1,
                label: "Off".to_string(),
            },
            EnumOptionSnapshot {
                id: "filter:2".to_string(),
                native_code: 2,
                label: "1 Sample".to_string(),
            },
        ],
        threshold: ThresholdCapabilitySnapshot {
            id: "threshold:vth-range".to_string(),
            kind: "voltage-range".to_string(),
            current_volts: Some(1.8),
            min_volts: 0.7,
            max_volts: 5.0,
            step_volts: 0.1,
            legacy_metadata: None,
        },
    }
}

fn classify_artifact_path_error(error: &CaptureArtifactPathError) -> ErrorResponse {
    match error {
        CaptureArtifactPathError::InvalidVcdExtension { path } => ErrorResponse {
            code: "capture_output_path_invalid",
            message: format!(
                "VCD output path `{}` must use the .vcd extension",
                path.display()
            ),
            detail: None,
            native_error: None,
            terminal_event: None,
            cleanup: None,
        },
        CaptureArtifactPathError::InvalidMetadataExtension { path } => ErrorResponse {
            code: "capture_metadata_output_path_invalid",
            message: format!(
                "metadata output path `{}` must use the .json extension",
                path.display()
            ),
            detail: None,
            native_error: None,
            terminal_event: None,
            cleanup: None,
        },
        CaptureArtifactPathError::ConflictingArtifactPaths {
            vcd_path,
            metadata_path,
        } => ErrorResponse {
            code: "capture_artifact_paths_conflict",
            message: format!(
                "VCD output path `{}` and metadata output path `{}` must be different",
                vcd_path.display(),
                metadata_path.display()
            ),
            detail: None,
            native_error: None,
            terminal_event: None,
            cleanup: None,
        },
    }
}

fn invalid_handle_error() -> ErrorResponse {
    ErrorResponse {
        code: "invalid_selector",
        message: "--handle must be a non-zero device handle from `devices list`.".to_string(),
        detail: None,
        native_error: None,
        terminal_event: None,
        cleanup: None,
    }
}

fn classify_error(error: &BringUpError) -> ErrorResponse {
    match error {
        BringUpError::CurrentExecutableUnavailable { detail } => ErrorResponse {
            code: "current_executable_unavailable",
            message: format!(
                "could not determine the executable location used for bundled runtime discovery: {detail}"
            ),
            detail: Some(
                "The CLI resolves bundled `runtime/` and `resources/` relative to the executable; rerun from a normal filesystem location or rebuild the bundle."
                    .to_string(),
            ),
            native_error: None,
            terminal_event: None,
            cleanup: None,
        },
        BringUpError::BundledRuntimeMissing {
            path,
            executable_dir,
        } => ErrorResponse {
            code: "bundled_runtime_missing",
            message: format!(
                "bundled runtime `{}` was not found relative to executable directory `{}`",
                path.display(),
                executable_dir.display()
            ),
            detail: Some(
                "Build or unpack the CLI with its sibling runtime/ directory, or use `--resource-dir <PATH>` only to point at alternate DSLogic Plus resources."
                    .to_string(),
            ),
            native_error: None,
            terminal_event: None,
            cleanup: None,
        },
        BringUpError::MissingResourceDirectory { path } => ErrorResponse {
            code: "resource_dir_missing",
            message: format!("resource directory `{}` is missing", path.display()),
            detail: Some(
                "The CLI expects bundled resources at `resources/` next to the executable unless you pass `--resource-dir <PATH>`."
                    .to_string(),
            ),
            native_error: None,
            terminal_event: None,
            cleanup: None,
        },
        BringUpError::UnreadableResourceDirectory { path } => ErrorResponse {
            code: "resource_dir_unreadable",
            message: format!("resource directory `{}` is not readable", path.display()),
            detail: Some(
                "The CLI expects bundled resources at `resources/` next to the executable unless you pass `--resource-dir <PATH>`."
                    .to_string(),
            ),
            native_error: None,
            terminal_event: None,
            cleanup: None,
        },
        BringUpError::MissingResourceFiles { path, missing } => ErrorResponse {
            code: "resource_files_missing",
            message: format!(
                "resource directory `{}` is missing required DSLogic Plus files: {}",
                path.display(),
                missing.join(", ")
            ),
            detail: Some(
                "Check the bundled `resources/` directory next to the executable or pass `--resource-dir <PATH>` to a complete DSLogic Plus resource set."
                    .to_string(),
            ),
            native_error: None,
            terminal_event: None,
            cleanup: None,
        },
        BringUpError::UnsupportedSelection { selection_handle } => ErrorResponse {
            code: "unsupported_selection",
            message: format!("device handle `{selection_handle}` is not a supported DSLogic Plus"),
            detail: None,
            native_error: None,
            terminal_event: None,
            cleanup: None,
        },
        BringUpError::NoSupportedDevices => ErrorResponse {
            code: "no_supported_devices",
            message: "no supported DSLogic Plus devices are currently available".to_string(),
            detail: None,
            native_error: None,
            terminal_event: None,
            cleanup: None,
        },
        BringUpError::Runtime(runtime) => classify_runtime_error(runtime),
    }
}

fn classify_runtime_error(error: &RuntimeError) -> ErrorResponse {
    match error {
        RuntimeError::LibraryLoad { path, detail } => ErrorResponse {
            code: "library_load_failed",
            message: format!("failed to load `{}`: {detail}", path.display()),
            detail: None,
            native_error: None,
            terminal_event: None,
            cleanup: None,
        },
        RuntimeError::SymbolLoad { path, detail } => ErrorResponse {
            code: "symbol_load_failed",
            message: format!(
                "`{}` is missing required ds_* symbols: {detail}",
                path.display()
            ),
            detail: None,
            native_error: None,
            terminal_event: None,
            cleanup: None,
        },
        RuntimeError::BridgeNotLoaded => ErrorResponse {
            code: "runtime_not_loaded",
            message: "the native runtime bridge is not loaded".to_string(),
            detail: None,
            native_error: None,
            terminal_event: None,
            cleanup: None,
        },
        RuntimeError::NativeCall { operation, code } => ErrorResponse {
            code: match code {
                NativeErrorCode::FirmwareMissing => "firmware_missing",
                NativeErrorCode::DeviceExclusive => "device_busy",
                NativeErrorCode::FirmwareVersionLow => "device_reconnect_required",
                NativeErrorCode::DeviceUsbIo => "usb_io_error",
                NativeErrorCode::CallStatus => "call_status_error",
                _ => "native_call_failed",
            },
            message: format!(
                "{operation} failed: {} ({})",
                describe_native_error(*code),
                code.name()
            ),
            detail: None,
            native_error: Some(code.name()),
            terminal_event: None,
            cleanup: None,
        },
        other => ErrorResponse {
            code: "runtime_error",
            message: other.to_string(),
            detail: None,
            native_error: None,
            terminal_event: None,
            cleanup: None,
        },
    }
}

#[cfg_attr(not(test), allow(dead_code))]
fn classify_validation_error(error: &DeviceOptionValidationError) -> ErrorResponse {
    ErrorResponse {
        code: error.code(),
        message: error.to_string(),
        detail: None,
        native_error: None,
        terminal_event: None,
        cleanup: None,
    }
}

fn classify_capture_device_option_parse_error(
    error: &CaptureDeviceOptionParseError,
) -> ErrorResponse {
    match error {
        CaptureDeviceOptionParseError::UnsupportedOperationModeToken { token } => ErrorResponse {
            code: "operation_mode_unsupported",
            message: format!(
                "operation mode token `{token}` is not supported by the selected device"
            ),
            detail: Some(
                "Use `devices options --handle <HANDLE>` to inspect supported tokens and compatibility.".to_string(),
            ),
            native_error: None,
            terminal_event: None,
            cleanup: None,
        },
        CaptureDeviceOptionParseError::UnsupportedStopOptionToken { token } => ErrorResponse {
            code: "stop_option_unsupported",
            message: format!(
                "stop option token `{token}` is not supported by the selected device"
            ),
            detail: Some(
                "Use `devices options --handle <HANDLE>` to inspect supported tokens and compatibility.".to_string(),
            ),
            native_error: None,
            terminal_event: None,
            cleanup: None,
        },
        CaptureDeviceOptionParseError::UnsupportedChannelModeToken { token }
        | CaptureDeviceOptionParseError::AmbiguousChannelModeToken { token } => ErrorResponse {
            code: "channel_mode_unsupported",
            message: format!(
                "channel mode token `{token}` is not supported without a unique parent operation mode"
            ),
            detail: Some(
                "Use `devices options --handle <HANDLE>` to inspect supported tokens and pass `--operation-mode` when a channel mode token is ambiguous.".to_string(),
            ),
            native_error: None,
            terminal_event: None,
            cleanup: None,
        },
        CaptureDeviceOptionParseError::UnsupportedFilterToken { token } => ErrorResponse {
            code: "filter_unsupported",
            message: format!("filter token `{token}` is not supported by the selected device"),
            detail: Some(
                "Use `devices options --handle <HANDLE>` to inspect supported tokens and compatibility.".to_string(),
            ),
            native_error: None,
            terminal_event: None,
            cleanup: None,
        },
        CaptureDeviceOptionParseError::MissingCurrentOperationMode
        | CaptureDeviceOptionParseError::MissingCurrentChannelMode => ErrorResponse {
            code: "validation_runtime_error",
            message: "the selected device did not report enough current option state to validate the capture request".to_string(),
            detail: Some(
                "Re-run `devices options --handle <HANDLE>` to confirm the current device-option snapshot.".to_string(),
            ),
            native_error: None,
            terminal_event: None,
            cleanup: None,
        },
        CaptureDeviceOptionParseError::ConflictingOperationModeInference { sources } => ErrorResponse {
            code: "operation_mode_required",
            message: format!(
                "the provided child option tokens imply conflicting operation modes via {}",
                sources.join(" and ")
            ),
            detail: Some(
                "Pass `--operation-mode` explicitly or inspect `devices options --handle <HANDLE>` for compatible token combinations.".to_string(),
            ),
            native_error: None,
            terminal_event: None,
            cleanup: None,
        },
    }
}

fn classify_capture_config_error(error: &CaptureConfigError) -> ErrorResponse {
    let validation_code = match error {
        CaptureConfigError::Runtime(_) => {
            DeviceOptionValidationError::Runtime(error.to_string()).code()
        }
        CaptureConfigError::EmptySampleRate => DeviceOptionValidationError::EmptySampleRate.code(),
        CaptureConfigError::EmptySampleLimit => {
            DeviceOptionValidationError::EmptySampleLimit.code()
        }
        CaptureConfigError::NoEnabledChannels => {
            DeviceOptionValidationError::NoEnabledChannels.code()
        }
        CaptureConfigError::ChannelOutOfRange { channel } => {
            DeviceOptionValidationError::ChannelOutOfRange {
                channel: *channel,
                total_channel_count: channel.saturating_add(1),
            }
            .code()
        }
        CaptureConfigError::UnknownChannelMode { mode } => {
            DeviceOptionValidationError::UnknownChannelMode {
                channel_mode_id: format!("channel-mode:{mode}"),
            }
            .code()
        }
        CaptureConfigError::UnsupportedSampleRate {
            sample_rate_hz,
            mode_name,
        } => DeviceOptionValidationError::UnsupportedSampleRate {
            sample_rate_hz: *sample_rate_hz,
            channel_mode_id: mode_name.clone(),
        }
        .code(),
        CaptureConfigError::TooManyEnabledChannels {
            enabled_channel_count,
            max_enabled_channels,
        } => DeviceOptionValidationError::TooManyEnabledChannels {
            enabled_channel_count: *enabled_channel_count,
            max_enabled_channels: *max_enabled_channels,
        }
        .code(),
        CaptureConfigError::SampleLimitExceedsCapacity {
            effective_sample_limit,
            maximum_sample_limit,
            enabled_channel_count,
        } => DeviceOptionValidationError::SampleLimitExceedsCapacity {
            effective_sample_limit: *effective_sample_limit,
            maximum_sample_limit: *maximum_sample_limit,
            enabled_channel_count: *enabled_channel_count,
        }
        .code(),
    };

    ErrorResponse {
        code: validation_code,
        message: error.to_string(),
        detail: None,
        native_error: None,
        terminal_event: None,
        cleanup: None,
    }
}

pub(crate) fn classify_capture_error(error: &CaptureRunError) -> ErrorResponse {
    match error {
        CaptureRunError::BringUp(error) => classify_error(error),
        CaptureRunError::DeviceOptionApply(error) => classify_device_option_apply_failure(error),
        CaptureRunError::EnvironmentNotReady => ErrorResponse {
            code: "capture_environment_not_ready",
            message: "capture preflight did not confirm USB permissions, runtime resources, and open/config readiness".to_string(),
            detail: None,
            native_error: None,
            terminal_event: None,
            cleanup: None,
        },
        CaptureRunError::StartFailed {
            code,
            last_error,
            cleanup,
        } => ErrorResponse {
            code: "capture_start_failed",
            message: format!(
                "capture start failed with {} ({})",
                describe_native_error(*code),
                code.name()
            ),
            detail: Some(format!(
                "release attempted after start failure; native last_error was {}",
                last_error.name()
            )),
            native_error: Some(code.name()),
            terminal_event: None,
            cleanup: Some(capture_cleanup_response(cleanup)),
        },
        CaptureRunError::RunFailed { summary, cleanup } => ErrorResponse {
            code: "capture_run_failed",
            message: "capture reached a terminal runtime error after start".to_string(),
            detail: Some(format!(
                "terminal event {} with native last_error {}",
                terminal_event_name(summary),
                summary.last_error.name()
            )),
            native_error: Some(summary.last_error.name()),
            terminal_event: Some(terminal_event_name(summary)),
            cleanup: Some(capture_cleanup_response(cleanup)),
        },
        CaptureRunError::Detached { summary, cleanup } => ErrorResponse {
            code: "capture_detached",
            message: "capture ended because the device detached during acquisition".to_string(),
            detail: Some(format!(
                "terminal event {} with native last_error {}",
                terminal_event_name(summary),
                summary.last_error.name()
            )),
            native_error: Some(summary.last_error.name()),
            terminal_event: Some(terminal_event_name(summary)),
            cleanup: Some(capture_cleanup_response(cleanup)),
        },
        CaptureRunError::Incomplete { summary, cleanup } => ErrorResponse {
            code: "capture_incomplete",
            message: "capture did not satisfy the clean finite-capture success rule".to_string(),
            detail: Some(format!(
                "logic_packet={}, end_packet={}, terminal_event={}",
                summary.saw_logic_packet,
                summary.saw_end_packet,
                terminal_event_name(summary)
            )),
            native_error: Some(summary.last_error.name()),
            terminal_event: Some(terminal_event_name(summary)),
            cleanup: Some(capture_cleanup_response(cleanup)),
        },
        CaptureRunError::Timeout { summary, cleanup } => ErrorResponse {
            code: "capture_timeout",
            message: "capture did not reach natural completion before the timeout".to_string(),
            detail: Some(format!(
                "forced cleanup after bounded wait; terminal_event={}",
                terminal_event_name(summary)
            )),
            native_error: Some(summary.last_error.name()),
            terminal_event: Some(terminal_event_name(summary)),
            cleanup: Some(capture_cleanup_response(cleanup)),
        },
        CaptureRunError::CleanupFailed {
            during,
            summary,
            cleanup,
        } => ErrorResponse {
            code: "capture_cleanup_failed",
            message: format!(
                "capture cleanup failed after {during}; the device may require re-open validation"
            ),
            detail: Some(format!(
                "terminal_event={}, stop_error={:?}, release_error={:?}",
                terminal_event_name(summary),
                cleanup.stop_error,
                cleanup.release_error
            )),
            native_error: Some(summary.last_error.name()),
            terminal_event: Some(terminal_event_name(summary)),
            cleanup: Some(capture_cleanup_response(cleanup)),
        },
    }
}

fn classify_device_option_apply_failure(error: &DeviceOptionApplyFailure) -> ErrorResponse {
    let applied_steps = error
        .applied_steps
        .iter()
        .map(|step| step.as_str())
        .collect::<Vec<_>>()
        .join(",");
    let native_error = match &error.runtime_error {
        RuntimeError::NativeCall { code, .. } => Some(code.name()),
        _ => None,
    };

    ErrorResponse {
        code: "device_option_apply_failed",
        message: format!(
            "device option apply failed before acquisition started at {}",
            error.failed_step.as_str()
        ),
        detail: Some(format!(
            "applied_steps={applied_steps}; failed_step={}; runtime_error={}",
            error.failed_step.as_str(),
            error.runtime_error
        )),
        native_error,
        terminal_event: None,
        cleanup: None,
    }
}

pub(crate) fn classify_export_error(error: &CaptureExportError) -> ErrorResponse {
    match error {
        CaptureExportError::CaptureNotExportable { completion } => ErrorResponse {
            code: "capture_not_exportable",
            message: format!(
                "capture completion `{}` is not eligible for artifact generation",
                completion_name(*completion)
            ),
            detail: None,
            native_error: None,
            terminal_event: None,
            cleanup: None,
        },
        CaptureExportError::InvalidArtifactPaths(error) => classify_artifact_path_error(error),
        CaptureExportError::ExportFailed { path, kind, detail } => ErrorResponse {
            code: match kind {
                dsview_core::CaptureExportFailureKind::Precondition { .. } => {
                    "capture_export_precondition_failed"
                }
                dsview_core::CaptureExportFailureKind::Runtime => "capture_export_failed",
            },
            message: format!("failed to write VCD artifact `{}`", path.display()),
            detail: Some(detail.clone()),
            native_error: None,
            terminal_event: None,
            cleanup: None,
        },
        CaptureExportError::MetadataSerializationFailed { path, detail } => ErrorResponse {
            code: "capture_metadata_serialization_failed",
            message: format!("failed to serialize metadata artifact `{}`", path.display()),
            detail: Some(detail.clone()),
            native_error: None,
            terminal_event: None,
            cleanup: None,
        },
        CaptureExportError::MetadataWriteFailed { path, detail } => ErrorResponse {
            code: "capture_metadata_write_failed",
            message: format!("failed to write metadata artifact `{}`", path.display()),
            detail: Some(detail.clone()),
            native_error: None,
            terminal_event: None,
            cleanup: None,
        },
    }
}

fn completion_name(completion: CaptureCompletion) -> &'static str {
    match completion {
        CaptureCompletion::CleanSuccess => "clean_success",
        CaptureCompletion::StartFailure => "start_failure",
        CaptureCompletion::Detached => "detach",
        CaptureCompletion::RunFailure => "run_failure",
        CaptureCompletion::Incomplete => "incomplete",
        CaptureCompletion::CleanupFailure => "cleanup_failure",
        CaptureCompletion::Timeout => "timeout",
    }
}

fn terminal_event_name(summary: &AcquisitionSummary) -> &'static str {
    match summary.terminal_event {
        AcquisitionTerminalEvent::None => "none",
        AcquisitionTerminalEvent::NormalEnd => "normal_end",
        AcquisitionTerminalEvent::EndByDetached => "end_by_detached",
        AcquisitionTerminalEvent::EndByError => "end_by_error",
        AcquisitionTerminalEvent::Unknown(_) => "unknown",
    }
}

fn capture_cleanup_response(cleanup: &CaptureCleanup) -> CaptureCleanupResponse {
    CaptureCleanupResponse {
        stop_attempted: cleanup.stop_attempted,
        stop_succeeded: cleanup.stop_succeeded,
        callbacks_cleared: cleanup.callbacks_cleared,
        release_succeeded: cleanup.release_succeeded,
        stop_error: cleanup.stop_error.clone(),
        clear_callbacks_error: cleanup.clear_callbacks_error.clone(),
        release_error: cleanup.release_error.clone(),
    }
}

fn command_error(format: OutputFormat, error: ErrorResponse) -> FailedCommand {
    FailedCommand { format, error }
}

fn render_device_list_success(format: OutputFormat, response: &DeviceListResponse) {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(response).unwrap());
        }
        OutputFormat::Text => {
            for device in &response.devices {
                println!(
                    "{}\t{}\t{}\t{}",
                    device.handle, device.stable_id, device.model, device.native_name
                );
            }
        }
    }
}

fn render_json_or_ok<T: Serialize>(format: OutputFormat, payload: &T) {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(payload).unwrap());
        }
        OutputFormat::Text => {
            println!("ok");
        }
    }
}

fn render_capture_success(format: OutputFormat, response: &CaptureResponse) {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(response).unwrap());
        }
        OutputFormat::Text => {
            println!("{}", capture_success_text(response));
        }
    }
}

fn render_decode_list_success(format: OutputFormat, response: &DecodeListResponse) {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(response).unwrap());
        }
        OutputFormat::Text => {
            println!("{}", render_decode_list_text(response));
        }
    }
}

fn render_decode_inspect_success(format: OutputFormat, response: &DecodeInspectResponse) {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(response).unwrap());
        }
        OutputFormat::Text => {
            println!("{}", render_decode_inspect_text(response));
        }
    }
}

fn render_device_options_success(format: OutputFormat, response: &DeviceOptionsResponse) {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(response).unwrap());
        }
        OutputFormat::Text => {
            println!("{}", render_device_options_text(response));
        }
    }
}

fn render_effective_capture_options_text(facts: &CaptureDeviceOptionFacts) -> Vec<String> {
    let effective = &facts.effective;
    vec![
        "effective options:".to_string(),
        format!("operation mode: {}", effective.operation_mode_id),
        format!(
            "stop option: {}",
            effective.stop_option_id.as_deref().unwrap_or("none")
        ),
        format!("channel mode: {}", effective.channel_mode_id),
        format!(
            "enabled channels: {}",
            effective
                .enabled_channels
                .iter()
                .map(u16::to_string)
                .collect::<Vec<_>>()
                .join(",")
        ),
        format!(
            "threshold volts: {}",
            effective
                .threshold_volts
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string())
        ),
        format!(
            "filter: {}",
            effective.filter_id.as_deref().unwrap_or("none")
        ),
        format!("sample rate: {}", effective.sample_rate_hz),
        format!("sample limit: {}", effective.sample_limit),
    ]
}

pub(crate) fn capture_success_text(response: &CaptureResponse) -> String {
    let mut lines = vec![format!("capture {}", response.completion)];
    lines.extend(render_effective_capture_options_text(
        &response.device_options,
    ));
    lines.push(format!("vcd {}", response.artifacts.vcd_path));
    lines.push(format!("metadata {}", response.artifacts.metadata_path));
    lines.join("\n")
}

fn render_error(format: OutputFormat, error: &ErrorResponse) {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(error).unwrap());
        }
        OutputFormat::Text => {
            eprintln!("{}: {}", error.code, error.message);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use dsview_cli::capture_device_options::resolve_capture_device_option_request;
    use dsview_core::{
        AcquisitionSummary, AcquisitionTerminalEvent, CaptureConfigError, ChannelModeGroupSnapshot,
        ChannelModeOptionSnapshot, CurrentDeviceOptionValues, DeviceIdentitySnapshot,
        DeviceOptionValidationCapabilities, EnumOptionSnapshot,
        OperationModeValidationCapabilities, SelectionHandle, ThresholdCapabilitySnapshot,
    };

    #[test]
    fn invalid_handle_maps_to_stable_error_code() {
        assert_eq!(invalid_handle_error().code, "invalid_selector");
    }

    #[test]
    fn bundled_runtime_missing_maps_to_stable_error_code() {
        let error = classify_error(&BringUpError::BundledRuntimeMissing {
            path: PathBuf::from("runtime/libdsview_runtime.so"),
            executable_dir: PathBuf::from("bundle"),
        });
        assert_eq!(error.code, "bundled_runtime_missing");
        assert!(error.message.contains("bundled runtime"));
    }

    #[test]
    fn no_supported_devices_maps_to_stable_error_code() {
        let error = classify_error(&BringUpError::NoSupportedDevices);
        assert_eq!(
            error,
            ErrorResponse {
                code: "no_supported_devices",
                message: "no supported DSLogic Plus devices are currently available".to_string(),
                detail: None,
                native_error: None,
                terminal_event: None,
                cleanup: None,
            }
        );
    }

    #[test]
    fn firmware_missing_native_error_maps_to_machine_readable_code() {
        let error = classify_runtime_error(&RuntimeError::NativeCall {
            operation: "ds_active_device",
            code: NativeErrorCode::FirmwareMissing,
        });
        assert_eq!(error.code, "firmware_missing");
        assert!(error.message.contains("Firmware not exist!"));
    }

    #[test]
    fn current_executable_unavailable_maps_to_stable_error_code() {
        let error = classify_error(&BringUpError::CurrentExecutableUnavailable {
            detail: "sandbox denied path lookup".to_string(),
        });
        assert_eq!(error.code, "current_executable_unavailable");
    }

    #[test]
    fn capture_run_failure_uses_terminal_event_and_native_error() {
        let error = classify_capture_error(&CaptureRunError::RunFailed {
            summary: AcquisitionSummary {
                callback_registration_active: true,
                start_status: NativeErrorCode::Ok.raw(),
                saw_collect_task_start: true,
                saw_device_running: true,
                saw_device_stopped: true,
                saw_terminal_normal_end: false,
                saw_terminal_end_by_detached: false,
                saw_terminal_end_by_error: true,
                terminal_event: AcquisitionTerminalEvent::EndByError,
                saw_logic_packet: true,
                saw_end_packet: false,
                end_packet_status: None,
                saw_end_packet_ok: false,
                saw_data_error_packet: true,
                last_error: NativeErrorCode::Generic,
                is_collecting: false,
            },
            cleanup: CaptureCleanup {
                callbacks_cleared: true,
                release_succeeded: true,
                ..CaptureCleanup::default()
            },
        });

        assert_eq!(error.code, "capture_run_failed");
        assert_eq!(error.terminal_event, Some("end_by_error"));
        assert_eq!(error.native_error, Some("SR_ERR"));
    }

    #[test]
    fn capture_detached_uses_distinct_terminal_event_shape() {
        let error = classify_capture_error(&CaptureRunError::Detached {
            summary: AcquisitionSummary {
                callback_registration_active: true,
                start_status: NativeErrorCode::Ok.raw(),
                saw_collect_task_start: true,
                saw_device_running: true,
                saw_device_stopped: true,
                saw_terminal_normal_end: false,
                saw_terminal_end_by_detached: true,
                saw_terminal_end_by_error: false,
                terminal_event: AcquisitionTerminalEvent::EndByDetached,
                saw_logic_packet: true,
                saw_end_packet: false,
                end_packet_status: None,
                saw_end_packet_ok: false,
                saw_data_error_packet: false,
                last_error: NativeErrorCode::DeviceUsbIo,
                is_collecting: false,
            },
            cleanup: CaptureCleanup {
                callbacks_cleared: true,
                release_succeeded: true,
                ..CaptureCleanup::default()
            },
        });

        assert_eq!(error.code, "capture_detached");
        assert_eq!(error.terminal_event, Some("end_by_detached"));
        assert_eq!(error.native_error, Some("SR_ERR_DEVICE_USB_IO_ERROR"));
    }

    #[test]
    fn capture_timeout_maps_to_stable_error_code() {
        let error = classify_capture_error(&CaptureRunError::Timeout {
            summary: AcquisitionSummary {
                callback_registration_active: true,
                start_status: NativeErrorCode::Ok.raw(),
                saw_collect_task_start: true,
                saw_device_running: true,
                saw_device_stopped: false,
                saw_terminal_normal_end: false,
                saw_terminal_end_by_detached: false,
                saw_terminal_end_by_error: false,
                terminal_event: AcquisitionTerminalEvent::None,
                saw_logic_packet: false,
                saw_end_packet: false,
                end_packet_status: None,
                saw_end_packet_ok: false,
                saw_data_error_packet: false,
                last_error: NativeErrorCode::Ok,
                is_collecting: true,
            },
            cleanup: CaptureCleanup {
                stop_attempted: true,
                stop_succeeded: true,
                callbacks_cleared: true,
                release_succeeded: true,
                ..CaptureCleanup::default()
            },
        });
        assert_eq!(error.code, "capture_timeout");
        assert_eq!(error.terminal_event, Some("none"));
    }

    #[test]
    fn device_option_request_preserves_current_values_when_flags_are_omitted() {
        let snapshot = sample_device_options_snapshot();
        let capabilities = sample_validation_capabilities();
        let args = CaptureDeviceOptionArgs::default();

        let request = resolve_capture_device_option_request(
            &snapshot,
            &capabilities,
            &args,
            100_000_000,
            4096,
            &[0, 1, 2, 3],
        )
        .expect("request should resolve from current values");

        assert_eq!(request.operation_mode_id, "operation-mode:0");
        assert_eq!(request.stop_option_id.as_deref(), Some("stop-option:1"));
        assert_eq!(request.channel_mode_id, "channel-mode:20");
        assert_eq!(request.threshold_volts, Some(1.8));
        assert_eq!(request.filter_id.as_deref(), Some("filter:0"));
    }

    #[test]
    fn device_option_request_infers_operation_mode_from_channel_mode_token() {
        let mut snapshot = sample_device_options_snapshot();
        snapshot.current.operation_mode_id = Some("operation-mode:0".to_string());
        snapshot.current.operation_mode_code = Some(0);
        snapshot.current.channel_mode_id = Some("channel-mode:20".to_string());
        snapshot.current.channel_mode_code = Some(20);
        snapshot.current.stop_option_id = None;
        snapshot.current.stop_option_code = None;

        let capabilities = sample_validation_capabilities();
        let args = CaptureDeviceOptionArgs {
            channel_mode: Some("stream-100x16".to_string()),
            ..CaptureDeviceOptionArgs::default()
        };

        let request = resolve_capture_device_option_request(
            &snapshot,
            &capabilities,
            &args,
            100_000_000,
            4096,
            &[0, 1, 2, 3],
        )
        .expect("channel mode token should infer the unique parent mode");

        assert_eq!(request.operation_mode_id, "operation-mode:1");
        assert_eq!(request.channel_mode_id, "channel-mode:30");
    }

    #[test]
    fn explicit_operation_mode_is_not_overwritten_by_inference_candidates() {
        let snapshot = sample_device_options_snapshot();
        let capabilities = sample_validation_capabilities();
        let args = CaptureDeviceOptionArgs {
            operation_mode: Some("buffer".to_string()),
            channel_mode: Some("stream-100x16".to_string()),
            ..CaptureDeviceOptionArgs::default()
        };

        let request = resolve_capture_device_option_request(
            &snapshot,
            &capabilities,
            &args,
            100_000_000,
            4096,
            &[0, 1, 2, 3],
        )
        .expect("explicit operation mode should be preserved");

        assert_eq!(request.operation_mode_id, "operation-mode:0");
        assert_eq!(request.channel_mode_id, "channel-mode:30");
    }

    #[test]
    fn operation_mode_change_does_not_inherit_incompatible_stop_option() {
        let snapshot = sample_device_options_snapshot();
        let capabilities = sample_validation_capabilities();
        let args = CaptureDeviceOptionArgs {
            operation_mode: Some("stream".to_string()),
            channel_mode: Some("stream-100x16".to_string()),
            ..CaptureDeviceOptionArgs::default()
        };

        let request = resolve_capture_device_option_request(
            &snapshot,
            &capabilities,
            &args,
            100_000_000,
            4096,
            &[0, 1, 2, 3],
        )
        .expect("operation mode change should not inherit incompatible stop option");

        assert_eq!(request.operation_mode_id, "operation-mode:1");
        assert_eq!(request.channel_mode_id, "channel-mode:30");
        assert_eq!(request.stop_option_id, None);
    }

    #[test]
    fn operation_mode_change_defaults_to_first_compatible_channel_mode() {
        let snapshot = sample_device_options_snapshot();
        let capabilities = sample_validation_capabilities();
        let args = CaptureDeviceOptionArgs {
            operation_mode: Some("stream".to_string()),
            ..CaptureDeviceOptionArgs::default()
        };

        let request = resolve_capture_device_option_request(
            &snapshot,
            &capabilities,
            &args,
            100_000_000,
            4096,
            &[0, 1, 2, 3],
        )
        .expect("operation mode change should pick a compatible default channel mode");

        assert_eq!(request.operation_mode_id, "operation-mode:1");
        assert_eq!(request.channel_mode_id, "channel-mode:30");
        assert_eq!(request.stop_option_id, None);
    }

    #[test]
    fn device_option_request_carries_channels_into_enabled_channels() {
        let snapshot = sample_device_options_snapshot();
        let capabilities = sample_validation_capabilities();
        let args = CaptureDeviceOptionArgs::default();

        let request = resolve_capture_device_option_request(
            &snapshot,
            &capabilities,
            &args,
            100_000_000,
            4096,
            &[0, 2, 4, 6],
        )
        .expect("request should include selected channels");

        assert_eq!(
            request.enabled_channels,
            BTreeSet::from([0_u16, 2_u16, 4_u16, 6_u16])
        );
    }

    #[test]
    fn channels_only_capture_uses_device_option_validation() {
        let args = sample_capture_args();

        assert!(uses_device_option_validation(&args));
    }

    #[test]
    fn capture_device_option_parse_errors_use_stable_validation_codes() {
        let response = classify_capture_device_option_parse_error(
            &dsview_cli::capture_device_options::CaptureDeviceOptionParseError::UnsupportedChannelModeToken {
                token: "invalid-token".to_string(),
            },
        );

        assert_eq!(response.code, "channel_mode_unsupported");
        assert!(response.message.contains("invalid-token"));
    }

    #[test]
    fn capture_environment_not_ready_maps_to_stable_error_code() {
        let error = classify_capture_error(&CaptureRunError::EnvironmentNotReady);
        assert_eq!(error.code, "capture_environment_not_ready");
        assert_eq!(error.cleanup, None);
    }

    #[test]
    fn classify_device_option_apply_failure_includes_applied_and_failed_steps() {
        let error = classify_capture_error(&CaptureRunError::DeviceOptionApply(
            DeviceOptionApplyFailure {
                applied_steps: vec![
                    dsview_core::DeviceOptionApplyStep::OperationMode,
                    dsview_core::DeviceOptionApplyStep::StopOption,
                ],
                failed_step: dsview_core::DeviceOptionApplyStep::Filter,
                runtime_error: RuntimeError::NativeCall {
                    operation: "ds_set_filter",
                    code: NativeErrorCode::Arg,
                },
            },
        ));

        assert_eq!(error.code, "device_option_apply_failed");
        assert_eq!(error.native_error, Some("SR_ERR_ARG"));
        assert!(error
            .detail
            .as_deref()
            .unwrap()
            .contains("applied_steps=operation_mode,stop_option"));
        assert!(error
            .detail
            .as_deref()
            .unwrap()
            .contains("failed_step=filter"));
    }

    fn sample_device_options_snapshot() -> dsview_core::DeviceOptionsSnapshot {
        dsview_core::DeviceOptionsSnapshot {
            device: DeviceIdentitySnapshot {
                selection_handle: SelectionHandle::new(7).unwrap().raw(),
                native_handle: 77,
                stable_id: "dslogic-plus".to_string(),
                kind: "DSLogic Plus".to_string(),
                name: "DSLogic Plus".to_string(),
            },
            current: CurrentDeviceOptionValues {
                operation_mode_id: Some("operation-mode:0".to_string()),
                operation_mode_code: Some(0),
                stop_option_id: Some("stop-option:1".to_string()),
                stop_option_code: Some(1),
                filter_id: Some("filter:0".to_string()),
                filter_code: Some(0),
                channel_mode_id: Some("channel-mode:20".to_string()),
                channel_mode_code: Some(20),
            },
            operation_modes: vec![
                EnumOptionSnapshot {
                    id: "operation-mode:0".to_string(),
                    native_code: 0,
                    label: "Buffer Mode".to_string(),
                },
                EnumOptionSnapshot {
                    id: "operation-mode:1".to_string(),
                    native_code: 1,
                    label: "Stream Mode".to_string(),
                },
            ],
            stop_options: vec![
                EnumOptionSnapshot {
                    id: "stop-option:0".to_string(),
                    native_code: 0,
                    label: "Immediate".to_string(),
                },
                EnumOptionSnapshot {
                    id: "stop-option:1".to_string(),
                    native_code: 1,
                    label: "Stop after samples".to_string(),
                },
            ],
            filters: vec![
                EnumOptionSnapshot {
                    id: "filter:0".to_string(),
                    native_code: 0,
                    label: "Off".to_string(),
                },
                EnumOptionSnapshot {
                    id: "filter:1".to_string(),
                    native_code: 1,
                    label: "1 Sample".to_string(),
                },
            ],
            channel_modes_by_operation_mode: vec![
                ChannelModeGroupSnapshot {
                    operation_mode_id: "operation-mode:0".to_string(),
                    operation_mode_code: 0,
                    current_channel_mode_id: Some("channel-mode:20".to_string()),
                    current_channel_mode_code: Some(20),
                    channel_modes: vec![
                        ChannelModeOptionSnapshot {
                            id: "channel-mode:20".to_string(),
                            native_code: 20,
                            label: "Buffer 100x16".to_string(),
                            max_enabled_channels: 16,
                        },
                        ChannelModeOptionSnapshot {
                            id: "channel-mode:21".to_string(),
                            native_code: 21,
                            label: "Buffer 200x8".to_string(),
                            max_enabled_channels: 8,
                        },
                    ],
                },
                ChannelModeGroupSnapshot {
                    operation_mode_id: "operation-mode:1".to_string(),
                    operation_mode_code: 1,
                    current_channel_mode_id: None,
                    current_channel_mode_code: None,
                    channel_modes: vec![ChannelModeOptionSnapshot {
                        id: "channel-mode:30".to_string(),
                        native_code: 30,
                        label: "Stream 100x16".to_string(),
                        max_enabled_channels: 16,
                    }],
                },
            ],
            threshold: ThresholdCapabilitySnapshot {
                id: "threshold:vth-range".to_string(),
                kind: "voltage-range".to_string(),
                current_volts: Some(1.8),
                min_volts: 0.0,
                max_volts: 5.0,
                step_volts: 0.1,
                legacy_metadata: None,
            },
        }
    }

    fn sample_validation_capabilities() -> DeviceOptionValidationCapabilities {
        DeviceOptionValidationCapabilities {
            device: sample_device_options_snapshot().device,
            current: CurrentDeviceOptionValues {
                operation_mode_id: Some("operation-mode:0".to_string()),
                operation_mode_code: Some(0),
                stop_option_id: Some("stop-option:1".to_string()),
                stop_option_code: Some(1),
                filter_id: Some("filter:0".to_string()),
                filter_code: Some(0),
                channel_mode_id: Some("channel-mode:20".to_string()),
                channel_mode_code: Some(20),
            },
            total_channel_count: 16,
            hardware_sample_capacity: 16 * 4096,
            sample_limit_alignment: 1024,
            operation_modes: vec![
                OperationModeValidationCapabilities {
                    id: "operation-mode:0".to_string(),
                    native_code: 0,
                    label: "Buffer Mode".to_string(),
                    stop_option_ids: vec!["stop-option:0".to_string(), "stop-option:1".to_string()],
                    channel_modes: vec![],
                },
                OperationModeValidationCapabilities {
                    id: "operation-mode:1".to_string(),
                    native_code: 1,
                    label: "Stream Mode".to_string(),
                    stop_option_ids: vec![],
                    channel_modes: vec![],
                },
            ],
            filters: vec![],
            threshold: ThresholdCapabilitySnapshot {
                id: "threshold:vth-range".to_string(),
                kind: "voltage-range".to_string(),
                current_volts: Some(1.8),
                min_volts: 0.0,
                max_volts: 5.0,
                step_volts: 0.1,
                legacy_metadata: None,
            },
        }
    }

    fn sample_capture_args() -> CaptureArgs {
        CaptureArgs {
            runtime: SharedRuntimeArgs {
                resource_dir: None,
                format: OutputFormat::Json,
            },
            handle: 7,
            sample_rate_hz: 100_000_000,
            sample_limit: 4096,
            channels: vec![0, 1, 2, 3],
            device_options: CaptureDeviceOptionArgs::default(),
            output: PathBuf::from("artifacts/run.vcd"),
            metadata_output: None,
            wait_timeout_ms: 10_000,
            poll_interval_ms: 50,
        }
    }

    fn sample_capture_device_option_facts() -> dsview_core::CaptureDeviceOptionFacts {
        dsview_core::CaptureDeviceOptionFacts {
            requested: dsview_core::CaptureDeviceOptionSnapshot {
                operation_mode_id: "operation-mode:0".to_string(),
                stop_option_id: Some("stop-option:1".to_string()),
                channel_mode_id: "channel-mode:20".to_string(),
                enabled_channels: vec![0, 1, 2, 3],
                threshold_volts: Some(1.8),
                filter_id: Some("filter:0".to_string()),
                sample_rate_hz: 100_000_000,
                sample_limit: 4097,
            },
            effective: dsview_core::CaptureDeviceOptionSnapshot {
                operation_mode_id: "operation-mode:1".to_string(),
                stop_option_id: None,
                channel_mode_id: "channel-mode:30".to_string(),
                enabled_channels: vec![0, 2, 4, 6],
                threshold_volts: Some(2.4),
                filter_id: Some("filter:1".to_string()),
                sample_rate_hz: 200_000_000,
                sample_limit: 4096,
            },
        }
    }

    #[test]
    fn capture_response_json_includes_requested_and_effective_device_options() {
        let response = CaptureResponse {
            selected_handle: 7,
            completion: "clean_success",
            saw_logic_packet: true,
            saw_end_packet: true,
            saw_terminal_normal_end: true,
            cleanup_succeeded: true,
            artifacts: CaptureArtifactsResponse {
                vcd_path: "/tmp/run.vcd".to_string(),
                metadata_path: "/tmp/run.json".to_string(),
            },
            device_options: sample_capture_device_option_facts(),
        };

        let payload = serde_json::to_value(response).unwrap();

        assert_eq!(payload["device_options"]["requested"]["sample_limit"], 4097);
        assert_eq!(
            payload["device_options"]["effective"]["operation_mode_id"],
            "operation-mode:1"
        );
        assert_eq!(
            payload["device_options"]["effective"]["enabled_channels"],
            serde_json::json!([0, 2, 4, 6])
        );
    }

    #[test]
    fn render_capture_success_text_lists_effective_device_options() {
        let response = CaptureResponse {
            selected_handle: 7,
            completion: "clean_success",
            saw_logic_packet: true,
            saw_end_packet: true,
            saw_terminal_normal_end: true,
            cleanup_succeeded: true,
            artifacts: CaptureArtifactsResponse {
                vcd_path: "/tmp/run.vcd".to_string(),
                metadata_path: "/tmp/run.json".to_string(),
            },
            device_options: sample_capture_device_option_facts(),
        };

        let text = capture_success_text(&response);

        assert!(text.contains("capture clean_success"));
        assert!(text.contains("effective options:"));
        assert!(text.contains("operation mode: operation-mode:1"));
        assert!(text.contains("stop option: none"));
        assert!(text.contains("channel mode: channel-mode:30"));
        assert!(text.contains("enabled channels: 0,2,4,6"));
        assert!(text.contains("threshold volts: 2.4"));
        assert!(text.contains("filter: filter:1"));
        assert!(text.contains("sample rate: 200000000"));
        assert!(text.contains("sample limit: 4096"));
        assert!(text.contains("vcd /tmp/run.vcd"));
        assert!(text.contains("metadata /tmp/run.json"));
        assert!(!text.contains("requested options:"));
        assert!(!text.contains("operation-mode:0"));
    }

    #[test]
    fn stable_validation_error_codes() {
        let cases = [
            (
                DeviceOptionValidationError::UnknownOperationMode {
                    operation_mode_id: "operation-mode:404".to_string(),
                },
                "operation_mode_unsupported",
            ),
            (
                DeviceOptionValidationError::StopOptionIncompatibleWithMode {
                    stop_option_id: "stop-option:1".to_string(),
                    operation_mode_id: "operation-mode:202".to_string(),
                },
                "stop_option_incompatible",
            ),
            (
                DeviceOptionValidationError::UnknownChannelMode {
                    channel_mode_id: "channel-mode:404".to_string(),
                },
                "channel_mode_unsupported",
            ),
            (
                DeviceOptionValidationError::UnsupportedSampleRate {
                    sample_rate_hz: 123,
                    channel_mode_id: "channel-mode:11".to_string(),
                },
                "sample_rate_unsupported",
            ),
            (
                DeviceOptionValidationError::ThresholdStepInvalid {
                    threshold_volts: 1.85,
                    min_volts: 0.0,
                    step_volts: 0.1,
                },
                "threshold_step_invalid",
            ),
            (
                DeviceOptionValidationError::UnknownFilter {
                    filter_id: "filter:404".to_string(),
                },
                "filter_unsupported",
            ),
        ];

        for (error, expected_code) in cases {
            let response = classify_validation_error(&error);
            assert_eq!(response.code, expected_code);
            assert!(response.message.contains('`') || !response.message.is_empty());
        }
    }

    #[test]
    fn sample_rate_unsupported_maps_to_stable_validation_error_code() {
        let response =
            classify_validation_error(&DeviceOptionValidationError::UnsupportedSampleRate {
                sample_rate_hz: 200_000_000,
                channel_mode_id: "channel-mode:11".to_string(),
            });

        assert_eq!(response.code, "sample_rate_unsupported");
        assert!(response.message.contains("200000000"));
        assert_eq!(response.native_error, None);
        assert_eq!(response.terminal_event, None);
        assert_eq!(response.cleanup, None);
    }

    #[test]
    fn capture_reports_stable_validation_error_for_channel_limit_exceeded() {
        let response =
            classify_validation_error(&DeviceOptionValidationError::TooManyEnabledChannels {
                enabled_channel_count: 5,
                max_enabled_channels: 4,
            });

        assert_eq!(response.code, "enabled_channels_exceed_mode_limit");
    }

    #[test]
    fn sample_limit_exceeds_capacity_maps_to_stable_validation_error_code() {
        let response =
            classify_validation_error(&DeviceOptionValidationError::SampleLimitExceedsCapacity {
                effective_sample_limit: 4096,
                maximum_sample_limit: 3072,
                enabled_channel_count: 4,
            });

        assert_eq!(response.code, "sample_limit_exceeds_capacity");
    }

    #[test]
    fn threshold_out_of_range_maps_to_stable_validation_error_code() {
        let response =
            classify_validation_error(&DeviceOptionValidationError::ThresholdOutOfRange {
                threshold_volts: 6.0,
                min_volts: 0.0,
                max_volts: 5.0,
            });

        assert_eq!(response.code, "threshold_out_of_range");
    }

    #[test]
    fn filter_unsupported_maps_to_stable_validation_error_code() {
        let response = classify_validation_error(&DeviceOptionValidationError::UnknownFilter {
            filter_id: "filter:404".to_string(),
        });

        assert_eq!(response.code, "filter_unsupported");
    }

    #[test]
    fn stop_option_incompatible_maps_to_stable_validation_error_code() {
        let response = classify_validation_error(
            &DeviceOptionValidationError::StopOptionIncompatibleWithMode {
                stop_option_id: "stop-option:1".to_string(),
                operation_mode_id: "operation-mode:202".to_string(),
            },
        );

        assert_eq!(response.code, "stop_option_incompatible");
    }

    #[test]
    fn capture_config_sample_rate_validation_maps_to_stable_validation_error_code() {
        let response = classify_capture_config_error(&CaptureConfigError::UnsupportedSampleRate {
            sample_rate_hz: 200_000_000,
            mode_name: "Buffer 100x16".to_string(),
        });

        assert_eq!(response.code, "sample_rate_unsupported");
    }

    #[test]
    fn capture_config_enabled_channel_limit_maps_to_stable_validation_error_code() {
        let response = classify_capture_config_error(&CaptureConfigError::TooManyEnabledChannels {
            enabled_channel_count: 5,
            max_enabled_channels: 4,
        });

        assert_eq!(response.code, "enabled_channels_exceed_mode_limit");
        assert!(response.message.contains("exceeds"));
    }

    #[test]
    fn capture_config_sample_limit_maps_to_stable_validation_error_code() {
        let response =
            classify_capture_config_error(&CaptureConfigError::SampleLimitExceedsCapacity {
                effective_sample_limit: 4096,
                maximum_sample_limit: 3072,
                enabled_channel_count: 4,
            });

        assert_eq!(response.code, "sample_limit_exceeds_capacity");
        assert!(response.message.contains("effective sample limit"));
    }

    #[test]
    fn capture_start_failure_preserves_cleanup_detail() {
        let error = classify_capture_error(&CaptureRunError::StartFailed {
            code: NativeErrorCode::DeviceExclusive,
            last_error: NativeErrorCode::DeviceExclusive,
            cleanup: CaptureCleanup {
                stop_attempted: true,
                stop_succeeded: true,
                callbacks_cleared: true,
                release_succeeded: true,
                ..CaptureCleanup::default()
            },
        });

        assert_eq!(error.code, "capture_start_failed");
        assert_eq!(error.native_error, Some("SR_ERR_DEVICE_IS_EXCLUSIVE"));
        assert!(error.cleanup.is_some());
    }

    #[test]
    fn capture_incomplete_mentions_logic_and_end_markers() {
        let error = classify_capture_error(&CaptureRunError::Incomplete {
            summary: AcquisitionSummary {
                callback_registration_active: true,
                start_status: NativeErrorCode::Ok.raw(),
                saw_collect_task_start: true,
                saw_device_running: true,
                saw_device_stopped: true,
                saw_terminal_normal_end: true,
                saw_terminal_end_by_detached: false,
                saw_terminal_end_by_error: false,
                terminal_event: AcquisitionTerminalEvent::NormalEnd,
                saw_logic_packet: false,
                saw_end_packet: false,
                end_packet_status: None,
                saw_end_packet_ok: false,
                saw_data_error_packet: false,
                last_error: NativeErrorCode::Ok,
                is_collecting: false,
            },
            cleanup: CaptureCleanup {
                callbacks_cleared: true,
                release_succeeded: true,
                ..CaptureCleanup::default()
            },
        });

        assert_eq!(error.code, "capture_incomplete");
        assert!(error.detail.unwrap().contains("logic_packet=false"));
    }

    #[test]
    fn capture_cleanup_failure_uses_stable_code() {
        let error = classify_capture_error(&CaptureRunError::CleanupFailed {
            during: "clean_success",
            summary: AcquisitionSummary {
                callback_registration_active: true,
                start_status: NativeErrorCode::Ok.raw(),
                saw_collect_task_start: true,
                saw_device_running: true,
                saw_device_stopped: true,
                saw_terminal_normal_end: true,
                saw_terminal_end_by_detached: false,
                saw_terminal_end_by_error: false,
                terminal_event: AcquisitionTerminalEvent::NormalEnd,
                saw_logic_packet: true,
                saw_end_packet: true,
                end_packet_status: None,
                saw_end_packet_ok: true,
                saw_data_error_packet: false,
                last_error: NativeErrorCode::AlreadyDone,
                is_collecting: false,
            },
            cleanup: CaptureCleanup {
                stop_attempted: true,
                stop_succeeded: false,
                callbacks_cleared: false,
                release_succeeded: false,
                stop_error: Some("stop failed".to_string()),
                release_error: Some("release failed".to_string()),
                ..CaptureCleanup::default()
            },
        });

        assert_eq!(error.code, "capture_cleanup_failed");
        assert_eq!(error.terminal_event, Some("normal_end"));
    }

    #[test]
    fn capture_completion_names_are_machine_readable() {
        assert_eq!(
            completion_name(CaptureCompletion::CleanSuccess),
            "clean_success"
        );
        assert_eq!(
            completion_name(CaptureCompletion::RunFailure),
            "run_failure"
        );
        assert_eq!(completion_name(CaptureCompletion::Detached), "detach");
        assert_eq!(completion_name(CaptureCompletion::Timeout), "timeout");
    }
}
