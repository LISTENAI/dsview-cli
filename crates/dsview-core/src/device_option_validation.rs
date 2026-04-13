use std::collections::BTreeSet;

use thiserror::Error;

use crate::{
    CurrentDeviceOptionValues, DeviceIdentitySnapshot, EnumOptionSnapshot, SupportedDevice,
    ThresholdCapabilitySnapshot,
};
use dsview_sys::DeviceOptionValidationSnapshot as NativeDeviceOptionValidationSnapshot;

pub(crate) const OPERATION_MODE_PREFIX: &str = "operation-mode";
pub(crate) const STOP_OPTION_PREFIX: &str = "stop-option";
pub(crate) const FILTER_PREFIX: &str = "filter";
pub(crate) const CHANNEL_MODE_PREFIX: &str = "channel-mode";

#[derive(Debug, Clone, PartialEq)]
pub struct DeviceOptionValidationRequest {
    pub operation_mode_id: String,
    pub stop_option_id: Option<String>,
    pub channel_mode_id: String,
    pub sample_rate_hz: u64,
    pub sample_limit: u64,
    pub enabled_channels: BTreeSet<u16>,
    pub threshold_volts: Option<f64>,
    pub filter_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelModeValidationCapabilities {
    pub id: String,
    pub native_code: i16,
    pub label: String,
    pub max_enabled_channels: u16,
    pub supported_sample_rates: Vec<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationModeValidationCapabilities {
    pub id: String,
    pub native_code: i16,
    pub label: String,
    pub stop_option_ids: Vec<String>,
    pub channel_modes: Vec<ChannelModeValidationCapabilities>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeviceOptionValidationCapabilities {
    pub device: DeviceIdentitySnapshot,
    pub current: CurrentDeviceOptionValues,
    pub total_channel_count: u16,
    pub hardware_sample_capacity: u64,
    pub sample_limit_alignment: u64,
    pub operation_modes: Vec<OperationModeValidationCapabilities>,
    pub filters: Vec<EnumOptionSnapshot>,
    pub threshold: ThresholdCapabilitySnapshot,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValidatedDeviceOptionRequest {
    pub operation_mode_id: String,
    pub operation_mode_code: i16,
    pub stop_option_id: Option<String>,
    pub stop_option_code: Option<i16>,
    pub channel_mode_id: String,
    pub channel_mode_code: i16,
    pub sample_rate_hz: u64,
    pub requested_sample_limit: u64,
    pub effective_sample_limit: u64,
    pub enabled_channels: Vec<u16>,
    pub threshold_volts: Option<f64>,
    pub filter_id: Option<String>,
    pub filter_code: Option<i16>,
}

#[derive(Debug, Error, Clone, PartialEq)]
pub enum DeviceOptionValidationError {
    #[error("validation capability load failed: {0}")]
    Runtime(String),
    #[error("operation mode `{operation_mode_id}` is not supported by the selected device")]
    UnknownOperationMode { operation_mode_id: String },
    #[error("stop option `{stop_option_id}` is not supported by the selected device")]
    UnknownStopOption { stop_option_id: String },
    #[error(
        "stop option `{stop_option_id}` is not compatible with operation mode `{operation_mode_id}`"
    )]
    StopOptionIncompatibleWithMode {
        stop_option_id: String,
        operation_mode_id: String,
    },
    #[error("channel mode `{channel_mode_id}` is not supported by the selected device")]
    UnknownChannelMode { channel_mode_id: String },
    #[error(
        "channel mode `{channel_mode_id}` is not available for operation mode `{operation_mode_id}`"
    )]
    ChannelModeIncompatibleWithOperationMode {
        operation_mode_id: String,
        channel_mode_id: String,
    },
    #[error("sample rate must be greater than zero")]
    EmptySampleRate,
    #[error("sample limit must be greater than zero")]
    EmptySampleLimit,
    #[error("at least one logic channel must be enabled")]
    NoEnabledChannels,
    #[error(
        "channel {channel} is outside the supported DSLogic Plus range 0..{total_channel_count}"
    )]
    ChannelOutOfRange {
        channel: u16,
        total_channel_count: u16,
    },
    #[error(
        "sample rate {sample_rate_hz} Hz is not supported in channel mode `{channel_mode_id}`"
    )]
    UnsupportedSampleRate {
        sample_rate_hz: u64,
        channel_mode_id: String,
    },
    #[error(
        "enabled channel count {enabled_channel_count} exceeds the channel mode limit of {max_enabled_channels}"
    )]
    TooManyEnabledChannels {
        enabled_channel_count: usize,
        max_enabled_channels: u16,
    },
    #[error(
        "effective sample limit {effective_sample_limit} exceeds the maximum {maximum_sample_limit} for {enabled_channel_count} enabled channels"
    )]
    SampleLimitExceedsCapacity {
        effective_sample_limit: u64,
        maximum_sample_limit: u64,
        enabled_channel_count: usize,
    },
    #[error(
        "threshold {threshold_volts} V is outside the supported range {min_volts}..={max_volts} V"
    )]
    ThresholdOutOfRange {
        threshold_volts: f64,
        min_volts: f64,
        max_volts: f64,
    },
    #[error(
        "threshold {threshold_volts} V does not align to the supported {step_volts} V step from {min_volts} V"
    )]
    ThresholdStepInvalid {
        threshold_volts: f64,
        min_volts: f64,
        step_volts: f64,
    },
    #[error("filter `{filter_id}` is not supported by the selected device")]
    UnknownFilter { filter_id: String },
}

