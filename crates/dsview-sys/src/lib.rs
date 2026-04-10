//! Raw native integration boundary for DSView CLI.
//!
//! This crate is the only allowed home for unsafe FFI when Phase 1 adds
//! bindings to `DSView/libsigrok4DSL`.

use std::ffi::{CStr, CString};
use std::fmt;
use std::fs;
use std::os::raw::{c_char, c_int};
use std::path::{Path, PathBuf};

use thiserror::Error;

/// Returns the platform-specific runtime library filename.
///
/// This is the shared naming contract between build.rs, packaging helpers,
/// and runtime discovery logic.
pub fn runtime_library_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "dsview_runtime.dll"
    } else if cfg!(target_os = "macos") {
        "libdsview_runtime.dylib"
    } else {
        "libdsview_runtime.so"
    }
}

#[cfg(dsview_runtime_smoke_available)]
unsafe extern "C" {
    /// Public frontend symbol exported by `DSView/libsigrok4DSL`.
    pub fn sr_get_lib_version_string() -> *const c_char;
}

unsafe extern "C" {
    fn dsview_bridge_load_library(path: *const c_char) -> c_int;
    fn dsview_bridge_unload_library();
    fn dsview_bridge_is_loaded() -> c_int;
    fn dsview_bridge_last_loader_error() -> *const c_char;
    fn dsview_bridge_ds_lib_init() -> c_int;
    fn dsview_bridge_ds_lib_exit() -> c_int;
    fn dsview_bridge_ds_set_firmware_resource_dir(dir: *const c_char);
    fn dsview_bridge_ds_get_device_list(
        out_list: *mut *mut RawDeviceBaseInfo,
        out_count: *mut c_int,
    ) -> c_int;
    fn dsview_bridge_free_device_list(list: *mut RawDeviceBaseInfo);
    fn dsview_bridge_ds_active_device(handle: u64) -> c_int;
    fn dsview_bridge_ds_release_actived_device() -> c_int;
    fn dsview_bridge_ds_get_last_error() -> c_int;
    fn dsview_bridge_ds_get_actived_device_init_status(status: *mut c_int) -> c_int;
    fn dsview_bridge_ds_get_current_samplerate(value: *mut u64) -> c_int;
    fn dsview_bridge_ds_get_current_sample_limit(value: *mut u64) -> c_int;
    fn dsview_bridge_ds_get_total_channel_count(value: *mut c_int) -> c_int;
    fn dsview_bridge_ds_get_valid_channel_count(value: *mut c_int) -> c_int;
    fn dsview_bridge_ds_get_current_channel_mode(value: *mut c_int) -> c_int;
    fn dsview_bridge_ds_get_hw_depth(value: *mut u64) -> c_int;
    fn dsview_bridge_ds_get_vth(value: *mut f64) -> c_int;
    fn dsview_bridge_ds_get_samplerates(out_list: *mut RawSamplerateList) -> c_int;
    fn dsview_bridge_ds_get_channel_modes(
        out_modes: *mut RawChannelMode,
        max_modes: c_int,
        out_count: *mut c_int,
    ) -> c_int;
    fn dsview_bridge_ds_get_device_options(
        out_snapshot: *mut RawDeviceOptionsSnapshot,
    ) -> c_int;
    fn dsview_bridge_ds_set_samplerate(value: u64) -> c_int;
    fn dsview_bridge_ds_set_sample_limit(value: u64) -> c_int;
    fn dsview_bridge_ds_enable_channel(channel_index: c_int, enable: c_int) -> c_int;
    fn dsview_bridge_ds_register_acquisition_callbacks() -> c_int;
    fn dsview_bridge_ds_clear_acquisition_callbacks() -> c_int;
    fn dsview_bridge_ds_start_collect() -> c_int;
    fn dsview_bridge_ds_stop_collect() -> c_int;
    fn dsview_bridge_ds_is_collecting(value: *mut c_int) -> c_int;
    fn dsview_bridge_ds_reset_acquisition_summary() -> c_int;
    fn dsview_bridge_ds_get_acquisition_summary(out_summary: *mut RawAcquisitionSummary) -> c_int;
    fn dsview_bridge_ds_export_recorded_vcd(
        request: *const RawVcdExportRequest,
        out_buffer: *mut RawExportBuffer,
    ) -> c_int;
    fn dsview_bridge_render_vcd_from_samples(
        request: *const RawVcdExportRequest,
        sample_bytes: *const u8,
        sample_bytes_len: usize,
        unitsize: u16,
        out_buffer: *mut RawExportBuffer,
    ) -> c_int;
    fn dsview_bridge_render_vcd_from_logic_packets(
        request: *const RawVcdExportRequest,
        sample_bytes: *const u8,
        sample_bytes_len: usize,
        logic_packet_lengths: *const usize,
        logic_packet_count: usize,
        unitsize: u16,
        out_buffer: *mut RawExportBuffer,
    ) -> c_int;
    fn dsview_bridge_free_export_buffer(buffer: *mut RawExportBuffer);
}

const SR_OK: i32 = 0;
const SR_ERR: i32 = 1;
const SR_ERR_MALLOC: i32 = 2;
const SR_ERR_ARG: i32 = 3;
const SR_ERR_BUG: i32 = 4;
const SR_ERR_SAMPLERATE: i32 = 5;
const SR_ERR_NA: i32 = 6;
const SR_ERR_DEVICE_CLOSED: i32 = 7;
const SR_ERR_CALL_STATUS: i32 = 8;
const SR_ERR_HAVE_DONE: i32 = 9;
const SR_ERR_FIRMWARE_NOT_EXIST: i32 = 10;
const SR_ERR_DEVICE_IS_EXCLUSIVE: i32 = 11;
const SR_ERR_DEVICE_FIRMWARE_VERSION_LOW: i32 = 12;
const SR_ERR_DEVICE_USB_IO_ERROR: i32 = 13;

const DSVIEW_BRIDGE_ERR_ARG: i32 = -1;
const DSVIEW_BRIDGE_ERR_NOT_LOADED: i32 = -2;
const DSVIEW_BRIDGE_ERR_DLOPEN: i32 = -3;
const DSVIEW_BRIDGE_ERR_DLSYM: i32 = -4;
const DSVIEW_EXPORT_ERR_GENERIC: i32 = -100;
const DSVIEW_EXPORT_ERR_NO_STREAM: i32 = -101;
const DSVIEW_EXPORT_ERR_OVERFLOW: i32 = -102;
const DSVIEW_EXPORT_ERR_BAD_END_STATUS: i32 = -103;
const DSVIEW_EXPORT_ERR_MISSING_SAMPLERATE: i32 = -104;
const DSVIEW_EXPORT_ERR_NO_ENABLED_CHANNELS: i32 = -105;
const DSVIEW_EXPORT_ERR_OUTPUT_MODULE: i32 = -106;
const DSVIEW_EXPORT_ERR_RUNTIME: i32 = -107;

const DEVICE_NAME_CAPACITY: usize = 150;
const OPTION_LABEL_CAPACITY: usize = 64;
const OPTION_VALUE_CAPACITY: usize = 16;
const CHANNEL_MODE_GROUP_CAPACITY: usize = 8;
const CHANNEL_MODE_CAPACITY: usize = 16;
const THRESHOLD_KIND_CAPACITY: usize = 32;
const END_PACKET_STATUS_UNKNOWN: i32 = -1;

