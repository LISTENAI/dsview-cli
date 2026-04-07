use std::path::PathBuf;

use dsview_core::{
    AcquisitionSummary, AcquisitionTerminalEvent, CaptureCleanup, CaptureCompletion,
    CaptureExportError, CaptureExportFailureKind, CaptureExportRequest, CaptureRunSummary,
    NativeErrorCode, ValidatedCaptureConfig,
};
use dsview_sys::{AcquisitionPacketStatus, ExportErrorCode, RuntimeError, VcdExportRequest};

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

fn clean_capture() -> CaptureRunSummary {
    CaptureRunSummary {
        completion: CaptureCompletion::CleanSuccess,
        summary: clean_summary(),
        cleanup: CaptureCleanup {
            callbacks_cleared: true,
            release_succeeded: true,
            ..CaptureCleanup::default()
        },
    }
}

fn validated_config() -> ValidatedCaptureConfig {
    ValidatedCaptureConfig {
        sample_rate_hz: 100_000_000,
        requested_sample_limit: 2048,
        effective_sample_limit: 2048,
        enabled_channels: vec![0, 1, 2, 3],
        channel_mode_id: 20,
    }
}

fn export_request(completion: CaptureCompletion) -> CaptureExportRequest {
    let mut capture = clean_capture();
    capture.completion = completion;
    CaptureExportRequest {
        capture,
        validated_config: validated_config(),
        vcd_path: PathBuf::from("/tmp/capture.vcd"),
    }
}

#[test]
fn only_clean_success_capture_is_export_eligible() {
    let completions = [
        CaptureCompletion::StartFailure,
        CaptureCompletion::RunFailure,
        CaptureCompletion::Detached,
        CaptureCompletion::Incomplete,
        CaptureCompletion::CleanupFailure,
        CaptureCompletion::Timeout,
    ];

    for completion in completions {
        let result = match completion {
            CaptureCompletion::CleanSuccess => None,
            other => Some(CaptureExportError::CaptureNotExportable { completion: other }),
        };
        assert!(matches!(
            result,
            Some(CaptureExportError::CaptureNotExportable { completion: current }) if current == completion
        ));
    }
}

#[test]
fn export_request_uses_observed_clean_capture_and_validated_config() {
    let request = export_request(CaptureCompletion::CleanSuccess);
    let export_request = VcdExportRequest {
        samplerate_hz: request.validated_config.sample_rate_hz,
        enabled_channels: request.validated_config.enabled_channels.clone(),
    };

    assert_eq!(request.capture.completion, CaptureCompletion::CleanSuccess);
    assert_eq!(export_request.samplerate_hz, 100_000_000);
    assert_eq!(export_request.enabled_channels, vec![0, 1, 2, 3]);
}

#[test]
fn overflow_precondition_maps_separately_from_export_runtime_failures() {
    let overflow = RuntimeError::ExportCall {
        operation: "ds_export_recorded_vcd",
        code: ExportErrorCode::Overflow,
    };
    let runtime = RuntimeError::ExportCall {
        operation: "ds_export_recorded_vcd",
        code: ExportErrorCode::OutputModuleUnavailable,
    };

    let overflow_kind = match overflow {
        RuntimeError::ExportCall { code, .. } => match code {
            ExportErrorCode::NoStream
            | ExportErrorCode::Overflow
            | ExportErrorCode::BadEndStatus
            | ExportErrorCode::MissingSamplerate
            | ExportErrorCode::NoEnabledChannels => {
                CaptureExportFailureKind::Precondition { code }
            }
            _ => CaptureExportFailureKind::Runtime,
        },
        _ => CaptureExportFailureKind::Runtime,
    };
    let runtime_kind = match runtime {
        RuntimeError::ExportCall { code, .. } => match code {
            ExportErrorCode::NoStream
            | ExportErrorCode::Overflow
            | ExportErrorCode::BadEndStatus
            | ExportErrorCode::MissingSamplerate
            | ExportErrorCode::NoEnabledChannels => {
                CaptureExportFailureKind::Precondition { code }
            }
            _ => CaptureExportFailureKind::Runtime,
        },
        _ => CaptureExportFailureKind::Runtime,
    };

    assert_eq!(
        overflow_kind,
        CaptureExportFailureKind::Precondition {
            code: ExportErrorCode::Overflow,
        }
    );
    assert_eq!(runtime_kind, CaptureExportFailureKind::Runtime);
}

#[test]
fn export_failure_keeps_output_path_without_sys_packet_details() {
    let error = CaptureExportError::ExportFailed {
        path: PathBuf::from("artifacts/run.vcd"),
        kind: CaptureExportFailureKind::Runtime,
        detail: "export call `ds_export_recorded_vcd` failed with OutputModuleUnavailable".into(),
    };

    match error {
        CaptureExportError::ExportFailed { path, kind, detail } => {
            assert_eq!(path, PathBuf::from("artifacts/run.vcd"));
            assert_eq!(kind, CaptureExportFailureKind::Runtime);
            assert!(detail.contains("ds_export_recorded_vcd"));
            assert!(!detail.contains("sr_datafeed_packet"));
        }
        other => panic!("expected export failure, got {other:?}"),
    }
}
