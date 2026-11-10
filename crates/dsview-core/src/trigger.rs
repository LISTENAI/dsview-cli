//! Hardware trigger configuration for DSLogic captures.
//!
//! This module parses trigger requests (CLI tokens or a JSON config file),
//! validates them against the resolved device options, and builds the native
//! trigger program that `dsview-sys` replays through the DSView setters.
//! The program layout mirrors what DSView's trigger dock writes for the same
//! settings, so a CLI trigger behaves like the equivalent GUI trigger.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use dsview_sys::{
    TRIGGER_PROBE_CAPACITY, TRIGGER_STAGE_CAPACITY, TriggerObservation, TriggerProgram,
    TriggerProgramMode, TriggerStageProgram,
};

use crate::capture_config::is_stream_mode_label;
use crate::device_option_validation::{
    DeviceOptionValidationCapabilities, ValidatedDeviceOptionRequest,
};

/// Version accepted in trigger config files.
pub const TRIGGER_CONFIG_VERSION: u32 = 1;
/// Default pre-trigger share of the capture, as in DSView.
pub const TRIGGER_POSITION_DEFAULT_PERCENT: u8 = 1;
/// Smallest trigger position DSView accepts.
pub const TRIGGER_POSITION_MIN_PERCENT: u8 = 1;
/// Largest buffer-mode trigger position (`DS_MAX_TRIG_PERCENT`).
pub const TRIGGER_POSITION_MAX_PERCENT: u8 = 90;
/// Stream mode fixes the trigger position at this value.
pub const TRIGGER_STREAM_POSITION_PERCENT: u8 = 1;
/// Number of sequential stages in the stage trigger.
pub const TRIGGER_MAX_STAGES: usize = TRIGGER_STAGE_CAPACITY;
/// Number of logic channels a trigger pattern covers.
pub const TRIGGER_CHANNEL_CAPACITY: u16 = TRIGGER_PROBE_CAPACITY as u16;
/// Widest serial trigger data word.
pub const TRIGGER_MAX_SERIAL_DATA_BITS: u8 = 16;
/// Largest stage counter DSView's trigger dock accepts.
pub const TRIGGER_MAX_STAGE_COUNT: u32 = i32::MAX as u32;

const DONT_CARE: u8 = b'X';

/// Per-channel trigger condition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggerCondition {
    Low,
    High,
    Rising,
    Falling,
    Edge,
}

impl TriggerCondition {
    pub const ALL: [Self; 5] = [
        Self::Low,
        Self::High,
        Self::Rising,
        Self::Falling,
        Self::Edge,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::High => "high",
            Self::Rising => "rising",
            Self::Falling => "falling",
            Self::Edge => "edge",
        }
    }

    /// The DSView pattern character for this condition.
    /// True for conditions that match a transition rather than a level.
    pub const fn is_edge(self) -> bool {
        matches!(self, Self::Rising | Self::Falling | Self::Edge)
    }

    pub const fn dsview_symbol(self) -> char {
        match self {
            Self::Low => '0',
            Self::High => '1',
            Self::Rising => 'R',
            Self::Falling => 'F',
            Self::Edge => 'C',
        }
    }

    /// Parses a condition name or its DSView character, ignoring case.
    pub fn parse(token: &str) -> Option<Self> {
        let token = token.trim();
        Self::ALL.into_iter().find(|condition| {
            token.eq_ignore_ascii_case(condition.as_str())
                || token.eq_ignore_ascii_case(condition.dsview_symbol().encode_utf8(&mut [0; 4]))
        })
    }

    const fn native(self) -> u8 {
        self.dsview_symbol() as u8
    }
}

impl<'de> Deserialize<'de> for TriggerCondition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let token = String::deserialize(deserializer)?;
        Self::parse(&token).ok_or_else(|| {
            serde::de::Error::custom(format!(
                "unknown trigger condition `{token}`; expected one of {}",
                condition_names()
            ))
        })
    }
}

fn condition_names() -> String {
    TriggerCondition::ALL
        .iter()
        .map(|condition| condition.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

/// Conditions keyed by logic channel index. Channels not listed are "don't care".
pub type TriggerChannelConditions = BTreeMap<u16, TriggerCondition>;

/// How a stage combines its two patterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggerCombine {
    #[default]
    And,
    Or,
}

/// A set of channel conditions that must hold on one sample, optionally inverted.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct TriggerPattern {
    pub channels: TriggerChannelConditions,
    pub invert: bool,
}

impl TriggerPattern {
    pub fn is_empty(&self) -> bool {
        self.channels.is_empty()
    }
}

/// One stage of a stage trigger.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TriggerStageSpec {
    pub pattern_a: TriggerPattern,
    pub pattern_b: TriggerPattern,
    pub combine: TriggerCombine,
    pub count: u32,
    pub contiguous: bool,
}

/// Serial trigger: match a data word sampled on one channel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TriggerSerialSpec {
    pub start: TriggerChannelConditions,
    pub stop: TriggerChannelConditions,
    pub clock: TriggerChannelConditions,
    pub data_channel: u16,
    pub data_bits: u8,
    /// MSB-first bit pattern of `0`, `1`, and `X`, `data_bits` characters long.
    pub data_value: String,
}

/// A parsed trigger condition before device validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum TriggerSpec {
    Simple { channels: TriggerChannelConditions },
    Stage { stages: Vec<TriggerStageSpec> },
    Serial(TriggerSerialSpec),
}

