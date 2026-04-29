//! Raw native integration boundary for DSView CLI.
//!
//! This crate is the only allowed home for unsafe FFI when Phase 1 adds
//! bindings to `DSView/libsigrok4DSL`.

use std::cell::{Cell, RefCell};
use std::env;
use std::ffi::{CStr, CString, OsString};
use std::fmt;
use std::fs;
use std::os::raw::{c_char, c_int};
use std::path::{Path, PathBuf};
use std::ptr::NonNull;

use thiserror::Error;

/// Returns the platform-specific runtime library filename.
///
/// This is the shared naming contract between build.rs, packaging helpers,
/// and runtime discovery logic.
pub fn runtime_library_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "dsview_runtime.dll"
    } else if cfg!(target_os = "macos") {
        "libdsview_runtime.dylib"
    } else {
        "libdsview_runtime.so"
    }
}

/// Returns the platform-specific decode runtime library filename.
///
/// This stays distinct from the capture runtime so decoder discovery can be
/// packaged and resolved without mutating the shipped capture dependency graph.
pub fn decode_runtime_library_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "dsview_decode_runtime.dll"
    } else if cfg!(target_os = "macos") {
        "libdsview_decode_runtime.dylib"
    } else {
        "libdsview_decode_runtime.so"
    }
}

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
    fn dsview_bridge_ds_get_current_operation_mode(value: *mut c_int) -> c_int;
    fn dsview_bridge_ds_get_current_stop_option(value: *mut c_int) -> c_int;
    fn dsview_bridge_ds_get_current_filter(value: *mut c_int) -> c_int;
    fn dsview_bridge_ds_get_current_channel_mode(value: *mut c_int) -> c_int;
    fn dsview_bridge_ds_get_hw_depth(value: *mut u64) -> c_int;
    fn dsview_bridge_ds_get_vth(value: *mut f64) -> c_int;
    fn dsview_bridge_ds_get_samplerates(out_list: *mut RawSamplerateList) -> c_int;
    fn dsview_bridge_ds_get_channel_modes(
        out_modes: *mut RawChannelMode,
        max_modes: c_int,
        out_count: *mut c_int,
    ) -> c_int;
    fn dsview_bridge_ds_get_device_options(out_snapshot: *mut RawDeviceOptionsSnapshot) -> c_int;
    fn dsview_bridge_ds_get_validation_capabilities(
        out_snapshot: *mut RawDeviceOptionValidationSnapshot,
    ) -> c_int;
    fn dsview_bridge_ds_set_operation_mode(value: c_int) -> c_int;
    fn dsview_bridge_ds_set_stop_option(value: c_int) -> c_int;
    fn dsview_bridge_ds_set_channel_mode(value: c_int) -> c_int;
    fn dsview_bridge_ds_set_vth(value: f64) -> c_int;
    fn dsview_bridge_ds_set_filter(value: c_int) -> c_int;
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
    fn dsview_bridge_ds_export_recorded_vcd(
        request: *const RawVcdExportRequest,
        out_buffer: *mut RawExportBuffer,
    ) -> c_int;
    fn dsview_bridge_ds_begin_streaming_vcd(
        request: *const RawVcdExportRequest,
        path: *const c_char,
    ) -> c_int;
    fn dsview_bridge_ds_finish_streaming_vcd(out_facts: *mut RawStreamExportFacts) -> c_int;
    fn dsview_bridge_ds_abort_streaming_vcd();
    fn dsview_bridge_render_vcd_from_samples(
        request: *const RawVcdExportRequest,
        sample_bytes: *const u8,
        sample_bytes_len: usize,
        unitsize: u16,
        out_buffer: *mut RawExportBuffer,
    ) -> c_int;
    fn dsview_bridge_render_vcd_from_logic_packets(
        request: *const RawVcdExportRequest,
        sample_bytes: *const u8,
        sample_bytes_len: usize,
        logic_packet_lengths: *const usize,
        logic_packet_count: usize,
        unitsize: u16,
        out_buffer: *mut RawExportBuffer,
    ) -> c_int;
    fn dsview_bridge_render_vcd_from_cross_logic_packets(
        request: *const RawVcdExportRequest,
        sample_bytes: *const u8,
        sample_bytes_len: usize,
        logic_packet_lengths: *const usize,
        logic_packet_count: usize,
        out_buffer: *mut RawExportBuffer,
    ) -> c_int;
    fn dsview_bridge_free_export_buffer(buffer: *mut RawExportBuffer);
    fn dsview_decode_runtime_load(path: *const c_char) -> c_int;
    fn dsview_decode_runtime_init(decoder_dir: *const c_char) -> c_int;
    fn dsview_decode_runtime_exit() -> c_int;
    fn dsview_decode_last_loader_error() -> *const c_char;
    fn dsview_decode_last_error() -> *const c_char;
    fn dsview_decode_last_error_name() -> *const c_char;
    fn dsview_decode_list(out_list: *mut *mut RawDecodeListEntry, out_count: *mut usize) -> c_int;
    fn dsview_decode_free_list(list: *mut RawDecodeListEntry, count: usize);
    fn dsview_decode_inspect(
        decoder_id: *const c_char,
        out_metadata: *mut RawDecodeMetadata,
    ) -> c_int;
    fn dsview_decode_free_metadata(metadata: *mut RawDecodeMetadata);
    fn dsview_decode_session_new(out_session: *mut *mut RawDecodeExecutionSession) -> c_int;
    fn dsview_decode_session_set_samplerate(
        session: *mut RawDecodeExecutionSession,
        samplerate_hz: u64,
    ) -> c_int;
    fn dsview_decode_session_build_linear_stack(
        session: *mut RawDecodeExecutionSession,
        root: *const RawDecodeInstanceSpec,
        stack: *const RawDecodeInstanceSpec,
        stack_count: usize,
    ) -> c_int;
    fn dsview_decode_session_start(session: *mut RawDecodeExecutionSession) -> c_int;
    fn dsview_decode_session_send_logic_chunk(
        session: *mut RawDecodeExecutionSession,
        chunk: *const RawDecodeLogicChunk,
    ) -> c_int;
    fn dsview_decode_session_end(session: *mut RawDecodeExecutionSession) -> c_int;
    fn dsview_decode_session_take_captured_annotations(
        session: *mut RawDecodeExecutionSession,
        out_annotations: *mut *mut RawDecodeCapturedAnnotation,
        out_count: *mut usize,
    ) -> c_int;
    fn dsview_decode_free_captured_annotations(
        annotations: *mut RawDecodeCapturedAnnotation,
        count: usize,
    );
    fn dsview_decode_session_destroy(session: *mut RawDecodeExecutionSession);
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
const DSVIEW_EXPORT_ERR_GENERIC: i32 = -100;
const DSVIEW_EXPORT_ERR_NO_STREAM: i32 = -101;
const DSVIEW_EXPORT_ERR_OVERFLOW: i32 = -102;
const DSVIEW_EXPORT_ERR_BAD_END_STATUS: i32 = -103;
const DSVIEW_EXPORT_ERR_MISSING_SAMPLERATE: i32 = -104;
const DSVIEW_EXPORT_ERR_NO_ENABLED_CHANNELS: i32 = -105;
const DSVIEW_EXPORT_ERR_OUTPUT_MODULE: i32 = -106;
const DSVIEW_EXPORT_ERR_RUNTIME: i32 = -107;
const DSVIEW_DECODE_ERR_ARG: i32 = -20;
const DSVIEW_DECODE_ERR_NOT_LOADED: i32 = -21;
const DSVIEW_DECODE_ERR_DECODER_DIR: i32 = -22;
const DSVIEW_DECODE_ERR_PYTHON: i32 = -23;
const DSVIEW_DECODE_ERR_DECODER_LOAD: i32 = -24;
const DSVIEW_DECODE_ERR_UNKNOWN_DECODER: i32 = -25;
const DSVIEW_DECODE_ERR_UPSTREAM: i32 = -26;
const DSVIEW_DECODE_ERR_MALLOC: i32 = -27;
const DSVIEW_DECODE_ERR_INPUT_SHAPE: i32 = -28;
const DSVIEW_DECODE_ERR_SESSION: i32 = -29;

