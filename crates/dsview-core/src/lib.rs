use std::fmt;
use std::fs;
use std::num::NonZeroU64;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

mod capture_config;

pub use capture_config::{
    CaptureCapabilities, CaptureConfigError, CaptureConfigRequest, ChannelModeCapability,
    ValidatedCaptureConfig,
};
pub use dsview_sys::{
    AcquisitionSummary, AcquisitionTerminalEvent, DeviceHandle, DeviceSummary, ExportErrorCode,
    NativeErrorCode, RuntimeError, VcdExportFacts, VcdExportRequest, source_runtime_library_path,
};
use dsview_sys::{AcquisitionPacketStatus, RuntimeBridge};
use serde::Serialize;
use thiserror::Error;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

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
    RunFailure,
    Incomplete,
    CleanupFailure,
    Timeout,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CaptureCleanup {
    pub collecting_before_cleanup: bool,
    pub stop_attempted: bool,
    pub stop_succeeded: bool,
    pub collecting_before_release: bool,
    pub callbacks_cleared: bool,
    pub release_succeeded: bool,
    pub stop_error: Option<String>,
    pub clear_callbacks_error: Option<String>,
    pub release_error: Option<String>,
}

impl CaptureCleanup {
    pub fn succeeded(&self) -> bool {
        (!self.stop_attempted || self.stop_succeeded)
            && !self.collecting_before_release
            && self.callbacks_cleared
            && self.release_succeeded
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureRunSummary {
    pub completion: CaptureCompletion,
    pub summary: AcquisitionSummary,
    pub cleanup: CaptureCleanup,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureExportRequest {
    pub capture: CaptureRunSummary,
    pub validated_config: ValidatedCaptureConfig,
    pub vcd_path: PathBuf,
    pub tool_name: String,
    pub tool_version: String,
    pub capture_started_at: SystemTime,
    pub device_model: String,
    pub device_stable_id: String,
    pub selected_handle: SelectionHandle,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MetadataToolInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MetadataCaptureInfo {
    pub timestamp_utc: String,
    pub device_model: String,
    pub device_stable_id: String,
    pub selected_handle: u64,
    pub sample_rate_hz: u64,
    pub requested_sample_limit: u64,
    pub actual_sample_count: u64,
    pub enabled_channels: Vec<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MetadataAcquisitionInfo {
    pub completion: String,
    pub terminal_event: String,
    pub saw_logic_packet: bool,
    pub saw_end_packet: bool,
    pub end_packet_status: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MetadataArtifactInfo {
    pub vcd_path: String,
    pub metadata_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CaptureMetadata {
    pub schema_version: u32,
    pub tool: MetadataToolInfo,
    pub capture: MetadataCaptureInfo,
    pub acquisition: MetadataAcquisitionInfo,
    pub artifacts: MetadataArtifactInfo,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureExportSuccess {
    pub vcd_path: PathBuf,
    pub metadata_path: PathBuf,
    pub export: VcdExportFacts,
    pub metadata: CaptureMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaptureExportFailureKind {
    Precondition { code: ExportErrorCode },
    Runtime,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CaptureExportError {
    #[error("capture completion `{completion:?}` is not export-eligible")]
    CaptureNotExportable { completion: CaptureCompletion },
    #[error("export failed for `{path}` with {kind:?}: {detail}")]
    ExportFailed {
        path: PathBuf,
        kind: CaptureExportFailureKind,
        detail: String,
    },
    #[error("metadata serialization failed for `{path}`: {detail}")]
    MetadataSerializationFailed { path: PathBuf, detail: String },
    #[error("metadata write failed for `{path}`: {detail}")]
    MetadataWriteFailed { path: PathBuf, detail: String },
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
    StartFailed {
        code: NativeErrorCode,
        last_error: NativeErrorCode,
        cleanup: CaptureCleanup,
    },
    #[error("capture ended with a terminal runtime error")]
    RunFailed {
        summary: AcquisitionSummary,
        cleanup: CaptureCleanup,
    },
    #[error("capture ended because the device detached")]
    Detached {
        summary: AcquisitionSummary,
        cleanup: CaptureCleanup,
    },
    #[error("capture completed without the clean finite-capture signal")]
    Incomplete {
        summary: AcquisitionSummary,
        cleanup: CaptureCleanup,
    },
    #[error("capture timed out before natural completion")]
    Timeout {
        summary: AcquisitionSummary,
        cleanup: CaptureCleanup,
    },
    #[error("capture cleanup failed during {during}: {cleanup:?}")]
    CleanupFailed {
        during: &'static str,
        summary: AcquisitionSummary,
        cleanup: CaptureCleanup,
    },
}

#[derive(Debug)]
pub struct CaptureSession<'a> {
    opened: OpenedDevice<'a>,
}

impl<'a> CaptureSession<'a> {
    pub fn device(&self) -> &SupportedDevice {
        self.opened.device()
    }

    fn cleanup(self, runtime: &RuntimeBridge, stop_if_collecting: bool) -> CaptureCleanup {
        let mut cleanup = CaptureCleanup::default();

        let collecting_state = runtime.acquisition_state().ok();
        cleanup.collecting_before_cleanup = collecting_state
            .map(|state| state.is_collecting)
            .unwrap_or(false);

        if stop_if_collecting && cleanup.collecting_before_cleanup {
            cleanup.stop_attempted = true;
            match runtime.stop_collect() {
                Ok(outcome) => {
                    cleanup.stop_succeeded = true;
                    cleanup.collecting_before_release = outcome.summary.is_collecting;
                }
                Err(error) => {
                    cleanup.stop_error = Some(error.to_string());
                    cleanup.collecting_before_release = runtime
                        .acquisition_state()
                        .map(|state| state.is_collecting)
                        .unwrap_or(true);
                }
            }
        } else {
            cleanup.collecting_before_release = cleanup.collecting_before_cleanup;
        }

        match runtime.clear_acquisition_callbacks() {
            Ok(_) => cleanup.callbacks_cleared = true,
            Err(error) => cleanup.clear_callbacks_error = Some(error.to_string()),
        }

        match self.opened.release() {
            Ok(()) => cleanup.release_succeeded = true,
            Err(error) => cleanup.release_error = Some(error.to_string()),
        }

        cleanup
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
            let cleanup = session.cleanup(&self.runtime, true);
            return Err(if cleanup.succeeded() {
                CaptureRunError::StartFailed {
                    code: started.start_status,
                    last_error: started.summary.last_error,
                    cleanup,
                }
            } else {
                CaptureRunError::CleanupFailed {
                    during: "start_failure",
                    summary: started.summary,
                    cleanup,
                }
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

        let completion = if summary.is_collecting {
            CaptureCompletion::Timeout
        } else {
            classify_capture_completion(&summary)
        };
        let cleanup = session.cleanup(&self.runtime, true);

        if !cleanup.succeeded() {
            return Err(CaptureRunError::CleanupFailed {
                during: capture_completion_stage(completion),
                summary,
                cleanup,
            });
        }

        match completion {
            CaptureCompletion::CleanSuccess => Ok(CaptureRunSummary {
                completion,
                summary,
                cleanup,
            }),
            CaptureCompletion::StartFailure => Err(CaptureRunError::StartFailed {
                code: NativeErrorCode::from_raw(summary.start_status),
                last_error: summary.last_error,
                cleanup,
            }),
            CaptureCompletion::Timeout => Err(CaptureRunError::Timeout { summary, cleanup }),
            CaptureCompletion::Detached => Err(CaptureRunError::Detached { summary, cleanup }),
            CaptureCompletion::RunFailure => Err(CaptureRunError::RunFailed { summary, cleanup }),
            CaptureCompletion::Incomplete => Err(CaptureRunError::Incomplete { summary, cleanup }),
            CaptureCompletion::CleanupFailure => Err(CaptureRunError::CleanupFailed {
                during: "capture_completion",
                summary,
                cleanup,
            }),
        }
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

pub fn metadata_path_for_vcd(vcd_path: &Path) -> PathBuf {
    let mut metadata_path = vcd_path.to_path_buf();
    metadata_path.set_extension("json");
    metadata_path
}

fn classify_capture_completion(summary: &AcquisitionSummary) -> CaptureCompletion {
    if !NativeErrorCode::from_raw(summary.start_status).is_ok() {
        return CaptureCompletion::StartFailure;
    }
    if summary.is_collecting {
        return CaptureCompletion::CleanupFailure;
    }
    match summary.terminal_event {
        AcquisitionTerminalEvent::EndByDetached => return CaptureCompletion::Detached,
        AcquisitionTerminalEvent::EndByError => return CaptureCompletion::RunFailure,
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
        Some(AcquisitionPacketStatus::Ok) => CaptureCompletion::CleanSuccess,
        None | Some(_) => CaptureCompletion::Incomplete,
    }
}

fn capture_completion_stage(completion: CaptureCompletion) -> &'static str {
    match completion {
        CaptureCompletion::CleanSuccess => "clean_success",
        CaptureCompletion::StartFailure => "start_failure",
        CaptureCompletion::Detached => "detach",
        CaptureCompletion::RunFailure => "run_failure",
        CaptureCompletion::Incomplete => "incomplete",
        CaptureCompletion::CleanupFailure => "cleanup",
        CaptureCompletion::Timeout => "timeout",
    }
}

fn export_failure_kind(error: &RuntimeError) -> CaptureExportFailureKind {
    match error {
        RuntimeError::ExportCall { code, .. } => match code {
            ExportErrorCode::NoStream
            | ExportErrorCode::Overflow
            | ExportErrorCode::BadEndStatus
            | ExportErrorCode::MissingSamplerate
            | ExportErrorCode::NoEnabledChannels => {
                CaptureExportFailureKind::Precondition { code: *code }
            }
            ExportErrorCode::Generic
            | ExportErrorCode::OutputModuleUnavailable
            | ExportErrorCode::Runtime
            | ExportErrorCode::Unknown(_) => CaptureExportFailureKind::Runtime,
        },
        RuntimeError::TempWrite { .. } | RuntimeError::TempPromote { .. } => {
            CaptureExportFailureKind::Runtime
        }
        _ => CaptureExportFailureKind::Runtime,
    }
}

fn build_vcd_export_request(config: &ValidatedCaptureConfig) -> VcdExportRequest {
    VcdExportRequest {
        samplerate_hz: config.sample_rate_hz,
        enabled_channels: config.enabled_channels.clone(),
    }
}

fn capture_timestamp_utc(started_at: SystemTime) -> Result<String, String> {
    let timestamp = OffsetDateTime::from(started_at)
        .format(&Rfc3339)
        .map_err(|error| error.to_string())?;
    Ok(timestamp)
}

fn end_packet_status_name(status: Option<AcquisitionPacketStatus>) -> Option<String> {
    status.map(|value| match value {
        AcquisitionPacketStatus::Ok => "ok".to_string(),
        AcquisitionPacketStatus::SourceError => "source_error".to_string(),
        AcquisitionPacketStatus::DataError => "data_error".to_string(),
        AcquisitionPacketStatus::Unknown(raw) => format!("unknown_{raw}"),
    })
}

fn terminal_event_name(event: AcquisitionTerminalEvent) -> String {
    match event {
        AcquisitionTerminalEvent::None => "none".to_string(),
        AcquisitionTerminalEvent::NormalEnd => "normal_end".to_string(),
        AcquisitionTerminalEvent::EndByDetached => "end_by_detached".to_string(),
        AcquisitionTerminalEvent::EndByError => "end_by_error".to_string(),
        AcquisitionTerminalEvent::Unknown(raw) => format!("unknown_{raw}"),
    }
}

fn completion_name(completion: CaptureCompletion) -> String {
    match completion {
        CaptureCompletion::CleanSuccess => "clean_success".to_string(),
        CaptureCompletion::StartFailure => "start_failure".to_string(),
        CaptureCompletion::Detached => "detach".to_string(),
        CaptureCompletion::RunFailure => "run_failure".to_string(),
        CaptureCompletion::Incomplete => "incomplete".to_string(),
        CaptureCompletion::CleanupFailure => "cleanup_failure".to_string(),
        CaptureCompletion::Timeout => "timeout".to_string(),
    }
}

fn build_capture_metadata(
    request: &CaptureExportRequest,
    metadata_path: &Path,
    export: &VcdExportFacts,
) -> Result<CaptureMetadata, String> {
    Ok(CaptureMetadata {
        schema_version: 1,
        tool: MetadataToolInfo {
            name: request.tool_name.clone(),
            version: request.tool_version.clone(),
        },
        capture: MetadataCaptureInfo {
            timestamp_utc: capture_timestamp_utc(request.capture_started_at)?,
            device_model: request.device_model.clone(),
            device_stable_id: request.device_stable_id.clone(),
            selected_handle: request.selected_handle.raw(),
            sample_rate_hz: request.validated_config.sample_rate_hz,
            requested_sample_limit: request.validated_config.requested_sample_limit,
            actual_sample_count: export.sample_count,
            enabled_channels: request.validated_config.enabled_channels.clone(),
        },
        acquisition: MetadataAcquisitionInfo {
            completion: completion_name(request.capture.completion),
            terminal_event: terminal_event_name(request.capture.summary.terminal_event),
            saw_logic_packet: request.capture.summary.saw_logic_packet,
            saw_end_packet: request.capture.summary.saw_end_packet,
            end_packet_status: end_packet_status_name(request.capture.summary.end_packet_status),
        },
        artifacts: MetadataArtifactInfo {
            vcd_path: request.vcd_path.display().to_string(),
            metadata_path: metadata_path.display().to_string(),
        },
    })
}

fn write_metadata_atomically(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("metadata path `{}` has no parent directory", path.display()))?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;

    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("metadata path `{}` has no valid file name", path.display()))?;
    let temp_path = parent.join(format!(".{file_name}.tmp"));

    fs::write(&temp_path, bytes).map_err(|error| error.to_string())?;
    if let Err(error) = fs::rename(&temp_path, path) {
        let _ = fs::remove_file(&temp_path);
        return Err(error.to_string());
    }

    Ok(())
}

impl Discovery {
    pub fn export_clean_capture_vcd(
        &self,
        request: &CaptureExportRequest,
    ) -> Result<CaptureExportSuccess, CaptureExportError> {
        if request.capture.completion != CaptureCompletion::CleanSuccess {
            return Err(CaptureExportError::CaptureNotExportable {
                completion: request.capture.completion,
            });
        }

        let export_request = build_vcd_export_request(&request.validated_config);
        let vcd_path = request.vcd_path.clone();
        let metadata_path = metadata_path_for_vcd(&vcd_path);
        let export = self
            .runtime
            .export_recorded_vcd_to_path(&export_request, &vcd_path)
            .map_err(|error| CaptureExportError::ExportFailed {
                path: vcd_path.clone(),
                kind: export_failure_kind(&error),
                detail: error.to_string(),
            })?;
        let metadata = build_capture_metadata(request, &metadata_path, &export).map_err(|detail| {
            CaptureExportError::MetadataSerializationFailed {
                path: metadata_path.clone(),
                detail,
            }
        })?;
        let metadata_bytes =
            serde_json::to_vec_pretty(&metadata).map_err(|error| {
                CaptureExportError::MetadataSerializationFailed {
                    path: metadata_path.clone(),
                    detail: error.to_string(),
                }
            })?;
        write_metadata_atomically(&metadata_path, &metadata_bytes).map_err(|detail| {
            CaptureExportError::MetadataWriteFailed {
                path: metadata_path.clone(),
                detail,
            }
        })?;

        Ok(CaptureExportSuccess {
            vcd_path,
            metadata_path,
            export,
            metadata,
        })
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

    fn export_request(vcd_path: PathBuf) -> CaptureExportRequest {
        CaptureExportRequest {
            capture: CaptureRunSummary {
                completion: CaptureCompletion::CleanSuccess,
                summary: clean_summary(),
                cleanup: CaptureCleanup {
                    callbacks_cleared: true,
                    release_succeeded: true,
                    ..CaptureCleanup::default()
                },
            },
            validated_config: ValidatedCaptureConfig {
                sample_rate_hz: 100_000_000,
                requested_sample_limit: 2048,
                effective_sample_limit: 2048,
                enabled_channels: vec![0, 1, 2, 3],
                channel_mode_id: 20,
            },
            vcd_path,
            tool_name: "dsview-cli".to_string(),
            tool_version: "0.1.0".to_string(),
            capture_started_at: UNIX_EPOCH + Duration::from_secs(1_744_018_496),
            device_model: "DSLogic Plus".to_string(),
            device_stable_id: "dslogic-plus".to_string(),
            selected_handle: SelectionHandle::new(7).unwrap(),
        }
    }

    #[test]
    fn clean_summary_maps_to_clean_success() {
        let summary = clean_summary();
        assert_eq!(
            classify_capture_completion(&summary),
            CaptureCompletion::CleanSuccess
        );
    }

    #[test]
    fn missing_logic_packet_maps_to_incomplete() {
        let mut summary = clean_summary();
        summary.saw_logic_packet = false;
        assert_eq!(
            classify_capture_completion(&summary),
            CaptureCompletion::Incomplete
        );
    }

    #[test]
    fn missing_normal_terminal_event_maps_to_incomplete() {
        let mut summary = clean_summary();
        summary.saw_terminal_normal_end = false;
        assert_eq!(
            classify_capture_completion(&summary),
            CaptureCompletion::Incomplete
        );
    }

    #[test]
    fn missing_end_packet_status_maps_to_incomplete() {
        let mut summary = clean_summary();
        summary.end_packet_status = None;
        assert_eq!(
            classify_capture_completion(&summary),
            CaptureCompletion::Incomplete
        );
    }

    #[test]
    fn detached_terminal_event_maps_to_detached() {
        let mut summary = clean_summary();
        summary.terminal_event = AcquisitionTerminalEvent::EndByDetached;
        summary.saw_terminal_normal_end = false;
        summary.saw_terminal_end_by_detached = true;
        assert_eq!(classify_capture_completion(&summary), CaptureCompletion::Detached);
    }

    #[test]
    fn error_terminal_event_maps_to_run_failure() {
        let mut summary = clean_summary();
        summary.terminal_event = AcquisitionTerminalEvent::EndByError;
        summary.saw_terminal_normal_end = false;
        summary.saw_terminal_end_by_error = true;
        assert_eq!(
            classify_capture_completion(&summary),
            CaptureCompletion::RunFailure
        );
    }

    #[test]
    fn collecting_summary_maps_to_cleanup_failure() {
        let mut summary = clean_summary();
        summary.is_collecting = true;
        assert_eq!(
            classify_capture_completion(&summary),
            CaptureCompletion::CleanupFailure
        );
    }

    #[test]
    fn export_clean_capture_writes_vcd_before_metadata_and_keeps_paths_in_sync() {
        let dir = temp_dir("export-artifacts-order");
        let vcd_path = dir.join("capture.vcd");
        let metadata_path = metadata_path_for_vcd(&vcd_path);
        let vcd_bytes = b"$date <normalized> $end\n#0 0!\n";

        fs::write(&vcd_path, vcd_bytes).unwrap();

        let request = export_request(vcd_path.clone());
        let export = VcdExportFacts {
            sample_count: 2048,
            packet_count: 3,
            output_bytes: vcd_bytes.len() as u64,
        };
        let metadata = build_capture_metadata(&request, &metadata_path, &export).unwrap();
        let metadata_bytes = serde_json::to_vec_pretty(&metadata).unwrap();
        write_metadata_atomically(&metadata_path, &metadata_bytes).unwrap();

        let vcd_meta = fs::metadata(&vcd_path).unwrap();
        let metadata_meta = fs::metadata(&metadata_path).unwrap();
        let metadata_json: serde_json::Value =
            serde_json::from_slice(&fs::read(&metadata_path).unwrap()).unwrap();

        assert_eq!(fs::read(&vcd_path).unwrap(), vcd_bytes);
        assert_eq!(metadata_path_for_vcd(&vcd_path), metadata_path);
        assert_eq!(metadata_json["artifacts"]["vcd_path"], vcd_path.display().to_string());
        assert_eq!(metadata_json["artifacts"]["metadata_path"], metadata_path.display().to_string());
        assert_eq!(metadata_json["capture"]["actual_sample_count"], export.sample_count);
        assert!(metadata_meta.modified().unwrap() >= vcd_meta.modified().unwrap());
    }

    #[test]
    fn write_metadata_atomically_cleans_up_temp_file_after_success() {
        let dir = temp_dir("metadata-atomic-write");
        let metadata_path = dir.join("capture.json");
        let temp_path = dir.join(".capture.json.tmp");

        write_metadata_atomically(&metadata_path, br#"{"ok":true}"#).unwrap();

        assert!(metadata_path.exists());
        assert!(!temp_path.exists());
    }

    #[test]
    fn build_capture_metadata_uses_export_sample_count_and_normal_end_shape() {
        let request = export_request(PathBuf::from("/tmp/capture.vcd"));
        let metadata_path = metadata_path_for_vcd(&request.vcd_path);
        let export = VcdExportFacts {
            sample_count: 1536,
            packet_count: 4,
            output_bytes: 512,
        };

        let metadata = build_capture_metadata(&request, &metadata_path, &export).unwrap();

        assert_eq!(metadata.capture.actual_sample_count, 1536);
        assert_eq!(metadata.capture.sample_rate_hz, 100_000_000);
        assert_eq!(metadata.capture.requested_sample_limit, 2048);
        assert_eq!(metadata.capture.enabled_channels, vec![0, 1, 2, 3]);
        assert_eq!(metadata.acquisition.completion, "clean_success");
        assert_eq!(metadata.acquisition.terminal_event, "normal_end");
        assert_eq!(metadata.acquisition.end_packet_status.as_deref(), Some("ok"));
        assert_eq!(metadata.artifacts.vcd_path, "/tmp/capture.vcd");
        assert_eq!(metadata.artifacts.metadata_path, "/tmp/capture.json");
    }

    #[test]
    fn failed_cleanup_maps_to_cleanup_failure() {
        let cleanup = CaptureCleanup {
            callbacks_cleared: true,
            release_succeeded: false,
            release_error: Some("release failed".to_string()),
            ..CaptureCleanup::default()
        };
        assert!(!cleanup.succeeded());
    }
}