impl TriggerSpec {
    pub const fn mode_name(&self) -> &'static str {
        match self {
            Self::Simple { .. } => "simple",
            Self::Stage { .. } => "stage",
            Self::Serial(_) => "serial",
        }
    }

    fn referenced_channels(&self) -> BTreeSet<u16> {
        let mut channels = BTreeSet::new();
        match self {
            Self::Simple {
                channels: conditions,
            } => channels.extend(conditions.keys()),
            Self::Stage { stages } => {
                for stage in stages {
                    channels.extend(stage.pattern_a.channels.keys());
                    channels.extend(stage.pattern_b.channels.keys());
                }
            }
            Self::Serial(serial) => {
                channels.extend(serial.start.keys());
                channels.extend(serial.stop.keys());
                channels.extend(serial.clock.keys());
                channels.insert(serial.data_channel);
            }
        }
        channels
    }
}

/// A trigger request: what to trigger on and where the trigger sits in the capture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TriggerRequest {
    pub spec: TriggerSpec,
    /// Pre-trigger share of the capture in percent; `None` uses the default.
    pub position_percent: Option<u8>,
}

/// Device facts a trigger request is validated against.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TriggerValidationContext {
    pub operation_mode_id: String,
    pub stream_mode: bool,
    pub channel_mode_id: String,
    /// Channels usable by the trigger in the selected channel mode (`0..limit`).
    pub channel_limit: u16,
    pub enabled_channels: BTreeSet<u16>,
}

impl TriggerValidationContext {
    pub fn from_device_options(
        capabilities: &DeviceOptionValidationCapabilities,
        request: &ValidatedDeviceOptionRequest,
    ) -> Self {
        let operation_mode = capabilities
            .operation_modes
            .iter()
            .find(|mode| mode.id == request.operation_mode_id);
        let stream_mode = operation_mode.is_some_and(|mode| is_stream_mode_label(&mode.label));
        let channel_limit = operation_mode
            .and_then(|mode| {
                mode.channel_modes
                    .iter()
                    .find(|channel_mode| channel_mode.id == request.channel_mode_id)
            })
            .map(|channel_mode| channel_mode.max_enabled_channels)
            .unwrap_or(capabilities.total_channel_count)
            .min(TRIGGER_CHANNEL_CAPACITY);

        Self {
            operation_mode_id: request.operation_mode_id.clone(),
            stream_mode,
            channel_mode_id: request.channel_mode_id.clone(),
            channel_limit,
            enabled_channels: request.enabled_channels.iter().copied().collect(),
        }
    }
}

/// A trigger request accepted for the selected device options.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedTrigger {
    pub spec: TriggerSpec,
    pub position_percent: u8,
    pub program: TriggerProgram,
}

impl ValidatedTrigger {
    pub const fn mode_name(&self) -> &'static str {
        self.spec.mode_name()
    }
}

/// Trigger facts for one finished capture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureTriggerOutcome {
    pub requested: ValidatedTrigger,
    pub observation: TriggerObservation,
}

/// Requested trigger as reported in capture output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TriggerRequestedReport {
    #[serde(flatten)]
    pub spec: TriggerSpec,
    pub position_percent: u8,
}

/// Device-reported trigger facts as reported in capture output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TriggerObservedReport {
    /// The device sent its trigger header (`SR_DF_TRIGGER`).
    pub trigger_packet: bool,
    /// The trigger header reported a trigger hit.
    pub fired: bool,
    /// Sample index of the trigger point in the exported capture.
    pub sample_index: Option<u64>,
    /// Raw `ds_trigger_pos.status` word.
    pub raw_status: Option<u32>,
}

/// The `trigger` block of capture metadata and CLI output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TriggerReport {
    pub requested: TriggerRequestedReport,
    pub observed: TriggerObservedReport,
}

