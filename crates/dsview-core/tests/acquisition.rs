use dsview_core::{
    AcquisitionSummary, AcquisitionTerminalEvent, CaptureCleanup, CaptureCompletion,
    NativeErrorCode,
};
use dsview_sys::AcquisitionPacketStatus;

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
