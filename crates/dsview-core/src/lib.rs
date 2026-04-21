use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fmt;
use std::fs;
use std::num::NonZeroU64;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

mod capture_config;
mod device_option_validation;
mod device_options;

pub use capture_config::{
    CaptureCapabilities, CaptureConfigError, CaptureConfigRequest, ChannelModeCapability,
    ValidatedCaptureConfig,
};
pub use device_option_validation::{
    ChannelModeValidationCapabilities, DeviceOptionValidationCapabilities,
    DeviceOptionValidationError, DeviceOptionValidationRequest,
    OperationModeValidationCapabilities, ValidatedDeviceOptionRequest,
};
pub use device_options::{
    normalize_device_options_snapshot, ChannelModeGroupSnapshot, ChannelModeOptionSnapshot,
    CurrentDeviceOptionValues, DeviceIdentitySnapshot, DeviceOptionsSnapshot, EnumOptionSnapshot,
    LegacyThresholdMetadataSnapshot, RawOptionMetadataSnapshot, ThresholdCapabilitySnapshot,
};
pub use dsview_sys::{
    decode_runtime_library_name, runtime_library_name, source_decode_runtime_library_path,
    source_runtime_library_path, AcquisitionSummary, AcquisitionTerminalEvent, DeviceHandle,
    DeviceSummary, DecodeExecutionLogicFormat, DecodeOptionValueKind, DecodeRuntimeError,
    DecodeRuntimeErrorCode, DecodeSessionChannelBinding, DecodeSessionInstance,
    DecodeSessionOption, DecodeSessionOptionValue, ExportErrorCode, NativeErrorCode,
    RuntimeError, VcdExportFacts, VcdExportRequest,
};
pub use dsview_sys::{
    DecodeRuntimeError as DecoderRuntimeError,
    DecodeRuntimeErrorCode as DecoderRuntimeErrorCode,
};
use dsview_sys::{
    AcquisitionPacketStatus, DecodeRuntimeBridge, RuntimeBridge,
};
use dsview_sys::{
    DecodeAnnotation as SysDecodeAnnotation, DecodeAnnotationRow as SysDecodeAnnotationRow,
    DecodeChannel as SysDecodeChannel, DecodeDecoder as SysDecodeDecoder,
    DecodeInput as SysDecodeInput, DecodeOption as SysDecodeOption,
    DecodeOutput as SysDecodeOutput,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

const DSLOGIC_PLUS_MODELS: &[&str] = &["DSLogic PLus"];
const DSLOGIC_PLUS_PRIMARY_FIRMWARES: &[&str] = &["DSLogicPlus.fw"];
const DSLOGIC_PLUS_FIRMWARE_FALLBACKS: &[&str] = &["DSLogic.fw"];
const DSLOGIC_PLUS_BITSTREAMS: &[&str] = &["DSLogicPlus.bin", "DSLogicPlus-pgl12.bin"];
const BUNDLED_RUNTIME_DIR: &str = "runtime";
const BUNDLED_RESOURCE_DIR: &str = "resources";
const BUNDLED_DECODE_RUNTIME_DIR: &str = "decode-runtime";
const BUNDLED_DECODER_DIR: &str = "decoders";

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DecoderInputDescriptor {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DecoderOutputDescriptor {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DecoderChannelDescriptor {
    pub id: String,
    pub name: String,
    pub description: String,
    pub order: i32,
    pub channel_type: i32,
    pub idn: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DecoderOptionDescriptor {
    pub id: String,
    pub idn: Option<String>,
    pub description: Option<String>,
    #[serde(skip_serializing)]
    pub value_kind: DecodeOptionValueKind,
    pub default_value: Option<String>,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DecoderAnnotationDescriptor {
    pub id: String,
    pub label: Option<String>,
    pub description: Option<String>,
    pub annotation_type: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DecoderAnnotationRowDescriptor {
    pub id: String,
    pub description: Option<String>,
    pub annotation_classes: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DecoderDescriptor {
    pub id: String,
    pub name: String,
    pub longname: String,
    pub description: String,
    pub license: String,
    pub inputs: Vec<DecoderInputDescriptor>,
    pub outputs: Vec<DecoderOutputDescriptor>,
    pub tags: Vec<String>,
    pub required_channels: Vec<DecoderChannelDescriptor>,
    pub optional_channels: Vec<DecoderChannelDescriptor>,
    pub options: Vec<DecoderOptionDescriptor>,
    pub annotations: Vec<DecoderAnnotationDescriptor>,
    pub annotation_rows: Vec<DecoderAnnotationRowDescriptor>,
}

pub fn normalize_decoder_registry(decoders: Vec<SysDecodeDecoder>) -> Vec<DecoderDescriptor> {
    decoders
        .into_iter()
        .map(normalize_decoder_descriptor)
        .collect()
}

pub fn normalize_decoder_descriptor(decoder: SysDecodeDecoder) -> DecoderDescriptor {
    DecoderDescriptor {
        id: decoder.id,
        name: decoder.name,
        longname: decoder.longname,
        description: decoder.description,
        license: decoder.license,
        inputs: decoder.inputs.into_iter().map(normalize_decoder_input).collect(),
        outputs: decoder
            .outputs
            .into_iter()
            .map(normalize_decoder_output)
            .collect(),
        tags: decoder.tags,
        required_channels: decoder
            .required_channels
            .into_iter()
            .map(normalize_decoder_channel)
            .collect(),
        optional_channels: decoder
            .optional_channels
            .into_iter()
            .map(normalize_decoder_channel)
            .collect(),
        options: decoder
            .options
            .into_iter()
            .map(normalize_decoder_option)
            .collect(),
        annotations: decoder
            .annotations
            .into_iter()
            .map(normalize_decoder_annotation)
            .collect(),
        annotation_rows: decoder
            .annotation_rows
            .into_iter()
            .map(normalize_decoder_annotation_row)
            .collect(),
    }
}

fn normalize_decoder_input(input: SysDecodeInput) -> DecoderInputDescriptor {
    DecoderInputDescriptor { id: input.id }
}

fn normalize_decoder_output(output: SysDecodeOutput) -> DecoderOutputDescriptor {
    DecoderOutputDescriptor { id: output.id }
}

fn normalize_decoder_channel(channel: SysDecodeChannel) -> DecoderChannelDescriptor {
    DecoderChannelDescriptor {
        id: channel.id,
        name: channel.name,
        description: channel.description,
        order: channel.order,
        channel_type: channel.channel_type,
        idn: channel.idn,
    }
}

fn normalize_decoder_option(option: SysDecodeOption) -> DecoderOptionDescriptor {
    DecoderOptionDescriptor {
        id: option.id,
        idn: option.idn,
        description: option.description,
        value_kind: option.value_kind,
        default_value: option.default_value,
        values: option.values,
    }
}

fn normalize_decoder_annotation(annotation: SysDecodeAnnotation) -> DecoderAnnotationDescriptor {
    DecoderAnnotationDescriptor {
        id: annotation.id,
        label: annotation.label,
        description: annotation.description,
        annotation_type: annotation.annotation_type,
    }
}

fn normalize_decoder_annotation_row(
    row: SysDecodeAnnotationRow,
) -> DecoderAnnotationRowDescriptor {
    DecoderAnnotationRowDescriptor {
        id: row.id,
        description: row.description,
        annotation_classes: row.annotation_classes,
    }
}

pub const DECODE_CONFIG_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecodeConfig {
    #[serde(default = "default_decode_config_version")]
    pub version: u32,
    pub decoder: DecodeDecoderConfig,
    #[serde(default)]
    pub stack: Vec<DecodeStackEntryConfig>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecodeDecoderConfig {
    pub id: String,
    #[serde(default)]
    pub channels: BTreeMap<String, u32>,
    #[serde(default)]
    pub options: BTreeMap<String, DecodeOptionValue>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecodeStackEntryConfig {
    pub id: String,
    #[serde(default)]
    pub options: BTreeMap<String, DecodeOptionValue>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DecodeOptionValue {
    String(String),
    Integer(i64),
    Float(f64),
}

#[derive(Debug, Error)]
pub enum DecodeConfigParseError {
    #[error("decode config JSON could not be parsed: {detail}")]
    InvalidJson {
        detail: String,
        #[source]
        source: serde_json::Error,
    },
    #[error("decode config JSON did not match the expected schema: {detail}")]
    Schema {
        detail: String,
        #[source]
        source: serde_json::Error,
    },
}

pub fn parse_decode_config(json: &str) -> Result<DecodeConfig, DecodeConfigParseError> {
    parse_decode_config_slice(json.as_bytes())
}

pub fn parse_decode_config_slice(json: &[u8]) -> Result<DecodeConfig, DecodeConfigParseError> {
    serde_json::from_slice::<DecodeConfig>(json).map_err(|source| match source.classify() {
        serde_json::error::Category::Data => DecodeConfigParseError::Schema {
            detail: source.to_string(),
            source,
        },
        serde_json::error::Category::Io
        | serde_json::error::Category::Syntax
        | serde_json::error::Category::Eof => DecodeConfigParseError::InvalidJson {
            detail: source.to_string(),
            source,
        },
    })
}

fn default_decode_config_version() -> u32 {
    DECODE_CONFIG_VERSION
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ValidatedDecodeConfig {
    pub version: u32,
    pub decoder: ValidatedDecodeDecoderConfig,
    pub stack: Vec<ValidatedDecodeStackEntryConfig>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ValidatedDecodeDecoderConfig {
    pub descriptor: DecoderDescriptor,
    pub channels: BTreeMap<String, u32>,
    pub options: BTreeMap<String, DecodeOptionValue>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ValidatedDecodeStackEntryConfig {
    pub descriptor: DecoderDescriptor,
    pub options: BTreeMap<String, DecodeOptionValue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OfflineDecodeDataFormat {
    SplitLogic,
    CrossLogic,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OfflineDecodeInput {
    pub samplerate_hz: u64,
    pub format: OfflineDecodeDataFormat,
    pub sample_bytes: Vec<u8>,
    pub unitsize: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel_count: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logic_packet_lengths: Option<Vec<usize>>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OfflineDecodeExecutionRequest {
    pub config: ValidatedDecodeConfig,
    pub input: OfflineDecodeInput,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum OfflineDecodeInputError {
    #[error("offline decode input samplerate_hz must be greater than zero")]
    MissingSamplerate,
    #[error("offline decode input sample_bytes must not be empty")]
    MissingSampleBytes,
    #[error("offline decode split-logic unitsize must be greater than zero")]
    MissingUnitsize,
    #[error("offline decode cross-logic inputs require channel_count")]
    MissingChannelCount,
    #[error("offline decode cross-logic channel_count must be greater than zero")]
    InvalidChannelCount,
    #[error("offline decode input sample_bytes do not align to the declared format")]
    MisalignedSampleBytes,
    #[error("offline decode input logic_packet_lengths are invalid for the declared format")]
    InvalidPacketLengths,
}

impl OfflineDecodeInput {
    pub fn sample_count(&self) -> Result<u64, OfflineDecodeInputError> {
        self.validate_basic_shape()?;
        Ok(match self.format {
            OfflineDecodeDataFormat::SplitLogic => {
                (self.sample_bytes.len() / self.unitsize as usize) as u64
            }
            OfflineDecodeDataFormat::CrossLogic => {
                let channel_count = self.channel_count.expect("validated above") as usize;
                ((self.sample_bytes.len() / (channel_count * std::mem::size_of::<u64>())) * 64)
                    as u64
            }
        })
    }

    pub fn validate_basic_shape(&self) -> Result<(), OfflineDecodeInputError> {
        if self.samplerate_hz == 0 {
            return Err(OfflineDecodeInputError::MissingSamplerate);
        }
        if self.sample_bytes.is_empty() {
            return Err(OfflineDecodeInputError::MissingSampleBytes);
        }

        let alignment = match self.format {
            OfflineDecodeDataFormat::SplitLogic => {
                if self.unitsize == 0 {
                    return Err(OfflineDecodeInputError::MissingUnitsize);
                }
                self.unitsize as usize
            }
            OfflineDecodeDataFormat::CrossLogic => {
                let channel_count =
                    self.channel_count.ok_or(OfflineDecodeInputError::MissingChannelCount)?;
                if channel_count == 0 {
                    return Err(OfflineDecodeInputError::InvalidChannelCount);
                }
                channel_count as usize * std::mem::size_of::<u64>()
            }
        };

        if self.sample_bytes.len() % alignment != 0 {
            return Err(OfflineDecodeInputError::MisalignedSampleBytes);
        }

        if let Some(packet_lengths) = &self.logic_packet_lengths {
            if packet_lengths.is_empty()
                || packet_lengths.iter().sum::<usize>() != self.sample_bytes.len()
                || packet_lengths
                    .iter()
                    .any(|length| *length == 0 || (*length % alignment) != 0)
            {
                return Err(OfflineDecodeInputError::InvalidPacketLengths);
            }
        }

        Ok(())
    }

    fn byte_alignment(&self) -> Result<usize, OfflineDecodeInputError> {
        self.validate_basic_shape()?;
        Ok(match self.format {
            OfflineDecodeDataFormat::SplitLogic => self.unitsize as usize,
            OfflineDecodeDataFormat::CrossLogic => {
                self.channel_count.expect("validated above") as usize * std::mem::size_of::<u64>()
            }
        })
    }

    fn sample_count_for_len(&self, sample_byte_len: usize) -> Result<u64, OfflineDecodeInputError> {
        let alignment = self.byte_alignment()?;
        if sample_byte_len == 0 || sample_byte_len % alignment != 0 {
            return Err(OfflineDecodeInputError::MisalignedSampleBytes);
        }

        Ok(match self.format {
            OfflineDecodeDataFormat::SplitLogic => {
                (sample_byte_len / self.unitsize as usize) as u64
            }
            OfflineDecodeDataFormat::CrossLogic => {
                let channel_count = self.channel_count.expect("validated above") as usize;
                ((sample_byte_len / (channel_count * std::mem::size_of::<u64>())) * 64) as u64
            }
        })
    }
}

pub const OFFLINE_DECODE_FIXED_CHUNK_BYTES: usize = 4096;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DecodeCapturedAnnotation {
    pub decoder_id: String,
    pub start_sample: u64,
    pub end_sample: u64,
    pub annotation_class: i32,
    pub annotation_type: i32,
    pub texts: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OfflineDecodeResult {
    annotations: Vec<DecodeCapturedAnnotation>,
    #[serde(skip_serializing)]
    diagnostics: OfflineDecodeDiagnostics,
}

impl OfflineDecodeResult {
    pub fn annotations(&self) -> &[DecodeCapturedAnnotation] {
        &self.annotations
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct OfflineDecodeDiagnostics {
    completed_chunks: usize,
    consumed_samples: u64,
    partial_annotations: Vec<DecodeCapturedAnnotation>,
}

#[derive(Debug, Error)]
pub enum OfflineDecodeRunError {
    #[error(transparent)]
    InvalidInput(#[from] OfflineDecodeInputError),
    #[error("offline decode runtime failed during {operation}: {source}")]
    Runtime {
        operation: &'static str,
        #[source]
        source: DecodeRuntimeError,
        diagnostics: OfflineDecodeDiagnostics,
    },
}

impl OfflineDecodeRunError {
    pub fn operation(&self) -> &'static str {
        match self {
            Self::InvalidInput(_) => "validate input",
            Self::Runtime { operation, .. } => operation,
        }
    }

    pub fn retained_annotations(&self) -> &[DecodeCapturedAnnotation] {
        match self {
            Self::InvalidInput(_) => &[],
            Self::Runtime { diagnostics, .. } => &diagnostics.partial_annotations,
        }
    }

    pub fn completed_chunks(&self) -> usize {
        match self {
            Self::InvalidInput(_) => 0,
            Self::Runtime { diagnostics, .. } => diagnostics.completed_chunks,
        }
    }
}

pub trait OfflineDecodeRuntimeSession {
    fn set_samplerate_hz(&mut self, samplerate_hz: u64) -> Result<(), DecodeRuntimeError>;
    fn build_linear_stack(
        &mut self,
        root: &DecodeSessionInstance,
        stack: &[DecodeSessionInstance],
    ) -> Result<(), DecodeRuntimeError>;
    fn start(&mut self) -> Result<(), DecodeRuntimeError>;
    fn send_logic_chunk(
        &mut self,
        abs_start_sample: u64,
        sample_bytes: &[u8],
        format: DecodeExecutionLogicFormat,
    ) -> Result<Vec<DecodeCapturedAnnotation>, DecodeRuntimeError>;
    fn end(&mut self) -> Result<Vec<DecodeCapturedAnnotation>, DecodeRuntimeError>;
}

pub trait OfflineDecodeRuntime {
    type Session: OfflineDecodeRuntimeSession;

    fn create_session(&self) -> Result<Self::Session, DecodeRuntimeError>;
}

impl OfflineDecodeRuntime for DecodeRuntimeBridge {
    type Session = dsview_sys::DecodeExecutionSession;

    fn create_session(&self) -> Result<Self::Session, DecodeRuntimeError> {
        dsview_sys::DecodeExecutionSession::new()
    }
}

impl OfflineDecodeRuntimeSession for dsview_sys::DecodeExecutionSession {
    fn set_samplerate_hz(&mut self, samplerate_hz: u64) -> Result<(), DecodeRuntimeError> {
        dsview_sys::DecodeExecutionSession::set_samplerate_hz(self, samplerate_hz)
    }

    fn build_linear_stack(
        &mut self,
        root: &DecodeSessionInstance,
        stack: &[DecodeSessionInstance],
    ) -> Result<(), DecodeRuntimeError> {
        dsview_sys::DecodeExecutionSession::build_linear_stack(self, root, stack)
    }

    fn start(&mut self) -> Result<(), DecodeRuntimeError> {
        dsview_sys::DecodeExecutionSession::start(self)
    }

    fn send_logic_chunk(
        &mut self,
        abs_start_sample: u64,
        sample_bytes: &[u8],
        format: DecodeExecutionLogicFormat,
    ) -> Result<Vec<DecodeCapturedAnnotation>, DecodeRuntimeError> {
        dsview_sys::session_send_logic_chunk(self, abs_start_sample, sample_bytes, format, None)?;
        Ok(self
            .take_captured_annotations()?
            .into_iter()
            .filter(|annotation| annotation.annotation_class >= 0)
            .map(|annotation| DecodeCapturedAnnotation {
                decoder_id: annotation.decoder_id,
                start_sample: annotation.start_sample,
                end_sample: annotation.end_sample,
                annotation_class: annotation.annotation_class,
                annotation_type: annotation.annotation_type,
                texts: annotation.texts,
            })
            .collect())
    }

    fn end(&mut self) -> Result<Vec<DecodeCapturedAnnotation>, DecodeRuntimeError> {
        dsview_sys::DecodeExecutionSession::end(self)?;
        Ok(self
            .take_captured_annotations()?
            .into_iter()
            .filter(|annotation| annotation.annotation_class >= 0)
            .map(|annotation| DecodeCapturedAnnotation {
                decoder_id: annotation.decoder_id,
                start_sample: annotation.start_sample,
                end_sample: annotation.end_sample,
                annotation_class: annotation.annotation_class,
                annotation_type: annotation.annotation_type,
                texts: annotation.texts,
            })
            .collect())
    }
}

pub fn run_offline_decode<R: OfflineDecodeRuntime>(
    config: &ValidatedDecodeConfig,
    input: &OfflineDecodeInput,
    runtime: &R,
) -> Result<OfflineDecodeResult, OfflineDecodeRunError> {
    input.validate_basic_shape()?;

    let mut session = runtime
        .create_session()
        .map_err(|source| OfflineDecodeRunError::Runtime {
            operation: "create session",
            source,
            diagnostics: OfflineDecodeDiagnostics::default(),
        })?;
    let mut diagnostics = OfflineDecodeDiagnostics::default();

    session
        .set_samplerate_hz(input.samplerate_hz)
        .map_err(|source| OfflineDecodeRunError::Runtime {
            operation: "set samplerate",
            source,
            diagnostics: diagnostics.clone(),
        })?;

    let root = build_root_decode_session(config);
    let stack = build_stacked_decode_sessions(config);
    session
        .build_linear_stack(&root, &stack)
        .map_err(|source| OfflineDecodeRunError::Runtime {
            operation: "build linear stack",
            source,
            diagnostics: diagnostics.clone(),
        })?;
    session
        .start()
        .map_err(|source| OfflineDecodeRunError::Runtime {
            operation: "start session",
            source,
            diagnostics: diagnostics.clone(),
        })?;

    let mut annotations = Vec::new();
    let format = offline_decode_logic_format(input);
    let mut abs_start_sample = 0_u64;

    for chunk in offline_decode_chunk_ranges(input)? {
        let chunk_annotations = session
            .send_logic_chunk(abs_start_sample, &input.sample_bytes[chunk.clone()], format)
            .map_err(|source| OfflineDecodeRunError::Runtime {
                operation: "send logic chunk",
                source,
                diagnostics: diagnostics.clone(),
            })?;
        diagnostics.completed_chunks += 1;
        diagnostics.consumed_samples += input.sample_count_for_len(chunk.len())?;
        diagnostics.partial_annotations.extend(chunk_annotations.clone());
        annotations.extend(chunk_annotations);
        abs_start_sample = diagnostics.consumed_samples;
    }

    let tail_annotations = session
        .end()
        .map_err(|source| OfflineDecodeRunError::Runtime {
            operation: "end session",
            source,
            diagnostics: diagnostics.clone(),
        })?;
    diagnostics.partial_annotations.extend(tail_annotations.clone());
    annotations.extend(tail_annotations);

    Ok(OfflineDecodeResult {
        annotations,
        diagnostics,
    })
}

fn build_root_decode_session(config: &ValidatedDecodeConfig) -> DecodeSessionInstance {
    DecodeSessionInstance {
        decoder_id: config.decoder.descriptor.id.clone(),
        channel_bindings: config
            .decoder
            .channels
            .iter()
            .map(|(channel_id, channel_index)| DecodeSessionChannelBinding {
                channel_id: channel_id.clone(),
                channel_index: *channel_index,
            })
            .collect(),
        options: decode_session_options(&config.decoder.options),
    }
}

fn build_stacked_decode_sessions(config: &ValidatedDecodeConfig) -> Vec<DecodeSessionInstance> {
    config
        .stack
        .iter()
        .map(|entry| DecodeSessionInstance {
            decoder_id: entry.descriptor.id.clone(),
            channel_bindings: Vec::new(),
            options: decode_session_options(&entry.options),
        })
        .collect()
}

fn decode_session_options(
    options: &BTreeMap<String, DecodeOptionValue>,
) -> Vec<DecodeSessionOption> {
    options
        .iter()
        .map(|(option_id, value)| DecodeSessionOption {
            option_id: option_id.clone(),
            value: match value {
                DecodeOptionValue::String(value) => DecodeSessionOptionValue::String(value.clone()),
                DecodeOptionValue::Integer(value) => DecodeSessionOptionValue::Integer(*value),
                DecodeOptionValue::Float(value) => DecodeSessionOptionValue::Float(*value),
            },
        })
        .collect()
}

fn offline_decode_logic_format(input: &OfflineDecodeInput) -> DecodeExecutionLogicFormat {
    match input.format {
        OfflineDecodeDataFormat::SplitLogic => {
            DecodeExecutionLogicFormat::SplitLogic {
                unitsize: input.unitsize,
            }
        }
        OfflineDecodeDataFormat::CrossLogic => DecodeExecutionLogicFormat::CrossLogic {
            channel_count: input.channel_count.expect("validated before execution"),
        },
    }
}

fn offline_decode_chunk_ranges(
    input: &OfflineDecodeInput,
) -> Result<Vec<std::ops::Range<usize>>, OfflineDecodeInputError> {
    let alignment = input.byte_alignment()?;
    if let Some(packet_lengths) = &input.logic_packet_lengths {
        let mut offset = 0_usize;
        return Ok(packet_lengths
            .iter()
            .map(|packet_len| {
                let range = offset..offset + packet_len;
                offset += packet_len;
                range
            })
            .collect());
    }

    let fixed_chunk_len = OFFLINE_DECODE_FIXED_CHUNK_BYTES
        .max(alignment)
        .checked_sub(OFFLINE_DECODE_FIXED_CHUNK_BYTES.max(alignment) % alignment)
        .filter(|chunk_len| *chunk_len > 0)
        .unwrap_or(alignment);
    let mut ranges = Vec::new();
    let mut offset = 0_usize;

    while offset < input.sample_bytes.len() {
        let next = (offset + fixed_chunk_len).min(input.sample_bytes.len());
        ranges.push(offset..next);
        offset = next;
    }

    Ok(ranges)
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DecodeConfigValidationError {
    #[error("decode config version `{version}` is not supported")]
    UnsupportedVersion { version: u32 },
    #[error("decoder `{decoder_id}` was not found in the loaded registry")]
    UnknownDecoder { decoder_id: String },
    #[error("decoder `{decoder_id}` is missing required channel `{channel_id}`")]
    MissingRequiredChannel {
        decoder_id: String,
        channel_id: String,
    },
    #[error("decoder `{decoder_id}` received unknown channel binding `{channel_id}`")]
    UnknownChannelBinding {
        decoder_id: String,
        channel_id: String,
    },
    #[error("decoder `{decoder_id}` received unknown option `{option_id}`")]
    UnknownOption {
        decoder_id: String,
        option_id: String,
    },
    #[error(
        "decoder `{decoder_id}` option `{option_id}` expected {expected:?} but received {actual:?}"
    )]
    InvalidOptionValueType {
        decoder_id: String,
        option_id: String,
        expected: DecodeOptionValueKind,
        actual: DecodeOptionValueKind,
    },
    #[error(
        "decoder `{decoder_id}` option `{option_id}` value `{value}` is not allowed; expected one of {allowed_values:?}"
    )]
    InvalidOptionValue {
        decoder_id: String,
        option_id: String,
        value: String,
        allowed_values: Vec<String>,
    },
    #[error(
        "decoder `{downstream_decoder_id}` cannot stack after `{upstream_decoder_id}` because inputs {downstream_inputs:?} do not match outputs {upstream_outputs:?}"
    )]
    IncompatibleStackLink {
        upstream_decoder_id: String,
        downstream_decoder_id: String,
        upstream_outputs: Vec<String>,
        downstream_inputs: Vec<String>,
    },
}

#[derive(Debug, Error)]
pub enum DecodeConfigLoadError {
    #[error("decode config file `{path}` is missing")]
    MissingFile { path: PathBuf },
    #[error("decode config file `{path}` is not readable: {detail}")]
    UnreadableFile { path: PathBuf, detail: String },
    #[error(transparent)]
    Discovery(#[from] DecodeBringUpError),
    #[error(transparent)]
    Parse(#[from] DecodeConfigParseError),
    #[error(transparent)]
    Validation(#[from] DecodeConfigValidationError),
}

pub fn validate_decode_config(
    config: &DecodeConfig,
    registry: &[DecoderDescriptor],
) -> Result<ValidatedDecodeConfig, DecodeConfigValidationError> {
    if config.version != DECODE_CONFIG_VERSION {
        return Err(DecodeConfigValidationError::UnsupportedVersion {
            version: config.version,
        });
    }

    let validated_decoder = validate_root_decode_decoder(&config.decoder, registry)?;
    let mut validated_stack = Vec::with_capacity(config.stack.len());
    let mut previous_descriptor = validated_decoder.descriptor.clone();

    for entry in &config.stack {
        let validated_entry = validate_stack_decode_decoder(entry, registry)?;
        ensure_stack_link(&previous_descriptor, &validated_entry.descriptor)?;
        previous_descriptor = validated_entry.descriptor.clone();
        validated_stack.push(validated_entry);
    }

    Ok(ValidatedDecodeConfig {
        version: config.version,
        decoder: validated_decoder,
        stack: validated_stack,
    })
}

fn validate_root_decode_decoder(
    config: &DecodeDecoderConfig,
    registry: &[DecoderDescriptor],
) -> Result<ValidatedDecodeDecoderConfig, DecodeConfigValidationError> {
    let descriptor = lookup_decoder_descriptor(&config.id, registry)?;
    validate_channel_bindings(&descriptor, &config.channels)?;
    validate_option_values(&descriptor, &config.options)?;
    Ok(ValidatedDecodeDecoderConfig {
        descriptor,
        channels: config.channels.clone(),
        options: config.options.clone(),
    })
}

fn validate_stack_decode_decoder(
    config: &DecodeStackEntryConfig,
    registry: &[DecoderDescriptor],
) -> Result<ValidatedDecodeStackEntryConfig, DecodeConfigValidationError> {
    let descriptor = lookup_decoder_descriptor(&config.id, registry)?;
    validate_option_values(&descriptor, &config.options)?;
    Ok(ValidatedDecodeStackEntryConfig {
        descriptor,
        options: config.options.clone(),
    })
}

fn lookup_decoder_descriptor(
    decoder_id: &str,
    registry: &[DecoderDescriptor],
) -> Result<DecoderDescriptor, DecodeConfigValidationError> {
    registry
        .iter()
        .find(|decoder| decoder.id == decoder_id)
        .cloned()
        .ok_or_else(|| DecodeConfigValidationError::UnknownDecoder {
            decoder_id: decoder_id.to_string(),
        })
}

fn validate_channel_bindings(
    descriptor: &DecoderDescriptor,
    channels: &BTreeMap<String, u32>,
) -> Result<(), DecodeConfigValidationError> {
    let valid_channel_ids = descriptor
        .required_channels
        .iter()
        .chain(descriptor.optional_channels.iter())
        .map(|channel| channel.id.as_str())
        .collect::<BTreeSet<_>>();

    for channel_id in channels.keys() {
        if !valid_channel_ids.contains(channel_id.as_str()) {
            return Err(DecodeConfigValidationError::UnknownChannelBinding {
                decoder_id: descriptor.id.clone(),
                channel_id: channel_id.clone(),
            });
        }
    }

    for channel in &descriptor.required_channels {
        if !channels.contains_key(&channel.id) {
            return Err(DecodeConfigValidationError::MissingRequiredChannel {
                decoder_id: descriptor.id.clone(),
                channel_id: channel.id.clone(),
            });
        }
    }

    Ok(())
}

fn validate_option_values(
    descriptor: &DecoderDescriptor,
    options: &BTreeMap<String, DecodeOptionValue>,
) -> Result<(), DecodeConfigValidationError> {
    for (option_id, value) in options {
        let metadata =
            descriptor
                .options
                .iter()
                .find(|option| option.id == *option_id)
                .ok_or_else(|| DecodeConfigValidationError::UnknownOption {
                    decoder_id: descriptor.id.clone(),
                    option_id: option_id.clone(),
                })?;
        let actual_kind = decode_option_value_kind(value);

        if metadata.value_kind != DecodeOptionValueKind::Unknown && metadata.value_kind != actual_kind
        {
            return Err(DecodeConfigValidationError::InvalidOptionValueType {
                decoder_id: descriptor.id.clone(),
                option_id: option_id.clone(),
                expected: metadata.value_kind,
                actual: actual_kind,
            });
        }

        if !metadata.values.is_empty() {
            let rendered_value = render_decode_option_value(value);
            if !metadata.values.iter().any(|candidate| candidate == &rendered_value) {
                return Err(DecodeConfigValidationError::InvalidOptionValue {
                    decoder_id: descriptor.id.clone(),
                    option_id: option_id.clone(),
                    value: rendered_value,
                    allowed_values: metadata.values.clone(),
                });
            }
        }
    }

    Ok(())
}

fn ensure_stack_link(
    upstream: &DecoderDescriptor,
    downstream: &DecoderDescriptor,
) -> Result<(), DecodeConfigValidationError> {
    let upstream_outputs = upstream
        .outputs
        .iter()
        .map(|output| output.id.clone())
        .collect::<Vec<_>>();
    let downstream_inputs = downstream
        .inputs
        .iter()
        .map(|input| input.id.clone())
        .collect::<Vec<_>>();

    if upstream_outputs
        .iter()
        .any(|output_id| downstream_inputs.iter().any(|input_id| input_id == output_id))
    {
        return Ok(());
    }

    Err(DecodeConfigValidationError::IncompatibleStackLink {
        upstream_decoder_id: upstream.id.clone(),
        downstream_decoder_id: downstream.id.clone(),
        upstream_outputs,
        downstream_inputs,
    })
}

fn decode_option_value_kind(value: &DecodeOptionValue) -> DecodeOptionValueKind {
    match value {
        DecodeOptionValue::String(_) => DecodeOptionValueKind::String,
        DecodeOptionValue::Integer(_) => DecodeOptionValueKind::Integer,
        DecodeOptionValue::Float(_) => DecodeOptionValueKind::Float,
    }
}

fn render_decode_option_value(value: &DecodeOptionValue) -> String {
    match value {
        DecodeOptionValue::String(value) => value.clone(),
        DecodeOptionValue::Integer(value) => value.to_string(),
        DecodeOptionValue::Float(value) => value.to_string(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceDirectory {
    path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDiscoveryPaths {
    pub runtime_library: PathBuf,
    pub resource_dir: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodeDiscoveryPaths {
    pub runtime_library: PathBuf,
    pub decoder_dir: PathBuf,
}

impl RuntimeDiscoveryPaths {
    pub fn discover(resource_override: Option<impl AsRef<Path>>) -> Result<Self, BringUpError> {
        let executable =
            env::current_exe().map_err(|error| BringUpError::CurrentExecutableUnavailable {
                detail: error.to_string(),
            })?;
        let executable_dir =
            executable
                .parent()
                .ok_or_else(|| BringUpError::CurrentExecutableUnavailable {
                    detail: format!("path `{}` has no parent directory", executable.display()),
                })?;
        Self::from_executable_dir(executable_dir, resource_override)
    }

    pub fn from_executable_dir(
        executable_dir: impl AsRef<Path>,
        resource_override: Option<impl AsRef<Path>>,
    ) -> Result<Self, BringUpError> {
        let executable_dir = executable_dir.as_ref();
        let resource_override = resource_override.map(|path| path.as_ref().to_path_buf());
        let bundled_runtime = executable_dir
            .join(BUNDLED_RUNTIME_DIR)
            .join(runtime_library_name());
        let bundled_resources = executable_dir.join(BUNDLED_RESOURCE_DIR);
        let resource_dir = resource_override
            .clone()
            .unwrap_or_else(|| bundled_resources.clone());

        if bundled_runtime.is_file() {
            return Ok(Self {
                runtime_library: bundled_runtime,
                resource_dir,
            });
        }

        if let Some(source_runtime) = source_runtime_library_path().filter(|path| path.is_file()) {
            let resource_dir = resource_override.unwrap_or_else(developer_resource_dir);
            return Ok(Self {
                runtime_library: source_runtime.to_path_buf(),
                resource_dir,
            });
        }

        Err(BringUpError::BundledRuntimeMissing {
            path: bundled_runtime,
            executable_dir: executable_dir.to_path_buf(),
        })
    }
}

impl DecodeDiscoveryPaths {
    pub fn discover(
        runtime_override: Option<impl AsRef<Path>>,
        decoder_dir_override: Option<impl AsRef<Path>>,
    ) -> Result<Self, DecodeBringUpError> {
        let executable =
            env::current_exe().map_err(|error| DecodeBringUpError::CurrentExecutableUnavailable {
                detail: error.to_string(),
            })?;
        let executable_dir =
            executable
                .parent()
                .ok_or_else(|| DecodeBringUpError::CurrentExecutableUnavailable {
                    detail: format!("path `{}` has no parent directory", executable.display()),
                })?;
        Self::from_executable_dir(executable_dir, runtime_override, decoder_dir_override)
    }

    pub fn from_executable_dir(
        executable_dir: impl AsRef<Path>,
        runtime_override: Option<impl AsRef<Path>>,
        decoder_dir_override: Option<impl AsRef<Path>>,
    ) -> Result<Self, DecodeBringUpError> {
        let executable_dir = executable_dir.as_ref();
        let runtime_override = runtime_override.map(|path| path.as_ref().to_path_buf());
        let decoder_dir_override = decoder_dir_override.map(|path| path.as_ref().to_path_buf());

        let bundled_runtime = executable_dir
            .join(BUNDLED_DECODE_RUNTIME_DIR)
            .join(decode_runtime_library_name());
        let bundled_decoder_dir = executable_dir.join(BUNDLED_DECODER_DIR);

        let runtime_library = if let Some(path) = runtime_override {
            path
        } else if bundled_runtime.is_file() {
            bundled_runtime.clone()
        } else if let Some(source_runtime) =
            source_decode_runtime_library_path().filter(|path| path.is_file())
        {
            source_runtime.to_path_buf()
        } else {
            return Err(DecodeBringUpError::BundledRuntimeMissing {
                path: bundled_runtime,
                executable_dir: executable_dir.to_path_buf(),
            });
        };

        let decoder_dir = if let Some(path) = decoder_dir_override {
            path
        } else if runtime_library == bundled_runtime {
            bundled_decoder_dir
        } else {
            developer_decoder_dir()
        };

        ensure_decoder_script_dir(&decoder_dir)?;

        Ok(Self {
            runtime_library,
            decoder_dir,
        })
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum DeviceOptionApplyStep {
    OperationMode,
    StopOption,
    ChannelMode,
    ThresholdVolts,
    Filter,
    EnabledChannels,
    SampleLimit,
    SampleRate,
}

impl DeviceOptionApplyStep {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OperationMode => "operation_mode",
            Self::StopOption => "stop_option",
            Self::ChannelMode => "channel_mode",
            Self::ThresholdVolts => "threshold_volts",
            Self::Filter => "filter",
            Self::EnabledChannels => "enabled_channels",
            Self::SampleLimit => "sample_limit",
            Self::SampleRate => "sample_rate",
        }
    }
}

impl fmt::Display for DeviceOptionApplyStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EffectiveDeviceOptionState {
    pub operation_mode_code: Option<i16>,
    pub stop_option_code: Option<i16>,
    pub channel_mode_code: Option<i16>,
    pub threshold_volts: Option<f64>,
    pub filter_code: Option<i16>,
    pub enabled_channels: Vec<u16>,
    pub sample_limit: Option<u64>,
    pub sample_rate_hz: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CaptureDeviceOptionSnapshot {
    pub operation_mode_id: String,
    pub stop_option_id: Option<String>,
    pub channel_mode_id: String,
    pub enabled_channels: Vec<u16>,
    pub threshold_volts: Option<f64>,
    pub filter_id: Option<String>,
    pub sample_rate_hz: u64,
    pub sample_limit: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CaptureDeviceOptionFacts {
    pub requested: CaptureDeviceOptionSnapshot,
    pub effective: CaptureDeviceOptionSnapshot,
}

#[derive(Debug, Error)]
#[error("device option apply failed at {failed_step}: {runtime_error}")]
pub struct DeviceOptionApplyFailure {
    pub applied_steps: Vec<DeviceOptionApplyStep>,
    pub failed_step: DeviceOptionApplyStep,
    pub runtime_error: RuntimeError,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CaptureRunSummary {
    pub completion: CaptureCompletion,
    pub summary: AcquisitionSummary,
    pub cleanup: CaptureCleanup,
    pub effective_device_options: Option<EffectiveDeviceOptionState>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureArtifactPaths {
    pub vcd_path: PathBuf,
    pub metadata_path: PathBuf,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CaptureArtifactPathError {
    #[error("VCD output path `{path}` must use the .vcd extension")]
    InvalidVcdExtension { path: PathBuf },
    #[error("metadata output path `{path}` must use the .json extension")]
    InvalidMetadataExtension { path: PathBuf },
    #[error(
        "VCD output path `{vcd_path}` and metadata output path `{metadata_path}` must be different"
    )]
    ConflictingArtifactPaths {
        vcd_path: PathBuf,
        metadata_path: PathBuf,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct CaptureExportRequest {
    pub capture: CaptureRunSummary,
    pub validated_config: ValidatedCaptureConfig,
    pub vcd_path: PathBuf,
    pub metadata_path: Option<PathBuf>,
    pub tool_name: String,
    pub tool_version: String,
    pub capture_started_at: SystemTime,
    pub device_model: String,
    pub device_stable_id: String,
    pub selected_handle: SelectionHandle,
    pub validated_device_options: Option<ValidatedDeviceOptionRequest>,
    pub device_options_snapshot: DeviceOptionsSnapshot,
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

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CaptureMetadata {
    pub schema_version: u32,
    pub tool: MetadataToolInfo,
    pub capture: MetadataCaptureInfo,
    pub acquisition: MetadataAcquisitionInfo,
    pub artifacts: MetadataArtifactInfo,
    pub device_options: CaptureDeviceOptionFacts,
}

#[derive(Debug, Clone, PartialEq)]
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
    #[error(transparent)]
    InvalidArtifactPaths(#[from] CaptureArtifactPathError),
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

#[derive(Debug, Clone, PartialEq)]
pub struct CaptureRunRequest {
    pub selection_handle: SelectionHandle,
    pub config: CaptureConfigRequest,
    pub validated_device_options: Option<ValidatedDeviceOptionRequest>,
    pub wait_timeout: Duration,
    pub poll_interval: Duration,
}

#[derive(Debug, Error)]
pub enum CaptureRunError {
    #[error(transparent)]
    BringUp(#[from] BringUpError),
    #[error(transparent)]
    DeviceOptionApply(#[from] DeviceOptionApplyFailure),
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

#[derive(Debug)]
pub struct DecodeDiscovery {
    runtime: DecodeRuntimeBridge,
    paths: DecodeDiscoveryPaths,
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

    pub fn connect_auto(resource_override: Option<impl AsRef<Path>>) -> Result<Self, BringUpError> {
        let paths = RuntimeDiscoveryPaths::discover(resource_override)?;
        Self::connect(&paths.runtime_library, &paths.resource_dir)
    }

    pub fn discovery_paths(
        resource_override: Option<impl AsRef<Path>>,
    ) -> Result<RuntimeDiscoveryPaths, BringUpError> {
        RuntimeDiscoveryPaths::discover(resource_override)
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

impl DecodeDiscovery {
    pub fn connect(
        library_path: impl AsRef<Path>,
        decoder_dir: impl AsRef<Path>,
    ) -> Result<Self, DecodeBringUpError> {
        let library_path = library_path.as_ref().to_path_buf();
        let decoder_dir = decoder_dir.as_ref().to_path_buf();
        ensure_decoder_script_dir(&decoder_dir)?;
        let runtime =
            DecodeRuntimeBridge::load(&library_path).map_err(DecodeBringUpError::Runtime)?;
        runtime.init(&decoder_dir).map_err(DecodeBringUpError::Runtime)?;
        Ok(Self {
            runtime,
            paths: DecodeDiscoveryPaths {
                runtime_library: library_path,
                decoder_dir,
            },
        })
    }

    pub fn connect_auto(
        runtime_override: Option<impl AsRef<Path>>,
        decoder_dir_override: Option<impl AsRef<Path>>,
    ) -> Result<Self, DecodeBringUpError> {
        let paths = DecodeDiscoveryPaths::discover(runtime_override, decoder_dir_override)?;
        Self::connect(&paths.runtime_library, &paths.decoder_dir)
    }

    pub fn discovery_paths(
        runtime_override: Option<impl AsRef<Path>>,
        decoder_dir_override: Option<impl AsRef<Path>>,
    ) -> Result<DecodeDiscoveryPaths, DecodeBringUpError> {
        DecodeDiscoveryPaths::discover(runtime_override, decoder_dir_override)
    }

    pub fn paths(&self) -> &DecodeDiscoveryPaths {
        &self.paths
    }

    pub fn decode_list(&self) -> Result<Vec<DecoderDescriptor>, DecodeBringUpError> {
        let decoders = self
            .runtime
            .decode_list()
            .map_err(|error| map_decode_runtime_error(error, None))?;
        if decoders.is_empty() {
            return Err(DecodeBringUpError::DecoderScriptsMissing {
                path: self.paths.decoder_dir.clone(),
            });
        }
        Ok(normalize_decoder_registry(decoders))
    }

    pub fn decode_inspect(&self, decoder_id: &str) -> Result<DecoderDescriptor, DecodeBringUpError> {
        let decoder = self
            .runtime
            .decode_inspect(decoder_id)
            .map_err(|error| map_decode_runtime_error(error, Some(decoder_id)))?;
        Ok(normalize_decoder_descriptor(decoder))
    }
}

impl Drop for DecodeDiscovery {
    fn drop(&mut self) {
        let _ = self.runtime.exit();
    }
}

pub fn decode_list(
    runtime_override: Option<impl AsRef<Path>>,
    decoder_dir_override: Option<impl AsRef<Path>>,
) -> Result<Vec<DecoderDescriptor>, DecodeBringUpError> {
    let discovery = DecodeDiscovery::connect_auto(runtime_override, decoder_dir_override)?;
    discovery.decode_list()
}

pub fn decode_inspect(
    runtime_override: Option<impl AsRef<Path>>,
    decoder_dir_override: Option<impl AsRef<Path>>,
    decoder_id: &str,
) -> Result<DecoderDescriptor, DecodeBringUpError> {
    let discovery = DecodeDiscovery::connect_auto(runtime_override, decoder_dir_override)?;
    discovery.decode_inspect(decoder_id)
}

pub fn validate_decode_config_file(
    runtime_override: Option<impl AsRef<Path>>,
    decoder_dir_override: Option<impl AsRef<Path>>,
    config_path: impl AsRef<Path>,
) -> Result<ValidatedDecodeConfig, DecodeConfigLoadError> {
    let config_path = config_path.as_ref();
    let config_bytes = fs::read(config_path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            DecodeConfigLoadError::MissingFile {
                path: config_path.to_path_buf(),
            }
        } else {
            DecodeConfigLoadError::UnreadableFile {
                path: config_path.to_path_buf(),
                detail: error.to_string(),
            }
        }
    })?;
    let config = parse_decode_config_slice(&config_bytes)?;
    let registry = decode_list(runtime_override, decoder_dir_override)?;
    validate_decode_config(&config, &registry).map_err(DecodeConfigLoadError::from)
}

pub fn validated_capture_config_from_device_options(
    request: &ValidatedDeviceOptionRequest,
) -> ValidatedCaptureConfig {
    ValidatedCaptureConfig {
        sample_rate_hz: request.sample_rate_hz,
        requested_sample_limit: request.requested_sample_limit,
        effective_sample_limit: request.effective_sample_limit,
        enabled_channels: request.enabled_channels.iter().copied().collect(),
        channel_mode_id: request.channel_mode_code,
    }
}

pub fn apply_capture_request_device_options(
    runtime: &RuntimeBridge,
    request: &CaptureRunRequest,
    total_channel_count: u16,
) -> Result<Option<EffectiveDeviceOptionState>, DeviceOptionApplyFailure> {
    request
        .validated_device_options
        .as_ref()
        .map(|validated_device_options| {
            apply_validated_device_options(runtime, validated_device_options, total_channel_count)
        })
        .transpose()
}

fn effective_device_option_state(
    runtime: &RuntimeBridge,
    enabled_channels: &[u16],
) -> Result<EffectiveDeviceOptionState, RuntimeError> {
    Ok(EffectiveDeviceOptionState {
        operation_mode_code: runtime.current_operation_mode_code()?,
        stop_option_code: runtime.current_stop_option_code()?,
        channel_mode_code: runtime.current_channel_mode_code()?,
        threshold_volts: runtime.current_threshold_volts()?,
        filter_code: runtime.current_filter_code()?,
        enabled_channels: enabled_channels.to_vec(),
        sample_limit: runtime.current_sample_limit()?,
        sample_rate_hz: runtime.current_samplerate()?,
    })
}

fn apply_device_option_step<F>(
    applied_steps: &mut Vec<DeviceOptionApplyStep>,
    step: DeviceOptionApplyStep,
    action: F,
) -> Result<(), DeviceOptionApplyFailure>
where
    F: FnOnce() -> Result<(), RuntimeError>,
{
    action().map_err(|runtime_error| DeviceOptionApplyFailure {
        applied_steps: applied_steps.clone(),
        failed_step: step,
        runtime_error,
    })?;
    applied_steps.push(step);
    Ok(())
}

pub fn apply_validated_device_options(
    runtime: &RuntimeBridge,
    request: &ValidatedDeviceOptionRequest,
    total_channel_count: u16,
) -> Result<EffectiveDeviceOptionState, DeviceOptionApplyFailure> {
    let mut applied_steps = Vec::new();

    apply_device_option_step(
        &mut applied_steps,
        DeviceOptionApplyStep::OperationMode,
        || runtime.set_operation_mode(request.operation_mode_code),
    )?;

    if let Some(stop_option_code) = request.stop_option_code {
        apply_device_option_step(&mut applied_steps, DeviceOptionApplyStep::StopOption, || {
            runtime.set_stop_option(stop_option_code)
        })?;
    }

    apply_device_option_step(
        &mut applied_steps,
        DeviceOptionApplyStep::ChannelMode,
        || runtime.set_channel_mode(request.channel_mode_code),
    )?;

    if let Some(threshold_volts) = request.threshold_volts {
        apply_device_option_step(
            &mut applied_steps,
            DeviceOptionApplyStep::ThresholdVolts,
            || runtime.set_threshold_volts(threshold_volts),
        )?;
    }

    if let Some(filter_code) = request.filter_code {
        apply_device_option_step(&mut applied_steps, DeviceOptionApplyStep::Filter, || {
            runtime.set_filter(filter_code)
        })?;
    }

    apply_device_option_step(
        &mut applied_steps,
        DeviceOptionApplyStep::EnabledChannels,
        || runtime.set_enabled_channels(&request.enabled_channels, total_channel_count),
    )?;
    apply_device_option_step(&mut applied_steps, DeviceOptionApplyStep::SampleLimit, || {
        runtime.set_sample_limit(request.effective_sample_limit)
    })?;
    apply_device_option_step(&mut applied_steps, DeviceOptionApplyStep::SampleRate, || {
        runtime.set_samplerate(request.sample_rate_hz)
    })?;

    effective_device_option_state(runtime, &request.enabled_channels).map_err(|runtime_error| {
        DeviceOptionApplyFailure {
            applied_steps,
            failed_step: DeviceOptionApplyStep::SampleRate,
            runtime_error,
        }
    })
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

    pub fn load_device_option_validation_capabilities(
        &self,
        selection_handle: SelectionHandle,
    ) -> Result<DeviceOptionValidationCapabilities, BringUpError> {
        let opened = self.open_device(selection_handle)?;
        let native = self
            .runtime
            .device_option_validation_capabilities()
            .map_err(BringUpError::Runtime)?;
        let snapshot = device_option_validation::normalize_device_option_validation_capabilities(
            opened.device(),
            native,
        );
        opened.release()?;
        Ok(snapshot)
    }

    pub fn validate_device_option_request(
        &self,
        selection_handle: SelectionHandle,
        request: &DeviceOptionValidationRequest,
    ) -> Result<ValidatedDeviceOptionRequest, DeviceOptionValidationError> {
        let capabilities = self
            .load_device_option_validation_capabilities(selection_handle)
            .map_err(|error| DeviceOptionValidationError::Runtime(error.to_string()))?;
        capabilities.validate_request(request)
    }

    pub fn validate_capture_config(
        &self,
        selection_handle: SelectionHandle,
        request: &CaptureConfigRequest,
    ) -> Result<ValidatedCaptureConfig, CaptureConfigError> {
        let opened = self
            .open_device(selection_handle)
            .map_err(|error| match error {
                BringUpError::Runtime(runtime) => CaptureConfigError::from_runtime_error(runtime),
                other => CaptureConfigError::Runtime(other.to_string()),
            })?;
        let capabilities = self.dslogic_plus_capabilities_for_opened(&opened)?;
        let validated = capabilities.validate_request(request);
        opened
            .release()
            .map_err(|error| CaptureConfigError::Runtime(error.to_string()))?;
        validated
    }

    pub fn inspect_device_options(
        &self,
        selection_handle: SelectionHandle,
    ) -> Result<DeviceOptionsSnapshot, BringUpError> {
        let opened = self.open_device(selection_handle)?;
        let native = self
            .runtime
            .device_options()
            .map_err(BringUpError::Runtime)?;
        let snapshot = normalize_device_options_snapshot(opened.device(), native);
        opened.release()?;
        Ok(snapshot)
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

    fn prepare_option_aware_capture_session(
        &self,
        request: &CaptureRunRequest,
    ) -> Result<(CaptureSession<'_>, EffectiveDeviceOptionState), CaptureRunError> {
        let opened = self.open_device(request.selection_handle)?;
        let total_channel_count = self
            .dslogic_plus_capabilities_for_opened(&opened)
            .map_err(|error| {
                CaptureRunError::BringUp(BringUpError::Runtime(RuntimeError::InvalidArgument(
                    error.to_string(),
                )))
            })?
            .total_channel_count;
        let effective_device_options = apply_capture_request_device_options(
            &self.runtime,
            request,
            total_channel_count,
        )?
        .expect("option-aware session requires validated device options");
        self.runtime
            .reset_acquisition_summary()
            .map_err(BringUpError::Runtime)?;
        self.runtime
            .register_acquisition_callbacks()
            .map_err(BringUpError::Runtime)?;
        Ok((CaptureSession { opened }, effective_device_options))
    }

    pub fn run_capture(
        &self,
        request: &CaptureRunRequest,
    ) -> Result<CaptureRunSummary, CaptureRunError> {
        let (session, effective_device_options) = if request.validated_device_options.is_some() {
                let (session, effective_device_options) =
                    self.prepare_option_aware_capture_session(request)?;
                (session, Some(effective_device_options))
            } else {
                let validated = self
                    .validate_capture_config(request.selection_handle, &request.config)
                    .map_err(|error| {
                        CaptureRunError::BringUp(BringUpError::Runtime(
                            RuntimeError::InvalidArgument(error.to_string()),
                        ))
                    })?;
                let session = self.prepare_capture_session(request.selection_handle, &validated)?;
                (session, None)
            };

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
                effective_device_options,
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
    #[error("could not determine the current executable path: {detail}")]
    CurrentExecutableUnavailable { detail: String },
    #[error(
        "bundled runtime `{path}` was not found next to executable directory `{executable_dir}` and no developer source runtime is available"
    )]
    BundledRuntimeMissing {
        path: PathBuf,
        executable_dir: PathBuf,
    },
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

#[derive(Debug, Error)]
pub enum DecodeBringUpError {
    #[error(transparent)]
    Runtime(#[from] DecodeRuntimeError),
    #[error("could not determine the current executable path: {detail}")]
    CurrentExecutableUnavailable { detail: String },
    #[error(
        "bundled decode runtime `{path}` was not found next to executable directory `{executable_dir}` and no developer source decode runtime is available"
    )]
    BundledRuntimeMissing {
        path: PathBuf,
        executable_dir: PathBuf,
    },
    #[error("decoder scripts directory `{path}` is missing")]
    MissingDecoderDirectory { path: PathBuf },
    #[error("decoder scripts directory `{path}` is not readable")]
    UnreadableDecoderDirectory { path: PathBuf },
    #[error("decoder scripts directory `{path}` does not contain any decoder scripts")]
    DecoderScriptsMissing { path: PathBuf },
    #[error("decoder `{decoder_id}` was not found in the loaded registry")]
    UnknownDecoder { decoder_id: String },
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

pub fn resolve_capture_artifact_paths(
    vcd_path: impl AsRef<Path>,
    metadata_path: Option<impl AsRef<Path>>,
) -> Result<CaptureArtifactPaths, CaptureArtifactPathError> {
    let vcd_path = vcd_path.as_ref().to_path_buf();
    if vcd_path.extension().and_then(|ext| ext.to_str()) != Some("vcd") {
        return Err(CaptureArtifactPathError::InvalidVcdExtension { path: vcd_path });
    }

    let metadata_path = metadata_path
        .map(|path| path.as_ref().to_path_buf())
        .unwrap_or_else(|| metadata_path_for_vcd(&vcd_path));

    if metadata_path == vcd_path {
        return Err(CaptureArtifactPathError::ConflictingArtifactPaths {
            vcd_path,
            metadata_path,
        });
    }

    if metadata_path.extension().and_then(|ext| ext.to_str()) != Some("json") {
        return Err(CaptureArtifactPathError::InvalidMetadataExtension {
            path: metadata_path,
        });
    }

    Ok(CaptureArtifactPaths {
        vcd_path,
        metadata_path,
    })
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

fn required_code(field: &str, value: Option<i16>) -> Result<i16, String> {
    value.ok_or_else(|| format!("missing `{field}` for capture device option reporting"))
}

fn required_u64(field: &str, value: Option<u64>) -> Result<u64, String> {
    value.ok_or_else(|| format!("missing `{field}` for capture device option reporting"))
}

fn resolve_enum_option_id(
    field: &str,
    options: &[EnumOptionSnapshot],
    code: i16,
) -> Result<String, String> {
    options
        .iter()
        .find(|option| option.native_code == code)
        .map(|option| option.id.clone())
        .ok_or_else(|| format!("missing `{field}` id for native code {code}"))
}

fn resolve_optional_enum_option_id(
    field: &str,
    options: &[EnumOptionSnapshot],
    code: Option<i16>,
) -> Result<Option<String>, String> {
    code.map(|value| resolve_enum_option_id(field, options, value))
        .transpose()
}

fn resolve_channel_mode_option_id(
    snapshot: &DeviceOptionsSnapshot,
    channel_mode_code: i16,
    operation_mode_code: Option<i16>,
) -> Result<String, String> {
    let scoped_match = operation_mode_code.and_then(|mode_code| {
        snapshot
            .channel_modes_by_operation_mode
            .iter()
            .find(|group| group.operation_mode_code == mode_code)
            .and_then(|group| {
                group.channel_modes
                    .iter()
                    .find(|mode| mode.native_code == channel_mode_code)
            })
    });
    if let Some(mode) = scoped_match {
        return Ok(mode.id.clone());
    }

    snapshot
        .channel_modes_by_operation_mode
        .iter()
        .flat_map(|group| group.channel_modes.iter())
        .find(|mode| mode.native_code == channel_mode_code)
        .map(|mode| mode.id.clone())
        .ok_or_else(|| format!("missing `channel_mode_id` for native code {channel_mode_code}"))
}

fn capture_device_option_snapshot_from_validated_request(
    request: &ValidatedDeviceOptionRequest,
) -> CaptureDeviceOptionSnapshot {
    CaptureDeviceOptionSnapshot {
        operation_mode_id: request.operation_mode_id.clone(),
        stop_option_id: request.stop_option_id.clone(),
        channel_mode_id: request.channel_mode_id.clone(),
        enabled_channels: request.enabled_channels.clone(),
        threshold_volts: request.threshold_volts,
        filter_id: request.filter_id.clone(),
        sample_rate_hz: request.sample_rate_hz,
        sample_limit: request.requested_sample_limit,
    }
}

fn capture_device_option_snapshot_from_effective_state(
    effective: &EffectiveDeviceOptionState,
    snapshot: &DeviceOptionsSnapshot,
) -> Result<CaptureDeviceOptionSnapshot, String> {
    let operation_mode_code = required_code("operation_mode_code", effective.operation_mode_code)?;
    let channel_mode_code = required_code("channel_mode_code", effective.channel_mode_code)?;
    Ok(CaptureDeviceOptionSnapshot {
        operation_mode_id: resolve_enum_option_id(
            "operation_mode_id",
            &snapshot.operation_modes,
            operation_mode_code,
        )?,
        stop_option_id: resolve_optional_enum_option_id(
            "stop_option_id",
            &snapshot.stop_options,
            effective.stop_option_code,
        )?,
        channel_mode_id: resolve_channel_mode_option_id(
            snapshot,
            channel_mode_code,
            Some(operation_mode_code),
        )?,
        enabled_channels: effective.enabled_channels.clone(),
        threshold_volts: effective.threshold_volts,
        filter_id: resolve_optional_enum_option_id(
            "filter_id",
            &snapshot.filters,
            effective.filter_code,
        )?,
        sample_rate_hz: required_u64("sample_rate_hz", effective.sample_rate_hz)?,
        sample_limit: required_u64("sample_limit", effective.sample_limit)?,
    })
}

fn inherited_capture_device_option_snapshot(
    snapshot: &DeviceOptionsSnapshot,
    validated_config: &ValidatedCaptureConfig,
) -> Result<CaptureDeviceOptionSnapshot, String> {
    Ok(CaptureDeviceOptionSnapshot {
        operation_mode_id: snapshot
            .current
            .operation_mode_id
            .clone()
            .ok_or_else(|| "missing `operation_mode_id` for baseline capture reporting".to_string())?,
        stop_option_id: snapshot.current.stop_option_id.clone(),
        channel_mode_id: snapshot
            .current
            .channel_mode_id
            .clone()
            .ok_or_else(|| "missing `channel_mode_id` for baseline capture reporting".to_string())?,
        enabled_channels: validated_config.enabled_channels.clone(),
        threshold_volts: snapshot.threshold.current_volts,
        filter_id: snapshot.current.filter_id.clone(),
        sample_rate_hz: validated_config.sample_rate_hz,
        sample_limit: validated_config.effective_sample_limit,
    })
}

pub fn build_capture_device_option_facts(
    request: &CaptureExportRequest,
) -> Result<CaptureDeviceOptionFacts, String> {
    if let Some(validated_device_options) = request.validated_device_options.as_ref() {
        let effective = request
            .capture
            .effective_device_options
            .as_ref()
            .ok_or_else(|| {
                "missing effective device option state for validated capture export".to_string()
            })?;
        Ok(CaptureDeviceOptionFacts {
            requested: capture_device_option_snapshot_from_validated_request(
                validated_device_options,
            ),
            effective: capture_device_option_snapshot_from_effective_state(
                effective,
                &request.device_options_snapshot,
            )?,
        })
    } else {
        let inherited = inherited_capture_device_option_snapshot(
            &request.device_options_snapshot,
            &request.validated_config,
        )?;
        Ok(CaptureDeviceOptionFacts {
            requested: inherited.clone(),
            effective: inherited,
        })
    }
}

fn build_capture_metadata(
    request: &CaptureExportRequest,
    metadata_path: &Path,
    export: &VcdExportFacts,
) -> Result<CaptureMetadata, String> {
    let device_options = build_capture_device_option_facts(request)?;
    Ok(CaptureMetadata {
        schema_version: 2,
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
        device_options,
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
        let artifact_paths =
            resolve_capture_artifact_paths(&request.vcd_path, request.metadata_path.as_ref())?;
        let vcd_path = artifact_paths.vcd_path;
        let metadata_path = artifact_paths.metadata_path;
        let export = self
            .runtime
            .export_recorded_vcd_to_path(&export_request, &vcd_path)
            .map_err(|error| CaptureExportError::ExportFailed {
                path: vcd_path.clone(),
                kind: export_failure_kind(&error),
                detail: error.to_string(),
            })?;
        let metadata =
            build_capture_metadata(request, &metadata_path, &export).map_err(|detail| {
                CaptureExportError::MetadataSerializationFailed {
                    path: metadata_path.clone(),
                    detail,
                }
            })?;
        let metadata_bytes = serde_json::to_vec_pretty(&metadata).map_err(|error| {
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

fn developer_resource_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("DSView")
        .join("DSView")
        .join("res")
}

fn developer_decoder_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("DSView")
        .join("libsigrokdecode4DSL")
        .join("decoders")
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

fn ensure_decoder_script_dir(path: &Path) -> Result<(), DecodeBringUpError> {
    if !path.exists() {
        return Err(DecodeBringUpError::MissingDecoderDirectory {
            path: path.to_path_buf(),
        });
    }

    let metadata = fs::metadata(path).map_err(|_| DecodeBringUpError::UnreadableDecoderDirectory {
        path: path.to_path_buf(),
    })?;
    if !metadata.is_dir() {
        return Err(DecodeBringUpError::UnreadableDecoderDirectory {
            path: path.to_path_buf(),
        });
    }

    let mut entries =
        fs::read_dir(path).map_err(|_| DecodeBringUpError::UnreadableDecoderDirectory {
            path: path.to_path_buf(),
        })?;
    if entries.next().is_none() {
        return Err(DecodeBringUpError::DecoderScriptsMissing {
            path: path.to_path_buf(),
        });
    }

    Ok(())
}

fn map_decode_runtime_error(
    error: DecodeRuntimeError,
    decoder_id: Option<&str>,
) -> DecodeBringUpError {
    match error {
        DecodeRuntimeError::NativeCall {
            code: DecodeRuntimeErrorCode::UnknownDecoder,
            ..
        } => DecodeBringUpError::UnknownDecoder {
            decoder_id: decoder_id.unwrap_or("unknown").to_string(),
        },
        other => DecodeBringUpError::Runtime(other),
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
    fn discovery_prefers_bundle_relative_layout() {
        let exe_dir = temp_dir("bundle-layout");
        let runtime_dir = exe_dir.join("runtime");
        let resource_dir = exe_dir.join("resources");
        fs::create_dir_all(&runtime_dir).unwrap();
        fs::create_dir_all(&resource_dir).unwrap();
        fs::write(runtime_dir.join(runtime_library_name()), b"runtime").unwrap();
        fs::write(resource_dir.join("DSLogicPlus.fw"), b"fw").unwrap();
        fs::write(resource_dir.join("DSLogicPlus.bin"), b"bin").unwrap();
        fs::write(resource_dir.join("DSLogicPlus-pgl12.bin"), b"bin").unwrap();

        let discovered =
            RuntimeDiscoveryPaths::from_executable_dir(&exe_dir, None::<&Path>).unwrap();
        assert_eq!(
            discovered.runtime_library,
            runtime_dir.join(runtime_library_name())
        );
        assert_eq!(discovered.resource_dir, resource_dir);
    }

    #[test]
    fn discovery_preserves_explicit_resource_override() {
        let exe_dir = temp_dir("bundle-override");
        let runtime_dir = exe_dir.join("runtime");
        let bundled_resources = exe_dir.join("resources");
        let override_resources = temp_dir("bundle-override-resources");
        fs::create_dir_all(&runtime_dir).unwrap();
        fs::create_dir_all(&bundled_resources).unwrap();
        fs::write(runtime_dir.join(runtime_library_name()), b"runtime").unwrap();
        fs::write(bundled_resources.join("DSLogicPlus.fw"), b"fw").unwrap();
        fs::write(bundled_resources.join("DSLogicPlus.bin"), b"bin").unwrap();
        fs::write(bundled_resources.join("DSLogicPlus-pgl12.bin"), b"bin").unwrap();
        fs::write(override_resources.join("DSLogicPlus.fw"), b"fw").unwrap();
        fs::write(override_resources.join("DSLogicPlus.bin"), b"bin").unwrap();
        fs::write(override_resources.join("DSLogicPlus-pgl12.bin"), b"bin").unwrap();

        let discovered =
            RuntimeDiscoveryPaths::from_executable_dir(&exe_dir, Some(&override_resources))
                .unwrap();
        assert_eq!(
            discovered.runtime_library,
            runtime_dir.join(runtime_library_name())
        );
        assert_eq!(discovered.resource_dir, override_resources);
    }

    #[test]
    fn connect_auto_requires_source_runtime_when_not_built() {
        let dir = temp_dir("resources-auto");
        fs::write(dir.join("DSLogicPlus.fw"), b"fw").unwrap();
        fs::write(dir.join("DSLogicPlus.bin"), b"bin").unwrap();
        fs::write(dir.join("DSLogicPlus-pgl12.bin"), b"bin").unwrap();

        if source_runtime_library_path().is_none() {
            let exe_dir = temp_dir("missing-runtime");
            let error =
                RuntimeDiscoveryPaths::from_executable_dir(&exe_dir, Some(&dir)).unwrap_err();
            assert!(matches!(error, BringUpError::BundledRuntimeMissing { .. }));
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

    #[test]
    fn offline_decode_input_reports_split_logic_sample_count() {
        let input = OfflineDecodeInput {
            samplerate_hz: 1_000_000,
            format: OfflineDecodeDataFormat::SplitLogic,
            sample_bytes: vec![0b00, 0b01, 0b10, 0b11],
            unitsize: 1,
            channel_count: None,
            logic_packet_lengths: Some(vec![2, 2]),
        };

        assert_eq!(input.sample_count().unwrap(), 4);
        input.validate_basic_shape().unwrap();
    }

    #[test]
    fn offline_decode_input_rejects_misaligned_split_logic_packets() {
        let input = OfflineDecodeInput {
            samplerate_hz: 1_000_000,
            format: OfflineDecodeDataFormat::SplitLogic,
            sample_bytes: vec![0b00, 0b01, 0b10, 0b11],
            unitsize: 2,
            channel_count: None,
            logic_packet_lengths: Some(vec![2, 1, 1]),
        };

        let error = input.validate_basic_shape().unwrap_err();
        assert!(matches!(
            error,
            OfflineDecodeInputError::InvalidPacketLengths { .. }
        ));
    }

    #[test]
    fn offline_decode_input_requires_cross_logic_channel_count() {
        let input = OfflineDecodeInput {
            samplerate_hz: 1_000_000,
            format: OfflineDecodeDataFormat::CrossLogic,
            sample_bytes: vec![0_u8; 16],
            unitsize: 1,
            channel_count: None,
            logic_packet_lengths: None,
        };

        let error = input.validate_basic_shape().unwrap_err();
        assert!(matches!(
            error,
            OfflineDecodeInputError::MissingChannelCount
        ));
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
                effective_device_options: None,
            },
            validated_config: ValidatedCaptureConfig {
                sample_rate_hz: 100_000_000,
                requested_sample_limit: 2048,
                effective_sample_limit: 2048,
                enabled_channels: vec![0, 1, 2, 3],
                channel_mode_id: 20,
            },
            vcd_path,
            metadata_path: None,
            tool_name: "dsview-cli".to_string(),
            tool_version: "1.1.1".to_string(),
            capture_started_at: UNIX_EPOCH + Duration::from_secs(1_744_018_496),
            device_model: "DSLogic Plus".to_string(),
            device_stable_id: "dslogic-plus".to_string(),
            selected_handle: SelectionHandle::new(7).unwrap(),
            validated_device_options: None,
            device_options_snapshot: DeviceOptionsSnapshot {
                device: DeviceIdentitySnapshot {
                    selection_handle: 7,
                    native_handle: 77,
                    stable_id: "dslogic-plus".to_string(),
                    kind: "DSLogic Plus".to_string(),
                    name: "DSLogic Plus".to_string(),
                },
                current: CurrentDeviceOptionValues {
                    operation_mode_id: Some("operation-mode:0".to_string()),
                    operation_mode_code: Some(0),
                    stop_option_id: Some("stop-option:1".to_string()),
                    stop_option_code: Some(1),
                    filter_id: Some("filter:0".to_string()),
                    filter_code: Some(0),
                    channel_mode_id: Some("channel-mode:20".to_string()),
                    channel_mode_code: Some(20),
                },
                operation_modes: vec![EnumOptionSnapshot {
                    id: "operation-mode:0".to_string(),
                    native_code: 0,
                    label: "Buffer Mode".to_string(),
                }],
                stop_options: vec![EnumOptionSnapshot {
                    id: "stop-option:1".to_string(),
                    native_code: 1,
                    label: "Stop after samples".to_string(),
                }],
                filters: vec![EnumOptionSnapshot {
                    id: "filter:0".to_string(),
                    native_code: 0,
                    label: "Off".to_string(),
                }],
                channel_modes_by_operation_mode: vec![ChannelModeGroupSnapshot {
                    operation_mode_id: "operation-mode:0".to_string(),
                    operation_mode_code: 0,
                    current_channel_mode_id: Some("channel-mode:20".to_string()),
                    current_channel_mode_code: Some(20),
                    channel_modes: vec![ChannelModeOptionSnapshot {
                        id: "channel-mode:20".to_string(),
                        native_code: 20,
                        label: "Buffer 100x16".to_string(),
                        max_enabled_channels: 16,
                    }],
                }],
                threshold: ThresholdCapabilitySnapshot {
                    id: "threshold:vth-range".to_string(),
                    kind: "voltage-range".to_string(),
                    current_volts: Some(1.8),
                    min_volts: 0.0,
                    max_volts: 5.0,
                    step_volts: 0.1,
                    legacy_metadata: None,
                },
            },
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
        assert_eq!(
            classify_capture_completion(&summary),
            CaptureCompletion::Detached
        );
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
        assert_eq!(
            metadata_json["artifacts"]["vcd_path"],
            vcd_path.display().to_string()
        );
        assert_eq!(
            metadata_json["artifacts"]["metadata_path"],
            metadata_path.display().to_string()
        );
        assert_eq!(
            metadata_json["capture"]["actual_sample_count"],
            export.sample_count
        );
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
        assert_eq!(
            metadata.acquisition.end_packet_status.as_deref(),
            Some("ok")
        );
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
