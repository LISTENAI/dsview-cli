use std::path::{Path, PathBuf};
use std::time::{Duration, UNIX_EPOCH};

use dsview_core::{
    build_capture_device_option_facts, metadata_path_for_vcd, resolve_capture_artifact_paths,
    AcquisitionSummary, AcquisitionTerminalEvent, CaptureArtifactPathError, CaptureCleanup,
    CaptureCompletion, CaptureDeviceOptionFacts, CaptureDeviceOptionSnapshot, CaptureExportError,
    CaptureExportFailureKind, CaptureExportRequest, CaptureRunSummary, ChannelModeGroupSnapshot,
    ChannelModeOptionSnapshot, CurrentDeviceOptionValues, DeviceIdentitySnapshot,
    DeviceOptionsSnapshot, EnumOptionSnapshot, NativeErrorCode, ThresholdCapabilitySnapshot,
    ValidatedCaptureConfig, ValidatedDeviceOptionRequest,
};
use dsview_sys::{
    AcquisitionPacketStatus, ExportErrorCode, RuntimeError, VcdExportFacts, VcdExportRequest,
};

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
        effective_device_options: None,
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

fn device_option_snapshot() -> DeviceOptionsSnapshot {
    DeviceOptionsSnapshot {
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
        operation_modes: vec![
            EnumOptionSnapshot {
                id: "operation-mode:0".to_string(),
                native_code: 0,
                label: "Buffer Mode".to_string(),
            },
            EnumOptionSnapshot {
                id: "operation-mode:1".to_string(),
                native_code: 1,
                label: "Stream Mode".to_string(),
            },
        ],
        stop_options: vec![
            EnumOptionSnapshot {
                id: "stop-option:0".to_string(),
                native_code: 0,
                label: "Immediate".to_string(),
            },
            EnumOptionSnapshot {
                id: "stop-option:1".to_string(),
                native_code: 1,
                label: "Stop after samples".to_string(),
            },
        ],
        filters: vec![
            EnumOptionSnapshot {
                id: "filter:0".to_string(),
                native_code: 0,
                label: "Off".to_string(),
            },
            EnumOptionSnapshot {
                id: "filter:1".to_string(),
                native_code: 1,
                label: "1 Sample".to_string(),
            },
        ],
        channel_modes_by_operation_mode: vec![
            ChannelModeGroupSnapshot {
                operation_mode_id: "operation-mode:0".to_string(),
                operation_mode_code: 0,
                current_channel_mode_id: Some("channel-mode:20".to_string()),
                current_channel_mode_code: Some(20),
                channel_modes: vec![
                    ChannelModeOptionSnapshot {
                        id: "channel-mode:20".to_string(),
                        native_code: 20,
                        label: "Buffer 100x16".to_string(),
                        max_enabled_channels: 16,
                    },
                    ChannelModeOptionSnapshot {
                        id: "channel-mode:21".to_string(),
                        native_code: 21,
                        label: "Buffer 200x8".to_string(),
                        max_enabled_channels: 8,
                    },
                ],
            },
            ChannelModeGroupSnapshot {
                operation_mode_id: "operation-mode:1".to_string(),
                operation_mode_code: 1,
                current_channel_mode_id: None,
                current_channel_mode_code: None,
                channel_modes: vec![ChannelModeOptionSnapshot {
                    id: "channel-mode:30".to_string(),
                    native_code: 30,
                    label: "Stream 100x16".to_string(),
                    max_enabled_channels: 16,
                }],
            },
        ],
        threshold: ThresholdCapabilitySnapshot {
            id: "threshold:vth-range".to_string(),
            kind: "voltage-range".to_string(),
            current_volts: Some(1.8),
            min_volts: 0.0,
            max_volts: 5.0,
            step_volts: 0.1,
            legacy_metadata: None,
        },
    }
}

