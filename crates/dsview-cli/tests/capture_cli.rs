use assert_cmd::Command;
use predicates::prelude::*;
use dsview_core::{
    resolve_capture_artifact_paths, AcquisitionSummary, AcquisitionTerminalEvent,
    CaptureCleanup, CaptureCompletion, CaptureExportError, CaptureExportFailureKind,
    CaptureRunError, NativeErrorCode,
};
use dsview_sys::{AcquisitionPacketStatus, ExportErrorCode};
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

fn cli_command() -> Command {
    Command::cargo_bin("dsview-cli").expect("dsview-cli binary should build for CLI tests")
}

#[test]
fn capture_help_mentions_runtime_and_artifact_controls() {
    cli_command()
        .arg("capture")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--library <PATH>"))
        .stdout(predicate::str::contains("--use-source-runtime"))
        .stdout(predicate::str::contains("--resource-dir <PATH>"))
        .stdout(predicate::str::contains("--output <PATH>"))
        .stdout(predicate::str::contains("--metadata-output <PATH>"))
        .stdout(predicate::str::contains("json is stable for automation"))
        .stdout(predicate::str::contains("must end with .vcd"))
        .stdout(predicate::str::contains("defaults to the VCD path with a .json extension"));
}

#[test]
fn capture_missing_runtime_selector_fails_with_machine_readable_json() {
    cli_command()
        .args([
            "capture",
            "--resource-dir",
            ".",
            "--handle",
            "7",
            "--sample-rate-hz",
            "100000000",
            "--sample-limit",
            "2048",
            "--channels",
            "0,1,2,3",
            "--output",
            "artifacts/run.vcd",
        ])
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"code\": \"runtime_selector_missing\""))
        .stdout(predicate::str::contains(
            "pass either --library <PATH> or --use-source-runtime.",
        ))
        .stderr(predicate::str::is_empty());
}

#[test]
fn capture_invalid_output_path_fails_on_stderr_in_text_mode() {
    cli_command()
        .args([
            "capture",
            "--format",
            "text",
            "--resource-dir",
            ".",
            "--handle",
            "7",
            "--sample-rate-hz",
            "100000000",
            "--sample-limit",
            "2048",
            "--channels",
            "0,1,2,3",
            "--output",
            "artifacts/run.bin",
        ])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("capture_output_path_invalid:"))
        .stderr(predicate::str::contains("must use the .vcd extension"));
}

#[test]
fn capture_invalid_metadata_output_fails_on_stderr_in_text_mode() {
    cli_command()
        .args([
            "capture",
            "--format",
            "text",
            "--resource-dir",
            ".",
            "--handle",
            "7",
            "--sample-rate-hz",
            "100000000",
            "--sample-limit",
            "2048",
            "--channels",
            "0,1,2,3",
            "--output",
            "artifacts/run.vcd",
            "--metadata-output",
            "artifacts/run.txt",
        ])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("capture_metadata_output_path_invalid:"))
        .stderr(predicate::str::contains("must use the .json extension"));
}

#[test]
fn capture_conflicting_artifact_paths_fail_before_runtime_work() {
    cli_command()
        .args([
            "capture",
            "--resource-dir",
            ".",
            "--handle",
            "7",
            "--sample-rate-hz",
            "100000000",
            "--sample-limit",
            "2048",
            "--channels",
            "0,1,2,3",
            "--output",
            "artifacts/run.vcd",
            "--metadata-output",
            "artifacts/run.vcd",
        ])
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"code\": \"capture_artifact_paths_conflict\""))
        .stdout(predicate::str::contains("must be different"))
        .stderr(predicate::str::is_empty());
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
fn capture_text_success_reports_completion_and_artifact_paths() {
    let output = cli::capture_success_text(
        "clean_success",
        "artifacts/run.vcd",
        "artifacts/run.json",
    );

    assert_eq!(
        output,
        "capture clean_success\nvcd artifacts/run.vcd\nmetadata artifacts/run.json"
    );
}

#[test]
fn invalid_vcd_output_path_maps_to_distinct_cli_error_code() {
    let path_error = resolve_capture_artifact_paths("artifacts/run.bin", None::<&str>).unwrap_err();
    let error = cli::classify_export_error(&CaptureExportError::InvalidArtifactPaths(path_error));

    assert_eq!(error.code, "capture_output_path_invalid");
    assert!(error.message.contains(".vcd extension"));
}

#[test]
fn invalid_metadata_output_path_maps_to_distinct_cli_error_code() {
    let path_error =
        resolve_capture_artifact_paths("artifacts/run.vcd", Some("artifacts/run.txt")).unwrap_err();
    let error = cli::classify_export_error(&CaptureExportError::InvalidArtifactPaths(path_error));

    assert_eq!(error.code, "capture_metadata_output_path_invalid");
    assert!(error.message.contains(".json extension"));
}

#[test]
fn conflicting_artifact_paths_map_to_distinct_cli_error_code() {
    let path_error =
        resolve_capture_artifact_paths("artifacts/run.vcd", Some("artifacts/run.vcd")).unwrap_err();
    let error = cli::classify_export_error(&CaptureExportError::InvalidArtifactPaths(path_error));

    assert_eq!(error.code, "capture_artifact_paths_conflict");
    assert!(error.message.contains("must be different"));
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
fn export_precondition_failure_uses_distinct_cli_code() {
    let error = cli::classify_export_error(&CaptureExportError::ExportFailed {
        path: "artifacts/run.vcd".into(),
        kind: CaptureExportFailureKind::Precondition {
            code: ExportErrorCode::Overflow,
        },
        detail: "export call `ds_export_recorded_vcd` failed with Overflow".to_string(),
    });

    assert_eq!(error.code, "capture_export_precondition_failed");
    assert_eq!(error.message, "failed to write VCD artifact `artifacts/run.vcd`");
    assert!(error.detail.unwrap().contains("Overflow"));
}

#[test]
fn export_runtime_failure_keeps_runtime_specific_cli_code() {
    let error = cli::classify_export_error(&CaptureExportError::ExportFailed {
        path: "artifacts/run.vcd".into(),
        kind: CaptureExportFailureKind::Runtime,
        detail: "export call `ds_export_recorded_vcd` failed with OutputModuleUnavailable"
            .to_string(),
    });

    assert_eq!(error.code, "capture_export_failed");
    assert_eq!(error.message, "failed to write VCD artifact `artifacts/run.vcd`");
    assert!(error
        .detail
        .unwrap()
        .contains("OutputModuleUnavailable"));
}

#[test]
fn cleanup_failure_shape_preserves_callback_clear_errors() {
    let error = cli::classify_capture_error(&CaptureRunError::CleanupFailed {
        during: "run_failure",
        summary: clean_summary(),
        cleanup: CaptureCleanup {
            callbacks_cleared: false,
            clear_callbacks_error: Some("clear failed".to_string()),
            release_succeeded: true,
            ..CaptureCleanup::default()
        },
    });

    assert_eq!(error.code, "capture_cleanup_failed");
    let cleanup = error.cleanup.expect("cleanup detail should be present");
    assert!(!cleanup.callbacks_cleared);
    assert_eq!(cleanup.clear_callbacks_error.as_deref(), Some("clear failed"));
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
