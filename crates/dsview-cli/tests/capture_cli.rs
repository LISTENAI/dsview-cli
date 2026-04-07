use dsview_core::{
    AcquisitionSummary, AcquisitionTerminalEvent, CaptureCleanup, CaptureRunError,
    NativeErrorCode,
};
use dsview_sys::AcquisitionPacketStatus;

#[path = "../src/main.rs"]
mod cli;

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
fn preflight_not_ready_shape_is_stable() {
    let error = cli::classify_capture_error(&CaptureRunError::EnvironmentNotReady);
    assert_eq!(error.code, "capture_environment_not_ready");
    assert_eq!(error.native_error, None);
    assert_eq!(error.terminal_event, None);
    assert_eq!(error.cleanup, None);
}

#[test]
fn run_failure_shape_is_non_zero_class() {
    let mut summary = clean_summary();
    summary.saw_terminal_normal_end = false;
    summary.saw_terminal_end_by_error = true;
    summary.terminal_event = AcquisitionTerminalEvent::EndByError;
    summary.saw_end_packet = false;
    summary.end_packet_status = None;
    summary.saw_end_packet_ok = false;
    summary.last_error = NativeErrorCode::Generic;

    let error = cli::classify_capture_error(&CaptureRunError::RunFailed {
        summary,
        cleanup: CaptureCleanup {
            callbacks_cleared: true,
            release_succeeded: true,
            ..CaptureCleanup::default()
        },
    });

    assert_eq!(error.code, "capture_run_failed");
    assert_eq!(error.terminal_event, Some("end_by_error"));
    assert_eq!(error.native_error, Some("SR_ERR"));
}

#[test]
fn cleanup_failure_shape_preserves_cleanup_fields() {
    let error = cli::classify_capture_error(&CaptureRunError::CleanupFailed {
        during: "timeout",
        summary: clean_summary(),
        cleanup: CaptureCleanup {
            stop_attempted: true,
            stop_succeeded: false,
            callbacks_cleared: false,
            release_succeeded: false,
            stop_error: Some("stop failed".to_string()),
            release_error: Some("release failed".to_string()),
            ..CaptureCleanup::default()
        },
    });

    assert_eq!(error.code, "capture_cleanup_failed");
    let cleanup = error.cleanup.expect("cleanup detail should be present");
    assert!(cleanup.stop_attempted);
    assert!(!cleanup.stop_succeeded);
    assert!(!cleanup.callbacks_cleared);
    assert!(!cleanup.release_succeeded);
}
