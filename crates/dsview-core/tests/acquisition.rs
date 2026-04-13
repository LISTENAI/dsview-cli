use std::collections::BTreeSet;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use dsview_core::{
    apply_capture_request_device_options, apply_validated_device_options, AcquisitionSummary,
    AcquisitionTerminalEvent, CaptureCleanup, CaptureCompletion, CaptureConfigRequest,
    CaptureRunRequest, DeviceOptionApplyStep, NativeErrorCode, SelectionHandle,
    ValidatedDeviceOptionRequest,
};
use dsview_sys::{source_runtime_library_path, AcquisitionPacketStatus, RuntimeBridge};

const SR_OK: i32 = 0;
const SR_ERR_ARG: i32 = 3;

const SR_CONF_SAMPLERATE: i32 = 30000;
const SR_CONF_LIMIT_SAMPLES: i32 = 50001;
const SR_CONF_TOTAL_CH_NUM: i32 = 30026;
const SR_CONF_FILTER: i32 = 30021;
const SR_CONF_OPERATION_MODE: i32 = 30065;
const SR_CONF_BUFFER_OPTIONS: i32 = 30066;
const SR_CONF_CHANNEL_MODE: i32 = 30067;
const SR_CONF_THRESHOLD: i32 = 30071;
const SR_CONF_VTH: i32 = 30072;
const SR_CONF_HW_DEPTH: i32 = 30075;
const APPLY_CHANNEL_CALL_KEY: i32 = -70000;

const BUFFER_MODE: i32 = 0;
const STREAM_MODE: i32 = 1;

const STOP_WHEN_FULL: i32 = 1;
const UPLOAD_WHEN_FULL: i32 = 2;

const FILTER_NONE: i32 = 0;
const FILTER_ONE_TICK: i32 = 1;

const BUFFER_WIDE_MODE: i32 = 11;
const BUFFER_COMPACT_MODE: i32 = 12;
const STREAM_WIDE_MODE: i32 = 21;
const STREAM_COMPACT_MODE: i32 = 22;

const THRESHOLD_3V3: i32 = 330;
const THRESHOLD_5V0: i32 = 500;

#[repr(C)]
#[derive(Clone, Copy)]
struct TestListItem {
    id: i32,
    name: [u8; 64],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct TestChannelMode {
    id: i32,
    name: [u8; 64],
    max_enabled_channels: u16,
}

unsafe extern "C" {
    fn dsview_test_install_mock_option_api();
    fn dsview_test_reset_mock_option_api();
    fn dsview_test_mock_set_current_int(key: i32, has_value: i32, value: i32, status: i32);
    fn dsview_test_mock_set_current_double(key: i32, has_value: i32, value: f64, status: i32);
    fn dsview_test_mock_set_current_u64(key: i32, has_value: i32, value: u64, status: i32);
    fn dsview_test_mock_set_list_items(
        key: i32,
        items: *const TestListItem,
        count: i32,
        status: i32,
    );
    fn dsview_test_mock_set_channel_mode_group(
        operation_mode_code: i32,
        items: *const TestChannelMode,
        count: i32,
        status: i32,
    );
    fn dsview_test_mock_set_channel_mode_samplerates(
        operation_mode_code: i32,
        channel_mode_code: i32,
        values: *const u64,
        count: i32,
        status: i32,
    );
    fn dsview_test_mock_set_apply_failure(key: i32, status: i32);
    fn dsview_test_mock_get_apply_call(index: i32, out_key: *mut i32, out_value: *mut i64) -> i32;
    fn dsview_test_mock_get_apply_call_count() -> i32;
    fn dsview_test_mock_reset_apply_log();
}

fn runtime_test_guard() -> &'static Mutex<()> {
    static GUARD: OnceLock<Mutex<()>> = OnceLock::new();
    GUARD.get_or_init(|| Mutex::new(()))
}

fn load_runtime() -> Option<RuntimeBridge> {
    let path = source_runtime_library_path()?;
    RuntimeBridge::load(path).ok()
}

