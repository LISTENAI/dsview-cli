//! Raw native integration boundary for DSView CLI.
//!
//! This crate is the only allowed home for unsafe FFI when Phase 1 adds
//! bindings to `DSView/libsigrok4DSL`.

use std::ffi::{CStr, CString};
use std::fmt;
use std::os::raw::{c_char, c_int};
use std::path::{Path, PathBuf};

use thiserror::Error;

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

const DEVICE_NAME_CAPACITY: usize = 150;
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
struct RawSamplerateList {
    count: u32,
    values: [u64; 64],
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
    #[error("device list returned an invalid handle")]
    InvalidDeviceHandle,
    #[error("device name contains invalid UTF-8")]
    InvalidDeviceName,
    #[error("path contains an interior NUL byte: {path}")]
    PathContainsNul { path: PathBuf },
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

fn path_to_cstring(path: &Path) -> Result<CString, RuntimeError> {
    CString::new(path.as_os_str().to_string_lossy().as_bytes()).map_err(|_| {
        RuntimeError::PathContainsNul {
            path: path.to_path_buf(),
        }
    })
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
    fn native_error_codes_map_expected_values() {
        assert_eq!(
            NativeErrorCode::from_raw(SR_ERR_FIRMWARE_NOT_EXIST),
            NativeErrorCode::FirmwareMissing
        );
        assert_eq!(
            NativeErrorCode::from_raw(SR_ERR_DEVICE_IS_EXCLUSIVE),
            NativeErrorCode::DeviceExclusive
        );
        assert_eq!(
            NativeErrorCode::from_raw(12345),
            NativeErrorCode::Unknown(12345)
        );
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
