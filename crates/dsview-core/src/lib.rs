use std::fs;
use std::path::{Path, PathBuf};

pub use dsview_sys::{
    source_runtime_library_path, DeviceHandle, DeviceSummary, NativeErrorCode, RuntimeError,
};
use dsview_sys::RuntimeBridge;
use thiserror::Error;

const DSLOGIC_PLUS_MODELS: &[&str] = &["DSLogic PLus"];
const DSLOGIC_PLUS_PRIMARY_FIRMWARES: &[&str] = &["DSLogicPlus.fw"];
const DSLOGIC_PLUS_FIRMWARE_FALLBACKS: &[&str] = &["DSLogic.fw"];
const DSLOGIC_PLUS_BITSTREAMS: &[&str] = &["DSLogicPlus.bin", "DSLogicPlus-pgl12.bin"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportedDeviceKind {
    DsLogicPlus,
}

impl SupportedDeviceKind {
    pub const fn stable_id(self) -> &'static str {
        match self {
            Self::DsLogicPlus => "dslogic-plus",
        }
    }

    pub const fn display_name(self) -> &'static str {
        match self {
            Self::DsLogicPlus => "DSLogic Plus",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SupportedDevice {
    pub handle: DeviceHandle,
    pub name: String,
    pub kind: SupportedDeviceKind,
    pub stable_id: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceDirectory {
    path: PathBuf,
}

impl ResourceDirectory {
    pub fn discover(path: impl AsRef<Path>) -> Result<Self, BringUpError> {
        let path = path.as_ref();
        ensure_resource_file_set(path)?;
        Ok(Self {
            path: path.to_path_buf(),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[derive(Debug)]
pub struct Discovery {
    runtime: RuntimeBridge,
    resources: ResourceDirectory,
}

impl Discovery {
    pub fn connect(library_path: impl AsRef<Path>, resource_dir: impl AsRef<Path>) -> Result<Self, BringUpError> {
        let resources = ResourceDirectory::discover(resource_dir)?;
        let runtime = RuntimeBridge::load(library_path).map_err(BringUpError::Runtime)?;
        runtime
            .set_firmware_resource_dir(resources.path())
            .map_err(BringUpError::Runtime)?;
        runtime.init().map_err(BringUpError::Runtime)?;

        Ok(Self { runtime, resources })
    }

    pub fn connect_auto(resource_dir: impl AsRef<Path>) -> Result<Self, BringUpError> {
        let library_path = source_runtime_library_path().ok_or(BringUpError::SourceRuntimeUnavailable)?;
        Self::connect(library_path, resource_dir)
    }

    pub fn resources(&self) -> &ResourceDirectory {
        &self.resources
    }

    pub fn list_supported_devices(&self) -> Result<Vec<SupportedDevice>, BringUpError> {
        let devices = self.runtime.list_devices().map_err(BringUpError::Runtime)?;
        Ok(filter_supported_devices(&devices))
    }

    pub fn open_device(&self, handle: DeviceHandle) -> Result<OpenedDevice<'_>, BringUpError> {
        let devices = self.list_supported_devices()?;
        let selected = devices
            .into_iter()
            .find(|device| device.handle == handle)
            .ok_or(BringUpError::UnsupportedSelection { handle })?;

        self.runtime.open_device(handle).map_err(BringUpError::Runtime)?;
        let init_status = self.runtime.active_device_init_status().ok();
        let last_error = self.runtime.last_error();
        Ok(OpenedDevice {
            runtime: &self.runtime,
            device: selected,
            init_status,
            last_error,
            released: false,
        })
    }
}

impl Drop for Discovery {
    fn drop(&mut self) {
        let _ = self.runtime.exit();
    }
}

pub struct OpenedDevice<'a> {
    runtime: &'a RuntimeBridge,
    device: SupportedDevice,
    init_status: Option<i32>,
    last_error: NativeErrorCode,
    released: bool,
}

impl<'a> OpenedDevice<'a> {
    pub fn device(&self) -> &SupportedDevice {
        &self.device
    }

    pub fn init_status(&self) -> Option<i32> {
        self.init_status
    }

    pub fn last_error(&self) -> NativeErrorCode {
        self.last_error
    }

    pub fn release(mut self) -> Result<(), BringUpError> {
        self.released = true;
        self.runtime.release_device().map_err(BringUpError::Runtime)
    }
}

impl Drop for OpenedDevice<'_> {
    fn drop(&mut self) {
        if !self.released {
            let _ = self.runtime.release_device();
        }
    }
}

#[derive(Debug, Error)]
pub enum BringUpError {
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
    #[error("no source-built DSView runtime library is available in this build")]
    SourceRuntimeUnavailable,
    #[error("resource directory `{path}` is missing")]
    MissingResourceDirectory { path: PathBuf },
    #[error("resource directory `{path}` is not readable")]
    UnreadableResourceDirectory { path: PathBuf },
    #[error("resource directory `{path}` is missing required files: {missing:?}")]
    MissingResourceFiles { path: PathBuf, missing: Vec<&'static str> },
    #[error("selected device handle `{handle}` is not a supported DSLogic Plus")]
    UnsupportedSelection { handle: DeviceHandle },
    #[error("no supported DSLogic Plus devices are currently available")]
    NoSupportedDevices,
}

pub fn filter_supported_devices(devices: &[DeviceSummary]) -> Vec<SupportedDevice> {
    devices
        .iter()
        .filter_map(|device| classify_supported_device(device))
        .collect()
}

pub fn classify_supported_device(device: &DeviceSummary) -> Option<SupportedDevice> {
    if DSLOGIC_PLUS_MODELS.iter().any(|name| *name == device.name) {
        Some(SupportedDevice {
            handle: device.handle,
            name: device.name.clone(),
            kind: SupportedDeviceKind::DsLogicPlus,
            stable_id: SupportedDeviceKind::DsLogicPlus.stable_id(),
        })
    } else {
        None
    }
}

pub fn require_supported_devices(devices: &[DeviceSummary]) -> Result<Vec<SupportedDevice>, BringUpError> {
    let filtered = filter_supported_devices(devices);
    if filtered.is_empty() {
        Err(BringUpError::NoSupportedDevices)
    } else {
        Ok(filtered)
    }
}

pub fn describe_native_error(code: NativeErrorCode) -> &'static str {
    match code {
        NativeErrorCode::FirmwareVersionLow => "Please reconnect the device!",
        NativeErrorCode::FirmwareMissing => "Firmware not exist!",
        NativeErrorCode::DeviceUsbIo => "USB io error!",
        NativeErrorCode::DeviceExclusive => "Device is busy!",
        NativeErrorCode::CallStatus => "The device is not in a state that allows this operation.",
        NativeErrorCode::AlreadyDone => "The DSView runtime has already completed this lifecycle step.",
        NativeErrorCode::Arg => "The native runtime rejected an invalid argument.",
        _ => "The DSView runtime reported an unspecified error.",
    }
}

/// Safe orchestration entry point for the Rust CLI layers.
pub fn workspace_status() -> &'static str {
    "dsview-core ready"
}

fn ensure_resource_file_set(path: &Path) -> Result<(), BringUpError> {
    if !path.exists() {
        return Err(BringUpError::MissingResourceDirectory {
            path: path.to_path_buf(),
        });
    }

    let metadata = fs::metadata(path).map_err(|_| BringUpError::UnreadableResourceDirectory {
        path: path.to_path_buf(),
    })?;
    if !metadata.is_dir() {
        return Err(BringUpError::UnreadableResourceDirectory {
            path: path.to_path_buf(),
        });
    }

    let mut missing = Vec::new();
    if !has_any_file(path, DSLOGIC_PLUS_PRIMARY_FIRMWARES)
        && !has_any_file(path, DSLOGIC_PLUS_FIRMWARE_FALLBACKS)
    {
        missing.push(DSLOGIC_PLUS_PRIMARY_FIRMWARES[0]);
        missing.extend_from_slice(DSLOGIC_PLUS_FIRMWARE_FALLBACKS);
    }

    for required in DSLOGIC_PLUS_BITSTREAMS {
        if !path.join(required).is_file() {
            missing.push(*required);
        }
    }

    if missing.is_empty() {
        Ok(())
    } else {
        Err(BringUpError::MissingResourceFiles {
            path: path.to_path_buf(),
            missing,
        })
    }
}

fn has_any_file(path: &Path, candidates: &[&str]) -> bool {
    candidates.iter().any(|candidate| path.join(candidate).is_file())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("dsview-core-{name}-{unique}"));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn filters_only_dslogic_plus_models() {
        let devices = vec![
            DeviceSummary {
                handle: DeviceHandle::new(1).unwrap(),
                name: "DSLogic PLus".into(),
            },
            DeviceSummary {
                handle: DeviceHandle::new(2).unwrap(),
                name: "DSLogic U2Basic".into(),
            },
        ];

        let filtered = filter_supported_devices(&devices);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].stable_id, "dslogic-plus");
        assert_eq!(filtered[0].kind, SupportedDeviceKind::DsLogicPlus);
    }

    #[test]
    fn require_supported_devices_rejects_empty_supported_set() {
        let devices = vec![DeviceSummary {
            handle: DeviceHandle::new(5).unwrap(),
            name: "DSLogic Basic".into(),
        }];

        let error = require_supported_devices(&devices).unwrap_err();
        assert!(matches!(error, BringUpError::NoSupportedDevices));
    }

    #[test]
    fn resource_directory_requires_expected_files() {
        let dir = temp_dir("resources-missing");
        fs::write(dir.join("DSLogicPlus.fw"), b"fw").unwrap();

        let error = ResourceDirectory::discover(&dir).unwrap_err();
        match error {
            BringUpError::MissingResourceFiles { missing, .. } => {
                assert!(missing.contains(&"DSLogicPlus.bin"));
                assert!(missing.contains(&"DSLogicPlus-pgl12.bin"));
            }
            other => panic!("expected missing resource files error, got {other:?}"),
        }
    }

    #[test]
    fn resource_directory_accepts_dslogic_firmware_fallback() {
        let dir = temp_dir("resources-fallback");
        fs::write(dir.join("DSLogic.fw"), b"fw").unwrap();
        fs::write(dir.join("DSLogicPlus.bin"), b"bin").unwrap();
        fs::write(dir.join("DSLogicPlus-pgl12.bin"), b"bin").unwrap();

        let discovered = ResourceDirectory::discover(&dir).unwrap();
        assert_eq!(discovered.path(), dir.as_path());
    }

    #[test]
    fn connect_auto_requires_source_runtime_when_not_built() {
        let dir = temp_dir("resources-auto");
        fs::write(dir.join("DSLogicPlus.fw"), b"fw").unwrap();
        fs::write(dir.join("DSLogicPlus.bin"), b"bin").unwrap();
        fs::write(dir.join("DSLogicPlus-pgl12.bin"), b"bin").unwrap();

        if source_runtime_library_path().is_none() {
            let error = Discovery::connect_auto(&dir).unwrap_err();
            assert!(matches!(error, BringUpError::SourceRuntimeUnavailable));
        }
    }

    #[test]
    fn resource_directory_accepts_complete_file_set() {
        let dir = temp_dir("resources-complete");
        fs::write(dir.join("DSLogicPlus.fw"), b"fw").unwrap();
        fs::write(dir.join("DSLogicPlus.bin"), b"bin").unwrap();
        fs::write(dir.join("DSLogicPlus-pgl12.bin"), b"bin").unwrap();

        let discovered = ResourceDirectory::discover(&dir).unwrap();
        assert_eq!(discovered.path(), dir.as_path());
    }

    #[test]
    fn native_error_messages_match_upstream_wording() {
        assert_eq!(describe_native_error(NativeErrorCode::FirmwareVersionLow), "Please reconnect the device!");
        assert_eq!(describe_native_error(NativeErrorCode::FirmwareMissing), "Firmware not exist!");
        assert_eq!(describe_native_error(NativeErrorCode::DeviceUsbIo), "USB io error!");
        assert_eq!(describe_native_error(NativeErrorCode::DeviceExclusive), "Device is busy!");
    }
}
