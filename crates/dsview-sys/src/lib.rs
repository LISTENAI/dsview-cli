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
    fn dsview_bridge_ds_get_device_list(out_list: *mut *mut RawDeviceBaseInfo, out_count: *mut c_int) -> c_int;
    fn dsview_bridge_free_device_list(list: *mut RawDeviceBaseInfo);
    fn dsview_bridge_ds_active_device(handle: u64) -> c_int;
    fn dsview_bridge_ds_release_actived_device() -> c_int;
    fn dsview_bridge_ds_get_last_error() -> c_int;
    fn dsview_bridge_ds_get_actived_device_init_status(status: *mut c_int) -> c_int;
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

#[repr(C)]
struct RawDeviceBaseInfo {
    handle: u64,
    name: [u8; DEVICE_NAME_CAPACITY],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceHandle(u64);

impl DeviceHandle {
    pub const fn new(raw: u64) -> Option<Self> {
        if raw == 0 {
            None
        } else {
            Some(Self(raw))
        }
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
    NativeCall { operation: &'static str, code: NativeErrorCode },
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
            DSVIEW_BRIDGE_ERR_ARG => Err(RuntimeError::InvalidArgument("library path must not be empty".to_string())),
            DSVIEW_BRIDGE_ERR_DLOPEN => Err(RuntimeError::LibraryLoad {
                path: path.to_path_buf(),
                detail: bridge_last_error(),
            }),
            DSVIEW_BRIDGE_ERR_DLSYM => Err(RuntimeError::SymbolLoad {
                path: path.to_path_buf(),
                detail: bridge_last_error(),
            }),
            DSVIEW_BRIDGE_ERR_NOT_LOADED => Err(RuntimeError::BridgeNotLoaded),
            other => Err(RuntimeError::InvalidArgument(format!("unexpected bridge status {other}"))),
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
        native_call_status(
            "ds_get_device_list",
            unsafe { dsview_bridge_ds_get_device_list(&mut raw_list, &mut count) },
        )?;

        if raw_list.is_null() || count <= 0 {
            return Ok(Vec::new());
        }

        let result = unsafe {
            let slice = std::slice::from_raw_parts(raw_list, count as usize);
            slice
                .iter()
                .map(|entry| {
                    let handle = DeviceHandle::new(entry.handle).ok_or(RuntimeError::InvalidDeviceHandle)?;
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
        native_call_status("ds_active_device", unsafe { dsview_bridge_ds_active_device(handle.raw()) })
    }

    pub fn release_device(&self) -> Result<(), RuntimeError> {
        native_call_status(
            "ds_release_actived_device",
            unsafe { dsview_bridge_ds_release_actived_device() },
        )
    }

    pub fn last_error(&self) -> NativeErrorCode {
        NativeErrorCode::from_raw(unsafe { dsview_bridge_ds_get_last_error() })
    }

    pub fn active_device_init_status(&self) -> Result<i32, RuntimeError> {
        let mut status = 0;
        native_call_status(
            "ds_get_actived_device_init_status",
            unsafe { dsview_bridge_ds_get_actived_device_init_status(&mut status) },
        )?;
        Ok(status)
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

fn path_to_cstring(path: &Path) -> Result<CString, RuntimeError> {
    CString::new(path.as_os_str().to_string_lossy().as_bytes()).map_err(|_| RuntimeError::PathContainsNul {
        path: path.to_path_buf(),
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
        assert!(native_boundary_ready(), "build script should enable the dsview_native_boundary cfg");
    }

    #[test]
    fn runtime_bridge_cfg_is_enabled() {
        assert!(runtime_bridge_ready(), "build script should enable the dsview_runtime_bridge cfg");
    }

    #[test]
    fn runtime_smoke_matches_environment() {
        let expected = std::path::Path::new("/usr/include/glib-2.0/glib.h").exists()
            && std::path::Path::new("/usr/lib/x86_64-linux-gnu/glib-2.0/include/glibconfig.h").exists();
        assert_eq!(runtime_smoke_ready(), expected, "runtime smoke availability should reflect whether glib development headers are present");
    }

    #[test]
    fn upstream_header_path_points_at_dsview_header() {
        let path = upstream_header_path();
        assert!(path.ends_with("DSView/libsigrok4DSL/libsigrok.h"), "header path should point at the DSView public header, got {}", path.display());
    }

    #[test]
    fn source_runtime_path_matches_cfg() {
        assert_eq!(source_runtime_library_path().is_some(), source_runtime_available());
    }

    #[test]
    fn native_error_codes_map_expected_values() {
        assert_eq!(NativeErrorCode::from_raw(SR_ERR_FIRMWARE_NOT_EXIST), NativeErrorCode::FirmwareMissing);
        assert_eq!(NativeErrorCode::from_raw(SR_ERR_DEVICE_IS_EXCLUSIVE), NativeErrorCode::DeviceExclusive);
        assert_eq!(NativeErrorCode::from_raw(12345), NativeErrorCode::Unknown(12345));
    }

    #[test]
    fn device_handle_rejects_zero() {
        assert!(DeviceHandle::new(0).is_none());
        assert_eq!(DeviceHandle::new(42).unwrap().raw(), 42);
    }

    #[test]
    fn lib_version_smoke_returns_expected_version_when_runtime_is_available() {
        if runtime_smoke_ready() {
            let version = lib_version_string().expect("runtime smoke shim should return a version string");
            assert_eq!(version.to_str().unwrap(), "1.3.0");
        } else {
            assert!(lib_version_string().is_none(), "without the runtime smoke shim, version lookup should stay disabled");
        }
    }
}
