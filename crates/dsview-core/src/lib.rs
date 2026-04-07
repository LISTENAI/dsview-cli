use std::fmt;
use std::fs;
use std::num::NonZeroU64;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

mod capture_config;

pub use capture_config::{
    CaptureCapabilities, CaptureConfigError, CaptureConfigRequest, ChannelModeCapability,
    ValidatedCaptureConfig,
};
use dsview_sys::{
    AcquisitionPacketStatus, AcquisitionSummary, AcquisitionTerminalEvent, RuntimeBridge,
};
pub use dsview_sys::{
    DeviceHandle, DeviceSummary, NativeErrorCode, RuntimeError, source_runtime_library_path,
};
use thiserror::Error;

const DSLOGIC_PLUS_MODELS: &[&str] = &["DSLogic PLus"];
const DSLOGIC_PLUS_PRIMARY_FIRMWARES: &[&str] = &["DSLogicPlus.fw"];
const DSLOGIC_PLUS_FIRMWARE_FALLBACKS: &[&str] = &["DSLogic.fw"];
const DSLOGIC_PLUS_BITSTREAMS: &[&str] = &["DSLogicPlus.bin", "DSLogicPlus-pgl12.bin"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectionHandle(NonZeroU64);

impl SelectionHandle {
    pub fn for_supported_index(index: usize) -> Option<Self> {
        let raw = u64::try_from(index).ok()?.checked_add(1)?;
        NonZeroU64::new(raw).map(Self)
    }

    pub fn new(raw: u64) -> Option<Self> {
        NonZeroU64::new(raw).map(Self)
    }

    pub const fn raw(self) -> u64 {
        self.0.get()
    }
}

impl fmt::Display for SelectionHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.raw())
    }
}

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
    pub selection_handle: SelectionHandle,
    pub native_handle: DeviceHandle,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreflightStatus {
    Ready,
    EnvironmentNotReady,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcquisitionPreflight {
    pub usb_permissions_ready: bool,
    pub resource_dir_ready: bool,
    pub source_runtime_ready: bool,
    pub source_runtime_path: Option<PathBuf>,
    pub supported_devices_available: bool,
    pub selected_device_open_ready: bool,
    pub config_apply_ready: bool,
    pub status: PreflightStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureCompletion {
    CleanSuccess,
    StartFailure,
    Detached,
    ErrorTerminal,
    Incomplete,
    CleanupFailure,
    Timeout,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureRunSummary {
    pub completion: CaptureCompletion,
    pub summary: AcquisitionSummary,
    pub cleanup_succeeded: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureRunRequest {
    pub selection_handle: SelectionHandle,
    pub config: CaptureConfigRequest,
    pub wait_timeout: Duration,
    pub poll_interval: Duration,
}

#[derive(Debug, Error)]
pub enum CaptureRunError {
    #[error(transparent)]
    BringUp(#[from] BringUpError),
    #[error("capture preflight is not ready for execution")]
    EnvironmentNotReady,
    #[error("capture start failed with {code:?}")]
    StartFailed { code: NativeErrorCode },
    #[error("capture timed out before natural completion")]
    Timeout,
    #[error("capture completed without the clean finite-capture signal")]
    Incomplete,
    #[error("capture cleanup failed: {0}")]
    CleanupFailed(String),
}

#[derive(Debug)]
pub struct CaptureSession<'a> {
    opened: OpenedDevice<'a>,
}

impl<'a> CaptureSession<'a> {
    pub fn device(&self) -> &SupportedDevice {
        self.opened.device()
    }

    pub fn release(self) -> Result<(), BringUpError> {
        self.opened.release()
    }
}

#[derive(Debug)]
pub struct Discovery {
    runtime: RuntimeBridge,
    resources: ResourceDirectory,
}

impl Discovery {
    pub fn connect(
        library_path: impl AsRef<Path>,
        resource_dir: impl AsRef<Path>,
    ) -> Result<Self, BringUpError> {
        let resources = ResourceDirectory::discover(resource_dir)?;
        let runtime = RuntimeBridge::load(library_path).map_err(BringUpError::Runtime)?;
        runtime
            .set_firmware_resource_dir(resources.path())
            .map_err(BringUpError::Runtime)?;
        runtime.init().map_err(BringUpError::Runtime)?;

        Ok(Self { runtime, resources })
    }

    pub fn connect_auto(resource_dir: impl AsRef<Path>) -> Result<Self, BringUpError> {
        let library_path =
            source_runtime_library_path().ok_or(BringUpError::SourceRuntimeUnavailable)?;
        Self::connect(library_path, resource_dir)
    }

    pub fn resources(&self) -> &ResourceDirectory {
        &self.resources
    }

    pub fn list_supported_devices(&self) -> Result<Vec<SupportedDevice>, BringUpError> {
        let devices = self.runtime.list_devices().map_err(BringUpError::Runtime)?;
        Ok(filter_supported_devices(&devices))
    }

    pub fn open_device(
        &self,
        selection_handle: SelectionHandle,
    ) -> Result<OpenedDevice<'_>, BringUpError> {
        let devices = self.list_supported_devices()?;
        let selected = devices
            .into_iter()
            .find(|device| device.selection_handle == selection_handle)
            .ok_or(BringUpError::UnsupportedSelection { selection_handle })?;

        self.runtime
            .open_device(selected.native_handle)
            .map_err(BringUpError::Runtime)?;
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

#[derive(Debug)]
pub struct OpenedDevice<'a> {
    runtime: &'a RuntimeBridge,
    device: SupportedDevice,
    init_status: Option<i32>,
    last_error: NativeErrorCode,
    released: bool,
}

impl Discovery {
    pub fn dslogic_plus_capabilities(&self) -> Result<CaptureCapabilities, CaptureConfigError> {
        let opened = self
            .open_first_supported_device_for_capabilities()
            .map_err(CaptureConfigError::from_runtime_error)?;
        let capabilities = self.dslogic_plus_capabilities_for_opened(&opened)?;
        opened
            .release()
            .map_err(|error| CaptureConfigError::Runtime(error.to_string()))?;
        Ok(capabilities)
    }

    fn open_first_supported_device_for_capabilities(
        &self,
    ) -> Result<OpenedDevice<'_>, RuntimeError> {
        let devices = self.list_supported_devices().map_err(|error| match error {
            BringUpError::Runtime(runtime) => runtime,
            other => RuntimeError::InvalidArgument(other.to_string()),
        })?;
        let selected = devices.into_iter().next().ok_or_else(|| {
            RuntimeError::InvalidArgument(BringUpError::NoSupportedDevices.to_string())
        })?;

        self.open_device(selected.selection_handle)
            .map_err(|error| match error {
                BringUpError::Runtime(runtime) => runtime,
                other => RuntimeError::InvalidArgument(other.to_string()),
            })
    }

    fn dslogic_plus_capabilities_for_opened(
        &self,
        _opened: &OpenedDevice<'_>,
    ) -> Result<CaptureCapabilities, CaptureConfigError> {
        let native = self
            .runtime
            .capture_capabilities()
            .map_err(CaptureConfigError::from_runtime_error)?;
        Ok(CaptureCapabilities {
            total_channel_count: native.total_channel_count,
            active_channel_mode: native.active_channel_mode,
            channel_modes: native
                .channel_modes
                .into_iter()
                .map(|mode| {
                    let max_enabled_channels =
                        channel_mode_max_enabled_channels(&mode.name, mode.max_enabled_channels);
                    ChannelModeCapability {
                        id: mode.id,
                        name: mode.name.clone(),
                        max_enabled_channels: if max_enabled_channels == 0
                            && mode.id == native.active_channel_mode
                        {
                            native.valid_channel_count
                        } else {
                            max_enabled_channels
                        },
                        supported_sample_rates: native.samplerates_hz.clone(),
                    }
                })
                .collect(),
            hardware_sample_capacity: native.hardware_depth,
            sample_limit_alignment: 1024,
            threshold_volts: native.threshold_volts,
        })
    }

    pub fn validate_capture_config(
        &self,
        request: &CaptureConfigRequest,
    ) -> Result<ValidatedCaptureConfig, CaptureConfigError> {
        self.dslogic_plus_capabilities()?.validate_request(request)
    }

    pub fn apply_capture_config(
        &self,
        config: &ValidatedCaptureConfig,
        total_channel_count: u16,
    ) -> Result<(), BringUpError> {
        self.runtime
            .set_enabled_channels(&config.enabled_channels, total_channel_count)
            .map_err(BringUpError::Runtime)?;
        self.runtime
            .set_sample_limit(config.effective_sample_limit)
            .map_err(BringUpError::Runtime)?;
        self.runtime
            .set_samplerate(config.sample_rate_hz)
            .map_err(BringUpError::Runtime)?;
        Ok(())
    }

    pub fn acquisition_preflight(
        &self,
        selection_handle: SelectionHandle,
        request: &CaptureConfigRequest,
    ) -> AcquisitionPreflight {
        let supported_devices = self.list_supported_devices().unwrap_or_default();
        let supported_devices_available = !supported_devices.is_empty();
        let source_runtime_path = source_runtime_library_path().map(Path::to_path_buf);
        let source_runtime_ready = source_runtime_path.is_some();
        let usb_permissions_ready = supported_devices_available;
        let resource_dir_ready = true;

        let mut selected_device_open_ready = false;
        let mut config_apply_ready = false;

        if supported_devices
            .iter()
            .any(|device| device.selection_handle == selection_handle)
        {
            if let Ok(opened) = self.open_device(selection_handle) {
                selected_device_open_ready = true;
                if let Ok(capabilities) = self.dslogic_plus_capabilities_for_opened(&opened) {
                    if let Ok(validated) = capabilities.validate_request(request) {
                        config_apply_ready = self
                            .apply_capture_config(&validated, capabilities.total_channel_count)
                            .is_ok();
                    }
                }
                let _ = opened.release();
            }
        }

        let status = if usb_permissions_ready
            && resource_dir_ready
            && source_runtime_ready
            && selected_device_open_ready
            && config_apply_ready
        {
            PreflightStatus::Ready
        } else {
            PreflightStatus::EnvironmentNotReady
        };

        AcquisitionPreflight {
            usb_permissions_ready,
            resource_dir_ready,
            source_runtime_ready,
            source_runtime_path,
            supported_devices_available,
            selected_device_open_ready,
            config_apply_ready,
            status,
        }
    }

    pub fn prepare_capture_session(
        &self,
        selection_handle: SelectionHandle,
        config: &ValidatedCaptureConfig,
    ) -> Result<CaptureSession<'_>, CaptureRunError> {
        let request = CaptureConfigRequest {
            sample_rate_hz: config.sample_rate_hz,
            sample_limit: config.requested_sample_limit,
            enabled_channels: config.enabled_channels.iter().copied().collect(),
        };
        let preflight = self.acquisition_preflight(selection_handle, &request);
        if preflight.status != PreflightStatus::Ready {
            return Err(CaptureRunError::EnvironmentNotReady);
        }

        let opened = self.open_device(selection_handle)?;
        let capabilities = self
            .dslogic_plus_capabilities_for_opened(&opened)
            .map_err(|error| {
                CaptureRunError::BringUp(BringUpError::Runtime(RuntimeError::InvalidArgument(
                    error.to_string(),
                )))
            })?;
        self.apply_capture_config(config, capabilities.total_channel_count)?;
        self.runtime
            .reset_acquisition_summary()
            .map_err(BringUpError::Runtime)?;
        self.runtime
            .register_acquisition_callbacks()
            .map_err(BringUpError::Runtime)?;
        Ok(CaptureSession { opened })
    }

    pub fn run_capture(
        &self,
        request: &CaptureRunRequest,
    ) -> Result<CaptureRunSummary, CaptureRunError> {
        let validated = self
            .validate_capture_config(&request.config)
            .map_err(|error| {
                CaptureRunError::BringUp(BringUpError::Runtime(RuntimeError::InvalidArgument(
                    error.to_string(),
                )))
            })?;
        let session = self.prepare_capture_session(request.selection_handle, &validated)?;

        let started = self
            .runtime
            .start_collect()
            .map_err(BringUpError::Runtime)?;
        if !started.start_status.is_ok() {
            let _ = self.runtime.clear_acquisition_callbacks();
            let _ = session.release();
            return Err(CaptureRunError::StartFailed {
                code: started.start_status,
            });
        }

        let deadline = Instant::now() + request.wait_timeout;
        let mut summary = started.summary;
        while Instant::now() < deadline {
            summary = self
                .runtime
                .acquisition_summary()
                .map_err(BringUpError::Runtime)?;
            if matches!(
                summary.terminal_event,
                AcquisitionTerminalEvent::NormalEnd
                    | AcquisitionTerminalEvent::EndByDetached
                    | AcquisitionTerminalEvent::EndByError
            ) && !summary.is_collecting
            {
                break;
            }
            thread::sleep(request.poll_interval);
        }

        if summary.is_collecting {
            let _ = self.runtime.stop_collect();
            let _ = self.runtime.clear_acquisition_callbacks();
            let _ = session.release();
            return Err(CaptureRunError::Timeout);
        }

        let cleanup_succeeded =
            self.runtime.clear_acquisition_callbacks().is_ok() && session.release().is_ok();
        let completion = classify_capture_completion(&summary, cleanup_succeeded);
        if !cleanup_succeeded {
            return Err(CaptureRunError::CleanupFailed(
                "failed to clear acquisition callbacks or release the active device".to_string(),
            ));
        }
        if completion != CaptureCompletion::CleanSuccess {
            return Err(match completion {
                CaptureCompletion::Timeout => CaptureRunError::Timeout,
                CaptureCompletion::CleanupFailure => CaptureRunError::CleanupFailed(
                    "capture cleanup did not finish cleanly".to_string(),
                ),
                _ => CaptureRunError::Incomplete,
            });
        }

        Ok(CaptureRunSummary {
            completion,
            summary,
            cleanup_succeeded,
        })
    }
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
    MissingResourceFiles {
        path: PathBuf,
        missing: Vec<&'static str>,
    },
    #[error("selected device handle `{selection_handle}` is not a supported DSLogic Plus")]
    UnsupportedSelection { selection_handle: SelectionHandle },
    #[error("no supported DSLogic Plus devices are currently available")]
    NoSupportedDevices,
}

pub fn filter_supported_devices(devices: &[DeviceSummary]) -> Vec<SupportedDevice> {
    devices
        .iter()
        .filter_map(classify_supported_device_kind)
        .enumerate()
        .filter_map(|(index, (device, kind))| {
            Some(SupportedDevice {
                selection_handle: SelectionHandle::for_supported_index(index)?,
                native_handle: device.handle,
                name: device.name.clone(),
                kind,
                stable_id: kind.stable_id(),
            })
        })
        .collect()
}

pub fn classify_supported_device(device: &DeviceSummary) -> Option<SupportedDevice> {
    classify_supported_device_kind(device).map(|(device, kind)| SupportedDevice {
        selection_handle: SelectionHandle::for_supported_index(0).unwrap(),
        native_handle: device.handle,
        name: device.name.clone(),
        kind,
        stable_id: kind.stable_id(),
    })
}

fn classify_supported_device_kind(
    device: &DeviceSummary,
) -> Option<(&DeviceSummary, SupportedDeviceKind)> {
    if DSLOGIC_PLUS_MODELS.iter().any(|name| *name == device.name) {
        Some((device, SupportedDeviceKind::DsLogicPlus))
    } else {
        None
    }
}

pub fn require_supported_devices(
    devices: &[DeviceSummary],
) -> Result<Vec<SupportedDevice>, BringUpError> {
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
        NativeErrorCode::AlreadyDone => {
            "The DSView runtime has already completed this lifecycle step."
        }
        NativeErrorCode::Arg => "The native runtime rejected an invalid argument.",
        _ => "The DSView runtime reported an unspecified error.",
    }
}

/// Safe orchestration entry point for the Rust CLI layers.
pub fn workspace_status() -> &'static str {
    "dsview-core ready"
}

