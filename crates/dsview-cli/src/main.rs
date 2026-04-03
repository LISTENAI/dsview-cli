use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand, ValueEnum};
use dsview_core::{
    describe_native_error, BringUpError, DeviceHandle, Discovery, NativeErrorCode, RuntimeError,
    SupportedDevice,
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

#[derive(Serialize, Debug, PartialEq, Eq)]
struct ErrorResponse {
    code: &'static str,
    message: String,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Devices(args) => match args.command {
            DeviceCommand::List(args) => run_list(args),
            DeviceCommand::Open(args) => run_open(args),
        },
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
    let handle = DeviceHandle::new(args.handle)
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
        handle: device.handle.raw(),
        stable_id: device.stable_id,
        model: device.kind.display_name(),
        native_name: device.name.clone(),
    }
}

fn invalid_handle_error() -> ErrorResponse {
    ErrorResponse {
        code: "invalid_selector",
        message: "--handle must be a non-zero device handle from `devices list`.".to_string(),
    }
}

fn missing_runtime_selector_error() -> ErrorResponse {
    ErrorResponse {
        code: "runtime_selector_missing",
        message: "pass either --library <PATH> or --use-source-runtime.".to_string(),
    }
}

fn classify_error(error: &BringUpError) -> ErrorResponse {
    match error {
        BringUpError::SourceRuntimeUnavailable => ErrorResponse {
            code: "source_runtime_unavailable",
            message: "this build does not include a source-built DSView runtime library".to_string(),
        },
        BringUpError::MissingResourceDirectory { path } => ErrorResponse {
            code: "resource_dir_missing",
            message: format!("resource directory `{}` is missing", path.display()),
        },
        BringUpError::UnreadableResourceDirectory { path } => ErrorResponse {
            code: "resource_dir_unreadable",
            message: format!("resource directory `{}` is not readable", path.display()),
        },
        BringUpError::MissingResourceFiles { path, missing } => ErrorResponse {
            code: "resource_files_missing",
            message: format!(
                "resource directory `{}` is missing required files: {}",
                path.display(),
                missing.join(", ")
            ),
        },
        BringUpError::UnsupportedSelection { handle } => ErrorResponse {
            code: "unsupported_selection",
            message: format!("device handle `{handle}` is not a supported DSLogic Plus"),
        },
        BringUpError::NoSupportedDevices => ErrorResponse {
            code: "no_supported_devices",
            message: "no supported DSLogic Plus devices are currently available".to_string(),
        },
        BringUpError::Runtime(runtime) => classify_runtime_error(runtime),
    }
}

fn classify_runtime_error(error: &RuntimeError) -> ErrorResponse {
    match error {
        RuntimeError::LibraryLoad { path, detail } => ErrorResponse {
            code: "library_load_failed",
            message: format!("failed to load `{}`: {detail}", path.display()),
        },
        RuntimeError::SymbolLoad { path, detail } => ErrorResponse {
            code: "symbol_load_failed",
            message: format!("`{}` is missing required ds_* symbols: {detail}", path.display()),
        },
        RuntimeError::BridgeNotLoaded => ErrorResponse {
            code: "runtime_not_loaded",
            message: "the native runtime bridge is not loaded".to_string(),
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
            message: format!("{operation} failed: {} ({})", describe_native_error(*code), code.name()),
        },
        other => ErrorResponse {
            code: "runtime_error",
            message: other.to_string(),
        },
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
                    println!("{}\t{}\t{}\t{}", device.handle, device.stable_id, device.model, device.native_name);
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

    #[test]
    fn invalid_handle_maps_to_stable_error_code() {
        assert_eq!(invalid_handle_error().code, "invalid_selector");
    }

    #[test]
    fn missing_runtime_selector_maps_to_stable_error_code() {
        assert_eq!(missing_runtime_selector_error().code, "runtime_selector_missing");
    }

    #[test]
    fn no_supported_devices_maps_to_stable_error_code() {
        let error = classify_error(&BringUpError::NoSupportedDevices);
        assert_eq!(
            error,
            ErrorResponse {
                code: "no_supported_devices",
                message: "no supported DSLogic Plus devices are currently available".to_string(),
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
}