impl DeviceOptionValidationError {
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Runtime(_) => "validation_runtime_error",
            Self::UnknownOperationMode { .. } => "invalid_operation_mode",
            Self::UnknownStopOption { .. } => "invalid_stop_option",
            Self::StopOptionIncompatibleWithMode { .. } => "stop_option_incompatible",
            Self::UnknownChannelMode { .. } => "invalid_channel_mode",
            Self::ChannelModeIncompatibleWithOperationMode { .. } => "channel_mode_incompatible",
            Self::EmptySampleRate => "sample_rate_required",
            Self::EmptySampleLimit => "sample_limit_required",
            Self::NoEnabledChannels => "enabled_channels_required",
            Self::ChannelOutOfRange { .. } => "channel_out_of_range",
            Self::UnsupportedSampleRate { .. } => "sample_rate_unsupported",
            Self::TooManyEnabledChannels { .. } => "enabled_channels_exceed_mode_limit",
            Self::SampleLimitExceedsCapacity { .. } => "sample_limit_exceeds_capacity",
            Self::ThresholdOutOfRange { .. } => "threshold_out_of_range",
            Self::ThresholdStepInvalid { .. } => "threshold_step_invalid",
            Self::UnknownFilter { .. } => "invalid_filter",
        }
    }
}

pub(crate) fn operation_mode_id(code: i16) -> String {
    format!("{OPERATION_MODE_PREFIX}:{code}")
}

pub(crate) fn stop_option_id(code: i16) -> String {
    format!("{STOP_OPTION_PREFIX}:{code}")
}

pub(crate) fn filter_id(code: i16) -> String {
    format!("{FILTER_PREFIX}:{code}")
}

pub(crate) fn channel_mode_id(code: i16) -> String {
    format!("{CHANNEL_MODE_PREFIX}:{code}")
}

pub(crate) fn normalize_device_option_validation_capabilities(
    device: &SupportedDevice,
    native: NativeDeviceOptionValidationSnapshot,
) -> DeviceOptionValidationCapabilities {
    DeviceOptionValidationCapabilities {
        device: DeviceIdentitySnapshot {
            selection_handle: device.selection_handle.raw(),
            native_handle: device.native_handle.raw(),
            stable_id: device.stable_id.to_string(),
            kind: device.kind.display_name().to_string(),
            name: device.name.clone(),
        },
        current: CurrentDeviceOptionValues {
            operation_mode_id: native.current_operation_mode_code.map(operation_mode_id),
            operation_mode_code: native.current_operation_mode_code,
            stop_option_id: native.current_stop_option_code.map(stop_option_id),
            stop_option_code: native.current_stop_option_code,
            filter_id: native.current_filter_code.map(filter_id),
            filter_code: native.current_filter_code,
            channel_mode_id: native.current_channel_mode_code.map(channel_mode_id),
            channel_mode_code: native.current_channel_mode_code,
        },
        total_channel_count: native.total_channel_count,
        hardware_sample_capacity: native.hardware_sample_capacity,
        sample_limit_alignment: 1024,
        operation_modes: native
            .operation_modes
            .into_iter()
            .map(|operation_mode| OperationModeValidationCapabilities {
                id: operation_mode_id(operation_mode.code),
                native_code: operation_mode.code,
                label: operation_mode.label.clone(),
                stop_option_ids: if operation_mode.label.contains("Buffer") {
                    operation_mode
                        .stop_options
                        .iter()
                        .map(|option| stop_option_id(option.code))
                        .collect()
                } else {
                    Vec::new()
                },
                channel_modes: operation_mode
                    .channel_modes
                    .into_iter()
                    .map(|channel_mode| ChannelModeValidationCapabilities {
                        id: channel_mode_id(channel_mode.code),
                        native_code: channel_mode.code,
                        label: channel_mode.label,
                        max_enabled_channels: channel_mode.max_enabled_channels,
                        supported_sample_rates: channel_mode.supported_sample_rates,
                    })
                    .collect(),
            })
            .collect(),
        filters: native
            .filters
            .into_iter()
            .map(|filter| EnumOptionSnapshot {
                id: filter_id(filter.code),
                native_code: filter.code,
                label: filter.label,
            })
            .collect(),
        threshold: ThresholdCapabilitySnapshot {
            id: native.threshold.id,
            kind: native.threshold.kind,
            current_volts: native.threshold.current_volts,
            min_volts: native.threshold.min_volts,
            max_volts: native.threshold.max_volts,
            step_volts: native.threshold.step_volts,
            legacy_metadata: native.threshold.legacy.map(|legacy| {
                crate::LegacyThresholdMetadataSnapshot {
                    current_native_code: legacy.current_code,
                    options: legacy
                        .options
                        .into_iter()
                        .map(|option| crate::RawOptionMetadataSnapshot {
                            native_code: option.code,
                            label: option.label,
                        })
                        .collect(),
                }
            }),
        },
    }
}
