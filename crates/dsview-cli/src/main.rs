use std::collections::BTreeSet;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

use clap::{Args, Parser, Subcommand, ValueEnum};
use dsview_core::{
    AcquisitionSummary, AcquisitionTerminalEvent, BringUpError, CaptureCleanup,
    CaptureCompletion, CaptureConfigRequest, CaptureExportError, CaptureRunError,
    CaptureRunRequest, Discovery, NativeErrorCode, RuntimeError, SelectionHandle,
    SupportedDevice, describe_native_error,
};
use serde::Serialize;

#[derive(Parser, Debug)]
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
}

#[derive(Args, Debug)]
struct SharedRuntimeArgs {
    #[arg(long, value_name = "PATH", conflicts_with = "use_source_runtime")]
    library: Option<PathBuf>,
    #[arg(long = "use-source-runtime", default_value_t = false)]
    use_source_runtime: bool,
    #[arg(long = "resource-dir", value_name = "PATH")]
    resource_dir: PathBuf,
    #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
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
    #[arg(long, value_name = "HANDLE")]
    handle: u64,
}

#[derive(Args, Debug)]
struct CaptureArgs {
    #[command(flatten)]
    runtime: SharedRuntimeArgs,
    #[arg(long, value_name = "HANDLE")]
    handle: u64,
    #[arg(long = "sample-rate-hz", value_name = "HZ")]
    sample_rate_hz: u64,
    #[arg(long = "sample-limit", value_name = "SAMPLES")]
    sample_limit: u64,
    #[arg(long = "channels", value_delimiter = ',', value_name = "IDX[,IDX...]")]
    channels: Vec<u16>,
    #[arg(long = "output", value_name = "PATH")]
    output: PathBuf,
    #[arg(long = "wait-timeout-ms", default_value_t = 10_000)]
    wait_timeout_ms: u64,
    #[arg(long = "poll-interval-ms", default_value_t = 50)]
    poll_interval_ms: u64,
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
    render_success(args.runtime.format, &response, &response.devices);
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

    render_success(args.runtime.format, &response, &[]);
    Ok(())
}

fn run_capture(args: CaptureArgs) -> Result<(), FailedCommand> {
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
        .validate_capture_config(&run_request.config)
        .map_err(|error| command_error(args.runtime.format, classify_runtime_error(&RuntimeError::InvalidArgument(error.to_string()))))?;
    let result = discovery
        .run_capture(&run_request)
        .map_err(|error| command_error(args.runtime.format, classify_capture_error(&error)))?;
    let export = discovery
        .export_clean_capture_vcd(&dsview_core::CaptureExportRequest {
            capture: result.clone(),
            validated_config,
            vcd_path: args.output.clone(),
            tool_name: env!("CARGO_PKG_NAME").to_string(),
            tool_version: env!("CARGO_PKG_VERSION").to_string(),
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
    render_success(args.runtime.format, &response, &[]);
    Ok(())
}

fn connect_runtime(args: &SharedRuntimeArgs) -> Result<Discovery, FailedCommand> {
    let result = if args.use_source_runtime {
        Discovery::connect_auto(&args.resource_dir)
    } else if let Some(library) = &args.library {
        Discovery::connect(library, &args.resource_dir)
    } else {
        return Err(command_error(args.format, missing_runtime_selector_error()));
    };

    result.map_err(|error| command_error(args.format, classify_error(&error)))
}

fn device_record(device: &SupportedDevice) -> DeviceRecord {
    DeviceRecord {
        handle: device.selection_handle.raw(),
        stable_id: device.stable_id,
        model: device.kind.display_name(),
        native_name: device.name.clone(),
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

fn missing_runtime_selector_error() -> ErrorResponse {
    ErrorResponse {
        code: "runtime_selector_missing",
        message: "pass either --library <PATH> or --use-source-runtime.".to_string(),
        detail: None,
        native_error: None,
        terminal_event: None,
        cleanup: None,
    }
}

fn classify_error(error: &BringUpError) -> ErrorResponse {
    match error {
        BringUpError::SourceRuntimeUnavailable => ErrorResponse {
            code: "source_runtime_unavailable",
            message: "this build does not include a source-built DSView runtime library"
                .to_string(),
            detail: None,
            native_error: None,
            terminal_event: None,
            cleanup: None,
        },
        BringUpError::MissingResourceDirectory { path } => ErrorResponse {
            code: "resource_dir_missing",
            message: format!("resource directory `{}` is missing", path.display()),
            detail: None,
            native_error: None,
            terminal_event: None,
            cleanup: None,
        },
        BringUpError::UnreadableResourceDirectory { path } => ErrorResponse {
            code: "resource_dir_unreadable",
            message: format!("resource directory `{}` is not readable", path.display()),
            detail: None,
            native_error: None,
            terminal_event: None,
            cleanup: None,
        },
        BringUpError::MissingResourceFiles { path, missing } => ErrorResponse {
            code: "resource_files_missing",
            message: format!(
                "resource directory `{}` is missing required files: {}",
                path.display(),
                missing.join(", ")
            ),
            detail: None,
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

fn render_success<T: Serialize>(format: OutputFormat, payload: &T, text_devices: &[DeviceRecord]) {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(payload).unwrap());
        }
        OutputFormat::Text => {
            if text_devices.is_empty() {
                println!("ok");
            } else {
                for device in text_devices {
                    println!(
                        "{}\t{}\t{}\t{}",
                        device.handle, device.stable_id, device.model, device.native_name
                    );
                }
            }
        }
    }
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

    use dsview_core::{AcquisitionSummary, AcquisitionTerminalEvent};

    #[test]
    fn invalid_handle_maps_to_stable_error_code() {
        assert_eq!(invalid_handle_error().code, "invalid_selector");
    }

    #[test]
    fn missing_runtime_selector_maps_to_stable_error_code() {
        assert_eq!(
            missing_runtime_selector_error().code,
            "runtime_selector_missing"
        );
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
    fn source_runtime_unavailable_maps_to_stable_error_code() {
        let error = classify_error(&BringUpError::SourceRuntimeUnavailable);
        assert_eq!(error.code, "source_runtime_unavailable");
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
    fn capture_environment_not_ready_maps_to_stable_error_code() {
        let error = classify_capture_error(&CaptureRunError::EnvironmentNotReady);
        assert_eq!(error.code, "capture_environment_not_ready");
        assert_eq!(error.cleanup, None);
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
        assert_eq!(completion_name(CaptureCompletion::RunFailure), "run_failure");
        assert_eq!(completion_name(CaptureCompletion::Detached), "detach");
        assert_eq!(completion_name(CaptureCompletion::Timeout), "timeout");
    }
}