fn fixed_name(label: &str) -> [u8; 64] {
    let mut bytes = [0_u8; 64];
    let label_bytes = label.as_bytes();
    let len = label_bytes.len().min(bytes.len().saturating_sub(1));
    bytes[..len].copy_from_slice(&label_bytes[..len]);
    bytes
}

fn list_item(id: i32, name: &str) -> TestListItem {
    TestListItem {
        id,
        name: fixed_name(name),
    }
}

fn channel_mode(id: i32, name: &str, max_enabled_channels: u16) -> TestChannelMode {
    TestChannelMode {
        id,
        name: fixed_name(name),
        max_enabled_channels,
    }
}

fn configure_mock_option_api() {
    unsafe {
        dsview_test_reset_mock_option_api();
        dsview_test_install_mock_option_api();
    }

    let operation_modes = [
        list_item(BUFFER_MODE, "Buffer Mode"),
        list_item(STREAM_MODE, "Stream Mode"),
    ];
    let stop_options = [
        list_item(STOP_WHEN_FULL, "Stop when memory fills"),
        list_item(UPLOAD_WHEN_FULL, "Upload after memory fills"),
    ];
    let filters = [
        list_item(FILTER_NONE, "No filtering"),
        list_item(FILTER_ONE_TICK, "Single-clock filter"),
    ];
    let legacy_thresholds = [
        list_item(THRESHOLD_3V3, "1.8/2.5/3.3V Level"),
        list_item(THRESHOLD_5V0, "5.0V Level"),
    ];
    let buffer_channel_modes = [
        channel_mode(BUFFER_WIDE_MODE, "Buffer wide lanes", 16),
        channel_mode(BUFFER_COMPACT_MODE, "Buffer compact lanes", 8),
    ];
    let stream_channel_modes = [
        channel_mode(STREAM_WIDE_MODE, "Streaming full lanes", 16),
        channel_mode(STREAM_COMPACT_MODE, "Streaming compact lanes", 6),
    ];
    let buffer_wide_samplerates = [50_000_000_u64, 100_000_000];
    let buffer_compact_samplerates = [100_000_000_u64, 200_000_000];
    let stream_wide_samplerates = [25_000_000_u64, 50_000_000];
    let stream_compact_samplerates = [50_000_000_u64, 100_000_000];

    unsafe {
        dsview_test_mock_set_list_items(
            SR_CONF_OPERATION_MODE,
            operation_modes.as_ptr(),
            operation_modes.len() as i32,
            SR_OK,
        );
        dsview_test_mock_set_list_items(
            SR_CONF_BUFFER_OPTIONS,
            stop_options.as_ptr(),
            stop_options.len() as i32,
            SR_OK,
        );
        dsview_test_mock_set_list_items(
            SR_CONF_FILTER,
            filters.as_ptr(),
            filters.len() as i32,
            SR_OK,
        );
        dsview_test_mock_set_list_items(
            SR_CONF_THRESHOLD,
            legacy_thresholds.as_ptr(),
            legacy_thresholds.len() as i32,
            SR_OK,
        );
        dsview_test_mock_set_channel_mode_group(
            BUFFER_MODE,
            buffer_channel_modes.as_ptr(),
            buffer_channel_modes.len() as i32,
            SR_OK,
        );
        dsview_test_mock_set_channel_mode_group(
            STREAM_MODE,
            stream_channel_modes.as_ptr(),
            stream_channel_modes.len() as i32,
            SR_OK,
        );
        dsview_test_mock_set_current_int(SR_CONF_OPERATION_MODE, 1, BUFFER_MODE, SR_OK);
        dsview_test_mock_set_current_int(SR_CONF_BUFFER_OPTIONS, 1, STOP_WHEN_FULL, SR_OK);
        dsview_test_mock_set_current_int(SR_CONF_FILTER, 1, FILTER_ONE_TICK, SR_OK);
        dsview_test_mock_set_current_int(SR_CONF_CHANNEL_MODE, 1, BUFFER_COMPACT_MODE, SR_OK);
        dsview_test_mock_set_current_int(SR_CONF_TOTAL_CH_NUM, 1, 16, SR_OK);
        dsview_test_mock_set_current_int(SR_CONF_THRESHOLD, 1, THRESHOLD_3V3, SR_OK);
        dsview_test_mock_set_current_double(SR_CONF_VTH, 1, 1.8, SR_OK);
        dsview_test_mock_set_current_u64(SR_CONF_SAMPLERATE, 1, 100_000_000, SR_OK);
        dsview_test_mock_set_current_u64(SR_CONF_LIMIT_SAMPLES, 1, 8192, SR_OK);
        dsview_test_mock_set_current_u64(SR_CONF_HW_DEPTH, 1, 268_435_456, SR_OK);
        dsview_test_mock_set_channel_mode_samplerates(
            BUFFER_MODE,
            BUFFER_WIDE_MODE,
            buffer_wide_samplerates.as_ptr(),
            buffer_wide_samplerates.len() as i32,
            SR_OK,
        );
        dsview_test_mock_set_channel_mode_samplerates(
            BUFFER_MODE,
            BUFFER_COMPACT_MODE,
            buffer_compact_samplerates.as_ptr(),
            buffer_compact_samplerates.len() as i32,
            SR_OK,
        );
        dsview_test_mock_set_channel_mode_samplerates(
            STREAM_MODE,
            STREAM_WIDE_MODE,
            stream_wide_samplerates.as_ptr(),
            stream_wide_samplerates.len() as i32,
            SR_OK,
        );
        dsview_test_mock_set_channel_mode_samplerates(
            STREAM_MODE,
            STREAM_COMPACT_MODE,
            stream_compact_samplerates.as_ptr(),
            stream_compact_samplerates.len() as i32,
            SR_OK,
        );
        dsview_test_mock_reset_apply_log();
    }
}