fn validated_device_options() -> ValidatedDeviceOptionRequest {
    ValidatedDeviceOptionRequest {
        operation_mode_id: "operation-mode:1".to_string(),
        operation_mode_code: 1,
        stop_option_id: None,
        stop_option_code: None,
        channel_mode_id: "channel-mode:30".to_string(),
        channel_mode_code: 30,
        sample_rate_hz: 200_000_000,
        requested_sample_limit: 4097,
        effective_sample_limit: 4096,
        enabled_channels: vec![0, 2, 4, 6],
        threshold_volts: Some(2.4),
        filter_id: Some("filter:1".to_string()),
        filter_code: Some(1),
    }
}

fn export_request(completion: CaptureCompletion) -> CaptureExportRequest {
    let mut capture = clean_capture();
    capture.completion = completion;
    CaptureExportRequest {
        capture,
        validated_config: validated_config(),
        vcd_path: PathBuf::from("/tmp/capture.vcd"),
        metadata_path: None,
        tool_name: "dsview-cli".to_string(),
        tool_version: "0.1.0".to_string(),
        capture_started_at: UNIX_EPOCH + Duration::from_secs(1_744_018_496),
        device_model: "DSLogic Plus".to_string(),
        device_stable_id: "dslogic-plus".to_string(),
        selected_handle: dsview_core::SelectionHandle::new(7).unwrap(),
        validated_device_options: Some(validated_device_options()),
        device_options_snapshot: device_option_snapshot(),
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
        let request = export_request(completion);
        let result = match request.capture.completion {
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
fn metadata_sidecar_path_is_derived_from_vcd_path() {
    let vcd_path = Path::new("artifacts/run-01.vcd");
    assert_eq!(metadata_path_for_vcd(vcd_path), PathBuf::from("artifacts/run-01.json"));
}

#[test]
fn explicit_metadata_path_override_is_used_when_present() {
    let paths = resolve_capture_artifact_paths(
        Path::new("artifacts/run-01.vcd"),
        Some(Path::new("sidecars/run-01.json")),
    )
    .unwrap();

    assert_eq!(paths.vcd_path, PathBuf::from("artifacts/run-01.vcd"));
    assert_eq!(paths.metadata_path, PathBuf::from("sidecars/run-01.json"));
}

#[test]
fn invalid_vcd_extension_is_rejected_before_export() {
    let error = resolve_capture_artifact_paths(Path::new("artifacts/run-01.bin"), None::<&Path>)
        .unwrap_err();

    assert_eq!(
        error,
        CaptureArtifactPathError::InvalidVcdExtension {
            path: PathBuf::from("artifacts/run-01.bin"),
        }
    );
}

#[test]
fn invalid_metadata_extension_is_rejected_before_export() {
    let error = resolve_capture_artifact_paths(
        Path::new("artifacts/run-01.vcd"),
        Some(Path::new("artifacts/run-01.txt")),
    )
    .unwrap_err();

    assert_eq!(
        error,
        CaptureArtifactPathError::InvalidMetadataExtension {
            path: PathBuf::from("artifacts/run-01.txt"),
        }
    );
}

#[test]
fn conflicting_artifact_paths_are_rejected_before_export() {
    let error = resolve_capture_artifact_paths(
        Path::new("artifacts/run-01.vcd"),
        Some(Path::new("artifacts/run-01.vcd")),
    )
    .unwrap_err();

    assert_eq!(
        error,
        CaptureArtifactPathError::ConflictingArtifactPaths {
            vcd_path: PathBuf::from("artifacts/run-01.vcd"),
            metadata_path: PathBuf::from("artifacts/run-01.vcd"),
        }
    );
}

#[test]
fn metadata_sidecar_schema_uses_numeric_capture_fields_and_utc_timestamp() {
    let request = export_request(CaptureCompletion::CleanSuccess);
    let metadata_path = metadata_path_for_vcd(&request.vcd_path);
    let export = VcdExportFacts {
        sample_count: 1536,
        packet_count: 4,
        output_bytes: 512,
    };
    let metadata_json = serde_json::to_value(
        dsview_core::CaptureMetadata {
            schema_version: 2,
            tool: dsview_core::MetadataToolInfo {
                name: request.tool_name.clone(),
                version: request.tool_version.clone(),
            },
            capture: dsview_core::MetadataCaptureInfo {
                timestamp_utc: "2025-04-07T10:14:56Z".to_string(),
                device_model: request.device_model.clone(),
                device_stable_id: request.device_stable_id.clone(),
                selected_handle: request.selected_handle.raw(),
                sample_rate_hz: request.validated_config.sample_rate_hz,
                requested_sample_limit: request.validated_config.requested_sample_limit,
                actual_sample_count: export.sample_count,
                enabled_channels: request.validated_config.enabled_channels.clone(),
            },
            acquisition: dsview_core::MetadataAcquisitionInfo {
                completion: "clean_success".to_string(),
                terminal_event: "normal_end".to_string(),
                saw_logic_packet: true,
                saw_end_packet: true,
                end_packet_status: Some("ok".to_string()),
            },
            artifacts: dsview_core::MetadataArtifactInfo {
                vcd_path: request.vcd_path.display().to_string(),
                metadata_path: metadata_path.display().to_string(),
            },
            device_options: CaptureDeviceOptionFacts {
                requested: CaptureDeviceOptionSnapshot {
                    operation_mode_id: "operation-mode:0".to_string(),
                    stop_option_id: Some("stop-option:1".to_string()),
                    channel_mode_id: "channel-mode:20".to_string(),
                    enabled_channels: vec![0, 1, 2, 3],
                    threshold_volts: Some(1.8),
                    filter_id: Some("filter:0".to_string()),
                    sample_rate_hz: 100_000_000,
                    sample_limit: 2048,
                },
                effective: CaptureDeviceOptionSnapshot {
                    operation_mode_id: "operation-mode:1".to_string(),
                    stop_option_id: None,
                    channel_mode_id: "channel-mode:30".to_string(),
                    enabled_channels: vec![0, 2, 4, 6],
                    threshold_volts: Some(2.4),
                    filter_id: Some("filter:1".to_string()),
                    sample_rate_hz: 200_000_000,
                    sample_limit: 4096,
                },
            },
        }
    )
    .unwrap();

    assert_eq!(metadata_json["schema_version"], 2);
    assert_eq!(metadata_json["tool"]["name"], "dsview-cli");
    assert!(metadata_json["capture"]["timestamp_utc"]
        .as_str()
        .unwrap()
        .ends_with('Z'));
    assert!(metadata_json["capture"]["sample_rate_hz"].is_number());
    assert!(metadata_json["capture"]["requested_sample_limit"].is_number());
    assert!(metadata_json["capture"]["actual_sample_count"].is_number());
    assert_eq!(metadata_json["capture"]["device_model"], "DSLogic Plus");
    assert_eq!(metadata_json["capture"]["enabled_channels"], serde_json::json!([0, 1, 2, 3]));
    assert_eq!(metadata_json["artifacts"]["vcd_path"], "/tmp/capture.vcd");
    assert_eq!(metadata_json["artifacts"]["metadata_path"], "/tmp/capture.json");
    assert_eq!(
        metadata_json["device_options"]["requested"]["operation_mode_id"],
        "operation-mode:0"
    );
    assert_eq!(
        metadata_json["device_options"]["effective"]["channel_mode_id"],
        "channel-mode:30"
    );
    assert_eq!(
        metadata_json["device_options"]["effective"]["sample_limit"],
        4096
    );
}

#[test]
fn metadata_sidecar_includes_requested_and_effective_device_options() {
    let facts = build_capture_device_option_facts(&export_request(CaptureCompletion::CleanSuccess))
        .expect("device option facts should build");
    let metadata_json = serde_json::to_value(facts).unwrap();

    assert_eq!(
        metadata_json["requested"]["operation_mode_id"],
        "operation-mode:1"
    );
    assert_eq!(metadata_json["requested"]["filter_id"], "filter:1");
    assert_eq!(
        metadata_json["effective"]["channel_mode_id"],
        "channel-mode:30"
    );
    assert_eq!(metadata_json["effective"]["sample_rate_hz"], 200_000_000);
}

#[test]
fn metadata_schema_version_is_2_when_device_option_facts_are_present() {
    let metadata = dsview_core::CaptureMetadata {
        schema_version: 2,
        tool: dsview_core::MetadataToolInfo {
            name: "dsview-cli".to_string(),
            version: "0.1.0".to_string(),
        },
        capture: dsview_core::MetadataCaptureInfo {
            timestamp_utc: "2025-04-07T10:14:56Z".to_string(),
            device_model: "DSLogic Plus".to_string(),
            device_stable_id: "dslogic-plus".to_string(),
            selected_handle: 7,
            sample_rate_hz: 100_000_000,
            requested_sample_limit: 2048,
            actual_sample_count: 1536,
            enabled_channels: vec![0, 1, 2, 3],
        },
        acquisition: dsview_core::MetadataAcquisitionInfo {
            completion: "clean_success".to_string(),
            terminal_event: "normal_end".to_string(),
            saw_logic_packet: true,
            saw_end_packet: true,
            end_packet_status: Some("ok".to_string()),
        },
        artifacts: dsview_core::MetadataArtifactInfo {
            vcd_path: "/tmp/capture.vcd".to_string(),
            metadata_path: "/tmp/capture.json".to_string(),
        },
        device_options: build_capture_device_option_facts(&export_request(CaptureCompletion::CleanSuccess))
            .expect("device option facts should build"),
    };

    assert_eq!(serde_json::to_value(metadata).unwrap()["schema_version"], 2);
}

#[test]
fn requested_and_effective_sample_limit_values_can_differ() {
    let facts = build_capture_device_option_facts(&export_request(CaptureCompletion::CleanSuccess))
        .expect("device option facts should build");

    assert_eq!(facts.requested.sample_limit, 4097);
    assert_eq!(facts.effective.sample_limit, 4096);
}

#[test]
fn baseline_capture_metadata_includes_inherited_effective_device_options() {
    let mut request = export_request(CaptureCompletion::CleanSuccess);
    request.validated_device_options = None;
    request.capture.effective_device_options = None;
    request.validated_config.sample_rate_hz = 50_000_000;
    request.validated_config.effective_sample_limit = 8192;
    request.validated_config.enabled_channels = vec![0, 1, 3, 5];

    let facts = build_capture_device_option_facts(&request)
        .expect("baseline device option facts should be inherited from current values");

    assert_eq!(
        facts.requested,
        CaptureDeviceOptionSnapshot {
            operation_mode_id: "operation-mode:0".to_string(),
            stop_option_id: Some("stop-option:1".to_string()),
            channel_mode_id: "channel-mode:20".to_string(),
            enabled_channels: vec![0, 1, 3, 5],
            threshold_volts: Some(1.8),
            filter_id: Some("filter:0".to_string()),
            sample_rate_hz: 50_000_000,
            sample_limit: 8192,
        }
    );
    assert_eq!(facts.effective, facts.requested);
}

#[test]
fn metadata_sidecar_failure_variants_distinguish_serialization_from_write() {
    let serialization = CaptureExportError::MetadataSerializationFailed {
        path: PathBuf::from("artifacts/run.json"),
        detail: "timestamp out of range".into(),
    };
    let write = CaptureExportError::MetadataWriteFailed {
        path: PathBuf::from("artifacts/run.json"),
        detail: "permission denied".into(),
    };

    match serialization {
        CaptureExportError::MetadataSerializationFailed { path, detail } => {
            assert_eq!(path, PathBuf::from("artifacts/run.json"));
            assert!(detail.contains("timestamp"));
        }
        other => panic!("expected serialization failure, got {other:?}"),
    }

    match write {
        CaptureExportError::MetadataWriteFailed { path, detail } => {
            assert_eq!(path, PathBuf::from("artifacts/run.json"));
            assert!(detail.contains("permission denied"));
        }
        other => panic!("expected metadata write failure, got {other:?}"),
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