fn classify_capture_completion(
    summary: &AcquisitionSummary,
    cleanup_succeeded: bool,
) -> CaptureCompletion {
    if !NativeErrorCode::from_raw(summary.start_status).is_ok() {
        return CaptureCompletion::StartFailure;
    }
    if !cleanup_succeeded || summary.is_collecting {
        return CaptureCompletion::CleanupFailure;
    }
    match summary.terminal_event {
        AcquisitionTerminalEvent::EndByDetached => return CaptureCompletion::Detached,
        AcquisitionTerminalEvent::EndByError => return CaptureCompletion::ErrorTerminal,
        AcquisitionTerminalEvent::NormalEnd => {}
        AcquisitionTerminalEvent::None | AcquisitionTerminalEvent::Unknown(_) => {
            return CaptureCompletion::Incomplete;
        }
    }
    if !summary.saw_collect_task_start
        || !summary.saw_device_running
        || !summary.saw_device_stopped
        || !summary.saw_terminal_normal_end
        || summary.saw_terminal_end_by_detached
        || summary.saw_terminal_end_by_error
        || !summary.saw_logic_packet
        || !summary.saw_end_packet
        || !summary.saw_end_packet_ok
        || summary.saw_data_error_packet
    {
        return CaptureCompletion::Incomplete;
    }

    match summary.end_packet_status {
        Some(AcquisitionPacketStatus::Ok) | None => CaptureCompletion::CleanSuccess,
        _ => CaptureCompletion::Incomplete,
    }
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
    candidates
        .iter()
        .any(|candidate| path.join(candidate).is_file())
}

