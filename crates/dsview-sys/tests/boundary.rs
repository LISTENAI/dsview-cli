use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use dsview_sys::{
    runtime_library_name, source_runtime_library_path, upstream_header_path, ExportErrorCode, RuntimeBridge,
    RuntimeError, VcdExportRequest,
};

fn runtime_test_guard() -> &'static Mutex<()> {
    static GUARD: OnceLock<Mutex<()>> = OnceLock::new();
    GUARD.get_or_init(|| Mutex::new(()))
}

fn load_runtime() -> Option<RuntimeBridge> {
    let path = source_runtime_library_path()?;
    RuntimeBridge::load(path).ok()
}

fn vcd_string(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).expect("synthetic VCD should be utf-8")
}

fn normalize_vcd(vcd: &str) -> String {
    let mut normalized = Vec::new();
    let mut lines = vcd.lines();

    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        if trimmed.starts_with("$date") {
            normalized.push("$date <normalized> $end".to_string());
            continue;
        }
        if trimmed.starts_with("$version") {
            normalized.push("$version <normalized> $end".to_string());
            continue;
        }
        if trimmed == "$comment" {
            normalized.push("$comment".to_string());
            normalized.push("<normalized>".to_string());
            for comment_line in lines.by_ref() {
                if comment_line.trim() == "$end" {
                    normalized.push("$end".to_string());
                    break;
                }
            }
            continue;
        }
        normalized.push(trimmed.to_string());
    }

    normalized.join("\n") + "\n"
}

fn golden_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("goldens")
        .join(name)
}

fn temp_path(name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("dsview-sys-{name}-{nonce}.vcd"))
}

fn parse_transitions(vcd: &str) -> Vec<(u64, String)> {
    let mut now = 0_u64;
    let mut transitions = Vec::new();

    for line in vcd.lines().map(str::trim).filter(|line| !line.is_empty()) {
        if let Some(rest) = line.strip_prefix('#') {
            let mut parts = rest.split_whitespace();
            let timestamp = parts.next().expect("timestamp should exist");
            now = timestamp.parse().expect("timestamp should parse");
            for value in parts {
                transitions.push((now, value.to_string()));
            }
            continue;
        }
        if matches!(line, "$dumpvars" | "$end") || line.starts_with('$') {
            continue;
        }
        if matches!(line.as_bytes().first(), Some(b'0' | b'1' | b'x' | b'z')) {
            transitions.push((now, line.to_string()));
        }
    }

    transitions
}

fn final_timestamp(vcd: &str) -> Option<u64> {
    vcd.lines()
        .map(str::trim)
        .filter_map(|line| line.strip_prefix('#'))
        .filter_map(|raw| raw.split_whitespace().next())
        .filter_map(|raw| raw.parse::<u64>().ok())
        .last()
}

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
        assert!(path.to_string_lossy().ends_with(runtime_library_name()));
    }
}

#[test]
fn synthetic_vcd_goldens_match_split_packet_fixture() {
    let _guard = runtime_test_guard().lock().unwrap();
    let Some(runtime) = load_runtime() else {
        return;
    };
    let request = VcdExportRequest {
        samplerate_hz: 100_000_000,
        enabled_channels: vec![0, 1],
    };
    let sample_bytes = [0b00, 0b01, 0b10, 0b00];
    let packet_lengths = [2_usize, 2_usize];

    let export = runtime
        .render_vcd_from_logic_packets(&request, &sample_bytes, &packet_lengths, 1)
        .expect("split-packet replay should export VCD");
    let actual = normalize_vcd(&vcd_string(&export.bytes));
    let expected = fs::read_to_string(golden_path("two_channel_split_packets.vcd"))
        .expect("golden VCD should exist");

    assert_eq!(export.sample_count, 4);
    assert_eq!(export.packet_count, 4);
    assert_eq!(actual, expected);
}