const DEVICE_NAME_CAPACITY: usize = 150;
const OPTION_LABEL_CAPACITY: usize = 64;
const OPTION_VALUE_CAPACITY: usize = 16;
const CHANNEL_MODE_GROUP_CAPACITY: usize = 8;
const CHANNEL_MODE_CAPACITY: usize = 16;
const SAMPLERATE_CAPACITY: usize = 64;
const THRESHOLD_KIND_CAPACITY: usize = 32;
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
struct RawOptionValue {
    code: i32,
    label: [u8; OPTION_LABEL_CAPACITY],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct RawChannelModeGroup {
    operation_mode_code: i32,
    channel_mode_count: u16,
    channel_modes: [RawChannelMode; CHANNEL_MODE_CAPACITY],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct RawValidationChannelMode {
    code: i32,
    label: [u8; OPTION_LABEL_CAPACITY],
    max_enabled_channels: u16,
    samplerate_count: u32,
    samplerates: [u64; SAMPLERATE_CAPACITY],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct RawValidationOperationMode {
    code: i32,
    label: [u8; OPTION_LABEL_CAPACITY],
    stop_option_count: u16,
    stop_options: [RawOptionValue; OPTION_VALUE_CAPACITY],
    channel_mode_count: u16,
    channel_modes: [RawValidationChannelMode; CHANNEL_MODE_CAPACITY],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct RawThresholdRange {
    kind: [u8; THRESHOLD_KIND_CAPACITY],
    id: [u8; OPTION_LABEL_CAPACITY],
    has_current_volts: c_int,
    current_volts: f64,
    min_volts: f64,
    max_volts: f64,
    step_volts: f64,
    has_current_legacy_code: c_int,
    current_legacy_code: i32,
    legacy_option_count: u16,
    legacy_options: [RawOptionValue; OPTION_VALUE_CAPACITY],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct RawDeviceOptionsSnapshot {
    has_current_operation_mode: c_int,
    current_operation_mode_code: i32,
    operation_mode_count: u16,
    operation_modes: [RawOptionValue; OPTION_VALUE_CAPACITY],
    has_current_stop_option: c_int,
    current_stop_option_code: i32,
    stop_option_count: u16,
    stop_options: [RawOptionValue; OPTION_VALUE_CAPACITY],
    has_current_filter: c_int,
    current_filter_code: i32,
    filter_count: u16,
    filters: [RawOptionValue; OPTION_VALUE_CAPACITY],
    has_current_channel_mode: c_int,
    current_channel_mode_code: i32,
    channel_mode_group_count: u16,
    channel_mode_groups: [RawChannelModeGroup; CHANNEL_MODE_GROUP_CAPACITY],
    threshold: RawThresholdRange,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct RawDeviceOptionValidationSnapshot {
    has_current_operation_mode: c_int,
    current_operation_mode_code: i32,
    has_current_stop_option: c_int,
    current_stop_option_code: i32,
    has_current_filter: c_int,
    current_filter_code: i32,
    has_current_channel_mode: c_int,
    current_channel_mode_code: i32,
    total_channel_count: u16,
    hardware_sample_capacity: u64,
    filter_count: u16,
    filters: [RawOptionValue; OPTION_VALUE_CAPACITY],
    threshold: RawThresholdRange,
    operation_mode_count: u16,
    operation_modes: [RawValidationOperationMode; CHANNEL_MODE_GROUP_CAPACITY],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct RawSamplerateList {
    count: u32,
    values: [u64; 64],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct RawVcdExportRequest {
    samplerate_hz: u64,
    enabled_channels: *const u16,
    enabled_channel_count: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct RawExportBuffer {
    data: *mut u8,
    len: usize,
    sample_count: u64,
    packet_count: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct RawStreamExportFacts {
    sample_count: u64,
    packet_count: usize,
    output_bytes: u64,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct RawDecodeChannel {
    id: *mut c_char,
    name: *mut c_char,
    desc: *mut c_char,
    order: c_int,
    channel_type: c_int,
    idn: *mut c_char,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct RawDecodeOption {
    id: *mut c_char,
    idn: *mut c_char,
    desc: *mut c_char,
    value_kind: c_int,
    default_value: *mut c_char,
    values: *mut *mut c_char,
    value_count: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct RawDecodeAnnotation {
    id: *mut c_char,
    label: *mut c_char,
    description: *mut c_char,
    annotation_type: c_int,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct RawDecodeAnnotationRow {
    id: *mut c_char,
    desc: *mut c_char,
    annotation_classes: *mut usize,
    annotation_class_count: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct RawDecodeCapturedAnnotation {
    decoder_id: *mut c_char,
    start_sample: u64,
    end_sample: u64,
    ann_class: c_int,
    ann_type: c_int,
    texts: *mut *mut c_char,
    text_count: usize,
    number_hex: *mut c_char,
    has_numeric_value: c_int,
    numeric_value: i64,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct RawDecodeListEntry {
    id: *mut c_char,
    name: *mut c_char,
    longname: *mut c_char,
    desc: *mut c_char,
    license: *mut c_char,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct RawDecodeMetadata {
    id: *mut c_char,
    name: *mut c_char,
    longname: *mut c_char,
    desc: *mut c_char,
    license: *mut c_char,
    inputs: *mut *mut c_char,
    input_count: usize,
    outputs: *mut *mut c_char,
    output_count: usize,
    tags: *mut *mut c_char,
    tag_count: usize,
    required_channels: *mut RawDecodeChannel,
    required_channel_count: usize,
    optional_channels: *mut RawDecodeChannel,
    optional_channel_count: usize,
    options: *mut RawDecodeOption,
    option_count: usize,
    annotations: *mut RawDecodeAnnotation,
    annotation_count: usize,
    annotation_rows: *mut RawDecodeAnnotationRow,
    annotation_row_count: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct RawDecodeOptionValue {
    kind: c_int,
    string_value: *const c_char,
    integer_value: i64,
    float_value: f64,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct RawDecodeOptionEntry {
    option_id: *const c_char,
    value: RawDecodeOptionValue,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct RawDecodeChannelBinding {
    channel_id: *const c_char,
    channel_index: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct RawDecodeInstanceSpec {
    decoder_id: *const c_char,
    channel_bindings: *const RawDecodeChannelBinding,
    channel_binding_count: usize,
    options: *const RawDecodeOptionEntry,
    option_count: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct RawDecodeLogicChunk {
    format: c_int,
    unitsize: u16,
    channel_count: u16,
    abs_start_sample: u64,
    abs_end_sample: u64,
    sample_bytes: *const u8,
    sample_bytes_len: usize,
}

#[repr(C)]
struct RawDecodeExecutionSession {
    _private: [u8; 0],
}

#[derive(Debug, Clone, PartialEq)]
pub enum DecodeSessionOptionValue {
    String(String),
    Integer(i64),
    Float(f64),
}

#[derive(Debug, Clone, PartialEq)]
pub struct DecodeSessionOption {
    pub option_id: String,
    pub value: DecodeSessionOptionValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodeSessionChannelBinding {
    pub channel_id: String,
    pub channel_index: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DecodeSessionInstance {
    pub decoder_id: String,
    pub channel_bindings: Vec<DecodeSessionChannelBinding>,
    pub options: Vec<DecodeSessionOption>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeExecutionLogicFormat {
    SplitLogic { unitsize: u16 },
    CrossLogic { channel_count: u16 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VcdExportRequest {
    pub samplerate_hz: u64,
    pub enabled_channels: Vec<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VcdExport {
    pub bytes: Vec<u8>,
    pub sample_count: u64,
    pub packet_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VcdExportFacts {
    pub sample_count: u64,
    pub packet_count: usize,
    pub output_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportErrorCode {
    Generic,
    NoStream,
    Overflow,
    BadEndStatus,
    MissingSamplerate,
    NoEnabledChannels,
    OutputModuleUnavailable,
    Runtime,
    Unknown(i32),
}

impl ExportErrorCode {
    fn from_raw(raw: i32) -> Self {
        match raw {
            DSVIEW_EXPORT_ERR_GENERIC => Self::Generic,
            DSVIEW_EXPORT_ERR_NO_STREAM => Self::NoStream,
            DSVIEW_EXPORT_ERR_OVERFLOW => Self::Overflow,
            DSVIEW_EXPORT_ERR_BAD_END_STATUS => Self::BadEndStatus,
            DSVIEW_EXPORT_ERR_MISSING_SAMPLERATE => Self::MissingSamplerate,
            DSVIEW_EXPORT_ERR_NO_ENABLED_CHANNELS => Self::NoEnabledChannels,
            DSVIEW_EXPORT_ERR_OUTPUT_MODULE => Self::OutputModuleUnavailable,
            DSVIEW_EXPORT_ERR_RUNTIME => Self::Runtime,
            other => Self::Unknown(other),
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Generic => "export_generic",
            Self::NoStream => "export_no_stream",
            Self::Overflow => "export_overflow",
            Self::BadEndStatus => "export_bad_end_status",
            Self::MissingSamplerate => "export_missing_samplerate",
            Self::NoEnabledChannels => "export_no_enabled_channels",
            Self::OutputModuleUnavailable => "export_output_module_unavailable",
            Self::Runtime => "export_runtime",
            Self::Unknown(_) => "export_unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeRuntimeErrorCode {
    DecoderDirectory,
    Python,
    DecoderLoad,
    UnknownDecoder,
    Upstream,
    OutOfMemory,
    InputShape,
    SessionState,
    Unknown(i32),
}

impl DecodeRuntimeErrorCode {
    fn from_raw(raw: i32) -> Self {
        match raw {
            DSVIEW_DECODE_ERR_DECODER_DIR => Self::DecoderDirectory,
            DSVIEW_DECODE_ERR_PYTHON => Self::Python,
            DSVIEW_DECODE_ERR_DECODER_LOAD => Self::DecoderLoad,
            DSVIEW_DECODE_ERR_UNKNOWN_DECODER => Self::UnknownDecoder,
            DSVIEW_DECODE_ERR_UPSTREAM => Self::Upstream,
            DSVIEW_DECODE_ERR_MALLOC => Self::OutOfMemory,
            DSVIEW_DECODE_ERR_INPUT_SHAPE => Self::InputShape,
            DSVIEW_DECODE_ERR_SESSION => Self::SessionState,
            other => Self::Unknown(other),
        }
    }
}

#[derive(Debug, Error)]
pub enum DecodeRuntimeError {
    #[error("decode runtime bridge rejected argument: {0}")]
    InvalidArgument(String),
    #[error("decode runtime bridge shared library is not loaded")]
    BridgeNotLoaded,
    #[error("failed to load decode runtime shared library `{path}`: {detail}")]
    LibraryLoad { path: PathBuf, detail: String },
    #[error("failed to resolve required srd_* symbols from `{path}`: {detail}")]
    SymbolLoad { path: PathBuf, detail: String },
    #[error("decode native call `{operation}` failed with {code:?}: {detail}")]
    NativeCall {
        operation: &'static str,
        code: DecodeRuntimeErrorCode,
        detail: String,
    },
    #[error("path contains an interior NUL byte: {path}")]
    PathContainsNul { path: PathBuf },
    #[error("decode metadata contains invalid UTF-8")]
    InvalidUtf8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodeChannel {
    pub id: String,
    pub name: String,
    pub description: String,
    pub order: i32,
    pub channel_type: i32,
    pub idn: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodeOption {
    pub id: String,
    pub idn: Option<String>,
    pub description: Option<String>,
    pub value_kind: DecodeOptionValueKind,
    pub default_value: Option<String>,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeOptionValueKind {
    Unknown,
    String,
    Integer,
    Float,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodeAnnotation {
    pub id: String,
    pub label: Option<String>,
    pub description: Option<String>,
    pub annotation_type: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodeAnnotationRow {
    pub id: String,
    pub description: Option<String>,
    pub annotation_classes: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodeCapturedAnnotation {
    pub decoder_id: String,
    pub start_sample: u64,
    pub end_sample: u64,
    pub annotation_class: i32,
    pub annotation_type: i32,
    pub texts: Vec<String>,
    pub number_hex: Option<String>,
    pub numeric_value: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodeInput {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodeOutput {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodeDecoder {
    pub id: String,
    pub name: String,
    pub longname: String,
    pub description: String,
    pub license: String,
    pub inputs: Vec<DecodeInput>,
    pub outputs: Vec<DecodeOutput>,
    pub tags: Vec<String>,
    pub required_channels: Vec<DecodeChannel>,
    pub optional_channels: Vec<DecodeChannel>,
    pub options: Vec<DecodeOption>,
    pub annotations: Vec<DecodeAnnotation>,
    pub annotation_rows: Vec<DecodeAnnotationRow>,
}

pub type DecodeListEntry = DecodeDecoder;
pub type DecodeMetadata = DecodeDecoder;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceOptionValue {
    pub code: i16,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceOptionChannelMode {
    pub code: i16,
    pub label: String,
    pub max_enabled_channels: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceOptionChannelModeGroup {
    pub operation_mode_code: i16,
    pub channel_modes: Vec<DeviceOptionChannelMode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceOptionValidationChannelMode {
    pub code: i16,
    pub label: String,
    pub max_enabled_channels: u16,
    pub supported_sample_rates: Vec<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceOptionValidationOperationMode {
    pub code: i16,
    pub label: String,
    pub stop_options: Vec<DeviceOptionValue>,
    pub channel_modes: Vec<DeviceOptionValidationChannelMode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyThresholdMetadata {
    pub current_code: Option<i16>,
    pub options: Vec<DeviceOptionValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThresholdVoltageRange {
    pub kind: String,
    pub id: String,
    pub current_volts: Option<f64>,
    pub min_volts: f64,
    pub max_volts: f64,
    pub step_volts: f64,
    pub legacy: Option<LegacyThresholdMetadata>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeviceOptionsSnapshot {
    pub current_operation_mode_code: Option<i16>,
    pub operation_modes: Vec<DeviceOptionValue>,
    pub current_stop_option_code: Option<i16>,
    pub stop_options: Vec<DeviceOptionValue>,
    pub current_filter_code: Option<i16>,
    pub filters: Vec<DeviceOptionValue>,
    pub current_channel_mode_code: Option<i16>,
    pub channel_mode_groups: Vec<DeviceOptionChannelModeGroup>,
    pub threshold: ThresholdVoltageRange,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeviceOptionValidationSnapshot {
    pub current_operation_mode_code: Option<i16>,
    pub current_stop_option_code: Option<i16>,
    pub current_filter_code: Option<i16>,
    pub current_channel_mode_code: Option<i16>,
    pub total_channel_count: u16,
    pub hardware_sample_capacity: u64,
    pub filters: Vec<DeviceOptionValue>,
    pub threshold: ThresholdVoltageRange,
    pub operation_modes: Vec<DeviceOptionValidationOperationMode>,
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
    #[error("export call `{operation}` failed with {code:?}")]
    ExportCall {
        operation: &'static str,
        code: ExportErrorCode,
    },
    #[error("device list returned an invalid handle")]
    InvalidDeviceHandle,
    #[error("device name contains invalid UTF-8")]
    InvalidDeviceName,
    #[error("path contains an interior NUL byte: {path}")]
    PathContainsNul { path: PathBuf },
    #[error("failed to write VCD temp file `{path}`: {detail}")]
    TempWrite { path: PathBuf, detail: String },
    #[error("failed to promote VCD temp file `{from}` to `{to}`: {detail}")]
    TempPromote {
        from: PathBuf,
        to: PathBuf,
        detail: String,
    },
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

    pub fn device_options(&self) -> Result<DeviceOptionsSnapshot, RuntimeError> {
        let mut raw = RawDeviceOptionsSnapshot {
            has_current_operation_mode: 0,
            current_operation_mode_code: 0,
            operation_mode_count: 0,
            operation_modes: [RawOptionValue {
                code: 0,
                label: [0; OPTION_LABEL_CAPACITY],
            }; OPTION_VALUE_CAPACITY],
            has_current_stop_option: 0,
            current_stop_option_code: 0,
            stop_option_count: 0,
            stop_options: [RawOptionValue {
                code: 0,
                label: [0; OPTION_LABEL_CAPACITY],
            }; OPTION_VALUE_CAPACITY],
            has_current_filter: 0,
            current_filter_code: 0,
            filter_count: 0,
            filters: [RawOptionValue {
                code: 0,
                label: [0; OPTION_LABEL_CAPACITY],
            }; OPTION_VALUE_CAPACITY],
            has_current_channel_mode: 0,
            current_channel_mode_code: 0,
            channel_mode_group_count: 0,
            channel_mode_groups: [RawChannelModeGroup {
                operation_mode_code: 0,
                channel_mode_count: 0,
                channel_modes: [RawChannelMode {
                    id: 0,
                    name: [0; OPTION_LABEL_CAPACITY],
                    max_enabled_channels: 0,
                }; CHANNEL_MODE_CAPACITY],
            }; CHANNEL_MODE_GROUP_CAPACITY],
            threshold: RawThresholdRange {
                kind: [0; THRESHOLD_KIND_CAPACITY],
                id: [0; OPTION_LABEL_CAPACITY],
                has_current_volts: 0,
                current_volts: 0.0,
                min_volts: 0.0,
                max_volts: 0.0,
                step_volts: 0.0,
                has_current_legacy_code: 0,
                current_legacy_code: 0,
                legacy_option_count: 0,
                legacy_options: [RawOptionValue {
                    code: 0,
                    label: [0; OPTION_LABEL_CAPACITY],
                }; OPTION_VALUE_CAPACITY],
            },
        };
        native_call_status("ds_get_device_options", unsafe {
            dsview_bridge_ds_get_device_options(&mut raw)
        })?;
        decode_device_options_snapshot(&raw)
    }

    pub fn device_option_validation_capabilities(
        &self,
    ) -> Result<DeviceOptionValidationSnapshot, RuntimeError> {
        let mut raw = RawDeviceOptionValidationSnapshot {
            has_current_operation_mode: 0,
            current_operation_mode_code: 0,
            has_current_stop_option: 0,
            current_stop_option_code: 0,
            has_current_filter: 0,
            current_filter_code: 0,
            has_current_channel_mode: 0,
            current_channel_mode_code: 0,
            total_channel_count: 0,
            hardware_sample_capacity: 0,
            filter_count: 0,
            filters: [RawOptionValue {
                code: 0,
                label: [0; OPTION_LABEL_CAPACITY],
            }; OPTION_VALUE_CAPACITY],
            threshold: RawThresholdRange {
                kind: [0; THRESHOLD_KIND_CAPACITY],
                id: [0; OPTION_LABEL_CAPACITY],
                has_current_volts: 0,
                current_volts: 0.0,
                min_volts: 0.0,
                max_volts: 0.0,
                step_volts: 0.0,
                has_current_legacy_code: 0,
                current_legacy_code: 0,
                legacy_option_count: 0,
                legacy_options: [RawOptionValue {
                    code: 0,
                    label: [0; OPTION_LABEL_CAPACITY],
                }; OPTION_VALUE_CAPACITY],
            },
            operation_mode_count: 0,
            operation_modes: [RawValidationOperationMode {
                code: 0,
                label: [0; OPTION_LABEL_CAPACITY],
                stop_option_count: 0,
                stop_options: [RawOptionValue {
                    code: 0,
                    label: [0; OPTION_LABEL_CAPACITY],
                }; OPTION_VALUE_CAPACITY],
                channel_mode_count: 0,
                channel_modes: [RawValidationChannelMode {
                    code: 0,
                    label: [0; OPTION_LABEL_CAPACITY],
                    max_enabled_channels: 0,
                    samplerate_count: 0,
                    samplerates: [0; SAMPLERATE_CAPACITY],
                }; CHANNEL_MODE_CAPACITY],
            }; CHANNEL_MODE_GROUP_CAPACITY],
        };
        native_call_status("ds_get_validation_capabilities", unsafe {
            dsview_bridge_ds_get_validation_capabilities(&mut raw)
        })?;
        decode_device_option_validation_snapshot(&raw)
    }

    pub fn set_operation_mode(&self, value: i16) -> Result<(), RuntimeError> {
        native_call_status("ds_set_operation_mode", unsafe {
            dsview_bridge_ds_set_operation_mode(value as c_int)
        })
    }

    pub fn set_stop_option(&self, value: i16) -> Result<(), RuntimeError> {
        native_call_status("ds_set_stop_option", unsafe {
            dsview_bridge_ds_set_stop_option(value as c_int)
        })
    }

    pub fn set_channel_mode(&self, value: i16) -> Result<(), RuntimeError> {
        native_call_status("ds_set_channel_mode", unsafe {
            dsview_bridge_ds_set_channel_mode(value as c_int)
        })
    }

    pub fn set_threshold_volts(&self, value: f64) -> Result<(), RuntimeError> {
        native_call_status("ds_set_threshold_volts", unsafe {
            dsview_bridge_ds_set_vth(value)
        })
    }

    pub fn set_filter(&self, value: i16) -> Result<(), RuntimeError> {
        native_call_status("ds_set_filter", unsafe {
            dsview_bridge_ds_set_filter(value as c_int)
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

    pub fn current_operation_mode_code(&self) -> Result<Option<i16>, RuntimeError> {
        Ok(get_optional_i32_config(
            "ds_get_current_operation_mode",
            dsview_bridge_ds_get_current_operation_mode,
        )?
        .map(|value| value as i16))
    }

    pub fn current_stop_option_code(&self) -> Result<Option<i16>, RuntimeError> {
        Ok(get_optional_i32_config(
            "ds_get_current_stop_option",
            dsview_bridge_ds_get_current_stop_option,
        )?
        .map(|value| value as i16))
    }

    pub fn current_channel_mode_code(&self) -> Result<Option<i16>, RuntimeError> {
        Ok(get_optional_i32_config(
            "ds_get_current_channel_mode",
            dsview_bridge_ds_get_current_channel_mode,
        )?
        .map(|value| value as i16))
    }

    pub fn current_threshold_volts(&self) -> Result<Option<f64>, RuntimeError> {
        get_optional_f64_config("ds_get_vth", dsview_bridge_ds_get_vth)
    }

    pub fn current_filter_code(&self) -> Result<Option<i16>, RuntimeError> {
        Ok(
            get_optional_i32_config("ds_get_current_filter", dsview_bridge_ds_get_current_filter)?
                .map(|value| value as i16),
        )
    }

    pub fn current_sample_limit(&self) -> Result<Option<u64>, RuntimeError> {
        get_optional_u64_config(
            "ds_get_current_sample_limit",
            dsview_bridge_ds_get_current_sample_limit,
        )
    }

    pub fn current_samplerate(&self) -> Result<Option<u64>, RuntimeError> {
        get_optional_u64_config(
            "ds_get_current_samplerate",
            dsview_bridge_ds_get_current_samplerate,
        )
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

    pub fn export_recorded_vcd(
        &self,
        request: &VcdExportRequest,
    ) -> Result<VcdExport, RuntimeError> {
        let raw_request = raw_vcd_export_request(request)?;
        let mut raw = RawExportBuffer {
            data: std::ptr::null_mut(),
            len: 0,
            sample_count: 0,
            packet_count: 0,
        };
        export_call_status("ds_export_recorded_vcd", unsafe {
            dsview_bridge_ds_export_recorded_vcd(&raw_request, &mut raw)
        })?;
        export_from_raw(raw)
    }

    pub fn export_recorded_vcd_to_path(
        &self,
        request: &VcdExportRequest,
        final_path: impl AsRef<Path>,
    ) -> Result<VcdExportFacts, RuntimeError> {
        let export = self.export_recorded_vcd(request)?;
        write_vcd_atomically(final_path.as_ref(), &export.bytes)?;
        Ok(VcdExportFacts {
            sample_count: export.sample_count,
            packet_count: export.packet_count,
            output_bytes: export.bytes.len() as u64,
        })
    }

    pub fn begin_streaming_vcd_to_path(
        &self,
        request: &VcdExportRequest,
        path: impl AsRef<Path>,
    ) -> Result<(), RuntimeError> {
        let raw_request = raw_vcd_export_request(request)?;
        let path = path.as_ref();
        let path_text = path.to_string_lossy();
        let c_path =
            CString::new(path_text.as_bytes()).map_err(|_| RuntimeError::PathContainsNul {
                path: path.to_path_buf(),
            })?;
        export_call_status("ds_begin_streaming_vcd", unsafe {
            dsview_bridge_ds_begin_streaming_vcd(&raw_request, c_path.as_ptr())
        })
    }

    pub fn finish_streaming_vcd(&self) -> Result<VcdExportFacts, RuntimeError> {
        let mut raw = RawStreamExportFacts {
            sample_count: 0,
            packet_count: 0,
            output_bytes: 0,
        };
        export_call_status("ds_finish_streaming_vcd", unsafe {
            dsview_bridge_ds_finish_streaming_vcd(&mut raw)
        })?;
        Ok(VcdExportFacts {
            sample_count: raw.sample_count,
            packet_count: raw.packet_count,
            output_bytes: raw.output_bytes,
        })
    }

    pub fn abort_streaming_vcd(&self) {
        unsafe { dsview_bridge_ds_abort_streaming_vcd() };
    }

    pub fn render_vcd_from_samples(
        &self,
        request: &VcdExportRequest,
        sample_bytes: &[u8],
        unitsize: u16,
    ) -> Result<VcdExport, RuntimeError> {
        if sample_bytes.is_empty() {
            return Err(RuntimeError::InvalidArgument(
                "sample bytes must not be empty".to_string(),
            ));
        }
        let raw_request = raw_vcd_export_request(request)?;
        let mut raw = RawExportBuffer {
            data: std::ptr::null_mut(),
            len: 0,
            sample_count: 0,
            packet_count: 0,
        };
        export_call_status("render_vcd_from_samples", unsafe {
            dsview_bridge_render_vcd_from_samples(
                &raw_request,
                sample_bytes.as_ptr(),
                sample_bytes.len(),
                unitsize,
                &mut raw,
            )
        })?;
        export_from_raw(raw)
    }

    pub fn render_vcd_from_samples_to_path(
        &self,
        request: &VcdExportRequest,
        sample_bytes: &[u8],
        unitsize: u16,
        final_path: impl AsRef<Path>,
    ) -> Result<VcdExportFacts, RuntimeError> {
        let export = self.render_vcd_from_samples(request, sample_bytes, unitsize)?;
        write_vcd_atomically(final_path.as_ref(), &export.bytes)?;
        Ok(VcdExportFacts {
            sample_count: export.sample_count,
            packet_count: export.packet_count,
            output_bytes: export.bytes.len() as u64,
        })
    }

    pub fn render_vcd_from_logic_packets(
        &self,
        request: &VcdExportRequest,
        sample_bytes: &[u8],
        logic_packet_lengths: &[usize],
        unitsize: u16,
    ) -> Result<VcdExport, RuntimeError> {
        if sample_bytes.is_empty() {
            return Err(RuntimeError::InvalidArgument(
                "sample bytes must not be empty".to_string(),
            ));
        }
        if logic_packet_lengths.is_empty() {
            return Err(RuntimeError::InvalidArgument(
                "at least one logic packet is required".to_string(),
            ));
        }
        if unitsize == 0 {
            return Err(RuntimeError::InvalidArgument(
                "logic packet unitsize must be greater than zero".to_string(),
            ));
        }
        if (sample_bytes.len() % unitsize as usize) != 0 {
            return Err(RuntimeError::InvalidArgument(
                "sample bytes length must divide evenly by unitsize".to_string(),
            ));
        }
        let expected_total: usize = logic_packet_lengths.iter().sum();
        if expected_total != sample_bytes.len() {
            return Err(RuntimeError::InvalidArgument(
                "logic packet lengths must sum to the sample byte length".to_string(),
            ));
        }
        if logic_packet_lengths
            .iter()
            .any(|length| *length == 0 || (length % unitsize as usize) != 0)
        {
            return Err(RuntimeError::InvalidArgument(
                "each logic packet length must be non-zero and aligned to unitsize".to_string(),
            ));
        }

        let raw_request = raw_vcd_export_request(request)?;
        let mut raw = RawExportBuffer {
            data: std::ptr::null_mut(),
            len: 0,
            sample_count: 0,
            packet_count: 0,
        };
        export_call_status("render_vcd_from_logic_packets", unsafe {
            dsview_bridge_render_vcd_from_logic_packets(
                &raw_request,
                sample_bytes.as_ptr(),
                sample_bytes.len(),
                logic_packet_lengths.as_ptr(),
                logic_packet_lengths.len(),
                unitsize,
                &mut raw,
            )
        })?;
        export_from_raw(raw)
    }

    pub fn render_vcd_from_cross_logic_packets(
        &self,
        request: &VcdExportRequest,
        sample_bytes: &[u8],
        logic_packet_lengths: &[usize],
    ) -> Result<VcdExport, RuntimeError> {
        if sample_bytes.is_empty() {
            return Err(RuntimeError::InvalidArgument(
                "sample bytes must not be empty".to_string(),
            ));
        }
        if logic_packet_lengths.is_empty() {
            return Err(RuntimeError::InvalidArgument(
                "at least one logic packet is required".to_string(),
            ));
        }
        if request.enabled_channels.is_empty() {
            return Err(RuntimeError::InvalidArgument(
                "at least one enabled channel is required for cross-logic export".to_string(),
            ));
        }

        let cross_unitsize = request.enabled_channels.len() * std::mem::size_of::<u64>();
        if (sample_bytes.len() % cross_unitsize) != 0 {
            return Err(RuntimeError::InvalidArgument(
                "cross-logic sample bytes length must divide evenly by enabled_channel_count * 8"
                    .to_string(),
            ));
        }
        let expected_total: usize = logic_packet_lengths.iter().sum();
        if expected_total != sample_bytes.len() {
            return Err(RuntimeError::InvalidArgument(
                "logic packet lengths must sum to the sample byte length".to_string(),
            ));
        }
        if logic_packet_lengths
            .iter()
            .any(|length| *length == 0 || (length % cross_unitsize) != 0)
        {
            return Err(RuntimeError::InvalidArgument(
                "each cross-logic packet length must be non-zero and aligned to enabled_channel_count * 8"
                    .to_string(),
            ));
        }

        let raw_request = raw_vcd_export_request(request)?;
        let mut raw = RawExportBuffer {
            data: std::ptr::null_mut(),
            len: 0,
            sample_count: 0,
            packet_count: 0,
        };
        export_call_status("render_vcd_from_cross_logic_packets", unsafe {
            dsview_bridge_render_vcd_from_cross_logic_packets(
                &raw_request,
                sample_bytes.as_ptr(),
                sample_bytes.len(),
                logic_packet_lengths.as_ptr(),
                logic_packet_lengths.len(),
                &mut raw,
            )
        })?;
        export_from_raw(raw)
    }

    pub fn render_vcd_from_logic_packets_to_path(
        &self,
        request: &VcdExportRequest,
        sample_bytes: &[u8],
        logic_packet_lengths: &[usize],
        unitsize: u16,
        final_path: impl AsRef<Path>,
    ) -> Result<VcdExportFacts, RuntimeError> {
        let export = self.render_vcd_from_logic_packets(
            request,
            sample_bytes,
            logic_packet_lengths,
            unitsize,
        )?;
        write_vcd_atomically(final_path.as_ref(), &export.bytes)?;
        Ok(VcdExportFacts {
            sample_count: export.sample_count,
            packet_count: export.packet_count,
            output_bytes: export.bytes.len() as u64,
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

#[derive(Debug)]
pub struct DecodeRuntimeBridge {
    library_path: PathBuf,
    initialized: Cell<bool>,
    python_home_guard: RefCell<Option<PythonHomeGuard>>,
}

impl DecodeRuntimeBridge {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, DecodeRuntimeError> {
        let path = path.as_ref();
        if !path.is_file() {
            return Err(DecodeRuntimeError::LibraryLoad {
                path: path.to_path_buf(),
                detail: "decode runtime library path does not exist".to_string(),
            });
        }
        let c_path = path_to_decode_cstring(path)?;
        let status = unsafe { dsview_decode_runtime_load(c_path.as_ptr()) };

        match status {
            0 => Ok(Self {
                library_path: path.to_path_buf(),
                initialized: Cell::new(false),
                python_home_guard: RefCell::new(None),
            }),
            DSVIEW_BRIDGE_ERR_ARG | DSVIEW_DECODE_ERR_ARG => {
                Err(DecodeRuntimeError::InvalidArgument(
                    "decode runtime library path must not be empty".to_string(),
                ))
            }
            DSVIEW_BRIDGE_ERR_DLOPEN => Err(DecodeRuntimeError::LibraryLoad {
                path: path.to_path_buf(),
                detail: decode_last_loader_error(),
            }),
            DSVIEW_BRIDGE_ERR_DLSYM => Err(DecodeRuntimeError::SymbolLoad {
                path: path.to_path_buf(),
                detail: decode_last_loader_error(),
            }),
            DSVIEW_BRIDGE_ERR_NOT_LOADED | DSVIEW_DECODE_ERR_NOT_LOADED => {
                Err(DecodeRuntimeError::BridgeNotLoaded)
            }
            other => Err(DecodeRuntimeError::InvalidArgument(format!(
                "unexpected decode bridge status {other}"
            ))),
        }
    }

    pub fn library_path(&self) -> &Path {
        &self.library_path
    }

    pub fn init(&self, decoder_dir: impl AsRef<Path>) -> Result<(), DecodeRuntimeError> {
        self.init_with_python_home(decoder_dir, None::<&Path>)
    }

    pub fn init_with_python_home(
        &self,
        decoder_dir: impl AsRef<Path>,
        python_home: Option<impl AsRef<Path>>,
    ) -> Result<(), DecodeRuntimeError> {
        let guard = if let Some(path) = python_home {
            Some(PythonHomeGuard::activate(path.as_ref())?)
        } else {
            None
        };
        let c_path = path_to_decode_cstring(decoder_dir.as_ref())?;
        decode_native_call_status("decode runtime init", unsafe {
            dsview_decode_runtime_init(c_path.as_ptr())
        })?;
        *self.python_home_guard.borrow_mut() = guard;
        self.initialized.set(true);
        Ok(())
    }

    pub fn exit(&self) -> Result<(), DecodeRuntimeError> {
        decode_native_call_status("decode runtime exit", unsafe {
            dsview_decode_runtime_exit()
        })?;
        self.initialized.set(false);
        self.python_home_guard.borrow_mut().take();
        Ok(())
    }

    pub fn decode_list(&self) -> Result<Vec<DecodeDecoder>, DecodeRuntimeError> {
        let mut raw_list: *mut RawDecodeListEntry = std::ptr::null_mut();
        let mut count = 0_usize;
        decode_native_call_status("decode_list", unsafe {
            dsview_decode_list(&mut raw_list, &mut count)
        })?;

        if raw_list.is_null() || count == 0 {
            return Ok(Vec::new());
        }

        let result = unsafe {
            let slice = std::slice::from_raw_parts(raw_list, count);
            slice
                .iter()
                .map(decode_list_entry_from_raw)
                .collect::<Result<Vec<_>, DecodeRuntimeError>>()
        };

        unsafe { dsview_decode_free_list(raw_list, count) };
        result
    }

    pub fn list_decoders(&self) -> Result<Vec<DecodeDecoder>, DecodeRuntimeError> {
        self.decode_list()
    }

    pub fn decode_inspect(&self, decoder_id: &str) -> Result<DecodeDecoder, DecodeRuntimeError> {
        let decoder_id = CString::new(decoder_id).map_err(|_| {
            DecodeRuntimeError::InvalidArgument(
                "decoder id must not contain interior NUL bytes".to_string(),
            )
        })?;
        let mut raw = RawDecodeMetadata {
            id: std::ptr::null_mut(),
            name: std::ptr::null_mut(),
            longname: std::ptr::null_mut(),
            desc: std::ptr::null_mut(),
            license: std::ptr::null_mut(),
            inputs: std::ptr::null_mut(),
            input_count: 0,
            outputs: std::ptr::null_mut(),
            output_count: 0,
            tags: std::ptr::null_mut(),
            tag_count: 0,
            required_channels: std::ptr::null_mut(),
            required_channel_count: 0,
            optional_channels: std::ptr::null_mut(),
            optional_channel_count: 0,
            options: std::ptr::null_mut(),
            option_count: 0,
            annotations: std::ptr::null_mut(),
            annotation_count: 0,
            annotation_rows: std::ptr::null_mut(),
            annotation_row_count: 0,
        };
        decode_native_call_status("decode inspect", unsafe {
            dsview_decode_inspect(decoder_id.as_ptr(), &mut raw)
        })?;

        let metadata = decode_metadata_from_raw(&raw);
        unsafe { dsview_decode_free_metadata(&mut raw) };
        metadata
    }

    pub fn inspect_decoder(&self, decoder_id: &str) -> Result<DecodeDecoder, DecodeRuntimeError> {
        self.decode_inspect(decoder_id)
    }
}

impl Drop for DecodeRuntimeBridge {
    fn drop(&mut self) {
        if self.initialized.get() {
            unsafe {
                let _ = dsview_decode_runtime_exit();
            };
        }
        self.python_home_guard.get_mut().take();
    }
}

#[derive(Debug)]
struct PythonHomeGuard {
    previous_home: Option<OsString>,
}

impl PythonHomeGuard {
    fn activate(path: &Path) -> Result<Self, DecodeRuntimeError> {
        if !path.is_dir() {
            return Err(DecodeRuntimeError::InvalidArgument(format!(
                "python home path does not exist: {}",
                path.display()
            )));
        }

        let previous_home = env::var_os("PYTHONHOME");
        unsafe {
            env::set_var("PYTHONHOME", path);
        }
        Ok(Self { previous_home })
    }
}

impl Drop for PythonHomeGuard {
    fn drop(&mut self) {
        unsafe {
            if let Some(previous_home) = &self.previous_home {
                env::set_var("PYTHONHOME", previous_home);
            } else {
                env::remove_var("PYTHONHOME");
            }
        }
    }
}

pub struct DecodeExecutionSession {
    raw: NonNull<RawDecodeExecutionSession>,
    next_abs_start: Option<u64>,
}

impl DecodeExecutionSession {
    pub fn new() -> Result<Self, DecodeRuntimeError> {
        let mut raw = std::ptr::null_mut();
        decode_native_call_status("decode session new", unsafe {
            dsview_decode_session_new(&mut raw)
        })?;
        let raw = NonNull::new(raw).ok_or_else(|| DecodeRuntimeError::NativeCall {
            operation: "decode session new",
            code: DecodeRuntimeErrorCode::SessionState,
            detail: "decode session bridge returned a null session".to_string(),
        })?;
        Ok(Self {
            raw,
            next_abs_start: None,
        })
    }

    pub fn set_samplerate_hz(&mut self, samplerate_hz: u64) -> Result<(), DecodeRuntimeError> {
        if samplerate_hz == 0 {
            return Err(DecodeRuntimeError::InvalidArgument(
                "decode session samplerate must be greater than zero".to_string(),
            ));
        }
        decode_native_call_status("decode session set samplerate", unsafe {
            dsview_decode_session_set_samplerate(self.raw.as_ptr(), samplerate_hz)
        })
    }

    pub fn build_linear_stack(
        &mut self,
        root: &DecodeSessionInstance,
        stack: &[DecodeSessionInstance],
    ) -> Result<(), DecodeRuntimeError> {
        let raw_root = OwnedRawDecodeInstanceSpec::new(root)?;
        let raw_stack = stack
            .iter()
            .map(OwnedRawDecodeInstanceSpec::new)
            .collect::<Result<Vec<_>, _>>()?;
        let raw_stack_specs = raw_stack
            .iter()
            .map(OwnedRawDecodeInstanceSpec::as_raw)
            .collect::<Vec<_>>();
        let stack_ptr = if raw_stack_specs.is_empty() {
            std::ptr::null()
        } else {
            raw_stack_specs.as_ptr()
        };
        decode_native_call_status("decode session build linear stack", unsafe {
            dsview_decode_session_build_linear_stack(
                self.raw.as_ptr(),
                &raw_root.as_raw(),
                stack_ptr,
                raw_stack_specs.len(),
            )
        })
    }

    pub fn start(&mut self) -> Result<(), DecodeRuntimeError> {
        self.next_abs_start = None;
        decode_native_call_status("decode session start", unsafe {
            dsview_decode_session_start(self.raw.as_ptr())
        })
    }

    pub fn end(&mut self) -> Result<(), DecodeRuntimeError> {
        decode_native_call_status("decode session end", unsafe {
            dsview_decode_session_end(self.raw.as_ptr())
        })
    }

    pub fn take_captured_annotations(
        &mut self,
    ) -> Result<Vec<DecodeCapturedAnnotation>, DecodeRuntimeError> {
        let mut raw_annotations: *mut RawDecodeCapturedAnnotation = std::ptr::null_mut();
        let mut count = 0_usize;
        decode_native_call_status("decode session take captured annotations", unsafe {
            dsview_decode_session_take_captured_annotations(
                self.raw.as_ptr(),
                &mut raw_annotations,
                &mut count,
            )
        })?;

        if raw_annotations.is_null() || count == 0 {
            return Ok(Vec::new());
        }

        let result = unsafe {
            let slice = std::slice::from_raw_parts(raw_annotations, count);
            slice
                .iter()
                .map(decode_captured_annotation_from_raw)
                .collect::<Result<Vec<_>, DecodeRuntimeError>>()
        };
        unsafe { dsview_decode_free_captured_annotations(raw_annotations, count) };
        result
    }
}

impl Drop for DecodeExecutionSession {
    fn drop(&mut self) {
        unsafe {
            dsview_decode_session_destroy(self.raw.as_ptr());
        }
    }
}

pub fn session_send_logic_chunk(
    session: &mut DecodeExecutionSession,
    abs_start_sample: u64,
    sample_bytes: &[u8],
    format: DecodeExecutionLogicFormat,
    logic_packet_lengths: Option<&[usize]>,
) -> Result<(), DecodeRuntimeError> {
    if sample_bytes.is_empty() {
        return Err(DecodeRuntimeError::InvalidArgument(
            "sample bytes must not be empty".to_string(),
        ));
    }

    let alignment = logic_chunk_alignment(format)?;
    validate_logic_packet_lengths(sample_bytes.len(), alignment, logic_packet_lengths)?;

    let expected_start = session.next_abs_start.unwrap_or(abs_start_sample);
    if abs_start_sample != expected_start {
        return Err(DecodeRuntimeError::InvalidArgument(
            "absolute sample progression must continue from the previous chunk end".to_string(),
        ));
    }

    let packet_lengths = logic_packet_lengths.unwrap_or(&[]);
    let mut cursor = abs_start_sample;

    if packet_lengths.is_empty() {
        send_raw_logic_chunk(session, cursor, sample_bytes, format)?;
        session.next_abs_start = Some(
            cursor
                .checked_add(logic_chunk_sample_count(sample_bytes.len(), format)?)
                .ok_or_else(|| {
                    DecodeRuntimeError::InvalidArgument(
                        "absolute sample progression overflowed u64".to_string(),
                    )
                })?,
        );
        return Ok(());
    }

    let mut offset = 0;
    for packet_len in packet_lengths {
        let packet = &sample_bytes[offset..offset + packet_len];
        let packet_samples = logic_chunk_sample_count(packet.len(), format)?;
        send_raw_logic_chunk(session, cursor, packet, format)?;
        cursor = cursor.checked_add(packet_samples).ok_or_else(|| {
            DecodeRuntimeError::InvalidArgument(
                "absolute sample progression overflowed u64".to_string(),
            )
        })?;
        offset += packet_len;
    }

    session.next_abs_start = Some(cursor);
    Ok(())
}

struct OwnedRawDecodeOptionEntry {
    option_id: CString,
    string_value: Option<CString>,
    integer_value: i64,
    float_value: f64,
    kind: c_int,
}

impl OwnedRawDecodeOptionEntry {
    fn new(option: &DecodeSessionOption) -> Result<Self, DecodeRuntimeError> {
        let option_id = CString::new(option.option_id.as_str()).map_err(|_| {
            DecodeRuntimeError::InvalidArgument(
                "decode option ids must not contain interior NUL bytes".to_string(),
            )
        })?;

        let (kind, string_value, integer_value, float_value) = match &option.value {
            DecodeSessionOptionValue::String(value) => (
                1,
                Some(CString::new(value.as_str()).map_err(|_| {
                    DecodeRuntimeError::InvalidArgument(
                        "decode option string values must not contain interior NUL bytes"
                            .to_string(),
                    )
                })?),
                0,
                0.0,
            ),
            DecodeSessionOptionValue::Integer(value) => (2, None, *value, 0.0),
            DecodeSessionOptionValue::Float(value) => (3, None, 0, *value),
        };

        Ok(Self {
            option_id,
            string_value,
            integer_value,
            float_value,
            kind,
        })
    }

    fn as_raw(&self) -> RawDecodeOptionEntry {
        RawDecodeOptionEntry {
            option_id: self.option_id.as_ptr(),
            value: RawDecodeOptionValue {
                kind: self.kind,
                string_value: self
                    .string_value
                    .as_ref()
                    .map_or(std::ptr::null(), |value| value.as_ptr()),
                integer_value: self.integer_value,
                float_value: self.float_value,
            },
        }
    }
}

struct OwnedRawDecodeChannelBinding {
    channel_id: CString,
    channel_index: u32,
}

impl OwnedRawDecodeChannelBinding {
    fn new(binding: &DecodeSessionChannelBinding) -> Result<Self, DecodeRuntimeError> {
        Ok(Self {
            channel_id: CString::new(binding.channel_id.as_str()).map_err(|_| {
                DecodeRuntimeError::InvalidArgument(
                    "decode channel ids must not contain interior NUL bytes".to_string(),
                )
            })?,
            channel_index: binding.channel_index,
        })
    }

    fn as_raw(&self) -> RawDecodeChannelBinding {
        RawDecodeChannelBinding {
            channel_id: self.channel_id.as_ptr(),
            channel_index: self.channel_index,
        }
    }
}

struct OwnedRawDecodeInstanceSpec {
    decoder_id: CString,
    channel_bindings: Vec<OwnedRawDecodeChannelBinding>,
    options: Vec<OwnedRawDecodeOptionEntry>,
    raw_channel_bindings: Vec<RawDecodeChannelBinding>,
    raw_options: Vec<RawDecodeOptionEntry>,
}

impl OwnedRawDecodeInstanceSpec {
    fn new(instance: &DecodeSessionInstance) -> Result<Self, DecodeRuntimeError> {
        Ok(Self {
            decoder_id: CString::new(instance.decoder_id.as_str()).map_err(|_| {
                DecodeRuntimeError::InvalidArgument(
                    "decoder ids must not contain interior NUL bytes".to_string(),
                )
            })?,
            channel_bindings: instance
                .channel_bindings
                .iter()
                .map(OwnedRawDecodeChannelBinding::new)
                .collect::<Result<Vec<_>, _>>()?,
            options: instance
                .options
                .iter()
                .map(OwnedRawDecodeOptionEntry::new)
                .collect::<Result<Vec<_>, _>>()?,
            raw_channel_bindings: Vec::new(),
            raw_options: Vec::new(),
        }
        .with_raw_views())
    }

    fn with_raw_views(mut self) -> Self {
        self.raw_channel_bindings = self
            .channel_bindings
            .iter()
            .map(OwnedRawDecodeChannelBinding::as_raw)
            .collect::<Vec<_>>();
        self.raw_options = self
            .options
            .iter()
            .map(OwnedRawDecodeOptionEntry::as_raw)
            .collect::<Vec<_>>();
        self
    }

    fn as_raw(&self) -> RawDecodeInstanceSpec {
        RawDecodeInstanceSpec {
            decoder_id: self.decoder_id.as_ptr(),
            channel_bindings: if self.raw_channel_bindings.is_empty() {
                std::ptr::null()
            } else {
                self.raw_channel_bindings.as_ptr()
            },
            channel_binding_count: self.raw_channel_bindings.len(),
            options: if self.raw_options.is_empty() {
                std::ptr::null()
            } else {
                self.raw_options.as_ptr()
            },
            option_count: self.raw_options.len(),
        }
    }
}

fn logic_chunk_alignment(format: DecodeExecutionLogicFormat) -> Result<usize, DecodeRuntimeError> {
    match format {
        DecodeExecutionLogicFormat::SplitLogic { unitsize } => {
            if unitsize == 0 {
                return Err(DecodeRuntimeError::InvalidArgument(
                    "split-logic unitsize must be greater than zero".to_string(),
                ));
            }
            Ok(unitsize as usize)
        }
        DecodeExecutionLogicFormat::CrossLogic { channel_count } => {
            if channel_count == 0 {
                return Err(DecodeRuntimeError::InvalidArgument(
                    "cross-logic channel_count must be greater than zero".to_string(),
                ));
            }
            Ok(channel_count as usize * std::mem::size_of::<u64>())
        }
    }
}

fn logic_chunk_sample_count(
    sample_byte_len: usize,
    format: DecodeExecutionLogicFormat,
) -> Result<u64, DecodeRuntimeError> {
    let alignment = logic_chunk_alignment(format)?;
    if sample_byte_len == 0 {
        return Err(DecodeRuntimeError::InvalidArgument(
            "sample bytes must not be empty".to_string(),
        ));
    }
    if sample_byte_len % alignment != 0 {
        return Err(DecodeRuntimeError::InvalidArgument(
            "sample bytes must be aligned to the requested logic format".to_string(),
        ));
    }

    let sample_count = match format {
        DecodeExecutionLogicFormat::SplitLogic { unitsize } => sample_byte_len / unitsize as usize,
        DecodeExecutionLogicFormat::CrossLogic { channel_count } => {
            (sample_byte_len / (channel_count as usize * std::mem::size_of::<u64>())) * 64
        }
    };
    Ok(sample_count as u64)
}

fn validate_logic_packet_lengths(
    sample_byte_len: usize,
    alignment: usize,
    logic_packet_lengths: Option<&[usize]>,
) -> Result<(), DecodeRuntimeError> {
    let Some(lengths) = logic_packet_lengths else {
        return Ok(());
    };
    if lengths.is_empty() {
        return Err(DecodeRuntimeError::InvalidArgument(
            "logic packet lengths must not be empty when packet boundaries are supplied"
                .to_string(),
        ));
    }

    let expected_total = lengths.iter().sum::<usize>();
    if expected_total != sample_byte_len {
        return Err(DecodeRuntimeError::InvalidArgument(
            "logic packet lengths must sum to the sample byte length".to_string(),
        ));
    }
    if lengths
        .iter()
        .any(|length| *length == 0 || (*length % alignment) != 0)
    {
        return Err(DecodeRuntimeError::InvalidArgument(
            "logic packet lengths must be non-zero and aligned to the requested logic format"
                .to_string(),
        ));
    }

    Ok(())
}

fn send_raw_logic_chunk(
    session: &mut DecodeExecutionSession,
    abs_start_sample: u64,
    sample_bytes: &[u8],
    format: DecodeExecutionLogicFormat,
) -> Result<(), DecodeRuntimeError> {
    let sample_count = logic_chunk_sample_count(sample_bytes.len(), format)?;
    let (format_code, unitsize, channel_count) = match format {
        DecodeExecutionLogicFormat::SplitLogic { unitsize } => (1, unitsize, 0),
        DecodeExecutionLogicFormat::CrossLogic { channel_count } => (2, 0, channel_count),
    };
    let chunk = RawDecodeLogicChunk {
        format: format_code,
        unitsize,
        channel_count,
        abs_start_sample,
        abs_end_sample: abs_start_sample.checked_add(sample_count).ok_or_else(|| {
            DecodeRuntimeError::InvalidArgument(
                "absolute sample progression overflowed u64".to_string(),
            )
        })?,
        sample_bytes: sample_bytes.as_ptr(),
        sample_bytes_len: sample_bytes.len(),
    };

    decode_native_call_status("decode session send logic chunk", unsafe {
        dsview_decode_session_send_logic_chunk(session.raw.as_ptr(), &chunk)
    })
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

/// Reports whether a source-built DSView decode runtime library is available.
pub fn source_decode_runtime_available() -> bool {
    cfg!(dsview_source_decode_runtime_available)
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

pub fn source_decode_runtime_library_path() -> Option<&'static Path> {
    option_env!("DSVIEW_SOURCE_DECODE_RUNTIME_LIBRARY").map(Path::new)
}

fn write_vcd_atomically(final_path: &Path, bytes: &[u8]) -> Result<(), RuntimeError> {
    let parent = final_path.parent().ok_or_else(|| {
        RuntimeError::InvalidArgument(format!(
            "final VCD path `{}` must have a parent directory",
            final_path.display()
        ))
    })?;
    fs::create_dir_all(parent).map_err(|error| RuntimeError::TempWrite {
        path: parent.to_path_buf(),
        detail: error.to_string(),
    })?;

    let temp_path = final_path.with_extension(format!(
        "{}.tmp",
        final_path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or("vcd")
    ));

    fs::write(&temp_path, bytes).map_err(|error| RuntimeError::TempWrite {
        path: temp_path.clone(),
        detail: error.to_string(),
    })?;

    if let Err(error) = fs::rename(&temp_path, final_path) {
        let _ = fs::remove_file(&temp_path);
        return Err(RuntimeError::TempPromote {
            from: temp_path,
            to: final_path.to_path_buf(),
            detail: error.to_string(),
        });
    }

    Ok(())
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

fn decode_last_loader_error() -> String {
    unsafe {
        let raw = dsview_decode_last_loader_error();
        if raw.is_null() {
            String::new()
        } else {
            CStr::from_ptr(raw).to_string_lossy().into_owned()
        }
    }
}

fn decode_last_error() -> String {
    unsafe {
        let raw = dsview_decode_last_error();
        if raw.is_null() {
            String::new()
        } else {
            CStr::from_ptr(raw).to_string_lossy().into_owned()
        }
    }
}

fn decode_last_error_name() -> String {
    unsafe {
        let raw = dsview_decode_last_error_name();
        if raw.is_null() {
            String::new()
        } else {
            CStr::from_ptr(raw).to_string_lossy().into_owned()
        }
    }
}

fn decode_native_call_status(
    operation: &'static str,
    status: i32,
) -> Result<(), DecodeRuntimeError> {
    match status {
        SR_OK => Ok(()),
        DSVIEW_BRIDGE_ERR_NOT_LOADED | DSVIEW_DECODE_ERR_NOT_LOADED => {
            Err(DecodeRuntimeError::BridgeNotLoaded)
        }
        DSVIEW_BRIDGE_ERR_ARG | DSVIEW_DECODE_ERR_ARG => {
            Err(DecodeRuntimeError::InvalidArgument(decode_last_error()))
        }
        other => Err(DecodeRuntimeError::NativeCall {
            operation,
            code: DecodeRuntimeErrorCode::from_raw(other),
            detail: if decode_last_error().is_empty() {
                decode_last_error_name()
            } else {
                let name = decode_last_error_name();
                let detail = decode_last_error();
                if name.is_empty() {
                    detail
                } else {
                    format!("{name}: {detail}")
                }
            },
        }),
    }
}

fn decode_optional_c_string(raw: *const c_char) -> Result<Option<String>, DecodeRuntimeError> {
    if raw.is_null() {
        return Ok(None);
    }

    Ok(Some(
        unsafe { CStr::from_ptr(raw) }
            .to_str()
            .map_err(|_| DecodeRuntimeError::InvalidUtf8)?
            .to_string(),
    ))
}

fn decode_required_c_string(raw: *const c_char) -> Result<String, DecodeRuntimeError> {
    decode_optional_c_string(raw)?.ok_or_else(|| {
        DecodeRuntimeError::InvalidArgument("expected non-null decode string".to_string())
    })
}

fn decode_string_array(
    raw: *const *mut c_char,
    count: usize,
) -> Result<Vec<String>, DecodeRuntimeError> {
    if raw.is_null() || count == 0 {
        return Ok(Vec::new());
    }

    unsafe { std::slice::from_raw_parts(raw, count) }
        .iter()
        .map(|value| decode_required_c_string(*value as *const c_char))
        .collect()
}

fn decode_inputs(
    raw: *const *mut c_char,
    count: usize,
) -> Result<Vec<DecodeInput>, DecodeRuntimeError> {
    decode_string_array(raw, count)
        .map(|values| values.into_iter().map(|id| DecodeInput { id }).collect())
}

fn decode_outputs(
    raw: *const *mut c_char,
    count: usize,
) -> Result<Vec<DecodeOutput>, DecodeRuntimeError> {
    decode_string_array(raw, count)
        .map(|values| values.into_iter().map(|id| DecodeOutput { id }).collect())
}

fn decode_channel_from_raw(raw: &RawDecodeChannel) -> Result<DecodeChannel, DecodeRuntimeError> {
    Ok(DecodeChannel {
        id: decode_required_c_string(raw.id.cast_const())?,
        name: decode_required_c_string(raw.name.cast_const())?,
        description: decode_required_c_string(raw.desc.cast_const())?,
        order: raw.order,
        channel_type: raw.channel_type,
        idn: decode_optional_c_string(raw.idn.cast_const())?,
    })
}

fn decode_option_from_raw(raw: &RawDecodeOption) -> Result<DecodeOption, DecodeRuntimeError> {
    Ok(DecodeOption {
        id: decode_required_c_string(raw.id.cast_const())?,
        idn: decode_optional_c_string(raw.idn.cast_const())?,
        description: decode_optional_c_string(raw.desc.cast_const())?,
        value_kind: decode_option_value_kind_from_raw(raw.value_kind),
        default_value: decode_optional_c_string(raw.default_value.cast_const())?,
        values: decode_string_array(raw.values.cast_const(), raw.value_count)?,
    })
}

fn decode_option_value_kind_from_raw(raw: c_int) -> DecodeOptionValueKind {
    match raw {
        1 => DecodeOptionValueKind::String,
        2 => DecodeOptionValueKind::Integer,
        3 => DecodeOptionValueKind::Float,
        _ => DecodeOptionValueKind::Unknown,
    }
}

fn decode_annotation_from_raw(
    raw: &RawDecodeAnnotation,
) -> Result<DecodeAnnotation, DecodeRuntimeError> {
    Ok(DecodeAnnotation {
        id: decode_required_c_string(raw.id.cast_const())?,
        label: decode_optional_c_string(raw.label.cast_const())?,
        description: decode_optional_c_string(raw.description.cast_const())?,
        annotation_type: raw.annotation_type,
    })
}

fn decode_annotation_row_from_raw(
    raw: &RawDecodeAnnotationRow,
) -> Result<DecodeAnnotationRow, DecodeRuntimeError> {
    let annotation_classes = if raw.annotation_classes.is_null() || raw.annotation_class_count == 0
    {
        Vec::new()
    } else {
        unsafe { std::slice::from_raw_parts(raw.annotation_classes, raw.annotation_class_count) }
            .to_vec()
    };

    Ok(DecodeAnnotationRow {
        id: decode_required_c_string(raw.id.cast_const())?,
        description: decode_optional_c_string(raw.desc.cast_const())?,
        annotation_classes,
    })
}

fn decode_captured_annotation_from_raw(
    raw: &RawDecodeCapturedAnnotation,
) -> Result<DecodeCapturedAnnotation, DecodeRuntimeError> {
    let texts = if raw.texts.is_null() || raw.text_count == 0 {
        Vec::new()
    } else {
        unsafe { std::slice::from_raw_parts(raw.texts, raw.text_count) }
            .iter()
            .map(|text| {
                decode_optional_c_string((*text).cast_const())
                    .map(|value| value.unwrap_or_default())
            })
            .collect::<Result<Vec<_>, _>>()?
    };

    Ok(DecodeCapturedAnnotation {
        decoder_id: decode_required_c_string(raw.decoder_id.cast_const())?,
        start_sample: raw.start_sample,
        end_sample: raw.end_sample,
        annotation_class: raw.ann_class,
        annotation_type: raw.ann_type,
        texts,
        number_hex: decode_optional_c_string(raw.number_hex.cast_const())?,
        numeric_value: if raw.has_numeric_value != 0 {
            Some(raw.numeric_value)
        } else {
            None
        },
    })
}

fn decode_list_entry_from_raw(
    raw: &RawDecodeListEntry,
) -> Result<DecodeDecoder, DecodeRuntimeError> {
    Ok(DecodeDecoder {
        id: decode_required_c_string(raw.id.cast_const())?,
        name: decode_required_c_string(raw.name.cast_const())?,
        longname: decode_required_c_string(raw.longname.cast_const())?,
        description: decode_required_c_string(raw.desc.cast_const())?,
        license: decode_required_c_string(raw.license.cast_const())?,
        inputs: Vec::new(),
        outputs: Vec::new(),
        tags: Vec::new(),
        required_channels: Vec::new(),
        optional_channels: Vec::new(),
        options: Vec::new(),
        annotations: Vec::new(),
        annotation_rows: Vec::new(),
    })
}

fn decode_metadata_from_raw(raw: &RawDecodeMetadata) -> Result<DecodeDecoder, DecodeRuntimeError> {
    let required_channels = if raw.required_channels.is_null() || raw.required_channel_count == 0 {
        Vec::new()
    } else {
        unsafe {
            std::slice::from_raw_parts(raw.required_channels, raw.required_channel_count)
                .iter()
                .map(decode_channel_from_raw)
                .collect::<Result<Vec<_>, _>>()?
        }
    };
    let optional_channels = if raw.optional_channels.is_null() || raw.optional_channel_count == 0 {
        Vec::new()
    } else {
        unsafe {
            std::slice::from_raw_parts(raw.optional_channels, raw.optional_channel_count)
                .iter()
                .map(decode_channel_from_raw)
                .collect::<Result<Vec<_>, _>>()?
        }
    };
    let options = if raw.options.is_null() || raw.option_count == 0 {
        Vec::new()
    } else {
        unsafe {
            std::slice::from_raw_parts(raw.options, raw.option_count)
                .iter()
                .map(decode_option_from_raw)
                .collect::<Result<Vec<_>, _>>()?
        }
    };
    let annotations = if raw.annotations.is_null() || raw.annotation_count == 0 {
        Vec::new()
    } else {
        unsafe {
            std::slice::from_raw_parts(raw.annotations, raw.annotation_count)
                .iter()
                .map(decode_annotation_from_raw)
                .collect::<Result<Vec<_>, _>>()?
        }
    };
    let annotation_rows = if raw.annotation_rows.is_null() || raw.annotation_row_count == 0 {
        Vec::new()
    } else {
        unsafe {
            std::slice::from_raw_parts(raw.annotation_rows, raw.annotation_row_count)
                .iter()
                .map(decode_annotation_row_from_raw)
                .collect::<Result<Vec<_>, _>>()?
        }
    };

    Ok(DecodeDecoder {
        id: decode_required_c_string(raw.id.cast_const())?,
        name: decode_required_c_string(raw.name.cast_const())?,
        longname: decode_required_c_string(raw.longname.cast_const())?,
        description: decode_required_c_string(raw.desc.cast_const())?,
        license: decode_required_c_string(raw.license.cast_const())?,
        inputs: decode_inputs(raw.inputs.cast_const(), raw.input_count)?,
        outputs: decode_outputs(raw.outputs.cast_const(), raw.output_count)?,
        tags: decode_string_array(raw.tags.cast_const(), raw.tag_count)?,
        required_channels,
        optional_channels,
        options,
        annotations,
        annotation_rows,
    })
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

fn get_optional_i32_config(
    operation: &'static str,
    getter: unsafe extern "C" fn(*mut c_int) -> c_int,
) -> Result<Option<i32>, RuntimeError> {
    match get_i32_config(operation, getter) {
        Ok(value) => Ok(Some(value)),
        Err(RuntimeError::NativeCall {
            operation: _,
            code: NativeErrorCode::NotApplicable,
        }) => Ok(None),
        Err(error) => Err(error),
    }
}

fn get_f64_config(
    operation: &'static str,
    getter: unsafe extern "C" fn(*mut f64) -> c_int,
) -> Result<f64, RuntimeError> {
    let mut value = 0.0;
    native_call_status(operation, unsafe { getter(&mut value) })?;
    Ok(value)
}

fn get_optional_f64_config(
    operation: &'static str,
    getter: unsafe extern "C" fn(*mut f64) -> c_int,
) -> Result<Option<f64>, RuntimeError> {
    match get_f64_config(operation, getter) {
        Ok(value) => Ok(Some(value)),
        Err(RuntimeError::NativeCall {
            operation: _,
            code: NativeErrorCode::NotApplicable,
        }) => Ok(None),
        Err(error) => Err(error),
    }
}

fn get_optional_u64_config(
    operation: &'static str,
    getter: unsafe extern "C" fn(*mut u64) -> c_int,
) -> Result<Option<u64>, RuntimeError> {
    match get_u64_config(operation, getter) {
        Ok(value) => Ok(Some(value)),
        Err(RuntimeError::NativeCall {
            operation: _,
            code: NativeErrorCode::NotApplicable,
        }) => Ok(None),
        Err(error) => Err(error),
    }
}

fn decode_device_options_snapshot(
    raw: &RawDeviceOptionsSnapshot,
) -> Result<DeviceOptionsSnapshot, RuntimeError> {
    Ok(DeviceOptionsSnapshot {
        current_operation_mode_code: decode_optional_code(
            raw.has_current_operation_mode,
            raw.current_operation_mode_code,
        ),
        operation_modes: decode_option_values(
            &raw.operation_modes,
            raw.operation_mode_count as usize,
        )?,
        current_stop_option_code: decode_optional_code(
            raw.has_current_stop_option,
            raw.current_stop_option_code,
        ),
        stop_options: decode_option_values(&raw.stop_options, raw.stop_option_count as usize)?,
        current_filter_code: decode_optional_code(raw.has_current_filter, raw.current_filter_code),
        filters: decode_option_values(&raw.filters, raw.filter_count as usize)?,
        current_channel_mode_code: decode_optional_code(
            raw.has_current_channel_mode,
            raw.current_channel_mode_code,
        ),
        channel_mode_groups: decode_channel_mode_groups(
            &raw.channel_mode_groups,
            raw.channel_mode_group_count as usize,
        )?,
        threshold: decode_threshold_range(&raw.threshold)?,
    })
}

fn decode_device_option_validation_snapshot(
    raw: &RawDeviceOptionValidationSnapshot,
) -> Result<DeviceOptionValidationSnapshot, RuntimeError> {
    Ok(DeviceOptionValidationSnapshot {
        current_operation_mode_code: decode_optional_code(
            raw.has_current_operation_mode,
            raw.current_operation_mode_code,
        ),
        current_stop_option_code: decode_optional_code(
            raw.has_current_stop_option,
            raw.current_stop_option_code,
        ),
        current_filter_code: decode_optional_code(raw.has_current_filter, raw.current_filter_code),
        current_channel_mode_code: decode_optional_code(
            raw.has_current_channel_mode,
            raw.current_channel_mode_code,
        ),
        total_channel_count: raw.total_channel_count,
        hardware_sample_capacity: raw.hardware_sample_capacity,
        filters: decode_option_values(&raw.filters, raw.filter_count as usize)?,
        threshold: decode_threshold_range(&raw.threshold)?,
        operation_modes: decode_validation_operation_modes(
            &raw.operation_modes,
            raw.operation_mode_count as usize,
        )?,
    })
}

fn decode_option_values(
    raw: &[RawOptionValue; OPTION_VALUE_CAPACITY],
    count: usize,
) -> Result<Vec<DeviceOptionValue>, RuntimeError> {
    raw[..count.min(raw.len())]
        .iter()
        .map(|item| {
            Ok(DeviceOptionValue {
                code: item.code as i16,
                label: decode_fixed_string(&item.label)?,
            })
        })
        .collect()
}

fn decode_channel_mode_groups(
    raw: &[RawChannelModeGroup; CHANNEL_MODE_GROUP_CAPACITY],
    count: usize,
) -> Result<Vec<DeviceOptionChannelModeGroup>, RuntimeError> {
    raw[..count.min(raw.len())]
        .iter()
        .map(|group| {
            Ok(DeviceOptionChannelModeGroup {
                operation_mode_code: group.operation_mode_code as i16,
                channel_modes: group.channel_modes
                    [..(group.channel_mode_count as usize).min(group.channel_modes.len())]
                    .iter()
                    .map(|mode| {
                        Ok(DeviceOptionChannelMode {
                            code: mode.id as i16,
                            label: decode_fixed_string(&mode.name)?,
                            max_enabled_channels: mode.max_enabled_channels,
                        })
                    })
                    .collect::<Result<Vec<_>, RuntimeError>>()?,
            })
        })
        .collect()
}

fn decode_validation_operation_modes(
    raw: &[RawValidationOperationMode; CHANNEL_MODE_GROUP_CAPACITY],
    count: usize,
) -> Result<Vec<DeviceOptionValidationOperationMode>, RuntimeError> {
    raw[..count.min(raw.len())]
        .iter()
        .map(|operation_mode| {
            Ok(DeviceOptionValidationOperationMode {
                code: operation_mode.code as i16,
                label: decode_fixed_string(&operation_mode.label)?,
                stop_options: decode_option_values(
                    &operation_mode.stop_options,
                    operation_mode.stop_option_count as usize,
                )?,
                channel_modes: decode_validation_channel_modes(
                    &operation_mode.channel_modes,
                    operation_mode.channel_mode_count as usize,
                )?,
            })
        })
        .collect()
}

fn decode_validation_channel_modes(
    raw: &[RawValidationChannelMode; CHANNEL_MODE_CAPACITY],
    count: usize,
) -> Result<Vec<DeviceOptionValidationChannelMode>, RuntimeError> {
    raw[..count.min(raw.len())]
        .iter()
        .map(|channel_mode| {
            Ok(DeviceOptionValidationChannelMode {
                code: channel_mode.code as i16,
                label: decode_fixed_string(&channel_mode.label)?,
                max_enabled_channels: channel_mode.max_enabled_channels,
                supported_sample_rates: channel_mode.samplerates[..(channel_mode.samplerate_count
                    as usize)
                    .min(channel_mode.samplerates.len())]
                    .to_vec(),
            })
        })
        .collect()
}

fn decode_threshold_range(raw: &RawThresholdRange) -> Result<ThresholdVoltageRange, RuntimeError> {
    let legacy_options =
        decode_option_values(&raw.legacy_options, raw.legacy_option_count as usize)?;

    Ok(ThresholdVoltageRange {
        kind: decode_fixed_string(&raw.kind)?,
        id: decode_fixed_string(&raw.id)?,
        current_volts: if raw.has_current_volts != 0 {
            Some(raw.current_volts)
        } else {
            None
        },
        min_volts: raw.min_volts,
        max_volts: raw.max_volts,
        step_volts: raw.step_volts,
        legacy: if legacy_options.is_empty() && raw.has_current_legacy_code == 0 {
            None
        } else {
            Some(LegacyThresholdMetadata {
                current_code: decode_optional_code(
                    raw.has_current_legacy_code,
                    raw.current_legacy_code,
                ),
                options: legacy_options,
            })
        },
    })
}

fn decode_fixed_string<const N: usize>(bytes: &[u8; N]) -> Result<String, RuntimeError> {
    let nul = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    std::str::from_utf8(&bytes[..nul])
        .map(|value| value.to_string())
        .map_err(|_| RuntimeError::InvalidDeviceName)
}

fn decode_optional_code(flag: c_int, value: i32) -> Option<i16> {
    (flag != 0).then_some(value as i16)
}

fn path_to_cstring(path: &Path) -> Result<CString, RuntimeError> {
    CString::new(path.as_os_str().to_string_lossy().as_bytes()).map_err(|_| {
        RuntimeError::PathContainsNul {
            path: path.to_path_buf(),
        }
    })
}

fn path_to_decode_cstring(path: &Path) -> Result<CString, DecodeRuntimeError> {
    CString::new(path.as_os_str().to_string_lossy().as_bytes()).map_err(|_| {
        DecodeRuntimeError::PathContainsNul {
            path: path.to_path_buf(),
        }
    })
}

fn raw_vcd_export_request(request: &VcdExportRequest) -> Result<RawVcdExportRequest, RuntimeError> {
    if request.samplerate_hz == 0 {
        return Err(RuntimeError::InvalidArgument(
            "samplerate must be greater than zero".to_string(),
        ));
    }
    if request.enabled_channels.is_empty() {
        return Err(RuntimeError::InvalidArgument(
            "at least one enabled channel is required for VCD export".to_string(),
        ));
    }

    Ok(RawVcdExportRequest {
        samplerate_hz: request.samplerate_hz,
        enabled_channels: request.enabled_channels.as_ptr(),
        enabled_channel_count: request.enabled_channels.len(),
    })
}

fn export_from_raw(raw: RawExportBuffer) -> Result<VcdExport, RuntimeError> {
    if raw.data.is_null() {
        return Ok(VcdExport {
            bytes: Vec::new(),
            sample_count: raw.sample_count,
            packet_count: raw.packet_count,
        });
    }

    let bytes = unsafe {
        let slice = std::slice::from_raw_parts(raw.data, raw.len);
        let owned = slice.to_vec();
        let mut raw = raw;
        dsview_bridge_free_export_buffer(&mut raw);
        normalize_vcd_timestamp_lines(owned)
    };

    Ok(VcdExport {
        bytes,
        sample_count: raw.sample_count,
        packet_count: raw.packet_count,
    })
}

fn normalize_vcd_timestamp_lines(bytes: Vec<u8>) -> Vec<u8> {
    let Ok(vcd) = String::from_utf8(bytes.clone()) else {
        return bytes;
    };

    let mut normalized = String::with_capacity(vcd.len());
    for line in vcd.lines() {
        if let Some((timestamp, values)) = line.split_once(' ') {
            if timestamp.starts_with('#') && !values.is_empty() {
                normalized.push_str(timestamp);
                normalized.push('\n');
                for value in values.split_whitespace() {
                    normalized.push_str(value);
                    normalized.push('\n');
                }
                continue;
            }
        }
        normalized.push_str(line);
        normalized.push('\n');
    }

    normalized.into_bytes()
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

fn export_call_status(operation: &'static str, status: i32) -> Result<(), RuntimeError> {
    if status == SR_OK {
        Ok(())
    } else if status == DSVIEW_BRIDGE_ERR_NOT_LOADED {
        Err(RuntimeError::BridgeNotLoaded)
    } else if status == DSVIEW_BRIDGE_ERR_ARG {
        Err(RuntimeError::InvalidArgument(format!(
            "bridge rejected export request for {operation}"
        )))
    } else {
        Err(RuntimeError::ExportCall {
            operation,
            code: ExportErrorCode::from_raw(status),
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
        assert_eq!(
            summary.terminal_event,
            AcquisitionTerminalEvent::EndByDetached
        );
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
    fn export_error_codes_map_expected_values() {
        assert_eq!(
            ExportErrorCode::from_raw(DSVIEW_EXPORT_ERR_OVERFLOW),
            ExportErrorCode::Overflow
        );
        assert_eq!(
            ExportErrorCode::from_raw(DSVIEW_EXPORT_ERR_OUTPUT_MODULE),
            ExportErrorCode::OutputModuleUnavailable
        );
    }

    #[test]
    fn raw_vcd_export_request_rejects_missing_inputs() {
        let missing_samplerate = raw_vcd_export_request(&VcdExportRequest {
            samplerate_hz: 0,
            enabled_channels: vec![0],
        })
        .unwrap_err();
        assert!(matches!(
            missing_samplerate,
            RuntimeError::InvalidArgument(_)
        ));

        let missing_channels = raw_vcd_export_request(&VcdExportRequest {
            samplerate_hz: 1,
            enabled_channels: Vec::new(),
        })
        .unwrap_err();
        assert!(matches!(missing_channels, RuntimeError::InvalidArgument(_)));
    }

    #[test]
    fn raw_vcd_export_request_preserves_multichannel_packed_shape() {
        let request = VcdExportRequest {
            samplerate_hz: 100_000_000,
            enabled_channels: vec![0, 1, 2, 3, 8, 9, 10, 11, 12],
        };

        let raw = raw_vcd_export_request(&request).unwrap();
        assert_eq!(raw.samplerate_hz, 100_000_000);
        assert_eq!(raw.enabled_channel_count, 9);

        let channels =
            unsafe { std::slice::from_raw_parts(raw.enabled_channels, raw.enabled_channel_count) };
        assert_eq!(channels, &[0, 1, 2, 3, 8, 9, 10, 11, 12]);

        let packed_unitsize = request.enabled_channels.len().div_ceil(8) as u16;
        assert_eq!(packed_unitsize, 2);
    }

    #[test]
    fn export_from_raw_copies_owned_bytes() {
        unsafe extern "C" {
            fn malloc(size: usize) -> *mut std::ffi::c_void;
        }

        let raw = RawExportBuffer {
            data: unsafe { malloc(4) }.cast(),
            len: 4,
            sample_count: 2,
            packet_count: 0,
        };
        unsafe {
            std::ptr::copy_nonoverlapping(b"vcd\n".as_ptr(), raw.data, 4);
        }

        let export = export_from_raw(raw).unwrap();
        assert_eq!(export.bytes, b"vcd\n");
        assert_eq!(export.sample_count, 2);
        assert_eq!(export.packet_count, 0);
    }

    #[test]
    fn write_vcd_atomically_promotes_temp_file() {
        let dir = std::env::temp_dir().join(format!(
            "dsview-sys-vcd-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let final_path = dir.join("capture.vcd");
        let temp_path = dir.join("capture.vcd.tmp");

        write_vcd_atomically(&final_path, b"$date\n$end\n").unwrap();

        assert!(final_path.is_file());
        assert!(!temp_path.exists());
        assert_eq!(std::fs::read(&final_path).unwrap(), b"$date\n$end\n");
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
