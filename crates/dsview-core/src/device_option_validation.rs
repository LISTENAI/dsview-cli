use std::collections::BTreeSet;

use thiserror::Error;

use crate::{
    capture_config::{align_down, align_sample_limit},
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
    pub fn code(&self) -> &'static str {
        match self {
            Self::Runtime(_) => "validation_runtime_error",
            Self::UnknownOperationMode { .. } => "operation_mode_unsupported",
            Self::UnknownStopOption { .. } => "stop_option_unsupported",
            Self::StopOptionIncompatibleWithMode { .. } => "stop_option_incompatible",
            Self::UnknownChannelMode { .. } => "channel_mode_unsupported",
            Self::ChannelModeIncompatibleWithOperationMode { .. } => "channel_mode_incompatible",
            Self::EmptySampleRate => "sample_rate_missing",
            Self::EmptySampleLimit => "sample_limit_missing",
            Self::NoEnabledChannels => "enabled_channels_empty",
            Self::ChannelOutOfRange { .. } => "channel_out_of_range",
            Self::UnsupportedSampleRate { .. } => "sample_rate_unsupported",
            Self::TooManyEnabledChannels { .. } => "enabled_channels_exceed_mode_limit",
            Self::SampleLimitExceedsCapacity { .. } => "sample_limit_exceeds_capacity",
            Self::ThresholdOutOfRange { .. } => "threshold_out_of_range",
            Self::ThresholdStepInvalid { .. } => "threshold_step_invalid",
            Self::UnknownFilter { .. } => "filter_unsupported",
        }
    }
}

impl DeviceOptionValidationCapabilities {
    pub fn validate_request(
        &self,
        request: &DeviceOptionValidationRequest,
    ) -> Result<ValidatedDeviceOptionRequest, DeviceOptionValidationError> {
        if request.sample_rate_hz == 0 {
            return Err(DeviceOptionValidationError::EmptySampleRate);
        }
        if request.sample_limit == 0 {
            return Err(DeviceOptionValidationError::EmptySampleLimit);
        }
        if request.enabled_channels.is_empty() {
            return Err(DeviceOptionValidationError::NoEnabledChannels);
        }

        let operation_mode = self
            .operation_modes
            .iter()
            .find(|mode| mode.id == request.operation_mode_id)
            .ok_or_else(|| DeviceOptionValidationError::UnknownOperationMode {
                operation_mode_id: request.operation_mode_id.clone(),
            })?;
        let channel_mode = self.resolve_channel_mode(operation_mode, &request.channel_mode_id)?;

        for channel in request.enabled_channels.iter().copied() {
            if channel >= self.total_channel_count {
                return Err(DeviceOptionValidationError::ChannelOutOfRange {
                    channel,
                    total_channel_count: self.total_channel_count,
                });
            }
        }

        if request.enabled_channels.len() > channel_mode.max_enabled_channels as usize {
            return Err(DeviceOptionValidationError::TooManyEnabledChannels {
                enabled_channel_count: request.enabled_channels.len(),
                max_enabled_channels: channel_mode.max_enabled_channels,
            });
        }

        if !channel_mode
            .supported_sample_rates
            .contains(&request.sample_rate_hz)
        {
            return Err(DeviceOptionValidationError::UnsupportedSampleRate {
                sample_rate_hz: request.sample_rate_hz,
                channel_mode_id: channel_mode.id.clone(),
            });
        }

        let effective_sample_limit =
            align_sample_limit(request.sample_limit, self.sample_limit_alignment);
        let maximum_sample_limit = align_down(
            self.hardware_sample_capacity / request.enabled_channels.len() as u64,
            self.sample_limit_alignment,
        );
        if maximum_sample_limit == 0 || effective_sample_limit > maximum_sample_limit {
            return Err(DeviceOptionValidationError::SampleLimitExceedsCapacity {
                effective_sample_limit,
                maximum_sample_limit,
                enabled_channel_count: request.enabled_channels.len(),
            });
        }

        let stop_option_code =
            self.resolve_stop_option_code(operation_mode, request.stop_option_id.as_deref())?;
        let filter_code = self.resolve_filter_code(request.filter_id.as_deref())?;

        if let Some(threshold_volts) = request.threshold_volts {
            self.validate_threshold(threshold_volts)?;
        }

        Ok(ValidatedDeviceOptionRequest {
            operation_mode_id: operation_mode.id.clone(),
            operation_mode_code: operation_mode.native_code,
            stop_option_id: request.stop_option_id.clone(),
            stop_option_code,
            channel_mode_id: channel_mode.id.clone(),
            channel_mode_code: channel_mode.native_code,
            sample_rate_hz: request.sample_rate_hz,
            requested_sample_limit: request.sample_limit,
            effective_sample_limit,
            enabled_channels: request.enabled_channels.iter().copied().collect(),
            threshold_volts: request.threshold_volts,
            filter_id: request.filter_id.clone(),
            filter_code,
        })
    }

