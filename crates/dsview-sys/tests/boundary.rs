use std::path::Path;

use dsview_sys::{
    source_runtime_library_path, upstream_header_path, ExportErrorCode, RuntimeError,
    VcdExportFacts, VcdExportRequest,
};

#[test]
fn upstream_header_exists_for_boundary_tests() {
    let header = upstream_header_path();
    assert!(header.ends_with("DSView/libsigrok4DSL/libsigrok.h"));
    assert!(header.exists(), "expected upstream header at {}", header.display());
}

#[test]
fn source_runtime_path_shape_matches_cfg_state() {
    if let Some(path) = source_runtime_library_path() {
        assert!(Path::new(path).is_absolute());
        assert!(path.to_string_lossy().ends_with("libdsview_runtime.so"));
    }
}

#[test]
fn export_request_shape_matches_wave0_capacity_fit_fixture() {
    let request = VcdExportRequest {
        samplerate_hz: 100_000_000,
        enabled_channels: vec![0, 1],
    };

    assert_eq!(request.samplerate_hz, 100_000_000);
    assert_eq!(request.enabled_channels, vec![0, 1]);
    assert_eq!(request.enabled_channels.len(), 2);
}

#[test]
fn overflow_error_code_stays_stable_for_fail_fast_contract() {
    let error = RuntimeError::ExportCall {
        operation: "ds_export_recorded_vcd",
        code: ExportErrorCode::Overflow,
    };

    match error {
        RuntimeError::ExportCall { operation, code } => {
            assert_eq!(operation, "ds_export_recorded_vcd");
            assert_eq!(code, ExportErrorCode::Overflow);
            assert_eq!(code.name(), "export_overflow");
        }
        other => panic!("expected export overflow call error, got {other:?}"),
    }
}

#[test]
fn temp_promotion_failures_reference_temp_and_final_vcd_paths() {
    let error = RuntimeError::TempPromote {
        from: Path::new("/tmp/capture.vcd.tmp").to_path_buf(),
        to: Path::new("/tmp/capture.vcd").to_path_buf(),
        detail: "cross-device rename".to_string(),
    };

    match error {
        RuntimeError::TempPromote { from, to, detail } => {
            assert!(from.ends_with("capture.vcd.tmp"));
            assert!(to.ends_with("capture.vcd"));
            assert_eq!(detail, "cross-device rename");
        }
        other => panic!("expected temp promote failure, got {other:?}"),
    }
}

#[test]
fn export_facts_surface_sample_packet_and_byte_counts() {
    let facts = VcdExportFacts {
        sample_count: 2048,
        packet_count: 3,
        output_bytes: 512,
    };

    assert_eq!(facts.sample_count, 2048);
    assert_eq!(facts.packet_count, 3);
    assert_eq!(facts.output_bytes, 512);
}