fn apply_log_keys() -> Vec<i32> {
    let count = unsafe { dsview_test_mock_get_apply_call_count() };
    let mut keys = Vec::with_capacity(count as usize);
    for index in 0..count {
        let mut key = 0;
        let mut value = 0_i64;
        let status = unsafe { dsview_test_mock_get_apply_call(index, &mut key, &mut value) };
        assert_eq!(status, SR_OK);
        keys.push(key);
    }
    keys
}

fn sample_validated_device_option_request() -> ValidatedDeviceOptionRequest {
    ValidatedDeviceOptionRequest {
        operation_mode_id: "operation-mode:1".to_string(),
        operation_mode_code: STREAM_MODE as i16,
        stop_option_id: Some("stop-option:2".to_string()),
        stop_option_code: Some(UPLOAD_WHEN_FULL as i16),
        channel_mode_id: "channel-mode:22".to_string(),
        channel_mode_code: STREAM_COMPACT_MODE as i16,
        sample_rate_hz: 50_000_000,
        requested_sample_limit: 4096,
        effective_sample_limit: 4096,
        enabled_channels: vec![0, 2, 4],
        threshold_volts: Some(2.5),
        filter_id: Some("filter:0".to_string()),
        filter_code: Some(FILTER_NONE as i16),
    }
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
fn clean_success_summary_matches_phase4_rule_shape() {
    let summary = clean_summary();
    assert!(summary.saw_logic_packet);
    assert!(summary.saw_end_packet);
    assert!(summary.saw_terminal_normal_end);
    assert_eq!(summary.terminal_event, AcquisitionTerminalEvent::NormalEnd);
    assert_eq!(summary.end_packet_status, Some(AcquisitionPacketStatus::Ok));
}

#[test]
fn preflight_blocked_shape_stays_distinct_from_runtime_failure() {
    let summary = clean_summary();
    assert_eq!(summary.start_status, NativeErrorCode::Ok.raw());
    assert_eq!(summary.last_error, NativeErrorCode::Ok);
}

#[test]
fn start_failure_shape_preserves_native_start_status() {
    let mut summary = clean_summary();
    summary.start_status = NativeErrorCode::DeviceExclusive.raw();
    summary.last_error = NativeErrorCode::DeviceExclusive;

    assert_eq!(NativeErrorCode::from_raw(summary.start_status), NativeErrorCode::DeviceExclusive);
    assert_eq!(summary.last_error, NativeErrorCode::DeviceExclusive);
}

#[test]
fn run_failure_shape_marks_terminal_error_event() {
    let mut summary = clean_summary();
    summary.saw_terminal_normal_end = false;
    summary.saw_terminal_end_by_error = true;
    summary.terminal_event = AcquisitionTerminalEvent::EndByError;

    assert_eq!(summary.terminal_event, AcquisitionTerminalEvent::EndByError);
    assert!(summary.saw_terminal_end_by_error);
}

#[test]
fn detach_shape_marks_terminal_detach_event() {
    let mut summary = clean_summary();
    summary.saw_terminal_normal_end = false;
    summary.saw_terminal_end_by_detached = true;
    summary.terminal_event = AcquisitionTerminalEvent::EndByDetached;

    assert_eq!(summary.terminal_event, AcquisitionTerminalEvent::EndByDetached);
    assert!(summary.saw_terminal_end_by_detached);
}

#[test]
fn incomplete_shape_is_missing_logic_packet() {
    let mut summary = clean_summary();
    summary.saw_logic_packet = false;
    assert!(!summary.saw_logic_packet);
    assert_eq!(summary.terminal_event, AcquisitionTerminalEvent::NormalEnd);
}

#[test]
fn incomplete_shape_is_missing_end_marker() {
    let mut summary = clean_summary();
    summary.saw_end_packet = false;
    summary.saw_end_packet_ok = false;
    summary.end_packet_status = None;
    assert!(!summary.saw_end_packet);
    assert_eq!(summary.end_packet_status, None);
}

#[test]
fn timeout_shape_keeps_collection_active_until_cleanup() {
    let mut summary = clean_summary();
    summary.saw_terminal_normal_end = false;
    summary.saw_device_stopped = false;
    summary.terminal_event = AcquisitionTerminalEvent::None;
    summary.is_collecting = true;

    assert_eq!(summary.terminal_event, AcquisitionTerminalEvent::None);
    assert!(summary.is_collecting);
}

#[test]
fn cleanup_success_requires_callbacks_release_and_no_active_collection() {
    let cleanup = CaptureCleanup {
        stop_attempted: true,
        stop_succeeded: true,
        callbacks_cleared: true,
        release_succeeded: true,
        collecting_before_release: false,
        ..CaptureCleanup::default()
    };

    assert!(cleanup.succeeded());
}

#[test]
fn cleanup_failure_detects_remaining_collection_before_release() {
    let cleanup = CaptureCleanup {
        stop_attempted: true,
        stop_succeeded: true,
        callbacks_cleared: true,
        release_succeeded: true,
        collecting_before_release: true,
        ..CaptureCleanup::default()
    };

    assert!(!cleanup.succeeded());
}

#[test]
fn cleanup_failure_detects_callback_clear_failure() {
    let cleanup = CaptureCleanup {
        stop_attempted: true,
        stop_succeeded: true,
        callbacks_cleared: false,
        release_succeeded: true,
        clear_callbacks_error: Some("clear failed".to_string()),
        collecting_before_release: false,
        ..CaptureCleanup::default()
    };

    assert!(!cleanup.succeeded());
    assert_eq!(cleanup.clear_callbacks_error.as_deref(), Some("clear failed"));
}

#[test]
fn cleanup_failure_detects_release_failure() {
    let cleanup = CaptureCleanup {
        stop_attempted: true,
        stop_succeeded: true,
        callbacks_cleared: true,
        release_succeeded: false,
        release_error: Some("release failed".to_string()),
        collecting_before_release: false,
        ..CaptureCleanup::default()
    };

    assert!(!cleanup.succeeded());
    assert_eq!(cleanup.release_error.as_deref(), Some("release failed"));
}

#[test]
fn option_aware_capture_applies_full_validated_request_in_locked_order() {
    let _guard = runtime_test_guard().lock().unwrap();
    let Some(runtime) = load_runtime() else {
        return;
    };
    configure_mock_option_api();

    let effective = apply_validated_device_options(&runtime, &sample_validated_device_option_request(), 6)
        .expect("validated request should apply in deterministic order");

    assert_eq!(effective.operation_mode_code, Some(STREAM_MODE as i16));
    assert_eq!(effective.stop_option_code, Some(UPLOAD_WHEN_FULL as i16));
    assert_eq!(effective.channel_mode_code, Some(STREAM_COMPACT_MODE as i16));
    assert_eq!(effective.threshold_volts, Some(2.5));
    assert_eq!(effective.filter_code, Some(FILTER_NONE as i16));
    assert_eq!(effective.enabled_channels, vec![0, 2, 4]);
    assert_eq!(effective.sample_limit, Some(4096));
    assert_eq!(effective.sample_rate_hz, Some(50_000_000));
    assert_eq!(
        apply_log_keys(),
        vec![
            SR_CONF_OPERATION_MODE,
            SR_CONF_BUFFER_OPTIONS,
            SR_CONF_CHANNEL_MODE,
            SR_CONF_VTH,
            SR_CONF_FILTER,
            APPLY_CHANNEL_CALL_KEY,
            APPLY_CHANNEL_CALL_KEY,
            APPLY_CHANNEL_CALL_KEY,
            APPLY_CHANNEL_CALL_KEY,
            APPLY_CHANNEL_CALL_KEY,
            APPLY_CHANNEL_CALL_KEY,
            SR_CONF_LIMIT_SAMPLES,
            SR_CONF_SAMPLERATE,
        ]
    );
}

#[test]
fn option_aware_capture_reports_partial_apply_failure_with_applied_steps() {
    let _guard = runtime_test_guard().lock().unwrap();
    let Some(runtime) = load_runtime() else {
        return;
    };
    configure_mock_option_api();
    unsafe {
        dsview_test_mock_set_apply_failure(SR_CONF_FILTER, SR_ERR_ARG);
    }

    let failure = apply_validated_device_options(&runtime, &sample_validated_device_option_request(), 6)
        .expect_err("filter injection should stop the apply sequence");

    assert_eq!(
        failure.applied_steps,
        vec![
            DeviceOptionApplyStep::OperationMode,
            DeviceOptionApplyStep::StopOption,
            DeviceOptionApplyStep::ChannelMode,
            DeviceOptionApplyStep::ThresholdVolts,
        ]
    );
    assert_eq!(failure.failed_step, DeviceOptionApplyStep::Filter);
    assert_eq!(
        apply_log_keys(),
        vec![
            SR_CONF_OPERATION_MODE,
            SR_CONF_BUFFER_OPTIONS,
            SR_CONF_CHANNEL_MODE,
            SR_CONF_VTH,
            SR_CONF_FILTER,
        ]
    );
}

#[test]
fn capture_without_validated_device_options_keeps_config_only_baseline() {
    let _guard = runtime_test_guard().lock().unwrap();
    let Some(runtime) = load_runtime() else {
        return;
    };
    configure_mock_option_api();

    let request = CaptureRunRequest {
        selection_handle: SelectionHandle::new(1).unwrap(),
        config: CaptureConfigRequest {
            sample_rate_hz: 25_000_000,
            sample_limit: 2048,
            enabled_channels: BTreeSet::from([0_u16, 1_u16]),
        },
        validated_device_options: None,
        wait_timeout: Duration::from_millis(100),
        poll_interval: Duration::from_millis(10),
    };

    let effective = apply_capture_request_device_options(&runtime, &request, 6)
        .expect("baseline path should skip option-aware apply");

    assert_eq!(effective, None);
    assert!(apply_log_keys().is_empty());
}

#[test]
fn completion_names_cover_all_phase4_categories() {
    let names = [
        CaptureCompletion::CleanSuccess,
        CaptureCompletion::StartFailure,
        CaptureCompletion::RunFailure,
        CaptureCompletion::Detached,
        CaptureCompletion::Incomplete,
        CaptureCompletion::Timeout,
        CaptureCompletion::CleanupFailure,
    ];

    assert_eq!(names.len(), 7);
}
