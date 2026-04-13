use std::collections::BTreeSet;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

use clap::{Args, Parser, Subcommand, ValueEnum};
use dsview_cli::{
    build_device_options_response, render_device_options_text, DeviceOptionsResponse,
};
use dsview_core::{
    describe_native_error, resolve_capture_artifact_paths, AcquisitionSummary,
    AcquisitionTerminalEvent, BringUpError, CaptureArtifactPathError, CaptureCleanup,
    CaptureCompletion, CaptureConfigError, CaptureConfigRequest, CaptureExportError,
    CaptureRunError, CaptureRunRequest, DeviceOptionValidationError, Discovery, NativeErrorCode,
    RuntimeError, SelectionHandle, SupportedDevice,
};
use serde::Serialize;

const BUILD_VERSION: &str = match option_env!("DSVIEW_BUILD_VERSION") {
    Some(version) => version,
    None => env!("CARGO_PKG_VERSION"),
};

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
    let discovery = connect_runtime(&args.runtime)?;
    let handle = SelectionHandle::new(args.handle)
        .ok_or_else(|| command_error(args.runtime.format, invalid_handle_error()))?;
    let config_request = CaptureConfigRequest {
        sample_rate_hz: args.sample_rate_hz,
        sample_limit: args.sample_limit,
        enabled_channels: args.channels.iter().copied().collect::<BTreeSet<_>>(),
    };
    let run_request = CaptureRunRequest {
        selection_handle: handle,
        config: config_request,
        wait_timeout: Duration::from_millis(args.wait_timeout_ms),
        poll_interval: Duration::from_millis(args.poll_interval_ms),
    };
    let validated_config = discovery
        .validate_capture_config(handle, &run_request.config)
        .map_err(|error| {
            command_error(args.runtime.format, classify_capture_config_error(&error))
        })?;
    let result = discovery
        .run_capture(&run_request)
        .map_err(|error| command_error(args.runtime.format, classify_capture_error(&error)))?;
    let export = discovery
        .export_clean_capture_vcd(&dsview_core::CaptureExportRequest {
            capture: result.clone(),
            validated_config,
            vcd_path: artifact_paths.vcd_path,
            metadata_path: Some(artifact_paths.metadata_path),
            tool_name: env!("CARGO_PKG_NAME").to_string(),
            tool_version: BUILD_VERSION.to_string(),
            capture_started_at: std::time::SystemTime::now(),
            device_model: "DSLogic Plus".to_string(),
            device_stable_id: "dslogic-plus".to_string(),
            selected_handle: handle,
        })
        .map_err(|error| command_error(args.runtime.format, classify_export_error(&error)))?;

    let response = CaptureResponse {
        selected_handle: args.handle,
        completion: completion_name(result.completion),
        saw_logic_packet: result.summary.saw_logic_packet,
        saw_end_packet: result.summary.saw_end_packet,
        saw_terminal_normal_end: result.summary.saw_terminal_normal_end,
        cleanup_succeeded: result.cleanup.succeeded(),
        artifacts: CaptureArtifactsResponse {
            vcd_path: export.vcd_path.display().to_string(),
            metadata_path: export.metadata_path.display().to_string(),
        },
    };
    render_capture_success(args.runtime.format, &response);
    Ok(())
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

fn device_record(device: &SupportedDevice) -> DeviceRecord {
    DeviceRecord {
        handle: device.selection_handle.raw(),
        stable_id: device.stable_id,
        model: device.kind.display_name(),
        native_name: device.name.clone(),
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
            println!(
                "{}",
                capture_success_text(
                    response.completion,
                    &response.artifacts.vcd_path,
                    &response.artifacts.metadata_path,
                )
            );
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

pub(crate) fn capture_success_text(
    completion: &str,
    vcd_path: &str,
    metadata_path: &str,
) -> String {
    format!("capture {completion}\nvcd {vcd_path}\nmetadata {metadata_path}")
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
        AcquisitionSummary, AcquisitionTerminalEvent, CaptureConfigError,
        ChannelModeGroupSnapshot, ChannelModeOptionSnapshot, CurrentDeviceOptionValues,
        DeviceIdentitySnapshot, DeviceOptionValidationCapabilities,
        EnumOptionSnapshot, OperationModeValidationCapabilities, SelectionHandle,
        ThresholdCapabilitySnapshot,
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
                    stop_option_ids: vec![
                        "stop-option:0".to_string(),
                        "stop-option:1".to_string(),
                    ],
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
    fn enabled_channels_exceed_mode_limit_maps_to_stable_validation_error_code() {
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
