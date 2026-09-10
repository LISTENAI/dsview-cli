use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::str::FromStr;

use crate::{NativeErrorCode, RuntimeError};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureConfigRequest {
    pub sample_rate_hz: u64,
    pub sample_limit: u64,
    pub enabled_channels: BTreeSet<u16>,
    /// Optional simple trigger. `None` starts the capture immediately.
    pub trigger: Option<TriggerSpec>,
}

/// Condition a single channel must meet for the trigger to fire.
///
/// These map onto the uppercase characters libsigrok4DSL matches on; the
/// lowercase spellings are what a user types on the command line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerCondition {
    DontCare,
    Low,
    High,
    Rising,
    Falling,
    Either,
}

impl TriggerCondition {
    pub fn as_native_char(self) -> u8 {
        match self {
            TriggerCondition::DontCare => b'X',
            TriggerCondition::Low => b'0',
            TriggerCondition::High => b'1',
            TriggerCondition::Rising => b'R',
            TriggerCondition::Falling => b'F',
            TriggerCondition::Either => b'C',
        }
    }

    pub fn token(self) -> &'static str {
        match self {
            TriggerCondition::DontCare => "x",
            TriggerCondition::Low => "0",
            TriggerCondition::High => "1",
            TriggerCondition::Rising => "r",
            TriggerCondition::Falling => "f",
            TriggerCondition::Either => "c",
        }
    }
}

impl fmt::Display for TriggerCondition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.token())
    }
}

impl FromStr for TriggerCondition {
    type Err = TriggerParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "x" => Ok(TriggerCondition::DontCare),
            "0" | "low" => Ok(TriggerCondition::Low),
            "1" | "high" => Ok(TriggerCondition::High),
            "r" | "rising" => Ok(TriggerCondition::Rising),
            "f" | "falling" => Ok(TriggerCondition::Falling),
            "c" | "change" | "either" => Ok(TriggerCondition::Either),
            other => Err(TriggerParseError::UnknownCondition(other.to_string())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TriggerParseError {
    #[error("trigger term '{0}' is not CHANNEL=CONDITION")]
    MalformedTerm(String),
    #[error("'{0}' is not a channel index")]
    BadChannel(String),
    #[error("unknown trigger condition '{0}'; expected one of 0 1 r f c x")]
    UnknownCondition(String),
    #[error("channel {0} appears more than once")]
    DuplicateChannel(u16),
    #[error("trigger position must be 0-100, got {0}")]
    BadPosition(u32),
    #[error("every channel is 'x', so the trigger would never fire")]
    AllDontCare,
}

/// A simple (single-stage) trigger: a condition per channel, plus how much of
/// the capture buffer to keep from before the trigger fires.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TriggerSpec {
    pub conditions: BTreeMap<u16, TriggerCondition>,
    pub position_percent: u16,
}

impl TriggerSpec {
    pub const DEFAULT_POSITION_PERCENT: u16 = 0;

    /// Parse a spec like `0=r,1=1,3=f`. Whitespace around terms is ignored.
    pub fn parse(spec: &str, position_percent: u16) -> Result<Self, TriggerParseError> {
        if position_percent > 100 {
            return Err(TriggerParseError::BadPosition(u32::from(position_percent)));
        }

        let mut conditions = BTreeMap::new();
        for term in spec.split(',').map(str::trim).filter(|t| !t.is_empty()) {
            let (channel_text, condition_text) = term
                .split_once('=')
                .ok_or_else(|| TriggerParseError::MalformedTerm(term.to_string()))?;
            let channel_text = channel_text.trim();
            // accept both "0" and the "D0" spelling the VCD output uses
            let normalised = channel_text
                .strip_prefix(['D', 'd'])
                .unwrap_or(channel_text);
            let channel: u16 = normalised
                .parse()
                .map_err(|_| TriggerParseError::BadChannel(channel_text.to_string()))?;
            let condition: TriggerCondition = condition_text.parse()?;
            if conditions.insert(channel, condition).is_some() {
                return Err(TriggerParseError::DuplicateChannel(channel));
            }
        }

        if conditions
            .values()
            .all(|c| matches!(c, TriggerCondition::DontCare))
        {
            return Err(TriggerParseError::AllDontCare);
        }

        Ok(Self {
            conditions,
            position_percent,
        })
    }