#[repr(C)]
#[derive(Clone, Copy)]
struct RawDeviceBaseInfo {
    handle: u64,
    name: [u8; DEVICE_NAME_CAPACITY],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct RawChannelMode {
    id: i32,
    name: [u8; 64],
    max_enabled_channels: u16,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct RawOptionValue {
    code: i32,
    label: [u8; OPTION_LABEL_CAPACITY],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct RawChannelModeGroup {
    operation_mode_code: i32,
    channel_mode_count: u16,
    channel_modes: [RawChannelMode; CHANNEL_MODE_CAPACITY],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct RawThresholdRange {
    kind: [u8; THRESHOLD_KIND_CAPACITY],
    id: [u8; OPTION_LABEL_CAPACITY],
    has_current_volts: c_int,
    current_volts: f64,
    min_volts: f64,
    max_volts: f64,
    step_volts: f64,
    has_current_legacy_code: c_int,
    current_legacy_code: i32,
    legacy_option_count: u16,
    legacy_options: [RawOptionValue; OPTION_VALUE_CAPACITY],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct RawDeviceOptionsSnapshot {
    has_current_operation_mode: c_int,
    current_operation_mode_code: i32,
    operation_mode_count: u16,
    operation_modes: [RawOptionValue; OPTION_VALUE_CAPACITY],
    has_current_stop_option: c_int,
    current_stop_option_code: i32,
    stop_option_count: u16,
    stop_options: [RawOptionValue; OPTION_VALUE_CAPACITY],
    has_current_filter: c_int,
    current_filter_code: i32,
    filter_count: u16,
    filters: [RawOptionValue; OPTION_VALUE_CAPACITY],
    has_current_channel_mode: c_int,
    current_channel_mode_code: i32,
    channel_mode_group_count: u16,
    channel_mode_groups: [RawChannelModeGroup; CHANNEL_MODE_GROUP_CAPACITY],
    threshold: RawThresholdRange,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct RawSamplerateList {
    count: u32,
    values: [u64; 64],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct RawVcdExportRequest {
    samplerate_hz: u64,
    enabled_channels: *const u16,
    enabled_channel_count: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct RawExportBuffer {
    data: *mut u8,
    len: usize,
    sample_count: u64,
    packet_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VcdExportRequest {
    pub samplerate_hz: u64,
    pub enabled_channels: Vec<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VcdExport {
    pub bytes: Vec<u8>,
    pub sample_count: u64,
    pub packet_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VcdExportFacts {
    pub sample_count: u64,
    pub packet_count: usize,
    pub output_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportErrorCode {
    Generic,
    NoStream,
    Overflow,
    BadEndStatus,
    MissingSamplerate,
    NoEnabledChannels,
    OutputModuleUnavailable,
    Runtime,
    Unknown(i32),
}

impl ExportErrorCode {
    fn from_raw(raw: i32) -> Self {
        match raw {
            DSVIEW_EXPORT_ERR_GENERIC => Self::Generic,
            DSVIEW_EXPORT_ERR_NO_STREAM => Self::NoStream,
            DSVIEW_EXPORT_ERR_OVERFLOW => Self::Overflow,
            DSVIEW_EXPORT_ERR_BAD_END_STATUS => Self::BadEndStatus,
            DSVIEW_EXPORT_ERR_MISSING_SAMPLERATE => Self::MissingSamplerate,
            DSVIEW_EXPORT_ERR_NO_ENABLED_CHANNELS => Self::NoEnabledChannels,
            DSVIEW_EXPORT_ERR_OUTPUT_MODULE => Self::OutputModuleUnavailable,
            DSVIEW_EXPORT_ERR_RUNTIME => Self::Runtime,
            other => Self::Unknown(other),
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Generic => "export_generic",
            Self::NoStream => "export_no_stream",
            Self::Overflow => "export_overflow",
            Self::BadEndStatus => "export_bad_end_status",
            Self::MissingSamplerate => "export_missing_samplerate",
            Self::NoEnabledChannels => "export_no_enabled_channels",
            Self::OutputModuleUnavailable => "export_output_module_unavailable",
            Self::Runtime => "export_runtime",
            Self::Unknown(_) => "export_unknown",
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct RawAcquisitionSummary {
    callback_registration_active: c_int,
    start_status: c_int,
    saw_collect_task_start: c_int,
    saw_device_running: c_int,
    saw_device_stopped: c_int,
    saw_terminal_normal_end: c_int,
    saw_terminal_end_by_detached: c_int,
    saw_terminal_end_by_error: c_int,
    terminal_event: c_int,
    saw_logic_packet: c_int,
    saw_end_packet: c_int,
    end_packet_status: c_int,
    saw_end_packet_ok: c_int,
    saw_data_error_packet: c_int,
    last_error: c_int,
    is_collecting: c_int,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcquisitionTerminalEvent {
    None,
    NormalEnd,
    EndByDetached,
    EndByError,
    Unknown(i32),
}

impl AcquisitionTerminalEvent {
    fn from_raw(raw: i32) -> Self {
        match raw {
            0 => Self::None,
            1 => Self::NormalEnd,
            2 => Self::EndByDetached,
            3 => Self::EndByError,
            other => Self::Unknown(other),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcquisitionPacketStatus {
    Ok,
    SourceError,
    DataError,
    Unknown(i32),
}

impl AcquisitionPacketStatus {
    fn from_raw(raw: i32) -> Self {
        match raw {
            0 => Self::Ok,
            1 => Self::SourceError,
            2 => Self::DataError,
            other => Self::Unknown(other),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AcquisitionSummary {
    pub callback_registration_active: bool,
    pub start_status: i32,
    pub saw_collect_task_start: bool,
    pub saw_device_running: bool,
    pub saw_device_stopped: bool,
    pub saw_terminal_normal_end: bool,
    pub saw_terminal_end_by_detached: bool,
    pub saw_terminal_end_by_error: bool,
    pub terminal_event: AcquisitionTerminalEvent,
    pub saw_logic_packet: bool,
    pub saw_end_packet: bool,
    pub end_packet_status: Option<AcquisitionPacketStatus>,
    pub saw_end_packet_ok: bool,
    pub saw_data_error_packet: bool,
    pub last_error: NativeErrorCode,
    pub is_collecting: bool,
}

impl AcquisitionSummary {
    fn from_raw(raw: RawAcquisitionSummary) -> Self {
        Self {
            callback_registration_active: raw.callback_registration_active != 0,
            start_status: raw.start_status,
            saw_collect_task_start: raw.saw_collect_task_start != 0,
            saw_device_running: raw.saw_device_running != 0,
            saw_device_stopped: raw.saw_device_stopped != 0,
            saw_terminal_normal_end: raw.saw_terminal_normal_end != 0,
            saw_terminal_end_by_detached: raw.saw_terminal_end_by_detached != 0,
            saw_terminal_end_by_error: raw.saw_terminal_end_by_error != 0,
            terminal_event: AcquisitionTerminalEvent::from_raw(raw.terminal_event),
            saw_logic_packet: raw.saw_logic_packet != 0,
            saw_end_packet: raw.saw_end_packet != 0,
            end_packet_status: if raw.end_packet_status == END_PACKET_STATUS_UNKNOWN {
                None
            } else {
                Some(AcquisitionPacketStatus::from_raw(raw.end_packet_status))
            },
            saw_end_packet_ok: raw.saw_end_packet_ok != 0,
            saw_data_error_packet: raw.saw_data_error_packet != 0,
            last_error: NativeErrorCode::from_raw(raw.last_error),
            is_collecting: raw.is_collecting != 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AcquisitionStartResult {
    pub start_status: NativeErrorCode,
    pub summary: AcquisitionSummary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AcquisitionState {
    pub is_collecting: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AcquisitionOutcome {
    pub summary: AcquisitionSummary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CaptureWaitStatus {
    pub saw_terminal_event: bool,
    pub is_collecting: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CaptureProgress {
    pub saw_logic_packet: bool,
    pub saw_end_packet: bool,
    pub saw_device_running: bool,
    pub saw_device_stopped: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AcquisitionRegistrationState {
    pub active: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AcquisitionFailureContext {
    pub last_error: NativeErrorCode,
    pub start_status: NativeErrorCode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceHandle(u64);

impl DeviceHandle {
    pub const fn new(raw: u64) -> Option<Self> {
        if raw == 0 { None } else { Some(Self(raw)) }
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

impl fmt::Display for DeviceHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceSummary {
    pub handle: DeviceHandle,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CaptureCapabilities {
    pub current_samplerate_hz: u64,
    pub current_sample_limit: u64,
    pub total_channel_count: u16,
    pub valid_channel_count: u16,
    pub active_channel_mode: i16,
    pub hardware_depth: u64,
    pub threshold_volts: Option<f64>,
    pub samplerates_hz: Vec<u64>,
    pub channel_modes: Vec<ChannelMode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelMode {
    pub id: i16,
    pub name: String,
    pub max_enabled_channels: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceOptionValue {
    pub code: i16,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceOptionChannelMode {
    pub code: i16,
    pub label: String,
    pub max_enabled_channels: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceOptionChannelModeGroup {
    pub operation_mode_code: i16,
    pub channel_modes: Vec<DeviceOptionChannelMode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyThresholdMetadata {
    pub current_code: Option<i16>,
    pub options: Vec<DeviceOptionValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThresholdVoltageRange {
    pub kind: String,
    pub id: String,
    pub current_volts: Option<f64>,
    pub min_volts: f64,
    pub max_volts: f64,
    pub step_volts: f64,
    pub legacy: Option<LegacyThresholdMetadata>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeviceOptionsSnapshot {
    pub current_operation_mode_code: Option<i16>,
    pub operation_modes: Vec<DeviceOptionValue>,
    pub current_stop_option_code: Option<i16>,
    pub stop_options: Vec<DeviceOptionValue>,
    pub current_filter_code: Option<i16>,
    pub filters: Vec<DeviceOptionValue>,
    pub current_channel_mode_code: Option<i16>,
    pub channel_mode_groups: Vec<DeviceOptionChannelModeGroup>,
    pub threshold: ThresholdVoltageRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeErrorCode {
    Ok,
    Generic,
    Malloc,
    Arg,
    Bug,
    SampleRate,
    NotApplicable,
    DeviceClosed,
    CallStatus,
    AlreadyDone,
    FirmwareMissing,
    DeviceExclusive,
    FirmwareVersionLow,
    DeviceUsbIo,
    Unknown(i32),
}

impl NativeErrorCode {
    pub const fn from_raw(raw: i32) -> Self {
        match raw {
            SR_OK => Self::Ok,
            SR_ERR => Self::Generic,
            SR_ERR_MALLOC => Self::Malloc,
            SR_ERR_ARG => Self::Arg,
            SR_ERR_BUG => Self::Bug,
            SR_ERR_SAMPLERATE => Self::SampleRate,
            SR_ERR_NA => Self::NotApplicable,
            SR_ERR_DEVICE_CLOSED => Self::DeviceClosed,
            SR_ERR_CALL_STATUS => Self::CallStatus,
            SR_ERR_HAVE_DONE => Self::AlreadyDone,
            SR_ERR_FIRMWARE_NOT_EXIST => Self::FirmwareMissing,
            SR_ERR_DEVICE_IS_EXCLUSIVE => Self::DeviceExclusive,
            SR_ERR_DEVICE_FIRMWARE_VERSION_LOW => Self::FirmwareVersionLow,
            SR_ERR_DEVICE_USB_IO_ERROR => Self::DeviceUsbIo,
            other => Self::Unknown(other),
        }
    }

    pub const fn raw(self) -> i32 {
        match self {
            Self::Ok => SR_OK,
            Self::Generic => SR_ERR,
            Self::Malloc => SR_ERR_MALLOC,
            Self::Arg => SR_ERR_ARG,
            Self::Bug => SR_ERR_BUG,
            Self::SampleRate => SR_ERR_SAMPLERATE,
            Self::NotApplicable => SR_ERR_NA,
            Self::DeviceClosed => SR_ERR_DEVICE_CLOSED,
            Self::CallStatus => SR_ERR_CALL_STATUS,
            Self::AlreadyDone => SR_ERR_HAVE_DONE,
            Self::FirmwareMissing => SR_ERR_FIRMWARE_NOT_EXIST,
            Self::DeviceExclusive => SR_ERR_DEVICE_IS_EXCLUSIVE,
            Self::FirmwareVersionLow => SR_ERR_DEVICE_FIRMWARE_VERSION_LOW,
            Self::DeviceUsbIo => SR_ERR_DEVICE_USB_IO_ERROR,
            Self::Unknown(raw) => raw,
        }
    }

    pub const fn is_ok(self) -> bool {
        matches!(self, Self::Ok)
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Ok => "SR_OK",
            Self::Generic => "SR_ERR",
            Self::Malloc => "SR_ERR_MALLOC",
            Self::Arg => "SR_ERR_ARG",
            Self::Bug => "SR_ERR_BUG",
            Self::SampleRate => "SR_ERR_SAMPLERATE",
            Self::NotApplicable => "SR_ERR_NA",
            Self::DeviceClosed => "SR_ERR_DEVICE_CLOSED",
            Self::CallStatus => "SR_ERR_CALL_STATUS",
            Self::AlreadyDone => "SR_ERR_HAVE_DONE",
            Self::FirmwareMissing => "SR_ERR_FIRMWARE_NOT_EXIST",
            Self::DeviceExclusive => "SR_ERR_DEVICE_IS_EXCLUSIVE",
            Self::FirmwareVersionLow => "SR_ERR_DEVICE_FIRMWARE_VERSION_LOW",
            Self::DeviceUsbIo => "SR_ERR_DEVICE_USB_IO_ERROR",
            Self::Unknown(_) => "SR_ERR_UNKNOWN",
        }
    }
}

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("runtime bridge rejected argument: {0}")]
    InvalidArgument(String),
    #[error("runtime bridge shared library is not loaded")]
    BridgeNotLoaded,
    #[error("failed to load runtime shared library `{path}`: {detail}")]
    LibraryLoad { path: PathBuf, detail: String },
    #[error("failed to resolve required ds_* symbols from `{path}`: {detail}")]
    SymbolLoad { path: PathBuf, detail: String },
    #[error("native call `{operation}` failed with {code:?}")]
    NativeCall {
        operation: &'static str,
        code: NativeErrorCode,
    },
    #[error("export call `{operation}` failed with {code:?}")]
    ExportCall {
        operation: &'static str,
        code: ExportErrorCode,
    },
    #[error("device list returned an invalid handle")]
    InvalidDeviceHandle,
    #[error("device name contains invalid UTF-8")]
    InvalidDeviceName,
    #[error("path contains an interior NUL byte: {path}")]
    PathContainsNul { path: PathBuf },
    #[error("failed to write VCD temp file `{path}`: {detail}")]
    TempWrite { path: PathBuf, detail: String },
    #[error("failed to promote VCD temp file `{from}` to `{to}`: {detail}")]
    TempPromote {
        from: PathBuf,
        to: PathBuf,
        detail: String,
    },
}

#[derive(Debug)]
pub struct RuntimeBridge {
    library_path: PathBuf,
}

impl RuntimeBridge {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, RuntimeError> {
        let path = path.as_ref();
        let c_path = path_to_cstring(path)?;
        let status = unsafe { dsview_bridge_load_library(c_path.as_ptr()) };

        match status {
            0 => Ok(Self {
                library_path: path.to_path_buf(),
            }),
            DSVIEW_BRIDGE_ERR_ARG => Err(RuntimeError::InvalidArgument(
                "library path must not be empty".to_string(),
            )),
            DSVIEW_BRIDGE_ERR_DLOPEN => Err(RuntimeError::LibraryLoad {
                path: path.to_path_buf(),
                detail: bridge_last_error(),
            }),
            DSVIEW_BRIDGE_ERR_DLSYM => Err(RuntimeError::SymbolLoad {
                path: path.to_path_buf(),
                detail: bridge_last_error(),
            }),
            DSVIEW_BRIDGE_ERR_NOT_LOADED => Err(RuntimeError::BridgeNotLoaded),
            other => Err(RuntimeError::InvalidArgument(format!(
                "unexpected bridge status {other}"
            ))),
        }
    }

    pub fn library_path(&self) -> &Path {
        &self.library_path
    }

    pub fn is_loaded(&self) -> bool {
        unsafe { dsview_bridge_is_loaded() != 0 }
    }

    pub fn set_firmware_resource_dir(&self, path: impl AsRef<Path>) -> Result<(), RuntimeError> {
        let c_path = path_to_cstring(path.as_ref())?;
        unsafe { dsview_bridge_ds_set_firmware_resource_dir(c_path.as_ptr()) };
        Ok(())
    }

    pub fn init(&self) -> Result<(), RuntimeError> {
        native_call_status("ds_lib_init", unsafe { dsview_bridge_ds_lib_init() })
    }

    pub fn exit(&self) -> Result<(), RuntimeError> {
        native_call_status("ds_lib_exit", unsafe { dsview_bridge_ds_lib_exit() })
    }

    pub fn list_devices(&self) -> Result<Vec<DeviceSummary>, RuntimeError> {
        let mut raw_list: *mut RawDeviceBaseInfo = std::ptr::null_mut();
        let mut count = 0;
        native_call_status("ds_get_device_list", unsafe {
            dsview_bridge_ds_get_device_list(&mut raw_list, &mut count)
        })?;

        if raw_list.is_null() || count <= 0 {
            return Ok(Vec::new());
        }

        let result = unsafe {
            let slice = std::slice::from_raw_parts(raw_list, count as usize);
            slice
                .iter()
                .map(|entry| {
                    let handle =
                        DeviceHandle::new(entry.handle).ok_or(RuntimeError::InvalidDeviceHandle)?;
                    let nul = entry
                        .name
                        .iter()
                        .position(|byte| *byte == 0)
                        .unwrap_or(DEVICE_NAME_CAPACITY);
                    let name = std::str::from_utf8(&entry.name[..nul])
                        .map_err(|_| RuntimeError::InvalidDeviceName)?
                        .to_string();
                    Ok(DeviceSummary { handle, name })
                })
                .collect::<Result<Vec<_>, RuntimeError>>()
        };

        unsafe { dsview_bridge_free_device_list(raw_list) };
        result
    }

    pub fn open_device(&self, handle: DeviceHandle) -> Result<(), RuntimeError> {
        native_call_status("ds_active_device", unsafe {
            dsview_bridge_ds_active_device(handle.raw())
        })
    }

    pub fn release_device(&self) -> Result<(), RuntimeError> {
        native_call_status("ds_release_actived_device", unsafe {
            dsview_bridge_ds_release_actived_device()
        })
    }

    pub fn last_error(&self) -> NativeErrorCode {
        NativeErrorCode::from_raw(unsafe { dsview_bridge_ds_get_last_error() })
    }

    pub fn active_device_init_status(&self) -> Result<i32, RuntimeError> {
        let mut status = 0;
        native_call_status("ds_get_actived_device_init_status", unsafe {
            dsview_bridge_ds_get_actived_device_init_status(&mut status)
        })?;
        Ok(status)
    }

    pub fn capture_capabilities(&self) -> Result<CaptureCapabilities, RuntimeError> {
        let current_samplerate_hz = get_u64_config(
            "ds_get_current_samplerate",
            dsview_bridge_ds_get_current_samplerate,
        )?;
        let current_sample_limit = get_u64_config(
            "ds_get_current_sample_limit",
            dsview_bridge_ds_get_current_sample_limit,
        )?;
        let total_channel_count = get_i32_config(
            "ds_get_total_channel_count",
            dsview_bridge_ds_get_total_channel_count,
        )? as u16;
        let valid_channel_count = get_i32_config(
            "ds_get_valid_channel_count",
            dsview_bridge_ds_get_valid_channel_count,
        )? as u16;
        let active_channel_mode = get_i32_config(
            "ds_get_current_channel_mode",
            dsview_bridge_ds_get_current_channel_mode,
        )? as i16;
        let hardware_depth = get_u64_config("ds_get_hw_depth", dsview_bridge_ds_get_hw_depth)?;

        let threshold_volts = match get_f64_config("ds_get_vth", dsview_bridge_ds_get_vth) {
            Ok(value) => Some(value),
            Err(RuntimeError::NativeCall {
                operation: _,
                code: NativeErrorCode::NotApplicable,
            }) => None,
            Err(error) => return Err(error),
        };

        let samplerates_hz = self.samplerates()?;
        let channel_modes = self.channel_modes()?;

        Ok(CaptureCapabilities {
            current_samplerate_hz,
            current_sample_limit,
            total_channel_count,
            valid_channel_count,
            active_channel_mode,
            hardware_depth,
            threshold_volts,
            samplerates_hz,
            channel_modes,
        })
    }

    pub fn device_options(&self) -> Result<DeviceOptionsSnapshot, RuntimeError> {
        let mut raw = RawDeviceOptionsSnapshot {
            has_current_operation_mode: 0,
            current_operation_mode_code: 0,
            operation_mode_count: 0,
            operation_modes: [RawOptionValue {
                code: 0,
                label: [0; OPTION_LABEL_CAPACITY],
            }; OPTION_VALUE_CAPACITY],
            has_current_stop_option: 0,
            current_stop_option_code: 0,
            stop_option_count: 0,
            stop_options: [RawOptionValue {
                code: 0,
                label: [0; OPTION_LABEL_CAPACITY],
            }; OPTION_VALUE_CAPACITY],
            has_current_filter: 0,
            current_filter_code: 0,
            filter_count: 0,
            filters: [RawOptionValue {
                code: 0,
                label: [0; OPTION_LABEL_CAPACITY],
            }; OPTION_VALUE_CAPACITY],
            has_current_channel_mode: 0,
            current_channel_mode_code: 0,
            channel_mode_group_count: 0,
            channel_mode_groups: [RawChannelModeGroup {
                operation_mode_code: 0,
                channel_mode_count: 0,
                channel_modes: [RawChannelMode {
                    id: 0,
                    name: [0; OPTION_LABEL_CAPACITY],
                    max_enabled_channels: 0,
                }; CHANNEL_MODE_CAPACITY],
            }; CHANNEL_MODE_GROUP_CAPACITY],
            threshold: RawThresholdRange {
                kind: [0; THRESHOLD_KIND_CAPACITY],
                id: [0; OPTION_LABEL_CAPACITY],
                has_current_volts: 0,
                current_volts: 0.0,
                min_volts: 0.0,
                max_volts: 0.0,
                step_volts: 0.0,
                has_current_legacy_code: 0,
                current_legacy_code: 0,
                legacy_option_count: 0,
                legacy_options: [RawOptionValue {
                    code: 0,
                    label: [0; OPTION_LABEL_CAPACITY],
                }; OPTION_VALUE_CAPACITY],
            },
        };
        native_call_status("ds_get_device_options", unsafe {
            dsview_bridge_ds_get_device_options(&mut raw)
        })?;
        decode_device_options_snapshot(&raw)
    }

    pub fn set_samplerate(&self, value: u64) -> Result<(), RuntimeError> {
        native_call_status("ds_set_samplerate", unsafe {
            dsview_bridge_ds_set_samplerate(value)
        })
    }

    pub fn set_sample_limit(&self, value: u64) -> Result<(), RuntimeError> {
        native_call_status("ds_set_sample_limit", unsafe {
            dsview_bridge_ds_set_sample_limit(value)
        })
    }

    pub fn set_enabled_channels(
        &self,
        enabled_channels: &[u16],
        total_channel_count: u16,
    ) -> Result<(), RuntimeError> {
        for channel in 0..total_channel_count {
            let enable = enabled_channels.contains(&channel);
            native_call_status("ds_enable_channel", unsafe {
                dsview_bridge_ds_enable_channel(channel as c_int, if enable { 1 } else { 0 })
            })?;
        }
        Ok(())
    }

    pub fn register_acquisition_callbacks(
        &self,
    ) -> Result<AcquisitionRegistrationState, RuntimeError> {
        native_call_status("ds_register_acquisition_callbacks", unsafe {
            dsview_bridge_ds_register_acquisition_callbacks()
        })?;
        Ok(AcquisitionRegistrationState { active: true })
    }

    pub fn clear_acquisition_callbacks(
        &self,
    ) -> Result<AcquisitionRegistrationState, RuntimeError> {
        native_call_status("ds_clear_acquisition_callbacks", unsafe {
            dsview_bridge_ds_clear_acquisition_callbacks()
        })?;
        Ok(AcquisitionRegistrationState { active: false })
    }

    pub fn reset_acquisition_summary(&self) -> Result<(), RuntimeError> {
        native_call_status("ds_reset_acquisition_summary", unsafe {
            dsview_bridge_ds_reset_acquisition_summary()
        })
    }

    pub fn acquisition_summary(&self) -> Result<AcquisitionSummary, RuntimeError> {
        let mut raw = RawAcquisitionSummary {
            callback_registration_active: 0,
            start_status: 0,
            saw_collect_task_start: 0,
            saw_device_running: 0,
            saw_device_stopped: 0,
            saw_terminal_normal_end: 0,
            saw_terminal_end_by_detached: 0,
            saw_terminal_end_by_error: 0,
            terminal_event: 0,
            saw_logic_packet: 0,
            saw_end_packet: 0,
            end_packet_status: END_PACKET_STATUS_UNKNOWN,
            saw_end_packet_ok: 0,
            saw_data_error_packet: 0,
            last_error: SR_OK,
            is_collecting: 0,
        };
        native_call_status("ds_get_acquisition_summary", unsafe {
            dsview_bridge_ds_get_acquisition_summary(&mut raw)
        })?;
        Ok(AcquisitionSummary::from_raw(raw))
    }

    pub fn start_collect(&self) -> Result<AcquisitionStartResult, RuntimeError> {
        let status = unsafe { dsview_bridge_ds_start_collect() };
        let summary = self.acquisition_summary()?;
        Ok(AcquisitionStartResult {
            start_status: NativeErrorCode::from_raw(status),
            summary,
        })
    }

    pub fn stop_collect(&self) -> Result<AcquisitionOutcome, RuntimeError> {
        native_call_status("ds_stop_collect", unsafe {
            dsview_bridge_ds_stop_collect()
        })?;
        Ok(AcquisitionOutcome {
            summary: self.acquisition_summary()?,
        })
    }

    pub fn acquisition_state(&self) -> Result<AcquisitionState, RuntimeError> {
        let mut is_collecting = 0;
        native_call_status("ds_is_collecting", unsafe {
            dsview_bridge_ds_is_collecting(&mut is_collecting)
        })?;
        Ok(AcquisitionState {
            is_collecting: is_collecting != 0,
        })
    }

    pub fn export_recorded_vcd(
        &self,
        request: &VcdExportRequest,
    ) -> Result<VcdExport, RuntimeError> {
        let raw_request = raw_vcd_export_request(request)?;
        let mut raw = RawExportBuffer {
            data: std::ptr::null_mut(),
            len: 0,
            sample_count: 0,
            packet_count: 0,
        };
        export_call_status("ds_export_recorded_vcd", unsafe {
            dsview_bridge_ds_export_recorded_vcd(&raw_request, &mut raw)
        })?;
        export_from_raw(raw)
    }

    pub fn export_recorded_vcd_to_path(
        &self,
        request: &VcdExportRequest,
        final_path: impl AsRef<Path>,
    ) -> Result<VcdExportFacts, RuntimeError> {
        let export = self.export_recorded_vcd(request)?;
        write_vcd_atomically(final_path.as_ref(), &export.bytes)?;
        Ok(VcdExportFacts {
            sample_count: export.sample_count,
            packet_count: export.packet_count,
            output_bytes: export.bytes.len() as u64,
        })
    }

    pub fn render_vcd_from_samples(
        &self,
        request: &VcdExportRequest,
        sample_bytes: &[u8],
        unitsize: u16,
    ) -> Result<VcdExport, RuntimeError> {
        if sample_bytes.is_empty() {
            return Err(RuntimeError::InvalidArgument(
                "sample bytes must not be empty".to_string(),
            ));
        }
        let raw_request = raw_vcd_export_request(request)?;
        let mut raw = RawExportBuffer {
            data: std::ptr::null_mut(),
            len: 0,
            sample_count: 0,
            packet_count: 0,
        };
        export_call_status("render_vcd_from_samples", unsafe {
            dsview_bridge_render_vcd_from_samples(
                &raw_request,
                sample_bytes.as_ptr(),
                sample_bytes.len(),
                unitsize,
                &mut raw,
            )
        })?;
        export_from_raw(raw)
    }

    pub fn render_vcd_from_samples_to_path(
        &self,
        request: &VcdExportRequest,
        sample_bytes: &[u8],
        unitsize: u16,
        final_path: impl AsRef<Path>,
    ) -> Result<VcdExportFacts, RuntimeError> {
        let export = self.render_vcd_from_samples(request, sample_bytes, unitsize)?;
        write_vcd_atomically(final_path.as_ref(), &export.bytes)?;
        Ok(VcdExportFacts {
            sample_count: export.sample_count,
            packet_count: export.packet_count,
            output_bytes: export.bytes.len() as u64,
        })
    }

    pub fn render_vcd_from_logic_packets(
        &self,
        request: &VcdExportRequest,
        sample_bytes: &[u8],
        logic_packet_lengths: &[usize],
        unitsize: u16,
    ) -> Result<VcdExport, RuntimeError> {
        if sample_bytes.is_empty() {
            return Err(RuntimeError::InvalidArgument(
                "sample bytes must not be empty".to_string(),
            ));
        }
        if logic_packet_lengths.is_empty() {
            return Err(RuntimeError::InvalidArgument(
                "at least one logic packet is required".to_string(),
            ));
        }
        if unitsize == 0 {
            return Err(RuntimeError::InvalidArgument(
                "logic packet unitsize must be greater than zero".to_string(),
            ));
        }
        if (sample_bytes.len() % unitsize as usize) != 0 {
            return Err(RuntimeError::InvalidArgument(
                "sample bytes length must divide evenly by unitsize".to_string(),
            ));
        }
        let expected_total: usize = logic_packet_lengths.iter().sum();
        if expected_total != sample_bytes.len() {
            return Err(RuntimeError::InvalidArgument(
                "logic packet lengths must sum to the sample byte length".to_string(),
            ));
        }
        if logic_packet_lengths
            .iter()
            .any(|length| *length == 0 || (length % unitsize as usize) != 0)
        {
            return Err(RuntimeError::InvalidArgument(
                "each logic packet length must be non-zero and aligned to unitsize"
                    .to_string(),
            ));
        }

        let raw_request = raw_vcd_export_request(request)?;
        let mut raw = RawExportBuffer {
            data: std::ptr::null_mut(),
            len: 0,
            sample_count: 0,
            packet_count: 0,
        };
        export_call_status("render_vcd_from_logic_packets", unsafe {
            dsview_bridge_render_vcd_from_logic_packets(
                &raw_request,
                sample_bytes.as_ptr(),
                sample_bytes.len(),
                logic_packet_lengths.as_ptr(),
                logic_packet_lengths.len(),
                unitsize,
                &mut raw,
            )
        })?;
        export_from_raw(raw)
    }

    pub fn render_vcd_from_logic_packets_to_path(
        &self,
        request: &VcdExportRequest,
        sample_bytes: &[u8],
        logic_packet_lengths: &[usize],
        unitsize: u16,
        final_path: impl AsRef<Path>,
    ) -> Result<VcdExportFacts, RuntimeError> {
        let export = self.render_vcd_from_logic_packets(
            request,
            sample_bytes,
            logic_packet_lengths,
            unitsize,
        )?;
        write_vcd_atomically(final_path.as_ref(), &export.bytes)?;
        Ok(VcdExportFacts {
            sample_count: export.sample_count,
            packet_count: export.packet_count,
            output_bytes: export.bytes.len() as u64,
        })
    }

    fn samplerates(&self) -> Result<Vec<u64>, RuntimeError> {
        let mut raw = RawSamplerateList {
            count: 0,
            values: [0; 64],
        };
        native_call_status("ds_get_samplerates", unsafe {
            dsview_bridge_ds_get_samplerates(&mut raw)
        })?;
        Ok(raw.values[..raw.count as usize].to_vec())
    }

    fn channel_modes(&self) -> Result<Vec<ChannelMode>, RuntimeError> {
        let mut raw_modes = [RawChannelMode {
            id: 0,
            name: [0; 64],
            max_enabled_channels: 0,
        }; 16];
        let mut count = 0;
        native_call_status("ds_get_channel_modes", unsafe {
            dsview_bridge_ds_get_channel_modes(
                raw_modes.as_mut_ptr(),
                raw_modes.len() as c_int,
                &mut count,
            )
        })?;

        raw_modes[..count as usize]
            .iter()
            .map(|raw| {
                let nul = raw
                    .name
                    .iter()
                    .position(|byte| *byte == 0)
                    .unwrap_or(raw.name.len());
                let name = std::str::from_utf8(&raw.name[..nul])
                    .map_err(|_| RuntimeError::InvalidDeviceName)?
                    .to_string();
                Ok(ChannelMode {
                    id: raw.id as i16,
                    name,
                    max_enabled_channels: raw.max_enabled_channels,
                })
            })
            .collect()
    }
}

impl Drop for RuntimeBridge {
    fn drop(&mut self) {
        unsafe { dsview_bridge_unload_library() };
    }
}

/// Reports whether the sys boundary is wired to the DSView public frontend API.
pub fn native_boundary_ready() -> bool {
    cfg!(dsview_native_boundary)
}

/// Reports whether the dynamic runtime bridge shim is available on this machine.
pub fn runtime_bridge_ready() -> bool {
    cfg!(dsview_runtime_bridge)
}

/// Reports whether the scoped runtime smoke shim is available on this machine.
pub fn runtime_smoke_ready() -> bool {
    cfg!(dsview_runtime_smoke_available)
}

/// Reports whether a source-built DSView runtime library is available.
pub fn source_runtime_available() -> bool {
    cfg!(dsview_source_runtime_available)
}

/// Returns the public libsigrok4DSL version string when a native library is linked.
#[cfg(dsview_runtime_smoke_available)]
pub fn lib_version_string() -> Option<&'static CStr> {
    if !native_boundary_ready() {
        return None;
    }

    let raw = unsafe { sr_get_lib_version_string() };
    if raw.is_null() {
        None
    } else {
        Some(unsafe { CStr::from_ptr(raw) })
    }
}

/// Returns `None` when the scoped runtime smoke shim is unavailable.
#[cfg(not(dsview_runtime_smoke_available))]
pub fn lib_version_string() -> Option<&'static CStr> {
    None
}

pub fn upstream_header_path() -> &'static Path {
    Path::new(env!("DSVIEW_LIBSIGROK_HEADER"))
}

pub fn source_runtime_library_path() -> Option<&'static Path> {
    option_env!("DSVIEW_SOURCE_RUNTIME_LIBRARY").map(Path::new)
}

fn write_vcd_atomically(final_path: &Path, bytes: &[u8]) -> Result<(), RuntimeError> {
    let parent = final_path.parent().ok_or_else(|| RuntimeError::InvalidArgument(format!(
        "final VCD path `{}` must have a parent directory",
        final_path.display()
    )))?;
    fs::create_dir_all(parent).map_err(|error| RuntimeError::TempWrite {
        path: parent.to_path_buf(),
        detail: error.to_string(),
    })?;

    let temp_path = final_path.with_extension(format!(
        "{}.tmp",
        final_path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or("vcd")
    ));

    fs::write(&temp_path, bytes).map_err(|error| RuntimeError::TempWrite {
        path: temp_path.clone(),
        detail: error.to_string(),
    })?;

    if let Err(error) = fs::rename(&temp_path, final_path) {
        let _ = fs::remove_file(&temp_path);
        return Err(RuntimeError::TempPromote {
            from: temp_path,
            to: final_path.to_path_buf(),
            detail: error.to_string(),
        });
    }

    Ok(())
}

fn bridge_last_error() -> String {
    unsafe {
        let raw = dsview_bridge_last_loader_error();
        if raw.is_null() {
            String::new()
        } else {
            CStr::from_ptr(raw).to_string_lossy().into_owned()
        }
    }
}

fn get_u64_config(
    operation: &'static str,
    getter: unsafe extern "C" fn(*mut u64) -> c_int,
) -> Result<u64, RuntimeError> {
    let mut value = 0;
    native_call_status(operation, unsafe { getter(&mut value) })?;
    Ok(value)
}

fn get_i32_config(
    operation: &'static str,
    getter: unsafe extern "C" fn(*mut c_int) -> c_int,
) -> Result<i32, RuntimeError> {
    let mut value = 0;
    native_call_status(operation, unsafe { getter(&mut value) })?;
    Ok(value)
}

fn get_f64_config(
    operation: &'static str,
    getter: unsafe extern "C" fn(*mut f64) -> c_int,
) -> Result<f64, RuntimeError> {
    let mut value = 0.0;
    native_call_status(operation, unsafe { getter(&mut value) })?;
    Ok(value)
}

fn decode_device_options_snapshot(
    raw: &RawDeviceOptionsSnapshot,
) -> Result<DeviceOptionsSnapshot, RuntimeError> {
    let legacy_options = decode_option_values(
        &raw.threshold.legacy_options,
        raw.threshold.legacy_option_count as usize,
    )?;

    Ok(DeviceOptionsSnapshot {
        current_operation_mode_code: decode_optional_code(
            raw.has_current_operation_mode,
            raw.current_operation_mode_code,
        ),
        operation_modes: decode_option_values(&raw.operation_modes, raw.operation_mode_count as usize)?,
        current_stop_option_code: decode_optional_code(
            raw.has_current_stop_option,
            raw.current_stop_option_code,
        ),
        stop_options: decode_option_values(&raw.stop_options, raw.stop_option_count as usize)?,
        current_filter_code: decode_optional_code(raw.has_current_filter, raw.current_filter_code),
        filters: decode_option_values(&raw.filters, raw.filter_count as usize)?,
        current_channel_mode_code: decode_optional_code(
            raw.has_current_channel_mode,
            raw.current_channel_mode_code,
        ),
        channel_mode_groups: decode_channel_mode_groups(
            &raw.channel_mode_groups,
            raw.channel_mode_group_count as usize,
        )?,
        threshold: ThresholdVoltageRange {
            kind: decode_fixed_string(&raw.threshold.kind)?,
            id: decode_fixed_string(&raw.threshold.id)?,
            current_volts: if raw.threshold.has_current_volts != 0 {
                Some(raw.threshold.current_volts)
            } else {
                None
            },
            min_volts: raw.threshold.min_volts,
            max_volts: raw.threshold.max_volts,
            step_volts: raw.threshold.step_volts,
            legacy: if legacy_options.is_empty() && raw.threshold.has_current_legacy_code == 0 {
                None
            } else {
                Some(LegacyThresholdMetadata {
                    current_code: decode_optional_code(
                        raw.threshold.has_current_legacy_code,
                        raw.threshold.current_legacy_code,
                    ),
                    options: legacy_options,
                })
            },
        },
    })
}

fn decode_option_values(
    raw: &[RawOptionValue; OPTION_VALUE_CAPACITY],
    count: usize,
) -> Result<Vec<DeviceOptionValue>, RuntimeError> {
    raw[..count.min(raw.len())]
        .iter()
        .map(|item| {
            Ok(DeviceOptionValue {
                code: item.code as i16,
                label: decode_fixed_string(&item.label)?,
            })
        })
        .collect()
}

fn decode_channel_mode_groups(
    raw: &[RawChannelModeGroup; CHANNEL_MODE_GROUP_CAPACITY],
    count: usize,
) -> Result<Vec<DeviceOptionChannelModeGroup>, RuntimeError> {
    raw[..count.min(raw.len())]
        .iter()
        .map(|group| {
            Ok(DeviceOptionChannelModeGroup {
                operation_mode_code: group.operation_mode_code as i16,
                channel_modes: group.channel_modes[..(group.channel_mode_count as usize)
                    .min(group.channel_modes.len())]
                    .iter()
                    .map(|mode| {
                        Ok(DeviceOptionChannelMode {
                            code: mode.id as i16,
                            label: decode_fixed_string(&mode.name)?,
                            max_enabled_channels: mode.max_enabled_channels,
                        })
                    })
                    .collect::<Result<Vec<_>, RuntimeError>>()?,
            })
        })
        .collect()
}

fn decode_fixed_string<const N: usize>(bytes: &[u8; N]) -> Result<String, RuntimeError> {
    let nul = bytes.iter().position(|byte| *byte == 0).unwrap_or(bytes.len());
    std::str::from_utf8(&bytes[..nul])
        .map(|value| value.to_string())
        .map_err(|_| RuntimeError::InvalidDeviceName)
}

fn decode_optional_code(flag: c_int, value: i32) -> Option<i16> {
    (flag != 0).then_some(value as i16)
}

fn path_to_cstring(path: &Path) -> Result<CString, RuntimeError> {
    CString::new(path.as_os_str().to_string_lossy().as_bytes()).map_err(|_| {
        RuntimeError::PathContainsNul {
            path: path.to_path_buf(),
        }
    })
}

fn raw_vcd_export_request(request: &VcdExportRequest) -> Result<RawVcdExportRequest, RuntimeError> {
    if request.samplerate_hz == 0 {
        return Err(RuntimeError::InvalidArgument(
            "samplerate must be greater than zero".to_string(),
        ));
    }
    if request.enabled_channels.is_empty() {
        return Err(RuntimeError::InvalidArgument(
            "at least one enabled channel is required for VCD export".to_string(),
        ));
    }

    Ok(RawVcdExportRequest {
        samplerate_hz: request.samplerate_hz,
        enabled_channels: request.enabled_channels.as_ptr(),
        enabled_channel_count: request.enabled_channels.len(),
    })
}

fn export_from_raw(raw: RawExportBuffer) -> Result<VcdExport, RuntimeError> {
    if raw.data.is_null() {
        return Ok(VcdExport {
            bytes: Vec::new(),
            sample_count: raw.sample_count,
            packet_count: raw.packet_count,
        });
    }

    let bytes = unsafe {
        let slice = std::slice::from_raw_parts(raw.data, raw.len);
        let owned = slice.to_vec();
        let mut raw = raw;
        dsview_bridge_free_export_buffer(&mut raw);
        normalize_vcd_timestamp_lines(owned)
    };

    Ok(VcdExport {
        bytes,
        sample_count: raw.sample_count,
        packet_count: raw.packet_count,
    })
}

fn normalize_vcd_timestamp_lines(bytes: Vec<u8>) -> Vec<u8> {
    let Ok(vcd) = String::from_utf8(bytes.clone()) else {
        return bytes;
    };

    let mut normalized = String::with_capacity(vcd.len());
    for line in vcd.lines() {
        if let Some((timestamp, values)) = line.split_once(' ') {
            if timestamp.starts_with('#') && !values.is_empty() {
                normalized.push_str(timestamp);
                normalized.push('\n');
                for value in values.split_whitespace() {
                    normalized.push_str(value);
                    normalized.push('\n');
                }
                continue;
            }
        }
        normalized.push_str(line);
        normalized.push('\n');
    }

    normalized.into_bytes()
}

fn native_call_status(operation: &'static str, status: i32) -> Result<(), RuntimeError> {
    if status == SR_OK {
        Ok(())
    } else if status == DSVIEW_BRIDGE_ERR_NOT_LOADED {
        Err(RuntimeError::BridgeNotLoaded)
    } else {
        Err(RuntimeError::NativeCall {
            operation,
            code: NativeErrorCode::from_raw(status),
        })
    }
}

fn export_call_status(operation: &'static str, status: i32) -> Result<(), RuntimeError> {
    if status == SR_OK {
        Ok(())
    } else if status == DSVIEW_BRIDGE_ERR_NOT_LOADED {
        Err(RuntimeError::BridgeNotLoaded)
    } else if status == DSVIEW_BRIDGE_ERR_ARG {
        Err(RuntimeError::InvalidArgument(format!(
            "bridge rejected export request for {operation}"
        )))
    } else {
        Err(RuntimeError::ExportCall {
            operation,
            code: ExportErrorCode::from_raw(status),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_boundary_cfg_is_enabled() {
        assert!(
            native_boundary_ready(),
            "build script should enable the dsview_native_boundary cfg"
        );
    }

    #[test]
    fn runtime_bridge_cfg_is_enabled() {
        assert!(
            runtime_bridge_ready(),
            "build script should enable the dsview_runtime_bridge cfg"
        );
    }

    #[test]
    fn acquisition_summary_preserves_end_marker_and_collecting_state() {
        let raw = RawAcquisitionSummary {
            callback_registration_active: 1,
            start_status: SR_OK,
            saw_collect_task_start: 1,
            saw_device_running: 1,
            saw_device_stopped: 1,
            saw_terminal_normal_end: 1,
            saw_terminal_end_by_detached: 0,
            saw_terminal_end_by_error: 0,
            terminal_event: 1,
            saw_logic_packet: 1,
            saw_end_packet: 1,
            end_packet_status: 0,
            saw_end_packet_ok: 1,
            saw_data_error_packet: 0,
            last_error: SR_OK,
            is_collecting: 0,
        };

        let summary = AcquisitionSummary::from_raw(raw);
        assert_eq!(summary.terminal_event, AcquisitionTerminalEvent::NormalEnd);
        assert_eq!(summary.end_packet_status, Some(AcquisitionPacketStatus::Ok));
        assert!(!summary.is_collecting);
    }

    #[test]
    fn acquisition_summary_preserves_detach_terminal_event() {
        let raw = RawAcquisitionSummary {
            callback_registration_active: 1,
            start_status: SR_OK,
            saw_collect_task_start: 1,
            saw_device_running: 1,
            saw_device_stopped: 1,
            saw_terminal_normal_end: 0,
            saw_terminal_end_by_detached: 1,
            saw_terminal_end_by_error: 0,
            terminal_event: 2,
            saw_logic_packet: 1,
            saw_end_packet: 0,
            end_packet_status: END_PACKET_STATUS_UNKNOWN,
            saw_end_packet_ok: 0,
            saw_data_error_packet: 0,
            last_error: SR_ERR_DEVICE_USB_IO_ERROR,
            is_collecting: 0,
        };

        let summary = AcquisitionSummary::from_raw(raw);
        assert_eq!(summary.terminal_event, AcquisitionTerminalEvent::EndByDetached);
        assert!(summary.saw_terminal_end_by_detached);
        assert_eq!(summary.last_error, NativeErrorCode::DeviceUsbIo);
    }

    #[test]
    fn acquisition_summary_treats_unknown_end_status_as_none() {
        let raw = RawAcquisitionSummary {
            callback_registration_active: 0,
            start_status: SR_OK,
            saw_collect_task_start: 0,
            saw_device_running: 0,
            saw_device_stopped: 0,
            saw_terminal_normal_end: 0,
            saw_terminal_end_by_detached: 0,
            saw_terminal_end_by_error: 0,
            terminal_event: 0,
            saw_logic_packet: 0,
            saw_end_packet: 0,
            end_packet_status: END_PACKET_STATUS_UNKNOWN,
            saw_end_packet_ok: 0,
            saw_data_error_packet: 0,
            last_error: SR_OK,
            is_collecting: 1,
        };

        let summary = AcquisitionSummary::from_raw(raw);
        assert_eq!(summary.end_packet_status, None);
        assert!(summary.is_collecting);
    }

    #[test]
    fn runtime_smoke_matches_environment() {
        let expected = std::path::Path::new("/usr/include/glib-2.0/glib.h").exists()
            && std::path::Path::new("/usr/lib/x86_64-linux-gnu/glib-2.0/include/glibconfig.h")
                .exists();
        assert_eq!(
            runtime_smoke_ready(),
            expected,
            "runtime smoke availability should reflect whether glib development headers are present"
        );
    }

    #[test]
    fn upstream_header_path_points_at_dsview_header() {
        let path = upstream_header_path();
        assert!(
            path.ends_with("DSView/libsigrok4DSL/libsigrok.h"),
            "header path should point at the DSView public header, got {}",
            path.display()
        );
    }

    #[test]
    fn source_runtime_path_matches_cfg() {
        assert_eq!(
            source_runtime_library_path().is_some(),
            source_runtime_available()
        );
    }

    #[test]
    fn export_error_codes_map_expected_values() {
        assert_eq!(ExportErrorCode::from_raw(DSVIEW_EXPORT_ERR_OVERFLOW), ExportErrorCode::Overflow);
        assert_eq!(
            ExportErrorCode::from_raw(DSVIEW_EXPORT_ERR_OUTPUT_MODULE),
            ExportErrorCode::OutputModuleUnavailable
        );
    }

    #[test]
    fn raw_vcd_export_request_rejects_missing_inputs() {
        let missing_samplerate = raw_vcd_export_request(&VcdExportRequest {
            samplerate_hz: 0,
            enabled_channels: vec![0],
        })
        .unwrap_err();
        assert!(matches!(missing_samplerate, RuntimeError::InvalidArgument(_)));

        let missing_channels = raw_vcd_export_request(&VcdExportRequest {
            samplerate_hz: 1,
            enabled_channels: Vec::new(),
        })
        .unwrap_err();
        assert!(matches!(missing_channels, RuntimeError::InvalidArgument(_)));
    }

    #[test]
    fn raw_vcd_export_request_preserves_multichannel_packed_shape() {
        let request = VcdExportRequest {
            samplerate_hz: 100_000_000,
            enabled_channels: vec![0, 1, 2, 3, 8, 9, 10, 11, 12],
        };

        let raw = raw_vcd_export_request(&request).unwrap();
        assert_eq!(raw.samplerate_hz, 100_000_000);
        assert_eq!(raw.enabled_channel_count, 9);

        let channels = unsafe {
            std::slice::from_raw_parts(raw.enabled_channels, raw.enabled_channel_count)
        };
        assert_eq!(channels, &[0, 1, 2, 3, 8, 9, 10, 11, 12]);

        let packed_unitsize = request.enabled_channels.len().div_ceil(8) as u16;
        assert_eq!(packed_unitsize, 2);
    }

    #[test]
    fn export_from_raw_copies_owned_bytes() {
        unsafe extern "C" {
            fn malloc(size: usize) -> *mut std::ffi::c_void;
        }

        let raw = RawExportBuffer {
            data: unsafe { malloc(4) }.cast(),
            len: 4,
            sample_count: 2,
            packet_count: 0,
        };
        unsafe {
            std::ptr::copy_nonoverlapping(b"vcd\n".as_ptr(), raw.data, 4);
        }

        let export = export_from_raw(raw).unwrap();
        assert_eq!(export.bytes, b"vcd\n");
        assert_eq!(export.sample_count, 2);
        assert_eq!(export.packet_count, 0);
    }

    #[test]
    fn write_vcd_atomically_promotes_temp_file() {
        let dir = std::env::temp_dir().join(format!(
            "dsview-sys-vcd-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let final_path = dir.join("capture.vcd");
        let temp_path = dir.join("capture.vcd.tmp");

        write_vcd_atomically(&final_path, b"$date\n$end\n").unwrap();

        assert!(final_path.is_file());
        assert!(!temp_path.exists());
        assert_eq!(std::fs::read(&final_path).unwrap(), b"$date\n$end\n");
    }

    #[test]
    fn device_handle_rejects_zero() {
        assert!(DeviceHandle::new(0).is_none());
        assert_eq!(DeviceHandle::new(42).unwrap().raw(), 42);
    }

    #[test]
    fn lib_version_smoke_returns_expected_version_when_runtime_is_available() {
        if runtime_smoke_ready() {
            let version =
                lib_version_string().expect("runtime smoke shim should return a version string");
            assert_eq!(version.to_str().unwrap(), "1.3.0");
        } else {
            assert!(
                lib_version_string().is_none(),
                "without the runtime smoke shim, version lookup should stay disabled"
            );
        }
    }
}