fn channel_mode_max_enabled_channels(name: &str, native_limit: u16) -> u16 {
    if native_limit != 0 {
        return native_limit;
    }

    name.rsplit_once('x')
        .and_then(|(_, tail)| tail.parse::<u16>().ok())
        .unwrap_or(0)
}

#[cfg(test)]
fn dslogic_plus_capabilities() -> CaptureCapabilities {
    CaptureCapabilities {
        total_channel_count: 16,
        active_channel_mode: 20,
        channel_modes: vec![
            ChannelModeCapability {
                id: 20,
                name: "Buffer 100x16".to_string(),
                max_enabled_channels: 16,
                supported_sample_rates: vec![20_000_000, 25_000_000, 50_000_000, 100_000_000],
            },
            ChannelModeCapability {
                id: 21,
                name: "Buffer 200x8".to_string(),
                max_enabled_channels: 8,
                supported_sample_rates: vec![
                    20_000_000,
                    25_000_000,
                    50_000_000,
                    100_000_000,
                    200_000_000,
                ],
            },
            ChannelModeCapability {
                id: 22,
                name: "Buffer 400x4".to_string(),
                max_enabled_channels: 4,
                supported_sample_rates: vec![
                    20_000_000,
                    25_000_000,
                    50_000_000,
                    100_000_000,
                    200_000_000,
                    400_000_000,
                ],
            },
        ],
        hardware_sample_capacity: 268_435_456,
        sample_limit_alignment: 1024,
        threshold_volts: Some(3.3),
    }
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
        assert_eq!(filtered[0].selection_handle.raw(), 1);
        assert_eq!(filtered[0].stable_id, "dslogic-plus");
        assert_eq!(filtered[0].kind, SupportedDeviceKind::DsLogicPlus);
    }

    #[test]
    fn selection_handles_are_stable_across_native_handle_changes() {
        let first_scan = vec![DeviceSummary {
            handle: DeviceHandle::new(101).unwrap(),
            name: "DSLogic PLus".into(),
        }];
        let second_scan = vec![DeviceSummary {
            handle: DeviceHandle::new(202).unwrap(),
            name: "DSLogic PLus".into(),
        }];

        let first = filter_supported_devices(&first_scan);
        let second = filter_supported_devices(&second_scan);

        assert_eq!(first[0].selection_handle, second[0].selection_handle);
        assert_ne!(first[0].native_handle, second[0].native_handle);
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
        assert_eq!(
            describe_native_error(NativeErrorCode::FirmwareVersionLow),
            "Please reconnect the device!"
        );
        assert_eq!(
            describe_native_error(NativeErrorCode::FirmwareMissing),
            "Firmware not exist!"
        );
        assert_eq!(
            describe_native_error(NativeErrorCode::DeviceUsbIo),
            "USB io error!"
        );
        assert_eq!(
            describe_native_error(NativeErrorCode::DeviceExclusive),
            "Device is busy!"
        );
    }

    #[test]
    fn dslogic_plus_capabilities_expose_expected_defaults() {
        let capabilities = super::dslogic_plus_capabilities();
        assert_eq!(capabilities.total_channel_count, 16);
        assert_eq!(capabilities.active_channel_mode, 20);
        assert_eq!(capabilities.channel_modes.len(), 3);
    }

    fn clean_summary() -> AcquisitionSummary {
        AcquisitionSummary {
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
            end_packet_status: Some(AcquisitionPacketStatus::Ok),
            saw_end_packet_ok: true,
            saw_data_error_packet: false,
            last_error: NativeErrorCode::Ok,
            is_collecting: false,
        }
    }

    #[test]
    fn clean_summary_maps_to_clean_success() {
        let summary = clean_summary();
        assert_eq!(
            classify_capture_completion(&summary, true),
            CaptureCompletion::CleanSuccess
        );
    }

    #[test]
    fn missing_logic_packet_maps_to_incomplete() {
        let mut summary = clean_summary();
        summary.saw_logic_packet = false;
        assert_eq!(
            classify_capture_completion(&summary, true),
            CaptureCompletion::Incomplete
        );
    }

    #[test]
    fn detached_terminal_event_maps_to_detached() {
        let mut summary = clean_summary();
        summary.terminal_event = AcquisitionTerminalEvent::EndByDetached;
        summary.saw_terminal_normal_end = false;
        summary.saw_terminal_end_by_detached = true;
        assert_eq!(
            classify_capture_completion(&summary, true),
            CaptureCompletion::Detached
        );
    }

    #[test]
    fn failed_cleanup_maps_to_cleanup_failure() {
        let summary = clean_summary();
        assert_eq!(
            classify_capture_completion(&summary, false),
            CaptureCompletion::CleanupFailure
        );
    }
}