impl TriggerReport {
    pub fn from_outcome(outcome: &CaptureTriggerOutcome) -> Self {
        let position = outcome.observation.position;
        let fired = outcome.observation.fired();
        Self {
            requested: TriggerRequestedReport {
                spec: outcome.requested.spec.clone(),
                position_percent: outcome.requested.position_percent,
            },
            observed: TriggerObservedReport {
                trigger_packet: outcome.observation.saw_trigger_packet,
                fired,
                sample_index: position
                    .filter(|_| fired)
                    .map(|position| u64::from(position.real_pos)),
                raw_status: position.map(|position| position.raw_status),
            },
        }
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TriggerConfigParseError {
    #[error("trigger condition `{token}` must use CHANNEL=CONDITION, for example `0=rising`")]
    InvalidConditionToken { token: String },
    #[error("trigger channel `{token}` is not a channel index such as `0` or `D0`")]
    InvalidChannel { token: String },
    #[error("unknown trigger condition `{token}`; expected one of {expected}")]
    UnknownCondition { token: String, expected: String },
    #[error("trigger channel {channel} is listed more than once")]
    DuplicateChannel { channel: u16 },
    #[error("trigger config is not valid JSON: {detail}")]
    InvalidJson { detail: String },
    #[error("trigger config must be a JSON object")]
    NotAnObject,
    #[error("trigger config version {version} is not supported; expected {expected}")]
    UnsupportedVersion { version: u64, expected: u32 },
    #[error("trigger config field `mode` must be one of simple, stage, serial")]
    InvalidMode,
    #[error("trigger config for mode `{mode}` is invalid: {detail}")]
    InvalidField { mode: &'static str, detail: String },
    #[error("serial trigger data_value `{value}` is invalid: {detail}")]
    InvalidSerialDataValue { value: String, detail: String },
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TriggerValidationError {
    #[error("simple trigger needs at least one channel condition")]
    EmptySimpleTrigger,
    #[error(
        "trigger channel {channel} is not usable in channel mode `{channel_mode_id}`; it supports channels 0..{channel_limit}"
    )]
    ChannelOutsideChannelMode {
        channel: u16,
        channel_mode_id: String,
        channel_limit: u16,
    },
    #[error("trigger channel {channel} is not enabled; add it to --channels")]
    ChannelNotEnabled { channel: u16 },
    #[error("trigger mode `{mode}` is not supported in stream mode; use a simple trigger")]
    ModeUnsupportedInStream { mode: &'static str },
    #[error(
        "trigger position {position_percent}% is not supported in stream mode; stream mode fixes it at {fixed}%"
    )]
    StreamPositionFixed { position_percent: u8, fixed: u8 },
    #[error("trigger position {position_percent}% is outside {min}..={max}%")]
    PositionOutOfRange {
        position_percent: u8,
        min: u8,
        max: u8,
    },
    #[error("stage trigger needs 1..={max} stages, got {count}")]
    StageCountOutOfRange { count: usize, max: usize },
    #[error("trigger stage {stage}: {detail}")]
    InvalidStage { stage: usize, detail: String },
    #[error("serial trigger: {detail}")]
    InvalidSerial { detail: String },
}

/// Parses `CHANNEL=CONDITION` tokens into a simple trigger.
pub fn parse_simple_trigger_tokens<S: AsRef<str>>(
    tokens: &[S],
) -> Result<TriggerSpec, TriggerConfigParseError> {
    let mut channels = TriggerChannelConditions::new();
    for token in tokens {
        let token = token.as_ref().trim();
        let (channel_token, condition_token) = token
            .split_once('=')
            .or_else(|| token.split_once(':'))
            .ok_or_else(|| TriggerConfigParseError::InvalidConditionToken {
                token: token.to_string(),
            })?;
        let channel = parse_channel_token(channel_token)?;
        let condition = TriggerCondition::parse(condition_token).ok_or_else(|| {
            TriggerConfigParseError::UnknownCondition {
                token: condition_token.trim().to_string(),
                expected: condition_names(),
            }
        })?;
        if channels.insert(channel, condition).is_some() {
            return Err(TriggerConfigParseError::DuplicateChannel { channel });
        }
    }
    Ok(TriggerSpec::Simple { channels })
}

fn parse_channel_token(token: &str) -> Result<u16, TriggerConfigParseError> {
    let trimmed = token.trim();
    let digits = trimmed
        .strip_prefix('D')
        .or_else(|| trimmed.strip_prefix('d'))
        .unwrap_or(trimmed);
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(TriggerConfigParseError::InvalidChannel {
            token: trimmed.to_string(),
        });
    }
    digits
        .parse::<u16>()
        .map_err(|_| TriggerConfigParseError::InvalidChannel {
            token: trimmed.to_string(),
        })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSimpleConfig {
    channels: BTreeMap<String, TriggerCondition>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPattern {
    #[serde(default)]
    channels: BTreeMap<String, TriggerCondition>,
    #[serde(default)]
    invert: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawStage {
    pattern_a: RawPattern,
    #[serde(default)]
    pattern_b: Option<RawPattern>,
    #[serde(default)]
    combine: TriggerCombine,
    #[serde(default = "default_stage_count")]
    count: u32,
    #[serde(default)]
    contiguous: bool,
}

fn default_stage_count() -> u32 {
    1
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawStageConfig {
    stages: Vec<RawStage>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum RawChannelRef {
    Index(u16),
    Token(String),
}

#[derive(Deserialize)]
#[serde(untagged)]
enum RawSerialValue {
    Integer(u64),
    Text(String),
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSerialConfig {
    #[serde(default)]
    start: BTreeMap<String, TriggerCondition>,
    #[serde(default)]
    stop: BTreeMap<String, TriggerCondition>,
    clock: BTreeMap<String, TriggerCondition>,
    data_channel: RawChannelRef,
    data_bits: u8,
    data_value: RawSerialValue,
}

/// Parses a version-1 trigger config document.
pub fn parse_trigger_config(json: &str) -> Result<TriggerSpec, TriggerConfigParseError> {
    let value: Value =
        serde_json::from_str(json).map_err(|error| TriggerConfigParseError::InvalidJson {
            detail: error.to_string(),
        })?;
    let Value::Object(mut object) = value else {
        return Err(TriggerConfigParseError::NotAnObject);
    };

    if let Some(version) = object.remove("version") {
        let version = version.as_u64().unwrap_or(u64::MAX);
        if version != u64::from(TRIGGER_CONFIG_VERSION) {
            return Err(TriggerConfigParseError::UnsupportedVersion {
                version,
                expected: TRIGGER_CONFIG_VERSION,
            });
        }
    }
    let mode = match object.remove("mode") {
        Some(Value::String(mode)) => mode,
        _ => return Err(TriggerConfigParseError::InvalidMode),
    };
    let body = Value::Object(object);

    match mode.as_str() {
        "simple" => {
            let raw: RawSimpleConfig = from_body("simple", body)?;
            Ok(TriggerSpec::Simple {
                channels: convert_conditions(raw.channels)?,
            })
        }
        "stage" => {
            let raw: RawStageConfig = from_body("stage", body)?;
            let stages = raw
                .stages
                .into_iter()
                .map(convert_stage)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(TriggerSpec::Stage { stages })
        }
        "serial" => {
            let raw: RawSerialConfig = from_body("serial", body)?;
            convert_serial(raw).map(TriggerSpec::Serial)
        }
        _ => Err(TriggerConfigParseError::InvalidMode),
    }
}

fn from_body<T: for<'de> Deserialize<'de>>(
    mode: &'static str,
    body: Value,
) -> Result<T, TriggerConfigParseError> {
    serde_json::from_value(body).map_err(|error| TriggerConfigParseError::InvalidField {
        mode,
        detail: error.to_string(),
    })
}

fn convert_conditions(
    raw: BTreeMap<String, TriggerCondition>,
) -> Result<TriggerChannelConditions, TriggerConfigParseError> {
    let mut channels = TriggerChannelConditions::new();
    for (token, condition) in raw {
        let channel = parse_channel_token(&token)?;
        if channels.insert(channel, condition).is_some() {
            return Err(TriggerConfigParseError::DuplicateChannel { channel });
        }
    }
    Ok(channels)
}

fn convert_pattern(raw: RawPattern) -> Result<TriggerPattern, TriggerConfigParseError> {
    Ok(TriggerPattern {
        channels: convert_conditions(raw.channels)?,
        invert: raw.invert,
    })
}

fn convert_stage(raw: RawStage) -> Result<TriggerStageSpec, TriggerConfigParseError> {
    Ok(TriggerStageSpec {
        pattern_a: convert_pattern(raw.pattern_a)?,
        pattern_b: raw
            .pattern_b
            .map(convert_pattern)
            .transpose()?
            .unwrap_or_default(),
        combine: raw.combine,
        count: raw.count,
        contiguous: raw.contiguous,
    })
}

fn convert_serial(raw: RawSerialConfig) -> Result<TriggerSerialSpec, TriggerConfigParseError> {
    let data_channel = match raw.data_channel {
        RawChannelRef::Index(index) => index,
        RawChannelRef::Token(token) => parse_channel_token(&token)?,
    };
    if raw.data_bits == 0 || raw.data_bits > TRIGGER_MAX_SERIAL_DATA_BITS {
        return Err(TriggerConfigParseError::InvalidField {
            mode: "serial",
            detail: format!(
                "data_bits must be 1..={TRIGGER_MAX_SERIAL_DATA_BITS}, got {}",
                raw.data_bits
            ),
        });
    }
    Ok(TriggerSerialSpec {
        start: convert_conditions(raw.start)?,
        stop: convert_conditions(raw.stop)?,
        clock: convert_conditions(raw.clock)?,
        data_channel,
        data_bits: raw.data_bits,
        data_value: normalize_serial_data_value(raw.data_value, raw.data_bits)?,
    })
}

fn normalize_serial_data_value(
    raw: RawSerialValue,
    data_bits: u8,
) -> Result<String, TriggerConfigParseError> {
    let width = usize::from(data_bits);
    let invalid = |value: String, detail: String| TriggerConfigParseError::InvalidSerialDataValue {
        value,
        detail,
    };
    let integer_bits = |value: u64, original: String| {
        if width < 64 && value >> width != 0 {
            return Err(invalid(
                original,
                format!("value does not fit in {data_bits} data bits"),
            ));
        }
        Ok((0..width)
            .rev()
            .map(|bit| if value >> bit & 1 == 1 { '1' } else { '0' })
            .collect())
    };

    match raw {
        RawSerialValue::Integer(value) => integer_bits(value, value.to_string()),
        RawSerialValue::Text(text) => {
            let compact: String = text
                .chars()
                .filter(|character| *character != '_' && *character != ' ')
                .collect();
            if let Some(hex) = compact
                .strip_prefix("0x")
                .or_else(|| compact.strip_prefix("0X"))
            {
                let value = u64::from_str_radix(hex, 16).map_err(|_| {
                    invalid(text.clone(), "hex digits expected after 0x".to_string())
                })?;
                return integer_bits(value, text);
            }
            let bits = compact
                .strip_prefix("0b")
                .or_else(|| compact.strip_prefix("0B"))
                .unwrap_or(&compact)
                .to_ascii_uppercase();
            if bits.len() != width {
                return Err(invalid(
                    text,
                    format!("bit pattern must have exactly {data_bits} characters"),
                ));
            }
            if !bits.chars().all(|bit| matches!(bit, '0' | '1' | 'X')) {
                return Err(invalid(
                    text,
                    "bit pattern may only contain 0, 1, and X".to_string(),
                ));
            }
            Ok(bits)
        }
    }
}

/// Validates a trigger request against the selected device options and builds
/// the native program.
pub fn validate_trigger_request(
    request: &TriggerRequest,
    context: &TriggerValidationContext,
) -> Result<ValidatedTrigger, TriggerValidationError> {
    let position_percent = validate_position(request.position_percent, context)?;

    if context.stream_mode && !matches!(request.spec, TriggerSpec::Simple { .. }) {
        return Err(TriggerValidationError::ModeUnsupportedInStream {
            mode: request.spec.mode_name(),
        });
    }

    match &request.spec {
        TriggerSpec::Simple { channels } if channels.is_empty() => {
            return Err(TriggerValidationError::EmptySimpleTrigger);
        }
        TriggerSpec::Stage { stages } => validate_stages(stages)?,
        TriggerSpec::Serial(serial) => validate_serial(serial)?,
        TriggerSpec::Simple { .. } => {}
    }

    for channel in request.spec.referenced_channels() {
        if channel >= context.channel_limit {
            return Err(TriggerValidationError::ChannelOutsideChannelMode {
                channel,
                channel_mode_id: context.channel_mode_id.clone(),
                channel_limit: context.channel_limit,
            });
        }
        if !context.enabled_channels.contains(&channel) {
            return Err(TriggerValidationError::ChannelNotEnabled { channel });
        }
    }

    Ok(ValidatedTrigger {
        program: build_trigger_program(&request.spec, position_percent),
        spec: request.spec.clone(),
        position_percent,
    })
}

fn validate_position(
    position_percent: Option<u8>,
    context: &TriggerValidationContext,
) -> Result<u8, TriggerValidationError> {
    if context.stream_mode {
        return match position_percent {
            None => Ok(TRIGGER_STREAM_POSITION_PERCENT),
            Some(position) if position == TRIGGER_STREAM_POSITION_PERCENT => Ok(position),
            Some(position) => Err(TriggerValidationError::StreamPositionFixed {
                position_percent: position,
                fixed: TRIGGER_STREAM_POSITION_PERCENT,
            }),
        };
    }

    let position = position_percent.unwrap_or(TRIGGER_POSITION_DEFAULT_PERCENT);
    if !(TRIGGER_POSITION_MIN_PERCENT..=TRIGGER_POSITION_MAX_PERCENT).contains(&position) {
        return Err(TriggerValidationError::PositionOutOfRange {
            position_percent: position,
            min: TRIGGER_POSITION_MIN_PERCENT,
            max: TRIGGER_POSITION_MAX_PERCENT,
        });
    }
    Ok(position)
}

fn validate_stages(stages: &[TriggerStageSpec]) -> Result<(), TriggerValidationError> {
    if stages.is_empty() || stages.len() > TRIGGER_MAX_STAGES {
        return Err(TriggerValidationError::StageCountOutOfRange {
            count: stages.len(),
            max: TRIGGER_MAX_STAGES,
        });
    }

    for (index, stage) in stages.iter().enumerate() {
        let invalid = |detail: &str| TriggerValidationError::InvalidStage {
            stage: index,
            detail: detail.to_string(),
        };
        if stage.pattern_a.is_empty() && stage.pattern_b.is_empty() {
            return Err(invalid(
                "pattern_a or pattern_b needs at least one channel condition",
            ));
        }
        // An empty pattern matches every sample and an inverted empty pattern
        // matches none, so these combinations always or never fire.
        if stage.combine == TriggerCombine::Or
            && (stage.pattern_a.is_empty() || stage.pattern_b.is_empty())
        {
            return Err(invalid(
                "`or` with an empty pattern matches every sample; give both patterns conditions or use `and`",
            ));
        }
        if (stage.pattern_a.invert && stage.pattern_a.is_empty())
            || (stage.pattern_b.invert && stage.pattern_b.is_empty())
        {
            return Err(invalid(
                "an inverted pattern needs at least one channel condition",
            ));
        }
        if stage.count == 0 || stage.count > TRIGGER_MAX_STAGE_COUNT {
            return Err(invalid(&format!(
                "count must be 1..={TRIGGER_MAX_STAGE_COUNT}, got {}",
                stage.count
            )));
        }
    }
    Ok(())
}

fn validate_serial(serial: &TriggerSerialSpec) -> Result<(), TriggerValidationError> {
    // On DSLogic Plus hardware a serial trigger whose start, stop, or clock
    // flag has no edge condition never fires: an empty flag or a level-only
    // flag keeps re-arming the serial engine.
    for (flag, conditions) in [
        ("start", &serial.start),
        ("stop", &serial.stop),
        ("clock", &serial.clock),
    ] {
        if !conditions.values().any(|condition| condition.is_edge()) {
            return Err(TriggerValidationError::InvalidSerial {
                detail: format!(
                    "{flag} needs at least one edge condition (rising, falling, or edge); without one the hardware never fires"
                ),
            });
        }
    }
    if serial.data_bits == 0 || serial.data_bits > TRIGGER_MAX_SERIAL_DATA_BITS {
        return Err(TriggerValidationError::InvalidSerial {
            detail: format!(
                "data_bits must be 1..={TRIGGER_MAX_SERIAL_DATA_BITS}, got {}",
                serial.data_bits
            ),
        });
    }
    if serial.data_value.len() != usize::from(serial.data_bits) {
        return Err(TriggerValidationError::InvalidSerial {
            detail: "data_value length must match data_bits".to_string(),
        });
    }
    Ok(())
}

fn native_pattern(conditions: &TriggerChannelConditions) -> [u8; TRIGGER_PROBE_CAPACITY] {
    let mut pattern = [DONT_CARE; TRIGGER_PROBE_CAPACITY];
    for (&channel, condition) in conditions {
        if let Some(slot) = pattern.get_mut(usize::from(channel)) {
            *slot = condition.native();
        }
    }
    pattern
}

/// Builds the native program DSView's trigger dock would write for `spec`.
pub fn build_trigger_program(spec: &TriggerSpec, position_percent: u8) -> TriggerProgram {
    let position_percent = u16::from(position_percent);
    match spec {
        TriggerSpec::Simple { channels } => TriggerProgram {
            mode: TriggerProgramMode::Simple,
            position_percent,
            stage_selector: 0,
            simple: native_pattern(channels),
            stages: Vec::new(),
        },
        TriggerSpec::Stage { stages } => TriggerProgram {
            mode: TriggerProgramMode::Stage,
            position_percent,
            stage_selector: stages.len().saturating_sub(1) as u16,
            simple: [DONT_CARE; TRIGGER_PROBE_CAPACITY],
            stages: stages.iter().map(stage_program).collect(),
        },
        TriggerSpec::Serial(serial) => TriggerProgram {
            mode: TriggerProgramMode::Serial,
            position_percent,
            // DSView keeps its stage selector at the default single stage here.
            stage_selector: 0,
            simple: [DONT_CARE; TRIGGER_PROBE_CAPACITY],
            stages: serial_stage_programs(serial),
        },
    }
}

fn stage_program(stage: &TriggerStageSpec) -> TriggerStageProgram {
    let and = u8::from(stage.combine == TriggerCombine::And);
    TriggerStageProgram {
        value0: native_pattern(&stage.pattern_a.channels),
        value1: native_pattern(&stage.pattern_b.channels),
        logic: (u8::from(stage.contiguous) << 1) | and,
        inv0: stage.pattern_a.invert,
        inv1: stage.pattern_b.invert,
        count0: stage.count,
        count1: 0,
    }
}

fn serial_stage_programs(serial: &TriggerSerialSpec) -> Vec<TriggerStageProgram> {
    let mut stages = vec![TriggerStageProgram::dont_care(); 4];

    // Stage 0: frame start (value0) and stop (value1) flags.
    stages[0].value0 = native_pattern(&serial.start);
    stages[0].value1 = native_pattern(&serial.stop);

    // Stage 1: clock flag, sampled once per bit.
    stages[1].value0 = native_pattern(&serial.clock);
    stages[1].count0 = 1;

    // Stage 2: the data channel, marked with `0`.
    let mut data_channel = [DONT_CARE; TRIGGER_PROBE_CAPACITY];
    if let Some(slot) = data_channel.get_mut(usize::from(serial.data_channel)) {
        *slot = b'0';
    }
    stages[2].value0 = data_channel;

    // Stage 3: the data word, bit k in slot k, and the bit count minus one.
    let mut data_value = [DONT_CARE; TRIGGER_PROBE_CAPACITY];
    for (bit, symbol) in serial.data_value.bytes().rev().enumerate() {
        if let Some(slot) = data_value.get_mut(bit) {
            *slot = symbol;
        }
    }
    stages[3].value0 = data_value;
    stages[3].count0 = u32::from(serial.data_bits.saturating_sub(1));

    stages
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context(stream_mode: bool, channel_limit: u16, enabled: &[u16]) -> TriggerValidationContext {
        TriggerValidationContext {
            operation_mode_id: "operation-mode:0".to_string(),
            stream_mode,
            channel_mode_id: "channel-mode:20".to_string(),
            channel_limit,
            enabled_channels: enabled.iter().copied().collect(),
        }
    }

    fn pattern(conditions: &[(usize, u8)]) -> [u8; TRIGGER_PROBE_CAPACITY] {
        let mut values = [DONT_CARE; TRIGGER_PROBE_CAPACITY];
        for &(probe, condition) in conditions {
            values[probe] = condition;
        }
        values
    }

    #[test]
    fn condition_tokens_accept_names_and_dsview_symbols() {
        for (token, expected) in [
            ("low", TriggerCondition::Low),
            ("HIGH", TriggerCondition::High),
            ("r", TriggerCondition::Rising),
            ("F", TriggerCondition::Falling),
            ("edge", TriggerCondition::Edge),
            ("c", TriggerCondition::Edge),
            ("0", TriggerCondition::Low),
            ("1", TriggerCondition::High),
        ] {
            assert_eq!(TriggerCondition::parse(token), Some(expected), "{token}");
        }
        assert_eq!(TriggerCondition::parse("x"), None);
        assert_eq!(TriggerCondition::parse("up"), None);
    }

    #[test]
    fn simple_tokens_parse_channel_prefixes_and_reject_duplicates() {
        let spec = parse_simple_trigger_tokens(&["0=rising", "D3=high", "d15:F"]).unwrap();
        assert_eq!(
            spec,
            TriggerSpec::Simple {
                channels: BTreeMap::from([
                    (0, TriggerCondition::Rising),
                    (3, TriggerCondition::High),
                    (15, TriggerCondition::Falling),
                ]),
            }
        );

        assert_eq!(
            parse_simple_trigger_tokens(&["0=rising", "D0=low"]),
            Err(TriggerConfigParseError::DuplicateChannel { channel: 0 })
        );
        assert!(matches!(
            parse_simple_trigger_tokens(&["rising"]),
            Err(TriggerConfigParseError::InvalidConditionToken { .. })
        ));
        assert!(matches!(
            parse_simple_trigger_tokens(&["x0=rising"]),
            Err(TriggerConfigParseError::InvalidChannel { .. })
        ));
        assert!(matches!(
            parse_simple_trigger_tokens(&["0=up"]),
            Err(TriggerConfigParseError::UnknownCondition { .. })
        ));
    }

    #[test]
    fn simple_program_places_conditions_by_channel() {
        let spec = parse_simple_trigger_tokens(&["0=rising", "3=high", "7=edge"]).unwrap();
        let validated = validate_trigger_request(
            &TriggerRequest {
                spec,
                position_percent: Some(25),
            },
            &context(false, 16, &[0, 1, 2, 3, 7]),
        )
        .unwrap();

        assert_eq!(validated.position_percent, 25);
        assert_eq!(validated.program.mode, TriggerProgramMode::Simple);
        assert_eq!(validated.program.position_percent, 25);
        assert_eq!(
            validated.program.simple,
            pattern(&[(0, b'R'), (3, b'1'), (7, b'C')])
        );
        assert!(validated.program.stages.is_empty());
    }

    #[test]
    fn stage_config_parses_defaults_and_builds_dsview_layout() {
        let spec = parse_trigger_config(
            r#"{
                "version": 1,
                "mode": "stage",
                "stages": [
                    {"pattern_a": {"channels": {"0": "rising", "1": "low"}}},
                    {
                        "pattern_a": {"channels": {"D2": "high"}, "invert": true},
                        "pattern_b": {"channels": {"3": "falling"}},
                        "combine": "or",
                        "count": 4,
                        "contiguous": true
                    }
                ]
            }"#,
        )
        .unwrap();
        let validated = validate_trigger_request(
            &TriggerRequest {
                spec,
                position_percent: None,
            },
            &context(false, 16, &[0, 1, 2, 3]),
        )
        .unwrap();

        let program = &validated.program;
        assert_eq!(validated.position_percent, TRIGGER_POSITION_DEFAULT_PERCENT);
        assert_eq!(program.mode, TriggerProgramMode::Stage);
        assert_eq!(program.stage_selector, 1);
        assert_eq!(program.stages.len(), 2);
        assert_eq!(program.stages[0].value0, pattern(&[(0, b'R'), (1, b'0')]));
        assert_eq!(program.stages[0].value1, pattern(&[]));
        assert_eq!(program.stages[0].logic, 0b01);
        assert_eq!(program.stages[0].count0, 1);
        assert_eq!(program.stages[1].value0, pattern(&[(2, b'1')]));
        assert_eq!(program.stages[1].value1, pattern(&[(3, b'F')]));
        assert_eq!(program.stages[1].logic, 0b10);
        assert!(program.stages[1].inv0);
        assert!(!program.stages[1].inv1);
        assert_eq!(program.stages[1].count0, 4);
    }

    #[test]
    fn serial_config_builds_the_four_dsview_stages() {
        let spec = parse_trigger_config(
            r#"{
                "mode": "serial",
                "start": {"1": "falling", "3": "high"},
                "stop": {"1": "rising", "3": "high"},
                "clock": {"0": "rising"},
                "data_channel": "D2",
                "data_bits": 8,
                "data_value": "0xA5"
            }"#,
        )
        .unwrap();
        let TriggerSpec::Serial(serial) = &spec else {
            panic!("serial spec expected");
        };
        assert_eq!(serial.data_value, "10100101");

        let validated = validate_trigger_request(
            &TriggerRequest {
                spec,
                position_percent: Some(10),
            },
            &context(false, 16, &[0, 1, 2, 3]),
        )
        .unwrap();
        let program = &validated.program;
        assert_eq!(program.mode, TriggerProgramMode::Serial);
        assert_eq!(program.stage_selector, 0);
        assert_eq!(program.stages.len(), 4);
        assert_eq!(program.stages[0].value0, pattern(&[(1, b'F'), (3, b'1')]));
        assert_eq!(program.stages[0].value1, pattern(&[(1, b'R'), (3, b'1')]));
        assert_eq!(program.stages[1].value0, pattern(&[(0, b'R')]));
        assert_eq!(program.stages[1].count0, 1);
        assert_eq!(program.stages[2].value0, pattern(&[(2, b'0')]));
        assert_eq!(
            program.stages[3].value0,
            pattern(&[
                (0, b'1'),
                (1, b'0'),
                (2, b'1'),
                (3, b'0'),
                (4, b'0'),
                (5, b'1'),
                (6, b'0'),
                (7, b'1'),
            ])
        );
        assert_eq!(program.stages[3].count0, 7);
        for stage in &program.stages {
            assert_eq!(stage.logic, 1);
            assert!(!stage.inv0 && !stage.inv1);
        }
    }

    #[test]
    fn serial_data_value_accepts_bits_integers_and_wildcards() {
        let parse = |value: &str, bits: u8| {
            parse_trigger_config(&format!(
                r#"{{"mode":"serial","start":{{"2":"rising"}},"stop":{{"2":"falling"}},"clock":{{"0":"rising"}},"data_channel":1,"data_bits":{bits},"data_value":{value}}}"#
            ))
        };
        let value_of = |spec: TriggerSpec| match spec {
            TriggerSpec::Serial(serial) => serial.data_value,
            _ => unreachable!(),
        };

        assert_eq!(value_of(parse(r#""01x1""#, 4).unwrap()), "01X1");
        assert_eq!(value_of(parse(r#""0b1010_0101""#, 8).unwrap()), "10100101");
        assert_eq!(value_of(parse("5", 4).unwrap()), "0101");
        assert!(matches!(
            parse(r#""0x1FF""#, 8),
            Err(TriggerConfigParseError::InvalidSerialDataValue { .. })
        ));
        assert!(matches!(
            parse(r#""0101""#, 8),
            Err(TriggerConfigParseError::InvalidSerialDataValue { .. })
        ));
        assert!(matches!(
            parse(r#""01A1""#, 4),
            Err(TriggerConfigParseError::InvalidSerialDataValue { .. })
        ));
        assert!(matches!(
            parse(r#""0""#, 17),
            Err(TriggerConfigParseError::InvalidField { .. })
        ));
    }

    #[test]
    fn config_rejects_unknown_fields_versions_and_modes() {
        assert!(matches!(
            parse_trigger_config(r#"{"mode":"simple","channels":{"0":"rising"},"extra":1}"#),
            Err(TriggerConfigParseError::InvalidField { mode: "simple", .. })
        ));
        assert_eq!(
            parse_trigger_config(r#"{"version":2,"mode":"simple","channels":{}}"#),
            Err(TriggerConfigParseError::UnsupportedVersion {
                version: 2,
                expected: 1
            })
        );
        assert_eq!(
            parse_trigger_config(r#"{"mode":"window"}"#),
            Err(TriggerConfigParseError::InvalidMode)
        );
        assert_eq!(
            parse_trigger_config("[]"),
            Err(TriggerConfigParseError::NotAnObject)
        );
        assert!(matches!(
            parse_trigger_config(r#"{"mode":"simple","channels":{"0":"up"}}"#),
            Err(TriggerConfigParseError::InvalidField { .. })
        ));
    }

    #[test]
    fn validation_enforces_channels_position_and_stream_limits() {
        let simple = |tokens: &[&str], position_percent| TriggerRequest {
            spec: parse_simple_trigger_tokens(tokens).unwrap(),
            position_percent,
        };

        assert_eq!(
            validate_trigger_request(&simple(&["2=high"], None), &context(false, 16, &[0, 1])),
            Err(TriggerValidationError::ChannelNotEnabled { channel: 2 })
        );
        assert!(matches!(
            validate_trigger_request(&simple(&["5=high"], None), &context(false, 4, &[0, 5])),
            Err(TriggerValidationError::ChannelOutsideChannelMode {
                channel: 5,
                channel_limit: 4,
                ..
            })
        ));
        assert_eq!(
            validate_trigger_request(&simple(&["0=high"], Some(91)), &context(false, 16, &[0])),
            Err(TriggerValidationError::PositionOutOfRange {
                position_percent: 91,
                min: 1,
                max: 90
            })
        );
        assert_eq!(
            validate_trigger_request(&simple(&["0=high"], Some(0)), &context(false, 16, &[0]))
                .unwrap_err(),
            TriggerValidationError::PositionOutOfRange {
                position_percent: 0,
                min: 1,
                max: 90
            }
        );
        assert_eq!(
            validate_trigger_request(&simple(&["0=high"], Some(10)), &context(true, 16, &[0])),
            Err(TriggerValidationError::StreamPositionFixed {
                position_percent: 10,
                fixed: 1
            })
        );
        assert_eq!(
            validate_trigger_request(&simple(&["0=high"], None), &context(true, 16, &[0]))
                .unwrap()
                .position_percent,
            1
        );
        assert_eq!(
            validate_trigger_request(&simple(&[], None), &context(false, 16, &[0])),
            Err(TriggerValidationError::EmptySimpleTrigger)
        );

        let stage = TriggerRequest {
            spec: parse_trigger_config(
                r#"{"mode":"stage","stages":[{"pattern_a":{"channels":{"0":"rising"}}}]}"#,
            )
            .unwrap(),
            position_percent: None,
        };
        assert_eq!(
            validate_trigger_request(&stage, &context(true, 16, &[0])),
            Err(TriggerValidationError::ModeUnsupportedInStream { mode: "stage" })
        );
    }

    #[test]
    fn stage_validation_rejects_patterns_that_always_or_never_fire() {
        let validate = |json: &str| {
            validate_trigger_request(
                &TriggerRequest {
                    spec: parse_trigger_config(json).unwrap(),
                    position_percent: None,
                },
                &context(false, 16, &[0, 1]),
            )
        };

        assert!(matches!(
            validate(r#"{"mode":"stage","stages":[]}"#),
            Err(TriggerValidationError::StageCountOutOfRange { count: 0, .. })
        ));
        let seventeen = vec![r#"{"pattern_a":{"channels":{"0":"high"}}}"#; 17].join(",");
        assert!(matches!(
            validate(&format!(r#"{{"mode":"stage","stages":[{seventeen}]}}"#)),
            Err(TriggerValidationError::StageCountOutOfRange { count: 17, .. })
        ));
        assert!(matches!(
            validate(r#"{"mode":"stage","stages":[{"pattern_a":{}}]}"#),
            Err(TriggerValidationError::InvalidStage { stage: 0, .. })
        ));
        assert!(matches!(
            validate(
                r#"{"mode":"stage","stages":[{"pattern_a":{"channels":{"0":"high"}},"combine":"or"}]}"#
            ),
            Err(TriggerValidationError::InvalidStage { stage: 0, .. })
        ));
        assert!(matches!(
            validate(
                r#"{"mode":"stage","stages":[{"pattern_a":{"channels":{"0":"high"}},"pattern_b":{"invert":true}}]}"#
            ),
            Err(TriggerValidationError::InvalidStage { stage: 0, .. })
        ));
        assert!(matches!(
            validate(
                r#"{"mode":"stage","stages":[{"pattern_a":{"channels":{"0":"high"}}},{"pattern_a":{"channels":{"1":"low"}},"count":0}]}"#
            ),
            Err(TriggerValidationError::InvalidStage { stage: 1, .. })
        ));
    }

    #[test]
    fn serial_validation_requires_edge_start_stop_and_clock_flags() {
        let validate = |start: &str, stop: &str, clock: &str| {
            validate_trigger_request(
                &TriggerRequest {
                    spec: parse_trigger_config(&format!(
                        r#"{{"mode":"serial","start":{start},"stop":{stop},"clock":{clock},"data_channel":1,"data_bits":4,"data_value":"0101"}}"#
                    ))
                    .unwrap(),
                    position_percent: None,
                },
                &context(false, 16, &[0, 1, 2]),
            )
        };

        assert!(
            validate(
                r#"{"2":"rising"}"#,
                r#"{"2":"falling"}"#,
                r#"{"0":"rising"}"#
            )
            .is_ok()
        );
        for (start, stop, clock, flag) in [
            ("{}", r#"{"2":"falling"}"#, r#"{"0":"rising"}"#, "start"),
            (r#"{"2":"rising"}"#, "{}", r#"{"0":"rising"}"#, "stop"),
            (
                r#"{"2":"high"}"#,
                r#"{"2":"low"}"#,
                r#"{"0":"rising"}"#,
                "start",
            ),
            (
                r#"{"2":"rising"}"#,
                r#"{"2":"falling"}"#,
                r#"{"0":"high"}"#,
                "clock",
            ),
        ] {
            match validate(start, stop, clock) {
                Err(TriggerValidationError::InvalidSerial { detail }) => {
                    assert!(detail.starts_with(flag), "{detail}");
                }
                other => panic!("expected {flag} rejection, got {other:?}"),
            }
        }
    }

    #[test]
    fn report_exposes_sample_index_only_when_fired() {
        let validated = validate_trigger_request(
            &TriggerRequest {
                spec: parse_simple_trigger_tokens(&["0=rising"]).unwrap(),
                position_percent: Some(50),
            },
            &context(false, 16, &[0]),
        )
        .unwrap();
        let outcome = |raw_status| CaptureTriggerOutcome {
            requested: validated.clone(),
            observation: TriggerObservation {
                saw_trigger_packet: true,
                packet_status: Some(dsview_sys::AcquisitionPacketStatus::Ok),
                position: Some(dsview_sys::TriggerPosition {
                    raw_status,
                    real_pos: 4096,
                    remain_count: 0,
                }),
            },
        };

        let fired = TriggerReport::from_outcome(&outcome(1));
        assert!(fired.observed.fired);
        assert_eq!(fired.observed.sample_index, Some(4096));
        let json = serde_json::to_value(&fired).unwrap();
        assert_eq!(json["requested"]["mode"], "simple");
        assert_eq!(json["requested"]["channels"]["0"], "rising");
        assert_eq!(json["requested"]["position_percent"], 50);
        assert_eq!(json["observed"]["sample_index"], 4096);

        let missed = TriggerReport::from_outcome(&outcome(0));
        assert!(!missed.observed.fired);
        assert_eq!(missed.observed.sample_index, None);
        assert_eq!(missed.observed.raw_status, Some(0));
    }
}