    fn resolve_channel_mode<'a>(
        &'a self,
        operation_mode: &'a OperationModeValidationCapabilities,
        requested_channel_mode_id: &str,
    ) -> Result<&'a ChannelModeValidationCapabilities, DeviceOptionValidationError> {
        if let Some(channel_mode) = operation_mode
            .channel_modes
            .iter()
            .find(|channel_mode| channel_mode.id == requested_channel_mode_id)
        {
            return Ok(channel_mode);
        }

        if self
            .operation_modes
            .iter()
            .flat_map(|mode| mode.channel_modes.iter())
            .any(|channel_mode| channel_mode.id == requested_channel_mode_id)
        {
            return Err(
                DeviceOptionValidationError::ChannelModeIncompatibleWithOperationMode {
                    operation_mode_id: operation_mode.id.clone(),
                    channel_mode_id: requested_channel_mode_id.to_string(),
                },
            );
        }

        Err(DeviceOptionValidationError::UnknownChannelMode {
            channel_mode_id: requested_channel_mode_id.to_string(),
        })
    }

    fn resolve_stop_option_code(
        &self,
        operation_mode: &OperationModeValidationCapabilities,
        requested_stop_option_id: Option<&str>,
    ) -> Result<Option<i16>, DeviceOptionValidationError> {
        let Some(requested_stop_option_id) = requested_stop_option_id else {
            return Ok(None);
        };

        if operation_mode
            .stop_option_ids
            .iter()
            .any(|stop_option_id| stop_option_id == requested_stop_option_id)
        {
            return parse_native_code(STOP_OPTION_PREFIX, requested_stop_option_id)
                .ok_or_else(|| DeviceOptionValidationError::UnknownStopOption {
                    stop_option_id: requested_stop_option_id.to_string(),
                })
                .map(Some);
        }

        if self.operation_modes.iter().any(|mode| {
            mode.stop_option_ids
                .iter()
                .any(|id| id == requested_stop_option_id)
        }) {
            return Err(
                DeviceOptionValidationError::StopOptionIncompatibleWithMode {
                    stop_option_id: requested_stop_option_id.to_string(),
                    operation_mode_id: operation_mode.id.clone(),
                },
            );
        }

        Err(DeviceOptionValidationError::UnknownStopOption {
            stop_option_id: requested_stop_option_id.to_string(),
        })
    }

    fn resolve_filter_code(
        &self,
        requested_filter_id: Option<&str>,
    ) -> Result<Option<i16>, DeviceOptionValidationError> {
        let Some(requested_filter_id) = requested_filter_id else {
            return Ok(None);
        };

        self.filters
            .iter()
            .find(|filter| filter.id == requested_filter_id)
            .map(|filter| Some(filter.native_code))
            .ok_or_else(|| DeviceOptionValidationError::UnknownFilter {
                filter_id: requested_filter_id.to_string(),
            })
    }

    fn validate_threshold(&self, threshold_volts: f64) -> Result<(), DeviceOptionValidationError> {
        if threshold_volts < self.threshold.min_volts || threshold_volts > self.threshold.max_volts
        {
            return Err(DeviceOptionValidationError::ThresholdOutOfRange {
                threshold_volts,
                min_volts: self.threshold.min_volts,
                max_volts: self.threshold.max_volts,
            });
        }

        let normalized_steps =
            (threshold_volts - self.threshold.min_volts) / self.threshold.step_volts;
        let rounded_steps = normalized_steps.round();
        if (normalized_steps - rounded_steps).abs() > 1e-6 {
            return Err(DeviceOptionValidationError::ThresholdStepInvalid {
                threshold_volts,
                min_volts: self.threshold.min_volts,
                step_volts: self.threshold.step_volts,
            });
        }

        Ok(())
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

fn parse_native_code(prefix: &str, value: &str) -> Option<i16> {
    value
        .strip_prefix(prefix)?
        .strip_prefix(':')?
        .parse::<i16>()
        .ok()
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
                stop_option_ids: operation_mode
                    .stop_options
                    .iter()
                    .map(|option| stop_option_id(option.code))
                    .collect(),
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
