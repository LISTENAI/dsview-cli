use dsview_core::{
    AcquisitionSummary, AcquisitionTerminalEvent, CaptureCleanup, CaptureCompletion,
    CaptureExportError, CaptureRunError, NativeErrorCode,
};
use dsview_sys::AcquisitionPacketStatus;
use serde_json::json;

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
fn capture_success_reports_artifacts_json() {
    let response = serde_json::to_value(json!({
        "selected_handle": 7,
        "completion": "clean_success",
        "saw_logic_packet": true,
        "saw_end_packet": true,
        "saw_terminal_normal_end": true,
        "cleanup_succeeded": true,
        "artifacts": {
            "vcd_path": "artifacts/run.vcd",
            "metadata_path": "artifacts/run.json"
        }
    }))
    .unwrap();

    assert_eq!(response["completion"], "clean_success");
    assert_eq!(response["artifacts"]["vcd_path"], "artifacts/run.vcd");
    assert_eq!(response["artifacts"]["metadata_path"], "artifacts/run.json");
}

#[test]
fn metadata_serialization_failure_maps_to_distinct_cli_error_code() {
    let error = cli::classify_export_error(&CaptureExportError::MetadataSerializationFailed {
        path: "artifacts/run.json".into(),
        detail: "invalid timestamp".to_string(),
    });

    assert_eq!(error.code, "capture_metadata_serialization_failed");
    assert!(error.message.contains("metadata artifact"));
}

#[test]
fn metadata_write_failure_maps_to_non_zero_artifact_error_class() {
    let error = cli::classify_export_error(&CaptureExportError::MetadataWriteFailed {
        path: "artifacts/run.json".into(),
        detail: "permission denied".to_string(),
    });

    assert_eq!(error.code, "capture_metadata_write_failed");
    assert!(error.detail.unwrap().contains("permission denied"));
}

#[test]
fn capture_not_exportable_maps_separately_from_acquisition_failure() {
    let error = cli::classify_export_error(&CaptureExportError::CaptureNotExportable {
        completion: CaptureCompletion::Incomplete,
    });

    assert_eq!(error.code, "capture_not_exportable");
    assert!(error.message.contains("incomplete"));
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