    /// Channels referenced by the trigger that are not in `enabled`. The device
    /// cannot trigger on a channel it is not sampling, so this is an error the
    /// caller should surface rather than let the capture hang.
    pub fn channels_not_enabled(&self, enabled: &[u16]) -> Vec<u16> {
        self.conditions
            .keys()
            .copied()
            .filter(|c| !enabled.contains(c))
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelModeCapability {
    pub id: i16,
    pub name: String,
    pub max_enabled_channels: u16,
    pub supported_sample_rates: Vec<u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CaptureCapabilities {
    pub total_channel_count: u16,
    pub active_channel_mode: i16,
    pub channel_modes: Vec<ChannelModeCapability>,
    pub hardware_sample_capacity: u64,
    pub sample_limit_alignment: u64,
    pub threshold_volts: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedCaptureConfig {
    pub sample_rate_hz: u64,
    pub requested_sample_limit: u64,
    pub effective_sample_limit: u64,
    pub enabled_channels: Vec<u16>,
    pub channel_mode_id: i16,
    pub trigger: Option<TriggerSpec>,
}

impl CaptureConfigError {
    pub fn from_runtime_error(error: RuntimeError) -> Self {
        match error {
            RuntimeError::NativeCall {
                operation:
                    "ds_get_current_channel_mode"
                    | "ds_get_channel_modes"
                    | "ds_get_valid_channel_count",
                code: NativeErrorCode::NotApplicable,
            } => Self::UnknownChannelMode { mode: -1 },
            other => Self::Runtime(other.to_string()),
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CaptureConfigError {
    #[error("runtime capability load failed: {0}")]
    Runtime(String),
    #[error("sample rate must be greater than zero")]
    EmptySampleRate,
    #[error("sample limit must be greater than zero")]
    EmptySampleLimit,
    #[error("at least one logic channel must be enabled")]
    NoEnabledChannels,
    #[error("channel {channel} is outside the supported DSLogic Plus range")]
    ChannelOutOfRange { channel: u16 },
    #[error("active channel mode `{mode}` is not available in the capability snapshot")]
    UnknownChannelMode { mode: i16 },
    #[error(
        "sample rate {sample_rate_hz} Hz is not supported in active channel mode `{mode_name}`"
    )]
    UnsupportedSampleRate {
        sample_rate_hz: u64,
        mode_name: String,
    },
    #[error(
        "enabled channel count {enabled_channel_count} exceeds the active channel mode limit of {max_enabled_channels}"
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
}

impl CaptureCapabilities {
    pub fn active_mode(&self) -> Result<&ChannelModeCapability, CaptureConfigError> {
        self.channel_modes
            .iter()
            .find(|mode| mode.id == self.active_channel_mode)
            .ok_or(CaptureConfigError::UnknownChannelMode {
                mode: self.active_channel_mode,
            })
    }

    pub fn validate_request(
        &self,
        request: &CaptureConfigRequest,
    ) -> Result<ValidatedCaptureConfig, CaptureConfigError> {
        if request.sample_rate_hz == 0 {
            return Err(CaptureConfigError::EmptySampleRate);
        }
        if request.sample_limit == 0 {
            return Err(CaptureConfigError::EmptySampleLimit);
        }
        if request.enabled_channels.is_empty() {
            return Err(CaptureConfigError::NoEnabledChannels);
        }

        let active_mode = self.active_mode()?;
        let enabled_channels = request.enabled_channels.iter().copied().collect::<Vec<_>>();

        for channel in &enabled_channels {
            if *channel >= self.total_channel_count {
                return Err(CaptureConfigError::ChannelOutOfRange { channel: *channel });
            }
        }

        if enabled_channels.len() > active_mode.max_enabled_channels as usize {
            return Err(CaptureConfigError::TooManyEnabledChannels {
                enabled_channel_count: enabled_channels.len(),
                max_enabled_channels: active_mode.max_enabled_channels,
            });
        }

        if !active_mode
            .supported_sample_rates
            .contains(&request.sample_rate_hz)
        {
            return Err(CaptureConfigError::UnsupportedSampleRate {
                sample_rate_hz: request.sample_rate_hz,
                mode_name: active_mode.name.clone(),
            });
        }

        let effective_sample_limit =
            align_sample_limit(request.sample_limit, self.sample_limit_alignment);

        if !is_stream_mode_label(&active_mode.name) {
            let maximum_sample_limit = align_down(
                self.hardware_sample_capacity / enabled_channels.len() as u64,
                self.sample_limit_alignment,
            );

            if maximum_sample_limit == 0 || effective_sample_limit > maximum_sample_limit {
                return Err(CaptureConfigError::SampleLimitExceedsCapacity {
                    effective_sample_limit,
                    maximum_sample_limit,
                    enabled_channel_count: enabled_channels.len(),
                });
            }
        }

        Ok(ValidatedCaptureConfig {
            trigger: request.trigger.clone(),
            sample_rate_hz: request.sample_rate_hz,
            requested_sample_limit: request.sample_limit,
            effective_sample_limit,
            enabled_channels,
            channel_mode_id: active_mode.id,
        })
    }
}

pub(crate) fn is_stream_mode_label(label: &str) -> bool {
    label
        .split(|character: char| !character.is_ascii_alphanumeric())
        .any(|token| token.eq_ignore_ascii_case("stream"))
}

pub(crate) fn align_sample_limit(value: u64, alignment: u64) -> u64 {
    if alignment <= 1 {
        value
    } else {
        let remainder = value % alignment;
        if remainder == 0 {
            value
        } else {
            value.checked_add(alignment - remainder).unwrap_or(u64::MAX)
        }
    }
}

pub(crate) fn align_down(value: u64, alignment: u64) -> u64 {
    if alignment <= 1 {
        value
    } else {
        value - (value % alignment)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
                ChannelModeCapability {
                    id: 30,
                    name: "Stream 20x16".to_string(),
                    max_enabled_channels: 16,
                    supported_sample_rates: vec![20_000_000, 25_000_000],
                },
            ],
            hardware_sample_capacity: 268_435_456,
            sample_limit_alignment: 1024,
            threshold_volts: Some(3.3),
        }
    }

    fn request(
        sample_rate_hz: u64,
        sample_limit: u64,
        enabled_channels: &[u16],
    ) -> CaptureConfigRequest {
        CaptureConfigRequest {
            sample_rate_hz,
            sample_limit,
            enabled_channels: enabled_channels.iter().copied().collect(),
            trigger: None,
        }
    }

    #[test]
    fn validates_supported_configuration() {
        let capabilities = dslogic_plus_capabilities();
        let validated = capabilities
            .validate_request(&request(100_000_000, 1500, &[0, 1, 2, 3]))
            .unwrap();

        assert_eq!(validated.effective_sample_limit, 2048);
        assert_eq!(validated.channel_mode_id, 20);
        assert_eq!(validated.enabled_channels, vec![0, 1, 2, 3]);
    }

    #[test]
    fn rejects_zero_enabled_channels() {
        let capabilities = dslogic_plus_capabilities();
        let error = capabilities
            .validate_request(&request(100_000_000, 2048, &[]))
            .unwrap_err();
        assert_eq!(error, CaptureConfigError::NoEnabledChannels);
    }

    #[test]
    fn rejects_out_of_range_channel() {
        let capabilities = dslogic_plus_capabilities();
        let error = capabilities
            .validate_request(&request(100_000_000, 2048, &[16]))
            .unwrap_err();
        assert_eq!(error, CaptureConfigError::ChannelOutOfRange { channel: 16 });
    }

    #[test]
    fn rejects_unsupported_sample_rate_for_active_mode() {
        let capabilities = dslogic_plus_capabilities();
        let error = capabilities
            .validate_request(&request(200_000_000, 2048, &[0, 1, 2, 3]))
            .unwrap_err();
        assert_eq!(
            error,
            CaptureConfigError::UnsupportedSampleRate {
                sample_rate_hz: 200_000_000,
                mode_name: "Buffer 100x16".to_string(),
            }
        );
    }

    #[test]
    fn rejects_too_many_enabled_channels_for_active_mode() {
        let mut capabilities = dslogic_plus_capabilities();
        capabilities.active_channel_mode = 22;
        let error = capabilities
            .validate_request(&request(400_000_000, 2048, &[0, 1, 2, 3, 4]))
            .unwrap_err();
        assert_eq!(
            error,
            CaptureConfigError::TooManyEnabledChannels {
                enabled_channel_count: 5,
                max_enabled_channels: 4,
            }
        );
    }

    #[test]
    fn rejects_depth_that_exceeds_capacity_for_enabled_channel_count() {
        let capabilities = dslogic_plus_capabilities();
        let error = capabilities
            .validate_request(&request(100_000_000, 80_000_000, &[0, 1, 2, 3]))
            .unwrap_err();
        assert_eq!(
            error,
            CaptureConfigError::SampleLimitExceedsCapacity {
                effective_sample_limit: 80_000_000,
                maximum_sample_limit: 67_108_864,
                enabled_channel_count: 4,
            }
        );
    }

    #[test]
    fn enabled_channel_count_changes_maximum_depth() {
        let capabilities = dslogic_plus_capabilities();

        let valid_two_channel =
            capabilities.validate_request(&request(100_000_000, 80_000_000, &[0, 1]));
        assert!(valid_two_channel.is_ok());

        let invalid_four_channel =
            capabilities.validate_request(&request(100_000_000, 80_000_000, &[0, 1, 2, 3]));
        assert!(matches!(
            invalid_four_channel,
            Err(CaptureConfigError::SampleLimitExceedsCapacity {
                enabled_channel_count: 4,
                ..
            })
        ));
    }

    #[test]
    fn stream_mode_allows_sample_limit_beyond_buffer_capacity() {
        let mut capabilities = dslogic_plus_capabilities();
        capabilities.active_channel_mode = 30;

        let validated = capabilities
            .validate_request(&request(25_000_000, 300_000_000, &[0, 1, 2, 3]))
            .unwrap();

        assert_eq!(validated.channel_mode_id, 30);
        assert_eq!(validated.effective_sample_limit, 300_000_256);
    }

    #[test]
    fn rejects_sample_limit_that_overflows_during_alignment() {
        let capabilities = dslogic_plus_capabilities();
        let error = capabilities
            .validate_request(&request(100_000_000, u64::MAX, &[0, 1, 2, 3]))
            .unwrap_err();

        assert!(matches!(
            error,
            CaptureConfigError::SampleLimitExceedsCapacity {
                effective_sample_limit: u64::MAX,
                enabled_channel_count: 4,
                ..
            }
        ));
    }

    #[test]
    fn unrelated_not_applicable_errors_stay_runtime_errors() {
        let error = CaptureConfigError::from_runtime_error(RuntimeError::NativeCall {
            operation: "ds_get_hw_depth",
            code: NativeErrorCode::NotApplicable,
        });

        assert!(matches!(error, CaptureConfigError::Runtime(_)));
    }
}