#[test]
fn synthetic_vcd_goldens_lock_initial_values_deltas_and_final_timestamp() {
    let _guard = runtime_test_guard().lock().unwrap();
    let Some(runtime) = load_runtime() else {
        return;
    };
    let request = VcdExportRequest {
        samplerate_hz: 250_000_000,
        enabled_channels: vec![3],
    };
    let sample_bytes = [0b0000, 0b1000, 0b0000, 0b1000];

    let export = runtime
        .render_vcd_from_samples(&request, &sample_bytes, 1)
        .expect("single-channel replay should export VCD");
    let actual = normalize_vcd(&vcd_string(&export.bytes));
    let expected = fs::read_to_string(golden_path("single_channel_final_timestamp.vcd"))
        .expect("golden VCD should exist");

    assert_eq!(export.sample_count, 4);
    assert_eq!(export.packet_count, 3);
    assert_eq!(actual, expected);
}

#[test]
fn synthetic_vcd_goldens_verify_transition_times_and_sample_count_semantics() {
    let _guard = runtime_test_guard().lock().unwrap();
    let Some(runtime) = load_runtime() else {
        return;
    };
    let request = VcdExportRequest {
        samplerate_hz: 100_000_000,
        enabled_channels: vec![0, 1],
    };
    let sample_bytes = [0b00, 0b01, 0b10, 0b00];
    let packet_lengths = [2_usize, 2_usize];

    let export = runtime
        .render_vcd_from_logic_packets(&request, &sample_bytes, &packet_lengths, 1)
        .expect("split-packet replay should export VCD");
    let transitions = parse_transitions(&vcd_string(&export.bytes));

    assert_eq!(export.sample_count, sample_bytes.len() as u64);
    assert_eq!(
        transitions,
        vec![
            (0, "0!".to_string()),
            (0, "0\"".to_string()),
            (10, "1!".to_string()),
            (20, "0!".to_string()),
            (20, "1\"".to_string()),
            (30, "0\"".to_string()),
        ]
    );
    assert_eq!(final_timestamp(&vcd_string(&export.bytes)), Some(40));
}

#[test]
fn synthetic_vcd_goldens_exercise_cleanup_safe_write_contract() {
    let _guard = runtime_test_guard().lock().unwrap();
    let Some(runtime) = load_runtime() else {
        return;
    };
    let request = VcdExportRequest {
        samplerate_hz: 250_000_000,
        enabled_channels: vec![3],
    };
    let sample_bytes = [0b0000, 0b1000, 0b0000, 0b1000];
    let final_path = temp_path("cleanup-safe");
    let sibling_temp = final_path.with_extension("vcd.tmp");
    let _ = fs::remove_file(&final_path);
    let _ = fs::remove_file(&sibling_temp);

    let facts = runtime
        .render_vcd_from_samples_to_path(&request, &sample_bytes, 1, &final_path)
        .expect("render-to-path should succeed");

    assert!(final_path.exists());
    assert!(!sibling_temp.exists());
    assert_eq!(facts.sample_count, 4);
    assert_eq!(facts.packet_count, 3);
    assert_eq!(
        normalize_vcd(&fs::read_to_string(&final_path).expect("final VCD should exist")),
        fs::read_to_string(golden_path("single_channel_final_timestamp.vcd"))
            .expect("golden VCD should exist")
    );

    fs::remove_file(&final_path).expect("temp VCD cleanup should succeed");
}

#[test]
fn synthetic_vcd_goldens_reject_overflow_aligned_logic_packet_replay() {
    let _guard = runtime_test_guard().lock().unwrap();
    let Some(runtime) = load_runtime() else {
        return;
    };
    let request = VcdExportRequest {
        samplerate_hz: 100_000_000,
        enabled_channels: vec![0, 1],
    };
    let sample_bytes = [0b00, 0b01, 0b10, 0b00];
    let invalid_packet_lengths = [3_usize, 1_usize];
    let final_path = temp_path("overflow-precondition");
    let sibling_temp = final_path.with_extension("vcd.tmp");
    let _ = fs::remove_file(&final_path);
    let _ = fs::remove_file(&sibling_temp);

    let error = runtime
        .render_vcd_from_logic_packets_to_path(
            &request,
            &sample_bytes,
            &invalid_packet_lengths,
            2,
            &final_path,
        )
        .expect_err("misaligned packet replay should fail fast");

    assert!(matches!(error, RuntimeError::InvalidArgument(detail) if detail.contains("aligned to unitsize")));
    assert!(!final_path.exists());
    assert!(!sibling_temp.exists());
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
